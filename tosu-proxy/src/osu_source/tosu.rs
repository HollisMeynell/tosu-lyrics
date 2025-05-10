use std::sync::atomic::{AtomicI64, Ordering};

use crate::error::Error;
use crate::model::tosu_types::TosuApi;
use crate::osu_source::OsuSource;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, watch};
use tokio::time::{Duration, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};

type WebsocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Tosu WebSocket 客户端
/// 用于连接 Tosu 并接收歌曲状态信息
pub(super) struct TosuWebsocketClient {
    /// WebSocket 连接地址
    url: String,
    /// 当前谱面 ID
    bid: AtomicI64,
    /// 当前谱面集 ID
    sid: AtomicI64,
    /// 当前音频文件
    audio_file: Mutex<String>,
    /// 重连延迟（毫秒）
    reconnect_delay_ms: u64,
    /// 连接超时（毫秒）
    connection_timeout_ms: u64,
    /// 心跳超时（毫秒）
    heartbeat_timeout_ms: u64,
}

impl OsuSource for TosuWebsocketClient {
    async fn start(self) {
        tokio::spawn(async move {
            loop {
                self.connect_and_process().await;
                // 连接断开后等待一段时间再重连
                sleep(Duration::from_millis(self.reconnect_delay_ms)).await;
            }
        });
    }
}

impl TosuWebsocketClient {
    /// 创建新的 Tosu WebSocket 客户端
    pub(super) fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            bid: AtomicI64::default(),
            sid: AtomicI64::default(),
            audio_file: Mutex::new(String::new()),
            reconnect_delay_ms: 1000,
            connection_timeout_ms: 5000,
            heartbeat_timeout_ms: 3000,
        }
    }

    /// 使用自定义配置创建 Tosu WebSocket 客户端
    pub(super) fn with_config(
        url: &str,
        reconnect_delay_ms: u64,
        connection_timeout_ms: u64,
        heartbeat_timeout_ms: u64,
    ) -> Self {
        Self {
            url: url.to_string(),
            bid: AtomicI64::default(),
            sid: AtomicI64::default(),
            audio_file: Mutex::new(String::new()),
            reconnect_delay_ms,
            connection_timeout_ms,
            heartbeat_timeout_ms,
        }
    }

    /// 连接到 WebSocket 并处理消息
    async fn connect_and_process(&self) {
        debug!("正在连接到 Tosu WebSocket: {}", self.url);

        // 尝试连接，带超时
        let connection_result = timeout(
            Duration::from_millis(self.connection_timeout_ms),
            connect_async(&self.url),
        )
        .await;

        // 处理连接超时
        let ws_result = match connection_result {
            Ok(result) => result,
            Err(_) => {
                error!("连接到 Tosu 超时 ({}ms)", self.connection_timeout_ms);
                return;
            }
        };

        // 处理连接错误
        if let Err(err) = ws_result {
            error!("连接到 Tosu 失败: {}", err);
            return;
        }

        // 连接成功，设置流
        let (ws_stream, _) = ws_result.unwrap();
        info!("已成功连接到 Tosu");

        let (mut sender, receiver) = ws_stream.split();

        // 设置心跳检测
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let (heartbeat_tx, mut heartbeat_rx) = watch::channel(false);

        // 克隆心跳超时时间用于心跳任务
        let heartbeat_timeout_ms = self.heartbeat_timeout_ms;

        // 启动心跳检测任务
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;

                // 发送心跳
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    warn!("无法发送心跳，连接可能已断开");
                    let _ = shutdown_tx.send(true);
                    break;
                }

                // 等待心跳响应
                match timeout(
                    Duration::from_millis(heartbeat_timeout_ms),
                    heartbeat_rx.changed(),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        debug!("收到心跳响应");
                        continue;
                    }
                    _ => {
                        warn!("心跳超时 ({}ms)，断开连接", heartbeat_timeout_ms);
                        let _ = shutdown_tx.send(true);
                        break;
                    }
                }
            }
        });

        // 处理主消息循环或关闭信号
        tokio::select! {
            _ = self.process_messages(receiver, heartbeat_tx) => {
                debug!("消息处理循环已结束");
            }
            _ = shutdown_rx.changed() => {
                debug!("收到关闭信号");
            }
        }

        info!(
            "WebSocket 连接断开，将在 {}ms 后重连",
            self.reconnect_delay_ms
        );
    }

    /// 处理 WebSocket 消息
    async fn process_messages(
        &self,
        mut receiver: WebsocketRead,
        heartbeat_tx: watch::Sender<bool>,
    ) {
        while let Some(message_result) = receiver.next().await {
            if message_result.is_err() {
                let err = message_result.err().unwrap();
                error!("WebSocket 读取错误: {}", err);
                break;
            }
            let message = message_result.unwrap();

            if !self.on_message(message, &heartbeat_tx).await {
                break;
            }
        }
    }

    /// return 是否继续下次监听
    async fn on_message(&self, message: Message, heartbeat_tx: &watch::Sender<bool>) -> bool {
        match message {
            Message::Text(text) => {
                let tosu_data = TosuApi::try_from(text.as_str());
                match tosu_data {
                    Ok(data) => {
                        self.handle_tosu_message(data).await;
                    }
                    Err(err) => {
                        error!("无法解析 Tosu 消息: {}", err);
                        debug!("异常原始信息: \n{}", text.as_str())
                    }
                }
            }
            Message::Pong(_) => {
                let _ = heartbeat_tx.send(true);
            }
            Message::Close(_) => {
                info!("websocket 对方关闭");
                return true;
            }
            _ => error!("websocket 收到无法处理的类型消息"),
        }
        true
    }

    /// 处理 Tosu 消息
    async fn handle_tosu_message(&self, tosu_data: TosuApi) {
        let bid = tosu_data.beatmap.id;
        let sid = tosu_data.beatmap.set;
        let now = tosu_data.beatmap.time.live;

        let old_bid = self.bid.load(Ordering::SeqCst);
        let old_sid = self.sid.load(Ordering::SeqCst);

        // 检查是否需要更新歌曲信息
        let update_song_info = {
            let current_audio = self.audio_file.lock().await;
            old_sid != sid || (old_bid != bid && *current_audio != tosu_data.files.audio)
        };

        if update_song_info {
            self.update_song_info(&tosu_data, bid, sid, now).await;
        } else {
            // 仅更新时间
            self.on_osu_state_change(super::OsuState::Time(now)).await;
        }
    }

    /// 更新歌曲信息
    async fn update_song_info(&self, tosu_data: &TosuApi, bid: i64, sid: i64, now: i32) {
        // 更新状态
        self.sid.store(sid, Ordering::SeqCst);
        self.bid.store(bid, Ordering::SeqCst);

        // 更新音频文件
        {
            let mut audio_file = self.audio_file.lock().await;
            *audio_file = tosu_data.files.audio.clone();
        }

        // 获取音频长度
        let length = crate::util::read_audio_length(tosu_data.audio_path())
            .await
            .unwrap_or_else(|e| {
                error!("无法获取音频长度: {}", e);
                -1
            });

        // 创建歌曲信息并通知状态变化
        let info = super::OsuSongInfo {
            bid,
            sid,
            length,
            now,
            artist: &tosu_data.beatmap.artist,
            artist_unicode: &tosu_data.beatmap.artist_unicode,
            title: &tosu_data.beatmap.title,
            title_unicode: &tosu_data.beatmap.title_unicode,
        };

        self.on_osu_state_change(super::OsuState::Song(info)).await;
    }
}

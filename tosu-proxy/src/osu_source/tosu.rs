use crate::config::TosuConfig;
use crate::error::Result;
use crate::server::ALL_SESSIONS;
use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::model::tosu_types::TosuApi;
use crate::osu_source::OsuSource;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Sender, channel};
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{error, info};
use tungstenite::handshake::client::Response;

type WebsocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub(super) struct TosuWebsocketClient {
    url: String,
    bid: AtomicI64,
    sid: AtomicI64,
    audio_file: Mutex<String>,
}

impl OsuSource for TosuWebsocketClient {
    async fn start(self) {
        tokio::spawn(async move {
            loop {
                self.on_loop().await;
            }
        });
    }
}

impl TosuWebsocketClient {
    pub(super) fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            bid: AtomicI64::default(),
            sid: AtomicI64::default(),
            audio_file: Mutex::new("".to_string()),
        }
    }

    /// ws 连接循环
    async fn on_loop(&self) {
        let result = timeout(Duration::from_secs(1), connect_async(&self.url)).await;

        let result = match result {
            Ok(r) => r,
            Err(_) => {
                error!("connect to Tosu time out");
                return;
            }
        };

        if result.is_err() {
            error!("Failed to connect to Tosu: {}", result.err().unwrap());
            sleep(Duration::from_secs(1)).await;
            return;
        }

        let (ws_stream, _) = result.unwrap();
        info!("Tosu connected");
        let (mut ping_write, read) = ws_stream.split();
        // 当 ping 发送后超过一秒未给相应认为网络中断, shutdown_xx 是传递中断信号
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        // pong 信号
        let (ping_tx, mut ping_rx) = watch::channel(false);
        // 启动 ws 检查循环
        tokio::spawn(async move {
            loop {
                // 等待一秒发送 ping, 如果无法发送认为网络出错, 发送 shutdown 信号
                tokio::time::sleep(Duration::from_secs(1)).await;
                if let Err(_) = ping_write.send(Message::Ping(vec![].into())).await {
                    let _ = shutdown_tx.send(true);
                    break;
                }
                // ping 响应超过一秒, 发送 shutdown 信号
                match timeout(Duration::from_secs(1), ping_rx.changed()).await {
                    Ok(Ok(_)) => continue,
                    _ => {
                        let _ = shutdown_tx.send(true);
                        break;
                    }
                }
            }
        });
        // 启动事件处理循环, 同时监听 shutdown 信号
        tokio::select! {
            _ = self.ws_receive_loop(read, ping_tx) =>  {}
            _ = shutdown_rx.changed() => {}
        }
        info!("WebSocket disconnected. Reconnecting...");
    }

    // ws 消息处理循环
    async fn ws_receive_loop(&self, mut read: WebsocketRead, mut ping_tx: watch::Sender<bool>) {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        self.on_tosu_message(text.to_string()).await;
                    }
                    Message::Pong(_) => {
                        let _ = ping_tx.send(true);
                    }
                    Message::Close(_) => {
                        info!("Received close message from Tosu");
                        break;
                    }
                    _ => {
                        error!("Received non-text message from Tosu");
                    }
                },
                Err(e) => {
                    error!("WebSocket read error: {}", e);
                    break;
                }
            }
        }
    }

    // 收到 tosu 消息处理
    async fn on_tosu_message(&self, msg: String) {
        let msg = TosuApi::try_from(&msg as &str);
        if let Err(e) = msg {
            error!("can not deserialization {}", e);
            return;
        }
        let tosu_data = msg.unwrap();

        let bid = tosu_data.beatmap.id;
        let sid = tosu_data.beatmap.set;
        let now = tosu_data.beatmap.time.live;

        let old_bid = self.bid.load(Ordering::SeqCst);
        let old_sid = self.sid.load(Ordering::SeqCst);
        let old_audio_file: &str;
        {
            old_audio_file = self.audio_file.lock().await.as_str();
        }
        if old_sid != sid
            || (old_bid != bid && *self.audio_file.lock().await != tosu_data.files.audio)
        {
            self.sid.store(sid, Ordering::SeqCst);
            self.bid.store(bid, Ordering::SeqCst);
            {
                *self.audio_file.lock().await = tosu_data.files.audio.clone();
            }
            let length = crate::util::read_audio_length(tosu_data.print_audio_path())
                .await
                .unwrap_or_else(|e| {
                    error!("can not get audio length: {}", e);
                    -1
                });
            let artist = &tosu_data.beatmap.artist;
            let artist_unicode = &tosu_data.beatmap.artist_unicode;
            let title = &tosu_data.beatmap.title;
            let title_unicode = &tosu_data.beatmap.title_unicode;
            let info = super::SongInfo {
                bid,
                sid,
                length,
                now,
                artist,
                artist_unicode,
                title,
                title_unicode,
            };
            self.on_osu_state_change(super::OsuState::Song(info)).await;
            return;
        }

        self.on_osu_state_change(super::OsuState::Time(now)).await;
    }
}

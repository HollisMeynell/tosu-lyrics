use crate::config::TosuConfig;
use crate::error::Result;
use crate::server::ALL_SESSIONS;

use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Sender, channel};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{error, info};
use tungstenite::handshake::client::Response;
use crate::model::tosu_types::TosuApi;

pub(super) async fn init_tosu_client(config: &TosuConfig) -> Result<()> {
    use salvo::websocket::Message;
    let (tx, mut rx) = channel(5);
    let tosu_client = TosuWebsocketClient::new(&config.url, tx);
    // 启动 tosu
    tosu_client.start();

    // 异步接受来自 tosu 的消息
    tokio::spawn(async move {
        let mut bid_cache = 0;
        while let Some(msg) = rx.recv().await {
            let msg = TosuApi::try_from(&msg as &str);
            match msg {
                Ok(msg) => {
                    if bid_cache == msg.beatmap.id {
                        continue;
                    }
                    bid_cache = msg.beatmap.id;
                    info!("beatmap change: {bid_cache}");
                    ALL_SESSIONS.send_to_all_client(Message::text(msg.beatmap.title)).await;
                }
                Err(e) => {
                    error!("can not trans{}",e);
                }
            }
        }
    });
    Ok(())
}

struct TosuWebsocketClient {
    url: String,
    sender: Sender<String>,
}

impl TosuWebsocketClient {
    fn new(url: &str, sender: Sender<String>) -> Self {
        Self {
            url: url.to_string(),
            sender,
        }
    }

    fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                self.on_loop().await;
            }
        })
    }

    /// ws 连接循环
    async fn on_loop(&self) {
        let result = timeout(Duration::from_secs(1), connect_async(&self.url)).await;

        let result = match result {
            Ok(r) => { r }
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

        // 拿到 websocket stream
        let (ws_stream, _) = result.unwrap();
        info!("Tosu connected");
        let (mut ping_write, read) = ws_stream.split();
        // 当 ping 发送后超过一秒未给相应认为网络中断, shutdown_xx 是传递中断信号
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        // ping_xx 是受到的 pong 信号
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
    async fn ws_receive_loop(
        &self,
        mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        mut ping_tx: watch::Sender<bool>,
    ) {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        self.on_tosu_message(text.to_string()).await;
                    }
                    Message::Pong(_) => {
                        let _ = ping_tx.send(true).await;
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
        if let Err(e) = &self.sender.send(msg.to_string()).await {
            error!("Failed to send message to channel: {}", e);
        }
    }
}

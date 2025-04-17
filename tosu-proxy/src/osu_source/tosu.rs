use crate::config::TosuConfig;
use crate::error::Result;
use crate::server::ALL_SESSIONS;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{Sender, channel};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub(super) async fn init_tosu_client(config: &TosuConfig) -> Result<()> {
    use salvo::websocket::Message;
    let (tx, mut rx) = channel(5);
    let tosu_client = TosuWebsocketClient::new(&config.url, tx);
    tosu_client.start();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Received: {}", msg);
            ALL_SESSIONS.send_to_all_client(Message::text(msg)).await;
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

    async fn on_loop(&self) {
        let result = connect_async(&self.url).await;
        if result.is_err() {
            error!("Failed to connect to Tosu: {}", result.err().unwrap());
            sleep(Duration::from_secs(1)).await;
            return;
        }

        let (ws_stream, _) = result.unwrap();
        info!("Tosu connected");
        let (mut ping_write, read) = ws_stream.split();
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let (tx, mut rx) = channel::<u8>(1);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1));
                if let Err(_) = ping_write.send(Message::Ping(vec![].into())).await {
                    let _ = shutdown_tx.send(true);
                    break;
                }
                match timeout(Duration::from_secs(1), rx.recv()).await {
                    Ok(Some(_)) => continue,
                    Ok(None) | Err(_) => {
                        let _ = shutdown_tx.send(true);
                        break;
                    }
                }
            }
        });
        tokio::select! {
            _ = self.ws_receive_loop(read, tx) =>  {}
            _ = shutdown_rx.changed() => {}
        }
        info!("WebSocket disconnected. Reconnecting...");
    }

    async fn ws_receive_loop<S>(&self, mut read: S, mut tx: Sender<u8>)
    where
        S: futures::Stream<Item = std::result::Result<Message, tungstenite::Error>> + Unpin,
    {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        self.on_tosu_message(text.to_string()).await;
                    }
                    Message::Pong(_) => {
                        let _ = tx.send(1).await;
                    }
                    Message::Close(_) => {
                        info!("Received close message from Tosu.");
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

    async fn on_tosu_message(&self, msg: String) {
        if let Err(e) = &self.sender.send(msg.to_string()).await {
            error!("Failed to send message to channel: {}", e);
        }
    }
}

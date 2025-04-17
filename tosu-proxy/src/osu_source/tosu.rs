use crate::config::TosuConfig;
use crate::error::Result;
use crate::server::ALL_SESSIONS;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::connect_async;
use tracing::{error, info};

use salvo::websocket::Message;

pub(super) async fn init_tosu_client(config: &TosuConfig) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(5);
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
            let url = self.url.clone();
            loop {
                let result = connect_async(&url).await;
                if result.is_err() {
                    error!("Failed to connect to Tosu: {}", result.err().unwrap());
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                let (ws_stream, _) = result.unwrap();
                info!("Tosu connected");
                let (mut _write, mut read) = ws_stream.split();

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(msg) if msg.is_text() => {
                            if let Err(e) = self.sender.send(msg.to_string()).await {
                                error!("Failed to send message to channel: {}", e);
                            }
                        }
                        Ok(_) => {
                            error!("Received non-text message from Tosu");
                        }
                        Err(e) => {
                            error!("WebSocket read error: {}", e);
                            break;
                        }
                    }
                }
                info!("WebSocket disconnected. Reconnecting...");
            }
        })
    }
}

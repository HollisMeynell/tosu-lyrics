use std::collections::HashMap;

use crate::config::CONFIG_ENDPOINT_WEBSOCKET;
use crate::error::Result;
use crate::util::generate_random_string;
use salvo::websocket::{Message, WebSocket};
use salvo::{Request, Response, Router, handler};
use std::sync::LazyLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{RwLock, mpsc};
use tracing::{error, info};

static ALL_SESSIONS: LazyLock<WebsocketSession> = LazyLock::new(|| WebsocketSession::new());

type LyricWebsocketMessage = String;

fn lyric_message_to_str(msg: LyricWebsocketMessage) -> Message {
    Message::text(msg)
}

#[derive(Debug)]
enum ClientType {
    Client(UnboundedSender<Message>),
    Setter(UnboundedSender<Message>),
}

impl ClientType {
    fn create(is_client: bool, channel: UnboundedSender<Message>) -> Self {
        if is_client {
            ClientType::Client(channel)
        } else {
            ClientType::Setter(channel)
        }
    }

    fn get_channel(&self) -> &UnboundedSender<Message> {
        match self {
            ClientType::Client(ch) => ch,
            ClientType::Setter(ch) => ch,
        }
    }
}

#[derive(Debug)]
struct WebsocketSession(RwLock<HashMap<String, ClientType>>);

impl Default for WebsocketSession {
    fn default() -> Self {
        WebsocketSession(RwLock::new(HashMap::new()))
    }
}
impl WebsocketSession {
    fn new() -> Self {
        Self::default()
    }

    async fn add_client(&self, sender: ClientType) -> String {
        let mut key = generate_random_string();
        while self.0.read().await.contains_key(&key) {
            key = generate_random_string()
        }
        self.0.write().await.insert(key.clone(), sender);
        key
    }

    async fn remove_client<T>(&self, key: &T)
    where
        T: AsRef<str>,
    {
        let old = self.0.write().await.remove(key.as_ref());
        if let Some(old) = old {
            old.get_channel().closed().await;
        }
    }

    async fn send_to_client<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        self.0
            .read()
            .await
            .iter()
            .filter(|(k, client)| key.as_ref() != *k && matches!(client, &&ClientType::Client(_)))
            .for_each(|(_, client)| {
                let r = client.get_channel().send(message.clone());
                if let Err(e) = r {
                    error!("can not send message: {e}")
                }
            })
    }

    async fn send_message<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        if let Some(channel) = self.0.read().await.get(key.as_ref()) {
            if let Err(e) = channel.get_channel().send(message) {
                error!("can not send message: {e}")
            }
        }
    }
}

async fn on_ws_message(key: &str, msg: Message) {
    // todo: 实现处理 setter 发出的指令

    if msg.is_close() {
        ALL_SESSIONS.remove_client(&key).await;
    } else if msg.is_text() {
        let t = msg.as_str().unwrap();
        ALL_SESSIONS.send_message(&key, msg.clone()).await;
        info!("text: {t}");
    }
}

async fn handle_ws(ws: WebSocket, key: String, mut rx: UnboundedReceiver<Message>) {
    use futures_util::{SinkExt, StreamExt};
    let (mut ws_sender, mut ws_receiver) = ws.split();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(msg).await {
                error!("Failed to send message: {e}");
                break;
            }
        }
    });
    while let Some(data) = ws_receiver.next().await {
        let key = key.clone();
        match data {
            Ok(message) => {
                tokio::spawn(async move {
                    on_ws_message(&key, message).await;
                });
            }
            Err(e) => {
                error!("websocket err(id={key}): {e}");
            }
        }
    }
}

/// is setter client if url "ws://(ip:port)?setter=true"
#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<()> {
    use salvo::websocket::WebSocketUpgrade;
    // 因为要忽略向设置端发送歌词, 需要通过 ws 连接的参数 setter 来判断是否为设置器 "ws://127.0.0.1:41280?setter=true"
    let is_client = req.query::<bool>("setter").is_none();
    WebSocketUpgrade::new()
        .upgrade(req, res, async move |ws| {
            let (tx, rx) = mpsc::unbounded_channel::<Message>();
            let key = ALL_SESSIONS
                .add_client(ClientType::create(is_client, tx))
                .await;
            handle_ws(ws, key, rx).await;
        })
        .await?;
    Ok(())
}

pub fn get_ws_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_WEBSOCKET).goal(connect)
}

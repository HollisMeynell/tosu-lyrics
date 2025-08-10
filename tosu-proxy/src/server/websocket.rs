use std::collections::HashMap;

use crate::config::{CONFIG_ENDPOINT_WEBSOCKET, CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT};
use crate::error::Result;
use crate::util::generate_random_string;
use salvo::websocket::{Message, WebSocket};
use salvo::{Request, Response, Router, handler};
use std::sync::LazyLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info};

pub static ALL_SESSIONS: LazyLock<WebsocketSession> = LazyLock::new(WebsocketSession::new);

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
pub struct WebsocketSession(RwLock<HashMap<String, ClientType>>);

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
        self.0.write().await.remove(key.as_ref());
    }

    async fn find_clients<F: Fn(&String, &ClientType) -> bool>(&self, message: Message, f: F) {
        self.0
            .read()
            .await
            .iter()
            .filter(|(key, client)| f(key, client))
            .for_each(|(_, client)| {
                let r = client.get_channel().send(message.clone());
                if let Err(e) = r {
                    error!("can not send message: {e}")
                }
            });
    }

    pub async fn send_to_all_client(&self, message: Message) {
        self.find_clients(message, |_, client| {
            matches!(*client, ClientType::Client(_))
        })
        .await;
    }
    pub async fn send_to_one_client<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        self.find_clients(message, |k, client| {
            key.as_ref() != k && matches!(*client, ClientType::Client(_))
        })
        .await;
    }

    pub async fn send_message<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        if let Some(channel) = self.0.read().await.get(key.as_ref()) {
            if let Err(e) = channel.get_channel().send(message) {
                error!("can not send message: {e}")
            }
        }
    }
    pub async fn send_pong<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        let sessions = self.0.read().await;
        let channel = sessions.get(key.as_ref());
        let channel = match channel {
            Some(ch) => ch,
            _ => return,
        };
        let pong = Message::pong(message.as_bytes().to_vec());
        if let Err(e) = channel.get_channel().send(pong) {
            error!("can not send message: {e}")
        }
    }
}

async fn on_ws_message(key: &str, message: Message) {
    if message.is_ping() {
        ALL_SESSIONS.send_pong(&key, message).await;
        return;
    }

    if !message.is_text() {
        debug!("receive not text message");
        return;
    }

    if let Ok(message) = message.as_str() { crate::service::on_setting(key, message).await }
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
        _ = ws_sender.close().await;
    });

    while let Some(data) = ws_receiver.next().await {
        let key = key.clone();
        match data {
            Ok(message) => {
                // 提前处理关闭消息
                if message.is_close() {
                    break;
                }
                tokio::spawn(async move {
                    on_ws_message(&key, message).await;
                });
            }
            Err(e) => {
                error!("websocket err(id={key}): {e}");
                break;
            }
        }
    }
    ALL_SESSIONS.remove_client(&key).await;
}

/// is setter client if url "ws://(ip:port)?setter=true"
#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<()> {
    use salvo::websocket::WebSocketUpgrade;
    // 因为要忽略向设置端发送歌词, 需要通过 ws 连接的参数 setter 来判断是否为设置器 "ws://127.0.0.1:41280?setter=true"
    let is_client = req
        .query::<bool>(CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT)
        .is_none();
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

use std::collections::HashMap;

use crate::config::CONFIG_ENDPOINT_WEBSOCKET;
use crate::error::Result;
use crate::util::generate_random_string;
use salvo::websocket::{Message, WebSocket};
use salvo::{handler, Request, Response, Router};
use std::sync::LazyLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{mpsc, RwLock};
use tracing::log::{log, Level};

static ALL_SESSIONS: LazyLock<WebsocketSession> = LazyLock::new(|| WebsocketSession::new());

type LyricWebsocketMessage = String;

fn lyric_message_to_str(msg: LyricWebsocketMessage) -> Message {
    Message::text(msg)
}

#[derive(Debug)]
enum ClientType {
    Client(UnboundedSender<LyricWebsocketMessage>),
    Setter(UnboundedSender<LyricWebsocketMessage>),
}

impl ClientType {
    fn create(is_client: bool, channel: UnboundedSender<LyricWebsocketMessage>) -> Self {
        if is_client {
            ClientType::Client(channel)
        } else {
            ClientType::Setter(channel)
        }
    }

    fn get_channel(&self) -> &UnboundedSender<LyricWebsocketMessage> {
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

    async fn send_to_client<T>(&self, key: &T, message: LyricWebsocketMessage)
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
                    log!(Level::Error, "can not send message: {e:?}")
                }
            })
    }

    async fn send_message<T>(&self, key: &T, message: LyricWebsocketMessage)
    where
        T: AsRef<str>,
    {
        if let Some(channel) = self.0.read().await.get(key.as_ref()) {
            if let Err(e) = channel.get_channel().send(message) {
                log!(Level::Error, "can not send message: {e:?}")
            }
        }
    }
}

async fn on_ws_message(key: &str, msg: Message) {
    // todo: 实现处理 setter 发出的指令
}

async fn handle_ws(ws: WebSocket, key: String, mut rx: UnboundedReceiver<LyricWebsocketMessage>) {
    use futures_util::{SinkExt, StreamExt};
    let (mut ws_sender, mut ws_receiver) = ws.split();
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg = lyric_message_to_str(msg);
            if let Err(e) = ws_sender.send(msg).await {
                log!(Level::Error, "Failed to send message: {e:?}");
                break;
            }
        }
    });
    tokio::task::spawn(async move {
        while let Some(data) = ws_receiver.next().await {
            match data {
                Ok(message) => {
                    on_ws_message(&key, message).await;
                }
                Err(e) => {
                    log!(Level::Error, "websocket err(id={key}): {e:?}");
                }
            }
        }
    });
}

/// is setter client if url "ws://(ip:port)?setter=true"
#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<()> {
    use salvo::websocket::WebSocketUpgrade;
    // 因为要忽略向设置端发送歌词, 需要通过 ws 连接的参数 setter 来判断是否为设置器 "ws://127.0.0.1:41280?setter=true"
    let is_client = req.query::<bool>("setter").is_none();
    async fn salvo_sb(ws: WebSocket) {
        let (tx, rx) = mpsc::unbounded_channel::<LyricWebsocketMessage>();
        let key = ALL_SESSIONS
            .add_client(ClientType::create(is_client, tx))
            .await;
        tokio::task::spawn(async { handle_ws(ws, key, rx) });
    }
    WebSocketUpgrade::new().upgrade(req, res, salvo_sb).await?;
    Ok(())
}

fn get_ws_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_WEBSOCKET).goal(connect)
}

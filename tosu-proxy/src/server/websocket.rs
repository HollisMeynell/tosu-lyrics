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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshot {
    pub key: String,
    pub identity: Option<String>,
    pub connected_at: i64,
    pub user_agent: Option<String>,
}

fn lyric_message_to_str(msg: LyricWebsocketMessage) -> Message {
    Message::text(msg)
}

fn display_channel(channel: UnboundedSender<Message>) -> UnboundedSender<Message> {
    channel
}

/// key 是每次连接随机生成的会话标识；identity 是客户端通过 ?id= 自报的稳定身份。
#[derive(Debug)]
pub struct SessionEntry {
    channel: UnboundedSender<Message>,
    identity: Option<String>,
    connected_at: i64,
    user_agent: Option<String>,
}

impl SessionEntry {
    pub fn is_display(&self) -> bool {
        true
    }
    pub fn identity(&self) -> Option<&str> {
        self.identity.as_deref()
    }
    pub fn connected_at(&self) -> i64 {
        self.connected_at
    }
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }
}

#[derive(Debug)]
pub struct WebsocketSession(RwLock<HashMap<String, SessionEntry>>);

impl Default for WebsocketSession {
    fn default() -> Self {
        WebsocketSession(RwLock::new(HashMap::new()))
    }
}
impl WebsocketSession {
    fn new() -> Self {
        Self::default()
    }

    async fn add_client(
        &self,
        channel: UnboundedSender<Message>,
        identity: Option<String>,
        user_agent: Option<String>,
    ) -> String {
        let mut key = generate_random_string();
        while self.0.read().await.contains_key(&key) {
            key = generate_random_string()
        }
        let now = sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis();
        let entry = SessionEntry {
            channel: display_channel(channel),
            identity,
            connected_at: now,
            user_agent,
        };
        self.0.write().await.insert(key.clone(), entry);
        key
    }

    pub async fn display_clients(&self) -> Vec<(String, ClientSnapshot)> {
        self.0
            .read()
            .await
            .iter()
            .filter(|(_, e)| e.is_display())
            .map(|(key, e)| {
                (
                    key.clone(),
                    ClientSnapshot {
                        key: key.clone(),
                        identity: e.identity().map(|s| s.to_string()),
                        connected_at: e.connected_at(),
                        user_agent: e.user_agent().map(|s| s.to_string()),
                    },
                )
            })
            .collect()
    }

    pub async fn resolve_display_targets(&self, target: &str) -> Vec<String> {
        self.0
            .read()
            .await
            .iter()
            .filter(|(key, e)| e.is_display() && (key.as_str() == target || e.identity() == Some(target)))
            .map(|(key, _)| key.clone())
            .collect()
    }

    async fn remove_client<T>(&self, key: &T)
    where
        T: AsRef<str>,
    {
        self.0.write().await.remove(key.as_ref());
    }

    async fn find_clients<F: Fn(&String, &SessionEntry) -> bool>(&self, message: Message, f: F) {
        let sessions = self.0.read().await;

        let failed_keys: Vec<String> = sessions
            .iter()
            .filter(|(key, client)| f(key, client))
            .filter_map(|(key, client)| {
                client.channel.send(message.clone())
                    .err()
                    .map(|_| key.clone())
            })
            .collect();

        drop(sessions); // 释放读锁

        for key in failed_keys {
            self.remove_client(&key).await;
            debug!("Removed dead client: {key}");
        }
    }

    pub async fn send_to_all_client(&self, message: Message) {
        self.find_clients(message, |_, _| true).await;
    }
    pub async fn send_to_one_client<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        self.find_clients(message, |k, client| key.as_ref() == k && client.is_display())
        .await;
    }

    pub async fn send_message<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        let key_str = key.as_ref();
        let send_result = self.0.read().await
            .get(key_str)
            .map(|entry| entry.channel.send(message));

        if let Some(Err(e)) = send_result {
            error!("Failed to send message to {key_str}: {e}");
            self.remove_client(key).await;
            debug!("Removed dead client: {key_str}");
        }
    }
    pub async fn send_pong<T>(&self, key: &T, message: Message)
    where
        T: AsRef<str>,
    {
        let key_str = key.as_ref();
        let pong = Message::pong(message.as_bytes().to_vec());
        let send_result = self.0.read().await
            .get(key_str)
            .map(|entry| entry.channel.send(pong));

        if let Some(Err(e)) = send_result {
            error!("Failed to send pong to {key_str}: {e}");
            self.remove_client(key).await;
            debug!("Removed dead client: {key_str}");
        }
    }
}

/// WS 不再是管理通道：客户端发来的任何文本只记录，不做业务操作。
async fn on_ws_message(key: &str, message: Message) {
    if message.is_ping() {
        ALL_SESSIONS.send_pong(&key, message).await;
        return;
    }

    if !message.is_text() {
        debug!("receive not text message");
        return;
    }

    if let Ok(text) = message.as_str() {
        debug!("忽略展示端上行消息(WS 管理已移除, id={key}): {}", &text[..text.len().min(80)]);
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
        _ = ws_sender.close().await;
    });

    while let Some(data) = ws_receiver.next().await {
        let key = key.clone();
        match data {
            Ok(message) => {
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

#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<()> {
    use salvo::websocket::WebSocketUpgrade;
    // ?setter=true 仍被接受但不再有一任何特权，仅为兼容旧客户端保留
    let _legacy_setter_param = req.query::<bool>(CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT);
    let identity = req
        .query::<String>("id")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 64);
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(160).collect::<String>());
    WebSocketUpgrade::new()
        .upgrade(req, res, async move |ws| {
            let (tx, rx) = mpsc::unbounded_channel::<Message>();
            let key = ALL_SESSIONS
                .add_client(tx, identity, user_agent)
                .await;
            {
                crate::service::send_settings_snapshot(&key).await;
                // 无歌词时下发清屏，避免断线重连 / OBS 刷新后残留上一次歌词
                match crate::service::LYRIC_SERVICE.lock().await.get_snapshot() {
                    Some(snapshot) => ALL_SESSIONS.send_message(&key, snapshot.into()).await,
                    None => {
                        let clean = crate::model::websocket::setting::SettingPayload::new(
                            "setClear".to_string(),
                        );
                        ALL_SESSIONS
                            .send_message(
                                &key,
                                Into::<crate::model::websocket::WebSocketMessage>::into(clean)
                                    .into(),
                            )
                            .await;
                    }
                }
            }
            handle_ws(ws, key, rx).await;
        })
        .await?;
    Ok(())
}

pub fn get_ws_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_WEBSOCKET).goal(connect)
}

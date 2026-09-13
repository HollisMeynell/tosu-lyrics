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

/// 对外暴露的在线展示端信息（B-08）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshot {
    /// 会话标识 —— 每次连接都不同
    pub key: String,
    /// 客户端自报的稳定身份；匿名为 null
    pub identity: Option<String>,
    pub connected_at: i64,
    pub user_agent: Option<String>,
}

fn lyric_message_to_str(msg: LyricWebsocketMessage) -> Message {
    Message::text(msg)
}

// B-11：`ClientType` 已删除。
//
// 它原本用来区分"展示端"与"setter / 管理连接"。管理通道移除后，
// 一个 WS 连接除了接收展示事件之外没有任何用途，因此**所有连接都是展示端**。
// 客户端仍然可以带 `?setter=true`，但那只是个被忽略的参数 —— 它不再开启任何特权。

fn display_channel(channel: UnboundedSender<Message>) -> UnboundedSender<Message> {
    channel
}

/// 一个在线会话（B-08）。
///
/// **在线会话 ≠ 持久身份**：
/// - `key` 是每次连接随机生成的**会话标识**，断线重连就换一个
/// - `identity` 是客户端通过 `?id=<名字>` **自报的稳定身份**，同一台展示端
///   重连后仍然是它；没自报就是匿名会话（`identity` 为 `None`）
#[derive(Debug)]
pub struct SessionEntry {
    channel: UnboundedSender<Message>,
    /// 客户端自报的稳定身份
    identity: Option<String>,
    connected_at: i64,
    user_agent: Option<String>,
}

impl SessionEntry {
    /// B-11 之后所有连接都是展示端（见上方注释）
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

    /// 在线展示端列表（**不含管理/setter 连接**）
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

    /// 按**会话 key 或自报身份**找展示端；返回匹配到的会话 key 列表
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

        // 收集发送失败的客户端 key
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

        // 移除失效的客户端
        for key in failed_keys {
            self.remove_client(&key).await;
            debug!("Removed dead client: {key}");
        }
    }

    pub async fn send_to_all_client(&self, message: Message) {
        // 所有连接都是展示端
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

/// 处理展示端发来的 WS 消息（B-11 之后）。
///
/// **WS 不再是管理通道**：客户端发来的任何文本都只被记录，不做任何业务操作。
/// 管理一律走 HTTP。这里保留函数是为了心跳与后续可能的展示端上报，
/// 但**没有**任何 dispatch 到管理逻辑的分支。
async fn on_ws_message(key: &str, message: Message) {
    if message.is_ping() {
        ALL_SESSIONS.send_pong(&key, message).await;
        return;
    }

    if !message.is_text() {
        debug!("receive not text message");
        return;
    }

    // 明确不作为管理请求处理：只记一条 debug 日志。
    // 这样"向 WS 发旧管理 key"既查不到也改不了任何业务状态。
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
    // B-11：`?setter=true` 仍然被接受，但**不再有任何特权** ——
    // 管理通道已经移除，所有连接一律是展示端。这个参数只为兼容旧客户端保留。
    let _legacy_setter_param = req.query::<bool>(CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT);
    // 客户端可自报稳定身份：ws://host/ws?id=obs-main
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
            // 展示端接入后立即下发完整当前状态, 无需等待下一次歌词变化
            {
                // 1. 已保存的展示样式(主副颜色/字体/字号/对齐/翻译为主/副歌词显隐)
                crate::service::send_settings_snapshot(&key).await;
                // 2. 歌词快照; 后端当前无歌词时明确下发清屏,
                //    避免断线重连/OBS 刷新后残留上一次的歌词
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

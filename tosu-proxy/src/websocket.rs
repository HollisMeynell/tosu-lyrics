use actix_web::{rt, web, HttpRequest, HttpResponse};
use actix_ws::{AggregatedMessage, MessageStream, Session};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Deserialize, Serialize)]
struct ConfigMessage {
    #[serde(
        deserialize_with = "deserialize_to_str",
        serialize_with = "serialize_to_str"
    )]
    command: String,
    echo: Option<String>,
}

// 反序列化时, 将各种类型转换为字符串
fn deserialize_to_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => serde_json::to_string(&other).unwrap_or_else(|_| "unknown".to_string()),
    })
}

// 序列化时, 保持字符串格式
fn serialize_to_str<S: Serializer>(command: &String, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(command)
}

#[derive(Debug)]
struct WsSessions(RwLock<HashMap<usize, UnboundedSender<String>>>);

static ALL_WS_SESSION: LazyLock<WsSessions> = LazyLock::new(|| WsSessions::new());

static mut SESSION_KEY: usize = 0;

impl WsSessions {
    pub(self) fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    fn get_static() -> &'static Self {
        &*ALL_WS_SESSION
    }

    fn get_new_key() -> usize {
        unsafe {
            SESSION_KEY += 1;
            SESSION_KEY
        }
    }

    async fn add_session(&self, key: usize, sender: UnboundedSender<String>) {
        self.0.write().await.insert(key, sender);
    }

    async fn remove_session(&self, key: usize) {
        let old = self.0.write().await.remove(&key);
        if let Some(old) = old {
            old.closed().await;
        }
    }

    async fn send_all(&self, self_key: usize, message: String) {
        for (key, sender) in self.0.read().await.iter() {
            if self_key == *key {
                continue;
            }
            let _ = sender.send(message.clone());
        }
    }

    async fn send_one(&self, key: usize, message: String) {
        if let Some(sender) = self.0.read().await.get(&key) {
            let _ = sender.send(message);
        }
    }
}

pub async fn handle(
    request: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, stream) = actix_ws::handle(&request, stream)?;

    let (tx, rx) = mpsc::unbounded_channel::<String>();
    let key = WsSessions::get_new_key();
    WsSessions::get_static().add_session(key, tx).await;
    tokio::task::spawn_local(handle_ws(session, stream, key, rx));

    Ok(response)
}

pub async fn handle_ws(
    mut session: Session,
    stream: MessageStream,
    key: usize,
    mut rx: UnboundedReceiver<String>,
) {
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let mut session_clone = session.clone();
    rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(_) = session_clone.text(msg).await {
                WsSessions::get_static().remove_session(key).await;
            }
        }
    });

    while let Some(msg) = stream.recv().await {
        if msg.is_err() {
            WsSessions::get_static().remove_session(key).await;
            break;
        }

        let msg = msg.unwrap();
        match msg {
            AggregatedMessage::Text(text) => {
                // println!("n: {}", WsSessions::get_static().0.read().await.len());
                let massage = serde_json::from_str::<ConfigMessage>(text.trim());
                if let Ok(msg) = massage {
                    WsSessions::get_static()
                        .send_all(key, serde_json::to_string(&msg).unwrap())
                        .await
                } else {
                    WsSessions::get_static()
                        .send_one(key, "json parse error".to_string())
                        .await
                }
            }
            AggregatedMessage::Binary(data) => {
                let massage = serde_json::from_slice::<ConfigMessage>(data.as_ref());
                if let Ok(msg) = massage {
                    WsSessions::get_static()
                        .send_all(key, serde_json::to_string(&msg).unwrap())
                        .await
                } else {
                    WsSessions::get_static()
                        .send_one(key, "data json parse error".to_string())
                        .await
                }
            }
            AggregatedMessage::Ping(_) => {
                if let Err(_) = session.pong(&[0u8]).await {
                    WsSessions::get_static().remove_session(key).await;
                    break;
                }
            }
            AggregatedMessage::Pong(_) => {}
            AggregatedMessage::Close(_) => {
                let _ = session.close(None).await;
                WsSessions::get_static().remove_session(key).await;
                break;
            }
        };
    }
}

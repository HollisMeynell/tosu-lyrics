use actix_web::{HttpRequest, HttpResponse, rt, web};
use actix_ws::{AggregatedMessage, MessageStream, Session};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::util::generate_random_string;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{RwLock, mpsc};

#[derive(Debug, Deserialize, Serialize)]
struct ConfigMessage {
    command: Value,
    echo: Option<String>,
}

#[derive(Debug)]
struct WsSessions(RwLock<HashMap<String, UnboundedSender<String>>>);

static ALL_WS_SESSION: LazyLock<WsSessions> = LazyLock::new(|| WsSessions::new());

impl WsSessions {
    pub(self) fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    fn get_static() -> &'static Self {
        &*ALL_WS_SESSION
    }

    async fn get_new_key() -> String {
        let mut key = generate_random_string();
        while let Some(_) = ALL_WS_SESSION.0.read().await.get(&key) {
            key = generate_random_string();
        }
        key
    }

    async fn add_session(&self, key: String, sender: UnboundedSender<String>) {
        self.send_all(&key, WsSessions::get_online_json(&key)).await;
        self.0.write().await.insert(key, sender);
    }

    async fn remove_session(&self, key: &str) {
        let old = self.0.write().await.remove(key);
        let offline_message = WsSessions::get_offline_json(key);
        self.send_all(key, offline_message).await;
        if let Some(old) = old {
            old.closed().await;
        }
    }

    async fn send_all(&self, self_key: &str, message: String) {
        for (key, sender) in self.0.read().await.iter() {
            if self_key == *key {
                continue;
            }
            let _ = sender.send(message.clone());
        }
    }

    async fn send_one(&self, key: &str, message: String) {
        if let Some(sender) = self.0.read().await.get(key) {
            let _ = sender.send(message);
        }
    }

    fn get_online_json(key: &str) -> String {
        format!(r##"{{"command":{{"type":"online","id":"{key}","status":"online"}}}}"##)
    }

    async fn get_new_client_json(key: &str) -> String {
        let others = ALL_WS_SESSION
            .0
            .read()
            .await
            .keys()
            .filter_map(|x| {
                if x == key {
                    None
                } else {
                    Some(format!("\"{x}\""))
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            r##"{{"command":{{"type":"online","id":"{key}","status":"online","others":[{others}]}}}}"##
        )
    }

    fn get_offline_json(key: &str) -> String {
        format!(r##"{{"command":{{"type":"online","id":"{key}","status":"offline"}}}}"##)
    }
}

pub async fn handle(
    request: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, stream) = actix_ws::handle(&request, stream)?;

    let (tx, rx) = mpsc::unbounded_channel::<String>();
    let key = WsSessions::get_new_key().await;
    WsSessions::get_static().add_session(key.clone(), tx).await;
    tokio::task::spawn_local(handle_ws(session, stream, key, rx));
    Ok(response)
}

async fn handle_ws(
    mut session: Session,
    stream: MessageStream,
    key: String,
    mut rx: UnboundedReceiver<String>,
) {
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let mut session_clone = session.clone();
    let key_clone = key.clone();
    rt::spawn(async move {
        let online_message = WsSessions::get_new_client_json(&key_clone).await;
        let _ = session_clone.text(online_message).await;
        while let Some(msg) = rx.recv().await {
            if let Err(_) = session_clone.text(msg).await {
                WsSessions::get_static().remove_session(&key_clone).await;
            }
        }
    });

    while let Some(msg) = stream.recv().await {
        if msg.is_err() {
            WsSessions::get_static().remove_session(&key).await;
            break;
        }

        let msg = msg.unwrap();
        match msg {
            AggregatedMessage::Text(text) => {
                let massage = crate::util::to_json::<ConfigMessage>(text.trim());
                if let Ok(msg) = massage {
                    WsSessions::get_static()
                        .send_all(&key, serde_json::to_string(&msg).unwrap())
                        .await
                } else {
                    WsSessions::get_static()
                        .send_one(&key, "json parse error".to_string())
                        .await
                }
            }
            AggregatedMessage::Binary(data) => {
                let massage = serde_json::from_slice::<ConfigMessage>(data.as_ref());
                if let Ok(msg) = massage {
                    WsSessions::get_static()
                        .send_all(&key, serde_json::to_string(&msg).unwrap())
                        .await
                } else {
                    WsSessions::get_static()
                        .send_one(&key, "data json parse error".to_string())
                        .await
                }
            }
            AggregatedMessage::Ping(_) => {
                if let Err(_) = session.pong(&[0u8]).await {
                    WsSessions::get_static().remove_session(&key).await;
                    break;
                }
            }
            AggregatedMessage::Pong(_) => {}
            AggregatedMessage::Close(_) => {
                let _ = session.close(None).await;
                WsSessions::get_static().remove_session(&key).await;
                break;
            }
        };
    }
}

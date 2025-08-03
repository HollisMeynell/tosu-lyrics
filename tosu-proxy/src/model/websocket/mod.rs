pub mod lyric;
pub mod setting;

use crate::error::Result;

use crate::model::JsonStruct;
use lyric::*;
use salvo::websocket::Message;
use serde::{Deserialize, Serialize, Serializer};
use setting::*;
use std::fmt::{Display, Formatter};
use tracing::error;

/// 表示 WebSocket 消息的类型。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Lyric,
    Setting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebSocketMessage {
    Lyric(LyricPayload),
    Setting(SettingPayload),
}
impl Display for WebSocketMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        f.write_str(&json)
    }
}

impl From<LyricPayload> for WebSocketMessage {
    fn from(value: LyricPayload) -> Self {
        WebSocketMessage::Lyric(value)
    }
}

impl From<SettingPayload> for WebSocketMessage {
    fn from(value: SettingPayload) -> Self {
        WebSocketMessage::Setting(value)
    }
}

impl Into<Message> for WebSocketMessage {
    fn into(self) -> Message {
        if let Ok(txt) = self.to_json_string() {
            Message::text(txt)
        } else {
            error!("Failed to serialize `WebSocketMessage` {self:?}");
            Message::text("")
        }
    }
}

impl WebSocketMessage {
    pub fn new_error<T: AsRef<str>>(err: T) -> Self {
        Self::new_error_has_type("unknown", err)
    }
    pub fn new_error_has_type<T: AsRef<str>, E: AsRef<str>>(key: T, err: E) -> Self {
        let mut message = SettingPayload::new(key.as_ref().to_string());
        message.error.replace(err.as_ref().to_string());
        WebSocketMessage::Setting(message)
    }

    fn get_mut_setting(&mut self) -> Result<&mut SettingPayload> {
        match self {
            WebSocketMessage::Lyric(_) => Err("type `lyric` can not get setting".into()),
            WebSocketMessage::Setting(setting) => Ok(setting),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_setting() -> Result<()> {
        // setting
        let setting_str = r#"{"type":"setting","id":"123","key":"k","echo":"ok"}"#;
        let mut setting = serde_json::from_str::<WebSocketMessage>(setting_str)?;
        let s = setting.get_mut_setting()?;
        s.set_replay("hello")?;
        println!("{}", setting);
        Ok(())
    }
}

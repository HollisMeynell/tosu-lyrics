pub mod lyric;
pub mod setting;

use crate::error::Result;

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize, Serializer};

use lyric::*;
use setting::*;

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

impl WebSocketMessage {
    fn get_mut_setting(&mut self) -> Result<&mut SettingPayload> {
        match self {
            WebSocketMessage::Lyric(_) => { Err("type `lyric` can not get setting".into()) }
            WebSocketMessage::Setting(setting) => { Ok(setting) }
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
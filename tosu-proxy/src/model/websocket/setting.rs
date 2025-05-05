use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

/// 表示包含设置信息的消息负载。
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingPayload {
    pub key: String,
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

impl SettingPayload {
    pub fn set_replay<T: Serialize>(&mut self, value: T) -> crate::error::Result<()> {
        self.value = Some(serde_json::to_value(value)?);
        Ok(())
    }
}

use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
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
    pub fn set_replay<T: Serialize>(&mut self, value: T) -> Result<&mut Self> {
        self.value = Some(serde_json::to_value(value)?);
        Ok(self)
    }

    pub fn set_echo(&mut self, echo: &str) -> Result<&mut Self> {
        self.echo = Some(echo.to_string());
        Ok(self)
    }

    pub fn get_value_clone<T: DeserializeOwned>(&self) -> Result<T> {
        if let Some(value) = &self.value {
            Self::translate_json(value.clone())
        } else {
            Err(Error::Static("Cannot get value, because it is null"))
        }
    }

    pub fn get_value_borrow<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        let json_value = if let Some(value) = &self.value {
            value
        } else {
            return Err(Error::Static("Cannot get value, because it is null"));
        };
        Ok(T::deserialize(json_value)?)
    }

    pub fn get_value_take<T: DeserializeOwned>(&mut self) -> Result<T> {
        if let Some(value) = self.value.take() {
            Self::translate_json(value)
        } else {
            Err(Error::Static("Cannot get value, because it is null"))
        }

    }

    fn translate_json<T: DeserializeOwned>(json_value:Value) -> Result<T> {
        Ok(serde_json::from_value(json_value)?)
    }
}

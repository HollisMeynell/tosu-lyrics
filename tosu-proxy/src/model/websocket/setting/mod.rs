pub mod base;
pub mod block;
pub mod song_info;

use crate::error::{Error, Result};
use crate::model::JsonStruct;
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
    const NULL_ERROR_TIPS: &'static str = "Cannot get value, because it is null";

    pub fn new(key: String) -> Self {
        Self {
            key,
            value: None,
            error: None,
            echo: None,
        }
    }

    pub fn set_replay<T: Serialize>(&mut self, value: T) -> Result<&mut Self> {
        self.value = Some(value.to_value()?);
        Ok(self)
    }
    pub fn set_replay_json_string(&mut self, json: &str) -> Result<&mut Self> {
        let value: Value = serde_json::from_str(json)?;
        self.value = Some(value);
        Ok(self)
    }

    pub fn set_echo(&mut self, echo: &str) -> Result<&mut Self> {
        self.echo = Some(echo.to_string());
        Ok(self)
    }

    pub fn get_value_json_string(&self) -> Result<String> {
        if let Some(value) = &self.value {
            Ok(serde_json::to_string(value)?)
        } else {
            Err(Self::NULL_ERROR_TIPS.into())
        }
    }

    pub fn get_value_clone<T: DeserializeOwned>(&self) -> Result<T> {
        if let Some(value) = &self.value {
            Self::translate_json(value.clone())
        } else {
            Err(Self::NULL_ERROR_TIPS.into())
        }
    }

    pub fn get_value<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        if let Some(value) = &self.value {
            Ok(T::deserialize(value)?)
        } else {
            Err(Self::NULL_ERROR_TIPS.into())
        }
    }

    pub fn get_value_take<T: DeserializeOwned>(&mut self) -> Result<T> {
        if let Some(value) = self.value.take() {
            Self::translate_json(value)
        } else {
            Err(Self::NULL_ERROR_TIPS.into())
        }
    }

    fn translate_json<T: DeserializeOwned>(json_value: Value) -> Result<T> {
        Ok(serde_json::from_value(json_value)?)
    }
}

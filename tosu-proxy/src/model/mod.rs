use serde::Serialize;
use serde_json::Value;

pub mod setting;
pub mod tosu_types;
pub mod websocket;

pub trait JsonStruct {
    fn to_json_string(&self) -> crate::error::Result<String>;
    fn to_value(&self) -> crate::error::Result<Value>;
}

impl<T: Serialize> JsonStruct for T {
    fn to_json_string(&self) -> crate::error::Result<String> {
        let result = serde_json::to_string(self)?;
        Ok(result)
    }

    fn to_value(&self) -> crate::error::Result<Value> {
        let result = serde_json::to_value(self)?;
        Ok(result)
    }
}

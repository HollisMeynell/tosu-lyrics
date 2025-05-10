use crate::database::SettingEntity;
use lyric_macro::gen_setting;
use serde::{Deserialize, Serialize};

// 成员的类型要满足 Serialize + Default + Clone
#[gen_setting]
pub struct LyricSetting {
    pub align: String,
    pub show_second: bool,
}

/// 宏生成, 以 align 为例
/// ```
/// enum  LyricSettingType {
///     Align(String),
///     //...
/// }
///
/// impl LyricSetting {
///     fn get(&self, key:&str) -> LyricSettingType {
///         // 实际值
///         LyricSettingType::Align(String::default())
///     }
/// }
///
/// ```

#[cfg(test)]
mod test {
    use super::LyricSettingType;
    use crate::error::Result;
    use crate::model::websocket::setting::SettingPayload;

    #[tokio::test]
    async fn test_serialize() -> Result<()> {
        let v = LyricSettingType::Align(String::from("center"));
        let json = serde_json::to_string(&v)?;
        println!("{json}");
        let json = serde_json::from_str::<LyricSettingType>(&json)?;
        println!("{json:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_replay() -> Result<()> {
        let v = LyricSettingType::Align(String::from("left"));
        let mut payload = SettingPayload {
            key: "align".to_string(),
            value: None,
            error: None,
            echo: None,
        };
        payload.set_replay(v)?;
        let json = serde_json::to_string(&payload)?;
        println!("{json}");
        Ok(())
    }
}

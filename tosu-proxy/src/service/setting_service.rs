use crate::error::Result;
use crate::model::shared::setting::{
    ALL_SETTING_KEYS, LyricSettings, LyricSettingsPatch, SettingKey,
};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::server::ALL_SESSIONS;
use crate::setting::global_setting;
use tracing::debug;

pub async fn current_settings() -> LyricSettings {
    global_setting().await.read().await.clone()
}

pub async fn patch_settings(patch: LyricSettingsPatch) -> Result<(LyricSettings, Vec<SettingKey>)> {
    let mut guard = global_setting().await.write().await;
    let before = guard.clone();
    let next = before.with_patch(patch)?;

    if next == before {
        return Ok((next, Vec::new()));
    }

    next.save().await?;

    *guard = next.clone();
    drop(guard);

    let changed = before.changed_keys(&next);
    broadcast_settings(&next, &changed).await;
    Ok((next, changed))
}

pub async fn broadcast_settings(settings: &LyricSettings, keys: &[SettingKey]) {
    for key in keys {
        let mut payload = SettingPayload::new(key.ws_key().to_string());
        payload.value = Some(settings.key_value(*key));
        debug!("广播设置 {}", key.ws_key());
        ALL_SESSIONS
            .send_to_all_client(WebSocketMessage::Setting(payload).into())
            .await;
    }
}

pub async fn send_settings_snapshot(session_key: &str) {
    let settings = current_settings().await;
    for key in ALL_SETTING_KEYS {
        let mut payload = SettingPayload::new(key.ws_key().to_string());
        payload.value = Some(settings.key_value(key));
        ALL_SESSIONS
            .send_message(&session_key, WebSocketMessage::Setting(payload).into())
            .await;
    }
}

#[cfg(test)]
mod test {
    use crate::model::shared::setting::{LyricSettings, LyricSettingsPatch, SettingKey};

    #[test]
    fn changed_keys_map_to_frontend_ws_keys() {
        let before = LyricSettings::default();
        let patch: LyricSettingsPatch = serde_json::from_str(
            r##"{"textColor":{"first":"#ff0000","second":"#00ff00"},"secondShow":false}"##,
        )
        .expect("patch 解析");
        let after = before.with_patch(patch).expect("patch 应用");

        let changed = after.changed_keys(&before);
        assert_eq!(changed, vec![SettingKey::TextColor, SettingKey::SecondShow]);
        assert_eq!(changed[0].ws_key(), "setColor");
        assert_eq!(changed[1].ws_key(), "setSecondShow");
        assert_eq!(
            after.key_value(SettingKey::TextColor),
            serde_json::json!({"first":"#ff0000","second":"#00ff00"})
        );
        assert_eq!(
            after.key_value(SettingKey::SecondShow),
            serde_json::json!(false)
        );
    }

    #[test]
    fn identical_patch_reports_no_change() {
        let before = LyricSettings::default();
        let patch: LyricSettingsPatch =
            serde_json::from_str(r#"{"alignment":"center"}"#).expect("patch 解析");
        let after = before.with_patch(patch).expect("patch 应用");
        assert!(after.changed_keys(&before).is_empty());
    }

    #[test]
    fn alignment_broadcasts_as_pair() {
        let settings = LyricSettings::default();
        let value = settings.key_value(SettingKey::Alignment);
        assert_eq!(value["first"], "center");
        assert_eq!(value["second"], "center");
    }
}

pub fn keys_of(before: &LyricSettings, after: &LyricSettings) -> Vec<SettingKey> {
    after.changed_keys(before)
}

pub fn key_value_of(settings: &LyricSettings, key: SettingKey) -> serde_json::Value {
    settings.key_value(key)
}

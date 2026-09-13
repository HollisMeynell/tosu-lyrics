//! 展示设置的业务层：校验 → 落库 → 更新内存 → 广播。
//!
//! HTTP（`/api/settings`）与旧 WS 管理命令都走这里，保证只有一份状态、一套 key，
//! 并且**写入失败时绝不广播**。

use crate::error::Result;
use crate::model::setting::{ALL_SETTING_KEYS, LyricSettings, LyricSettingsPatch, SettingKey};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::server::ALL_SESSIONS;
use crate::setting::global_setting;
use tracing::debug;

/// 当前生效的设置快照
pub async fn current_settings() -> LyricSettings {
    global_setting().await.read().await.clone()
}

/// 局部更新设置。
///
/// 返回 `(最终设置, 本次变化的设置项)`。失败时数据库与内存**都没有被改动**。
pub async fn patch_settings(patch: LyricSettingsPatch) -> Result<(LyricSettings, Vec<SettingKey>)> {
    let mut guard = global_setting().await.write().await;
    let before = guard.clone();
    let next = before.with_patch(patch)?;

    if next == before {
        return Ok((next, Vec::new()));
    }

    // 先落库；写失败直接返回错误，不更新内存、不广播
    next.save().await?;

    *guard = next.clone();
    drop(guard);

    let changed = before.changed_keys(&next);
    broadcast_settings(&next, &changed).await;
    Ok((next, changed))
}

/// 把指定设置项广播给所有展示端
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

/// 展示端接入时下发**完整**设置快照（不只是变化项）
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
    use crate::model::setting::{LyricSettings, LyricSettingsPatch, SettingKey};

    /// 不依赖数据库: 直接验证「变化字段」的判定与 WS key 映射
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
        // 广播值形状必须与前端 handleSettingBroadcast 的期望一致
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

    /// 旧 WS 命令可能只给一个标量，广播出去仍然要是 `{first, second}`
    #[test]
    fn alignment_broadcasts_as_pair() {
        let settings = LyricSettings::default();
        let value = settings.key_value(SettingKey::Alignment);
        assert_eq!(value["first"], "center");
        assert_eq!(value["second"], "center");
    }
}


// ---------------- 供"单独客户端调整"复用的小助手 ----------------

/// 相对 `before` 发生变化的设置项（转发到 `LyricSettings::changed_keys`）
pub fn keys_of(before: &LyricSettings, after: &LyricSettings) -> Vec<SettingKey> {
    after.changed_keys(before)
}

/// 某一设置项在当前设置里的完整取值。
///
/// 单独客户端调整时推送的是**完整取值**而不是差量 —— 展示端拿到就能直接套用，
/// 不需要自己合并，也就不会出现"半边更新"。
pub fn key_value_of(settings: &LyricSettings, key: SettingKey) -> serde_json::Value {
    settings.key_value(key)
}

use crate::database::SettingEntity;
use crate::error::{Error, Result};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::osu_source::OsuState;
use crate::server::ALL_SESSIONS;
use crate::service::LYRIC_SERVICE;
use std::fmt::Display;

#[derive(Debug)]
enum WebsocketResult {
    None,
    Return(SettingPayload),
    Broadcast(SettingPayload),
}

pub async fn on_setting(session_key: &str, msg: &str) {
    let message: WebSocketMessage = match serde_json::from_str(msg) {
        Ok(m) => m,
        Err(e) => {
            let err = format!("failed to deserialize setting: {e}");
            ALL_SESSIONS
                .send_message(&session_key, WebSocketMessage::new_error(err).into())
                .await;
            return;
        }
    };
    let message = match message {
        WebSocketMessage::Lyric(_) => {
            // lyric 类型的消息直接忽略
            return;
        }
        WebSocketMessage::Setting(setting) => setting,
    };
    match handle_setting(message).await {
        WebsocketResult::None => {}
        WebsocketResult::Return(message) => {
            ALL_SESSIONS
                .send_message(&session_key, WebSocketMessage::Setting(message).into())
                .await;
        }
        WebsocketResult::Broadcast(message) => {
            ALL_SESSIONS
                .send_to_all_client(WebSocketMessage::Setting(message).into())
                .await;
        }
    }
}

async fn handle_setting(mut setting: SettingPayload) -> WebsocketResult {
    let key = setting.key.clone();
    let echo = setting.echo.take();
    println!("echo is {echo:?}");

    let result = match key.as_str() {
        "setClear" => set_clear(setting).await,
        "setFont" => set_fount(setting).await,
        "getFont" => get_fount(setting).await,
        "setFontSize" => set_fount_size(setting).await,
        "getFontSize" => get_fount_size(setting).await,
        "setAlignment" => set_alignment(setting).await,
        "getAlignment" => get_alignment(setting).await,
        "setColor" => set_color(setting).await,
        "getColor" => get_color(setting).await,
        "setTranslationMain" => set_translation_main(setting).await,
        "getTranslationMain" => get_translation_main(setting).await,
        "setSecondShow" => set_second_show(setting).await,
        "getSecondShow" => get_second_show(setting).await,
        "getLyricList" => get_second_show(setting).await,
        _ => {
            let err = format!("unknown key: {}", key);
            setting.error.replace(err);
            return WebsocketResult::Return(setting);
        }
    };
    let mut result = match result {
        Ok(result) => result,
        Err(e) => {
            let mut payload = SettingPayload::new(key);
            payload.error.replace(e.to_string());
            WebsocketResult::Return(payload)
        }
    };

    if let Some(echo) = echo {
        if let WebsocketResult::Return(payload) = &mut result {
            payload.echo.replace(echo);
        }
    }

    result
}

async fn set_clear(_: SettingPayload) -> Result<WebsocketResult> {
    super::on_osu_state_change(OsuState::Clean).await;
    Ok(WebsocketResult::None)
}

macro_rules! base_setter {
    ($name:ident, $e:expr) => {
        async fn $name(setting: SettingPayload) -> Result<WebsocketResult> {
            let fount_set = setting.get_value_json_string()?;
            SettingEntity::save_config($e.to_string(), fount_set).await;
            let result = WebsocketResult::Broadcast(setting);
            Ok(result)
        }
    };
}

macro_rules! base_getter {
    ($name:ident, $e:expr) => {
        async fn $name(mut setting: SettingPayload) -> Result<WebsocketResult> {
            let json = SettingEntity::get_config($e.get_key()).await;
            let json = json.ok_or(Error::Static("cannot get setting"))?;
            setting.set_replay_json_string(&json)?;
            let result = WebsocketResult::Return(setting);
            Ok(result)
        }
    };
}

base_setter!(set_fount, LyricSettingDatabaseKey::Font);
base_getter!(get_fount, LyricSettingDatabaseKey::Font);
base_setter!(set_fount_size, LyricSettingDatabaseKey::FontSize);
base_getter!(get_fount_size, LyricSettingDatabaseKey::FontSize);
base_setter!(set_alignment, LyricSettingDatabaseKey::Alignment);
base_getter!(get_alignment, LyricSettingDatabaseKey::Alignment);
base_setter!(set_color, LyricSettingDatabaseKey::Color);
base_getter!(get_color, LyricSettingDatabaseKey::Color);
base_setter!(
    set_translation_main,
    LyricSettingDatabaseKey::TranslationMain
);
base_getter!(
    get_translation_main,
    LyricSettingDatabaseKey::TranslationMain
);
base_setter!(set_second_show, LyricSettingDatabaseKey::SecondShow);
base_getter!(get_second_show, LyricSettingDatabaseKey::SecondShow);

enum LyricSettingDatabaseKey {
    Font,
    FontSize,
    Alignment,
    Color,
    TranslationMain,
    SecondShow,
}

impl Display for LyricSettingDatabaseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_key())?;
        Ok(())
    }
}

impl LyricSettingDatabaseKey {
    fn get_key(&self) -> &'static str {
        match self {
            LyricSettingDatabaseKey::Font => "font",
            LyricSettingDatabaseKey::FontSize => "font-size",
            LyricSettingDatabaseKey::Alignment => "alignment",
            LyricSettingDatabaseKey::Color => "color",
            LyricSettingDatabaseKey::TranslationMain => "translation-main",
            LyricSettingDatabaseKey::SecondShow => "second-show",
        }
    }
}

async fn get_lyric_list(mut setting: SettingPayload) -> Result<WebsocketResult> {
    let lyric_service = LYRIC_SERVICE.lock().await;
    let data = lyric_service.get_search_result().await;
    setting.set_replay(data)?;
    Ok(WebsocketResult::Return(setting))
}

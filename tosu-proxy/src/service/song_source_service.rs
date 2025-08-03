use super::LYRIC_SERVICE;
use crate::error::Result;
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::osu_source::{OsuSongInfo, OsuSource, OsuState};
use crate::server::ALL_SESSIONS;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::task::{AbortHandle, JoinHandle};
use tracing::{error, info};

static BEFORE_HANDLE: LazyLock<Mutex<Option<AbortHandle>>> = LazyLock::new(|| Mutex::new(None));

pub async fn init_song_service() -> Result<()> {
    use crate::config::GLOBAL_CONFIG;
    if let Some(tosu_config) = &GLOBAL_CONFIG.tosu {
        info!("use tosu: {}", tosu_config.url);
        let tosu = crate::osu_source::TosuWebsocketClient::new(&tosu_config.url);
        tosu.start().await;
    }
    info!("osu 数据源初始化完成");
    Ok(())
}

pub async fn on_osu_state_change(state: OsuState) {
    match state {
        OsuState::Time(time) => on_time_update(time).await,
        OsuState::Song(song) => on_song_update(song).await,
        OsuState::Clean => on_clean().await,
    }
}

async fn on_time_update(time: i32) {
    let mut lyric_service = LYRIC_SERVICE.lock().await;
    if let Err(e) = lyric_service.time_next(time).await {
        error!("time update error: {}", e);
    }
}

async fn on_song_update(song: OsuSongInfo) {
    if song.sid < 0 && song.artist == "nekodex" {
        // 忽略掉首页音乐
        return;
    }
    let mut handle = BEFORE_HANDLE.lock().await;
    if let Some(handle) = handle.take() {
        handle.abort()
    }

    let task = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let mut lyric_service = LYRIC_SERVICE.lock().await;
        if let Err(e) = lyric_service.song_change(song).await {
            error!("song update error: {}", e);
        }
    });

    handle.replace(task.abort_handle());
}

async fn on_clean() {
    let clean_message = SettingPayload::new("setClear".to_string());
    ALL_SESSIONS
        .send_to_all_client(Into::<WebSocketMessage>::into(clean_message).into())
        .await;
}

use super::LYRIC_SERVICE;
use crate::error::Result;
use crate::osu_source::{OsuSource, OsuState};
use tracing::{error, info};

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
    LYRIC_SERVICE.call(move |svc| Box::pin(async move {
        match state {
            OsuState::Time(time) => if let Err(err) = svc.time_next(time).await {
                error!("time update error: {err}");
            },
            OsuState::Song(song) => svc.observe_song(song).await,
            OsuState::Clean => svc.song_clean().await,
        }
    })).await;
}

mod tosu;

use crate::error::Result;
use crate::lyric::LyricService;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub static LYRIC_SERVICE: OnceLock<Mutex<LyricService>> = OnceLock::new();

fn get_lyric_service() -> &'static Mutex<LyricService> {
    LYRIC_SERVICE.get().expect("can not get LyricService")
}

#[derive(Debug)]
struct SongInfo<'i> {
    bid: i64,
    sid: i64,
    /// 毫秒
    length: i32,
    /// 毫秒
    now: i64,

    artist: &'i str,
    artist_unicode: &'i str,
    title: &'i str,
    title_unicode: &'i str,
}

enum OsuState<'i> {
    Time(i64),
    Song(SongInfo<'i>),
    Clean,
}

impl<'i> OsuState<'i> {
    async fn send(self) -> Result<()> {
        match self {
            OsuState::Time(t) => {
                Self::on_time_update(t).await;
            }
            OsuState::Song(song) => {
                Self::on_song_update(song).await;
            }
            OsuState::Clean => {}
        }
        Ok(())
    }

    async fn on_time_update(time: i64) {
        debug!("time: {time}");
    }
    async fn on_song_update(song: SongInfo<'i>) {
        debug!("song: {song:?}");
    }
}

trait OsuSource {
    async fn start(self);
    async fn on_osu_state_change<'i>(&self, state: OsuState<'i>) {
        if let Err(e) = state.send().await {
            error!("send osu state error: {}", e);
        }
    }
}

pub async fn init_osu_source() -> Result<()> {
    use crate::config::GLOBAL_CONFIG;
    if let Some(tosu_config) = &GLOBAL_CONFIG.tosu {
        info!("use tosu: {}", tosu_config.url);
        let tosu = tosu::TosuWebsocketClient::new(&tosu_config.url);
        tosu.start().await;
    }
    Ok(())
}

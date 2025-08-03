mod lyric_service;
mod song_source_service;
mod websocket_service;

use crate::error::Result;

pub use lyric_service::*;
pub use song_source_service::on_osu_state_change;

pub async fn init_service() -> Result<()> {
    song_source_service::init_song_service().await?;
    Ok(())
}

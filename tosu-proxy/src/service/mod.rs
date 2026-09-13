pub mod block_service;
pub mod cache_service;
pub mod font_service;
pub mod lyric_content_service;
mod lyric_service;
mod setting_service;
mod song_source_service;

use crate::error::Result;

pub use lyric_service::*;
pub use setting_service::{
    key_value_of, keys_of,
    broadcast_settings, current_settings, patch_settings, send_settings_snapshot,
};
pub use song_source_service::on_osu_state_change;

pub async fn init_service() -> Result<()> {
    song_source_service::init_song_service().await?;
    Ok(())
}

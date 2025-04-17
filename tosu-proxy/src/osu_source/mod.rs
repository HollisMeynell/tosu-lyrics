mod tosu;

use crate::error::Result;
use crate::lyric::LyricService;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tracing::info;

pub static LYRIC_SERVICE: OnceLock<Mutex<LyricService>> = OnceLock::new();

fn get_lyric_service() -> &'static Mutex<LyricService> {
    LYRIC_SERVICE.get().expect("can not get LyricService")
}

pub async fn init_osu_source() -> Result<()> {
    use crate::config::GLOBAL_CONFIG;
    if let Some(tosu_config) = &GLOBAL_CONFIG.tosu {
        tosu::init_tosu_client(tosu_config).await?;
    }
    info!("use tosu: {}", GLOBAL_CONFIG.tosu.is_some());
    Ok(())
}

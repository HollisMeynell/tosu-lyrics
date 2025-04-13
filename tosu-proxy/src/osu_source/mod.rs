use super::error::Result;

use crate::lyric::LyricService;
use std::sync::OnceLock;
use tokio::sync::Mutex;

pub static LYRIC_SERVICE: OnceLock<Mutex<LyricService>> = OnceLock::new();

fn get_lyric_service() -> &'static Mutex<LyricService> {
    LYRIC_SERVICE.get().expect("")
}

pub async fn init_osu_source() -> Result<()> {
    use crate::config::GLOBAL_CONFIG;
    println!("use tosu: {}", GLOBAL_CONFIG.tosu.is_some());
    Ok(())
}

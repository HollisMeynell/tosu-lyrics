use super::model::setting::LyricSetting;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::info;

pub static GLOBAL_SETTINGS: OnceLock<RwLock<LyricSetting>> = OnceLock::new();

//加载需要数据库, 务必在数据库初始化完毕后再调用
pub async fn init_setting() {
    let lyric_setting = LyricSetting::init().await;
    match GLOBAL_SETTINGS.set(RwLock::new(lyric_setting)) {
        Err(_) => panic!("无法初始化歌词配置"),
        _ => {}
    }
    info!("初始化配置完成");
}

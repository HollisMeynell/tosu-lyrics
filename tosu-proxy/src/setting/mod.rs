use super::model::shared::setting::LyricSettings;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::info;

pub static GLOBAL_SETTINGS: OnceLock<RwLock<LyricSettings>> = OnceLock::new();

pub async fn global_setting() -> &'static RwLock<LyricSettings> {
    GLOBAL_SETTINGS.get().expect("cannot get global config")
}

pub async fn init_setting() {
    let lyric_setting = LyricSettings::load().await;
    if let Err(err) = lyric_setting.validate() {
        tracing::error!("数据库中的设置非法({err}), 使用默认设置");
        if GLOBAL_SETTINGS
            .set(RwLock::new(LyricSettings::default()))
            .is_err()
        {
            panic!("无法初始化歌词配置, 请尝试删除数据库文件 (.db)")
        }
        return;
    }
    if GLOBAL_SETTINGS.set(RwLock::new(lyric_setting)).is_err() {
        panic!("无法初始化歌词配置, 请尝试删除数据库文件 (.db)")
    }
    info!("初始化配置完成");
}

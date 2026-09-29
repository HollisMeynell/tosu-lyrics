use super::model::setting::LyricSettings;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::info;

/// 进程内唯一的设置状态。WS 管理命令与 HTTP 接口都读写这一份。
pub static GLOBAL_SETTINGS: OnceLock<RwLock<LyricSettings>> = OnceLock::new();

pub async fn global_setting() -> &'static RwLock<LyricSettings> {
    GLOBAL_SETTINGS.get().expect("cannot get global config")
}

/// 加载需要数据库, 务必在数据库初始化完毕后再调用
pub async fn init_setting() {
    let lyric_setting = LyricSettings::load().await;
    if let Err(err) = lyric_setting.validate() {
        tracing::error!("数据库中的设置非法({err}), 使用默认设置");
        if GLOBAL_SETTINGS.set(RwLock::new(LyricSettings::default())).is_err() {
            panic!("无法初始化歌词配置, 请尝试删除数据库文件 (.db)")
        }
        return;
    }
    if GLOBAL_SETTINGS.set(RwLock::new(lyric_setting)).is_err() {
        panic!("无法初始化歌词配置, 请尝试删除数据库文件 (.db)")
    }
    info!("初始化配置完成");
}

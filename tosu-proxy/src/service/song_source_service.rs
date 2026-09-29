use super::LYRIC_SERVICE;
use crate::error::Result;
use crate::osu_source::{OsuSongInfo, OsuSource, OsuState};
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
        return;
    }
    let mut handle = BEFORE_HANDLE.lock().await;
    if let Some(handle) = handle.take() {
        handle.abort()
    }

    // 立即清空上一首的展示状态并广播清屏。
    // 搜索被防抖延迟了 100ms, 若不在这里清空, 这段时间内新歌的时间事件
    // 会继续驱动上一首的歌词, 推出错误的下标。
    {
        let mut lyric_service = LYRIC_SERVICE.lock().await;
        lyric_service.clear_state();
        // 换代必须发生在**观察到的这一刻**，而不是 100ms 防抖之后。
        // 否则在防抖窗口内完成的旧搜索仍会拿旧代际通过校验，
        // 把上一首的结果写进新歌状态。
        lyric_service.invalidate_async();
        super::LyricService::broadcast_clear().await;
    }

    let task = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        // song_change 自己管理加锁粒度（联网阶段不持锁），这里不再预先持锁
        if let Err(e) = super::LyricService::song_change(song).await {
            error!("song update error: {}", e);
        }
    });

    handle.replace(task.abort_handle());
}

async fn on_clean() {
    // 先取消在途的换歌任务。
    // 否则"搜索/下载还在跑 -> 回到菜单"时, 那个任务稍后会拿到锁、
    // 把 now_lyric 重新写回去并推一帧, 让已经清空的歌词在菜单里复活。
    if let Some(handle) = BEFORE_HANDLE.lock().await.take() {
        handle.abort();
    }

    let mut lyric_service = LYRIC_SERVICE.lock().await;
    lyric_service.song_clean().await;
}

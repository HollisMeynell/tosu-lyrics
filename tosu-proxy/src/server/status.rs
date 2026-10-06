use crate::config::{
    CONFIG_ENDPOINT_DISPLAY, CONFIG_ENDPOINT_DISPLAY_CLEAR, CONFIG_ENDPOINT_STATUS,
};
use crate::model::http::lyrics::LyricLineDto;
use crate::model::http::status::{CurrentSongResponse, StatusLyric, StatusResponse, StatusSong};
use crate::server::response::render_error;
use crate::service::LYRIC_SERVICE;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::json;

#[handler]
async fn get_status(res: &mut Response) {
    let service = crate::service::lyric_service().await;
    let song = service.get_now_song();
    let blocked = match &song {
        Some(song) => crate::service::block_service::blocked_rule(
            song.bid as i32,
            song.sid as i32,
            &song.title_unicode,
        )
        .await
        .ok()
        .flatten()
        .is_some(),
        None => false,
    };

    let line_count = service
        .get_now_all_lyrics()
        .map(|lines| lines.len())
        .unwrap_or(0);
    let lyric = match service.current_frame() {
        Some(frame) => StatusLyric::loaded(line_count, frame.current, frame.next_time),
        None => StatusLyric::empty(line_count),
    };

    res.render(Json(StatusResponse {
        song: service.get_now_song().map(|s| StatusSong {
            bid: s.bid,
            sid: s.sid,
            title: s.title_unicode.clone(),
            artist: s.artist_unicode.clone(),
            length: s.length,
        }),
        lyric,
        offset: service.get_offset(),
        // 当前歌是否被黑名单命中。与"没有歌词"是两回事。
        blocked,
    }));
}

#[handler]
async fn get_current_lyric(res: &mut Response) {
    let service = crate::service::lyric_service().await;
    let Some(song) = service.get_now_song() else {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            "no_song",
            "当前没有播放中的歌曲",
        );
        return;
    };
    let Some(lines) = service.get_now_all_lyrics() else {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            "no_lyric",
            "当前歌曲没有可用歌词(可能正在搜索或被清屏)",
        );
        return;
    };
    let frame = service.current_frame();
    res.render(Json(CurrentSongResponse {
        song: StatusSong {
            bid: song.bid,
            sid: song.sid,
            title: song.title_unicode.clone(),
            artist: song.artist_unicode.clone(),
            length: song.length,
        },
        offset: service.get_offset(),
        current: frame.map(|f| f.current).unwrap_or(-1),
        next_time: frame.map(|f| f.next_time).unwrap_or(-1),
        lyric: lines.iter().map(LyricLineDto::from).collect(),
    }));
}

/// 持续清屏，直到切歌 / 换源 / 上传歌词才恢复。保留当前歌曲信息。
#[handler]
async fn clear_display(res: &mut Response) {
    LYRIC_SERVICE
        .call(|service| Box::pin(async move { service.clear_display().await }))
        .await;
    res.render(Json(json!({ "ok": true })));
}

pub fn get_status_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_STATUS).get(get_status)
}

pub fn get_display_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_DISPLAY)
        .push(Router::with_path(CONFIG_ENDPOINT_DISPLAY_CLEAR).post(clear_display))
}

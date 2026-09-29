use crate::config::{
    CONFIG_ENDPOINT_DISPLAY, CONFIG_ENDPOINT_DISPLAY_CLEAR, CONFIG_ENDPOINT_LYRICS,
    CONFIG_ENDPOINT_LYRICS_CURRENT, CONFIG_ENDPOINT_STATUS,
};
use crate::lyric::LyricLine;
use crate::osu_source::OsuSongInfo;
use crate::service::LYRIC_SERVICE;
use crate::server::response::render_error;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::{Map, Value, json};

fn song_json(song: &OsuSongInfo) -> Value {
    json!({
        "bid": song.bid,
        "sid": song.sid,
        "title": song.title_unicode,
        "artist": song.artist_unicode,
        // 毫秒; -1 表示未能读取到音频时长
        "length": song.length,
    })
}

fn lyric_lines_json(lines: &[LyricLine]) -> Vec<Value> {
    lines
        .iter()
        .map(|line| {
            let mut item = Map::new();
            // 毫秒(与展示 WS 的时间单位统一)
            item.insert(
                "time".to_string(),
                json!((line.time * 1000.0).round() as i64),
            );
            if let Some(origin) = &line.origin {
                item.insert("origin".to_string(), json!(origin));
            }
            if let Some(translation) = &line.translation {
                item.insert("translation".to_string(), json!(translation));
            }
            Value::Object(item)
        })
        .collect()
}

#[handler]
async fn get_status(res: &mut Response) {
    // 黑名单**现查**，不使用任何缓存字段：
    // 规则可能在歌曲播放途中被增删，缓存会让状态在那一刻变得不真实。
    // 先取身份再放锁，避免在持锁期间去查数据库。
    let song = {
        let service = LYRIC_SERVICE.lock().await;
        service.get_now_song().cloned()
    };
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

    let service = LYRIC_SERVICE.lock().await;
    let line_count = service
        .get_now_all_lyrics()
        .map(|lines| lines.len())
        .unwrap_or(0);
    let lyric_state = match service.current_frame() {
        Some(frame) => json!({
            "loaded": true,
            "cleared": false,
            "lineCount": line_count,
            "current": frame.current,
            // 距离下一行开始的剩余毫秒; 末行为 -1
            "nextTime": frame.next_time,
        }),
        None => json!({
            "loaded": false,
            "cleared": true,
            "lineCount": line_count,
            "current": -1,
            "nextTime": -1,
        }),
    };

    res.render(Json(json!({
        "song": service.get_now_song().map(song_json),
        "lyric": lyric_state,
        "offset": service.get_offset(),
        // 当前歌是否被黑名单命中。与"没有歌词"是两回事：
        // 被屏蔽是用户显式决定，找不到歌词是源的问题，前端需要能区分。
        "blocked": blocked,
    })));
}

#[handler]
async fn get_current_lyric(res: &mut Response) {
    let service = LYRIC_SERVICE.lock().await;
    let Some(song) = service.get_now_song() else {
        render_error(res, StatusCode::NOT_FOUND, "no_song", "当前没有播放中的歌曲");
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
    let body = json!({
        "song": song_json(song),
        "offset": service.get_offset(),
        "current": frame.map(|f| f.current).unwrap_or(-1),
        "nextTime": frame.map(|f| f.next_time).unwrap_or(-1),
        "lyric": lyric_lines_json(lines),
    });
    res.render(Json(body));
}

/// 持续清屏，直到切歌 / 换源 / 上传歌词才恢复。保留当前歌曲信息。
#[handler]
async fn clear_display(res: &mut Response) {
    let mut service = LYRIC_SERVICE.lock().await;
    service.clear_display().await;
    res.render(Json(json!({ "ok": true })));
}

pub fn get_status_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_STATUS).get(get_status)
}

pub fn get_display_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_DISPLAY)
        .push(Router::with_path(CONFIG_ENDPOINT_DISPLAY_CLEAR).post(clear_display))
}


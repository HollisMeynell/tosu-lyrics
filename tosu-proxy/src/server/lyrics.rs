//! 歌词内容管理 HTTP 接口（B-05）。
//!
//! - `GET    /api/lyrics/current`        当前歌词 + 身份 + 偏移 + 绑定
//! - `GET    /api/lyrics/search-results` 已有候选
//! - `POST   /api/lyrics/search`         主动搜索（可带条件，缺省用当前歌）
//! - `GET    /api/lyrics/preview`        预览某候选（**不改播放**）
//! - `PUT    /api/lyrics/source`         应用来源绑定
//! - `DELETE /api/lyrics/source`         恢复自动匹配
//! - `PUT    /api/lyrics/offset`         设置偏移
//!
//! 陈旧请求统一返回 409 `song_changed`：切歌后旧操作被拒绝，而不是悄悄写坏状态。

use crate::config::{
    CONFIG_ENDPOINT_LYRICS, CONFIG_ENDPOINT_LYRICS_CURRENT, CONFIG_ENDPOINT_LYRICS_OFFSET,
    CONFIG_ENDPOINT_LYRICS_PREVIEW, CONFIG_ENDPOINT_LYRICS_SEARCH,
    CONFIG_ENDPOINT_LYRICS_SEARCH_RESULTS, CONFIG_ENDPOINT_LYRICS_SOURCE,
    CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK, CONFIG_ENDPOINT_LYRICS_UPLOAD,
};
use crate::error::Error;
use crate::server::response::{
    CODE_INVALID_PARAM, CODE_NO_LYRIC, CODE_NO_SONG, CODE_SONG_CHANGED, CODE_SOURCE_FAILED,
    render_error, render_service_error,
};
use crate::service::lyric_content_service as content;
use crate::service::STALE_REQUEST;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};

/// 把服务层的错误映射到统一的错误码与状态码。
///
/// 服务层用 `来源标记:` 前缀携带分类（`no_song:` / `source_failed:` / ...），
/// 这里集中翻译一次；错误体本身只暴露**去掉前缀后的**可读信息。
fn render_content_error(res: &mut Response, err: Error) {
    let message = err.to_string();
    let (status, code, text) = if let Some(rest) = message.strip_prefix("no_song:") {
        (StatusCode::CONFLICT, CODE_NO_SONG, rest)
    } else if let Some(rest) = message.strip_prefix("source_failed:") {
        (StatusCode::BAD_GATEWAY, CODE_SOURCE_FAILED, rest)
    } else if let Some(rest) = message.strip_prefix("lyric_parse:") {
        (StatusCode::UNPROCESSABLE_ENTITY, CODE_NO_LYRIC, rest)
    } else if message.contains(STALE_REQUEST) {
        (
            StatusCode::CONFLICT,
            CODE_SONG_CHANGED,
            "操作发起后歌曲已切换，结果已丢弃",
        )
    } else {
        render_service_error(res, err);
        return;
    };
    render_error(res, status, code, text);
}

async fn parse_body<T: serde::de::DeserializeOwned>(req: &mut Request) -> Result<T, String> {
    let payload = req
        .payload()
        .await
        .map_err(|e| format!("读取请求体失败: {e}"))?;
    let text = std::str::from_utf8(payload).map_err(|e| format!("请求体不是合法 UTF-8: {e}"))?;
    if text.trim().is_empty() {
        return Err("请求体为空".to_string());
    }
    crate::util::to_json::<T>(text).map_err(|e: Error| format!("请求体格式错误: {e}"))
}

// ---------------------------------------------------------------- 当前歌词

/// `GET /api/lyrics/current`
///
/// **契约变更（B-05）**：旧实现把"有歌但暂时没歌词"也当成 404 `no_lyric`。
/// 但那是**正常且瞬时的状态**（正在搜索 / 已清屏 / 被拉黑 / 没有这个源），
/// 不是错误；而且恰恰在这种状态下 Controller 最需要读 `blocked` 与 `source`。
/// 现在改为：
/// - 没有播放中的歌        -> 404 `no_song`
/// - 有歌但没有可用歌词    -> 200，`lyric: null`，并带 `lyricState` 说明原因
/// - 有歌且有歌词          -> 200，`lyric: [...]`
#[handler]
async fn get_current(res: &mut Response) {
    // 先取快照再放锁，避免持锁期间做数据库查询
    let snapshot = {
        let service = crate::service::LYRIC_SERVICE.lock().await;
        let Some(song) = service.get_now_song() else {
            render_error(res, StatusCode::NOT_FOUND, CODE_NO_SONG, "当前没有播放中的歌曲");
            return;
        };
        let lines = service.get_now_all_lyrics().map(|l| {
            l.iter()
                .map(|line| {
                    let mut item = serde_json::Map::new();
                    item.insert("time".into(), json!((line.time * 1000.0).round() as i64));
                    if let Some(o) = &line.origin {
                        item.insert("origin".into(), json!(o));
                    }
                    if let Some(t) = &line.translation {
                        item.insert("translation".into(), json!(t));
                    }
                    Value::Object(item)
                })
                .collect::<Vec<Value>>()
        });
        let frame = service.current_frame();
        json!({
            "song": {
                "bid": song.bid,
                "sid": song.sid,
                "title": song.title_unicode,
                "artist": song.artist_unicode,
                "length": song.length,
            },
            "offset": service.get_offset(),
            "current": frame.map(|f| f.current).unwrap_or(-1),
            "nextTime": frame.map(|f| f.next_time).unwrap_or(-1),
            "lyric": lines,
        })
    };

    // 绑定与屏蔽单独查（都不需要服务锁）
    let binding = content::binding().await;
    let context = content::current_context_summary().await;
    let mut body = snapshot;
    if let Some(obj) = body.as_object_mut() {
        obj.insert("source".into(), binding.unwrap_or(Value::Null));
        // 机器可读的"为什么没有歌词"，避免前端靠 lyric==null 猜
        obj.insert("lyricState".into(), json!(context.state));
        obj.insert("blocked".into(), json!(context.blocked));
    }
    res.render(Json(body));
}

// ---------------------------------------------------------------- 候选 / 搜索

#[handler]
async fn get_search_results(res: &mut Response) {
    let items = content::candidates().await;
    res.render(Json(json!({ "total": items.len(), "items": items })));
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchBody {
    title: Option<String>,
    artist: Option<String>,
}

#[handler]
async fn post_search(req: &mut Request, res: &mut Response) {
    // body 可以为空：表示"按当前播放歌曲搜索"
    let body = parse_body::<SearchBody>(req).await.unwrap_or(SearchBody {
        title: None,
        artist: None,
    });
    match content::search(body.title, body.artist).await {
        Ok(items) => res.render(Json(json!({ "total": items.len(), "items": items }))),
        Err(err) => render_content_error(res, err),
    }
}

// ---------------------------------------------------------------- 预览

#[derive(Debug, Deserialize)]
struct PreviewQuery {
    source: String,
    key: String,
}

#[handler]
async fn get_preview(req: &mut Request, res: &mut Response) {
    let Ok(query) = req.parse_queries::<PreviewQuery>() else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "必须同时提供 source 与 key",
        );
        return;
    };
    match content::preview(&query.source, &query.key).await {
        // 预览只回内容，不触碰当前播放状态
        Ok(preview) => res.render(Json(preview)),
        Err(err) => render_content_error(res, err),
    }
}

// ---------------------------------------------------------------- 来源绑定

#[derive(Debug, Deserialize)]
struct SourceBody {
    source: String,
    key: String,
}

#[handler]
async fn put_source(req: &mut Request, res: &mut Response) {
    let body = match parse_body::<SourceBody>(req).await {
        Ok(body) => body,
        Err(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message);
            return;
        }
    };
    if body.source.trim().is_empty() || body.key.trim().is_empty() {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "source 与 key 都不能为空",
        );
        return;
    }
    match content::apply_source(&body.source, &body.key).await {
        Ok(rule) => res.render(Json(json!({ "ok": true, "source": rule }))),
        Err(err) => render_content_error(res, err),
    }
}

#[handler]
async fn delete_source(res: &mut Response) {
    match content::clear_source().await {
        Ok(removed) => res.render(Json(json!({ "ok": true, "removed": removed }))),
        Err(err) => render_content_error(res, err),
    }
}

// ---------------------------------------------------------------- 偏移

#[derive(Debug, Deserialize)]
struct OffsetBody {
    /// 毫秒
    offset: i32,
}

#[handler]
async fn put_offset(req: &mut Request, res: &mut Response) {
    let body = match parse_body::<OffsetBody>(req).await {
        Ok(body) => body,
        Err(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message);
            return;
        }
    };
    // 合理范围：±30 秒。超出基本可以确定是误输入
    if body.offset.abs() > 30_000 {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "offset 必须在 ±30000 毫秒以内",
        );
        return;
    }
    let effective = content::set_offset(body.offset).await;
    res.render(Json(json!({ "ok": true, "offset": effective })));
}

/// `POST /api/lyrics/translation-check`
///
/// 批量询问"这些候选有没有翻译"。内容来自真实取词；
/// 单次请求的条数有上限，避免前端一次把上游打满。
#[handler]
async fn post_translation_check(req: &mut Request, res: &mut Response) {
    #[derive(serde::Deserialize)]
    struct Body {
        items: Vec<Item>,
    }
    #[derive(serde::Deserialize)]
    struct Item {
        source: String,
        key: String,
    }

    let body = match parse_body::<Body>(req).await {
        Ok(body) => body,
        Err(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message);
            return;
        }
    };
    if body.items.len() > 60 {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "一次最多检查 60 个候选",
        );
        return;
    }

    let input: Vec<(String, String)> = body
        .items
        .into_iter()
        .map(|i| (i.source, i.key))
        .collect();
    let results = content::check_translations(input).await;
    let out: Vec<Value> = results
        .into_iter()
        .map(|(source, key, has_translation)| {
            json!({ "source": source, "key": key, "hasTranslation": has_translation })
        })
        .collect();
    res.render(Json(json!({ "items": out })));
}

pub fn get_lyrics_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_LYRICS)
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_CURRENT).get(get_current))
        .push(
            Router::with_path(CONFIG_ENDPOINT_LYRICS_SEARCH_RESULTS).get(get_search_results),
        )
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_SEARCH).post(post_search))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_PREVIEW).get(get_preview))
        .push(
            Router::with_path(CONFIG_ENDPOINT_LYRICS_SOURCE)
                .put(put_source)
                .delete(delete_source),
        )
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_OFFSET).put(put_offset))
        // LRC 上传与其它内容接口同一前缀，避免出现 /lyric 与 /lyrics 两套
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_UPLOAD).post(crate::server::lyric::upload_lyric))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK).post(post_translation_check))
}

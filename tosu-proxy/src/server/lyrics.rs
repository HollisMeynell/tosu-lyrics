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

/// 没有播放中的歌 -> 404 no_song；有歌但没歌词 -> 200 + lyric:null + lyricState；
/// 有歌且有歌词 -> 200 + lyric:[...]
#[handler]
async fn get_current(res: &mut Response) {
    // 先取快照再放锁，避免持锁期间做数据库查询
    let snapshot = {
        let service = crate::service::lyric_service().await;
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

#[derive(Debug, Deserialize)]
struct OffsetBody {
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
    // 偏移不设上限：整首歌长度级别的偏移也允许（例如 +60000 / -60000 毫秒），
    // 由使用者自己决定，服务端只做原样保存与生效。
    let effective = content::set_offset(body.offset).await;
    res.render(Json(json!({ "ok": true, "offset": effective })));
}

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
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_UPLOAD).post(crate::server::lyric::upload_lyric))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK).post(post_translation_check))
}

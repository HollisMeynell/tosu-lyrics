use crate::config::{
    CONFIG_ENDPOINT_LYRICS, CONFIG_ENDPOINT_LYRICS_CURRENT, CONFIG_ENDPOINT_LYRICS_OFFSET,
    CONFIG_ENDPOINT_LYRICS_PREVIEW, CONFIG_ENDPOINT_LYRICS_SEARCH,
    CONFIG_ENDPOINT_LYRICS_SEARCH_RESULTS, CONFIG_ENDPOINT_LYRICS_SOURCE,
    CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK, CONFIG_ENDPOINT_LYRICS_UPLOAD,
};
use crate::error::Error;
use crate::model::http::lyrics::{
    CurrentLyricResponse, DeleteSourceResponse, LyricLineDto, OffsetResponse, PreviewResponse,
    PutSourceResponse, SearchResultsResponse, SongDto, TranslationCheckItem,
    TranslationCheckResponse,
};
use crate::server::response::{
    CODE_INVALID_PARAM, CODE_NO_LYRIC, CODE_NO_SONG, CODE_SONG_CHANGED, CODE_SOURCE_FAILED,
    render_error, render_service_error,
};
use crate::service::STALE_REQUEST;
use crate::service::lyric_content_service as content;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde::Deserialize;

/// 把服务层的错误映射到统一的错误码与状态码。
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

#[handler]
async fn get_current(res: &mut Response) {
    let snapshot = {
        let service = crate::service::lyric_service().await;
        let Some(song) = service.get_now_song() else {
            render_error(
                res,
                StatusCode::NOT_FOUND,
                CODE_NO_SONG,
                "当前没有播放中的歌曲",
            );
            return;
        };
        let lines = service
            .get_now_all_lyrics()
            .map(|l| l.iter().map(LyricLineDto::from).collect::<Vec<_>>());
        let frame = service.current_frame();
        CurrentLyricResponse {
            song: SongDto::from(song),
            offset: service.get_offset(),
            current: frame.map(|f| f.current).unwrap_or(-1),
            next_time: frame.map(|f| f.next_time).unwrap_or(-1),
            lyric: lines.unwrap_or_default(),
            source: None,
            lyric_state: "ok",
            blocked: false,
        }
    };

    let binding = content::binding().await;
    let context = content::current_context_summary().await;
    let mut body = snapshot;
    body.source = binding;
    body.lyric_state = context.state;
    body.blocked = context.blocked;
    res.render(Json(body));
}

#[handler]
async fn get_search_results(res: &mut Response) {
    let items = content::candidates().await;
    res.render(Json(SearchResultsResponse {
        total: items.len(),
        items,
    }));
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchBody {
    title: Option<String>,
    artist: Option<String>,
}

#[handler]
async fn post_search(req: &mut Request, res: &mut Response) {
    let body = parse_body::<SearchBody>(req).await.unwrap_or(SearchBody {
        title: None,
        artist: None,
    });
    match content::search(body.title, body.artist).await {
        Ok(items) => res.render(Json(SearchResultsResponse {
            total: items.len(),
            items,
        })),
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
        Ok(preview) => res.render(Json(PreviewResponse {
            source: preview.source,
            key: preview.key,
            line_count: preview.line_count,
            lines: preview.lines,
        })),
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
        Ok(rule) => res.render(Json(PutSourceResponse {
            ok: true,
            source: rule,
        })),
        Err(err) => render_content_error(res, err),
    }
}

#[handler]
async fn delete_source(res: &mut Response) {
    match content::clear_source().await {
        Ok(removed) => res.render(Json(DeleteSourceResponse { ok: true, removed })),
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
    let effective = content::set_offset(body.offset).await;
    res.render(Json(OffsetResponse {
        ok: true,
        offset: effective,
    }));
}

#[handler]
async fn post_translation_check(req: &mut Request, res: &mut Response) {
    #[derive(Deserialize)]
    struct Body {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
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

    let input: Vec<(String, String)> = body.items.into_iter().map(|i| (i.source, i.key)).collect();
    let results = content::check_translations(input).await;
    res.render(Json(TranslationCheckResponse {
        items: results
            .into_iter()
            .map(|(source, key, has_translation)| TranslationCheckItem {
                source,
                key,
                has_translation,
            })
            .collect(),
    }));
}

pub fn get_lyrics_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_LYRICS)
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_CURRENT).get(get_current))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_SEARCH_RESULTS).get(get_search_results))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_SEARCH).post(post_search))
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_PREVIEW).get(get_preview))
        .push(
            Router::with_path(CONFIG_ENDPOINT_LYRICS_SOURCE)
                .put(put_source)
                .delete(delete_source),
        )
        .push(Router::with_path(CONFIG_ENDPOINT_LYRICS_OFFSET).put(put_offset))
        .push(
            Router::with_path(CONFIG_ENDPOINT_LYRICS_UPLOAD)
                .post(crate::server::lyric::upload_lyric),
        )
        .push(
            Router::with_path(CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK)
                .post(post_translation_check),
        )
}

use crate::config::{
    CONFIG_ENDPOINT_CACHE, CONFIG_ENDPOINT_CACHE_CLEANUP, CONFIG_ENDPOINT_CACHE_COUNT,
    CONFIG_ENDPOINT_CACHE_CURRENT,
};
use crate::model::http::cache::{CleanupResponse, CountResponse, RemovedResponse};
use crate::server::response::{CODE_INVALID_PARAM, render_error, render_service_error};
use crate::service::cache_service;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PageQuery {
    page: Option<u64>,
    size: Option<u64>,
    q: Option<String>,
}

#[handler]
async fn list_cache(req: &mut Request, res: &mut Response) {
    let query = req.parse_queries::<PageQuery>().unwrap_or(PageQuery {
        page: None,
        size: None,
        q: None,
    });
    match cache_service::page(
        query.q.as_deref(),
        query.page.unwrap_or(1),
        query.size.unwrap_or(20),
    )
    .await
    {
        Ok(page) => res.render(Json(page)),
        Err(err) => render_service_error(res, err),
    }
}

#[handler]
async fn count_cache(res: &mut Response) {
    match cache_service::count().await {
        Ok(total) => res.render(Json(CountResponse { total })),
        Err(err) => render_service_error(res, err),
    }
}

#[handler]
async fn delete_cache(req: &mut Request, res: &mut Response) {
    let Some(bid) = req.param::<i32>("bid") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "路径参数 bid 必须是整数",
        );
        return;
    };
    match cache_service::delete(bid).await {
        Ok(removed) => res.render(Json(RemovedResponse {
            removed: if removed { 1 } else { 0 },
        })),
        Err(err) => render_service_error(res, err),
    }
}

#[derive(Debug, Deserialize)]
struct DeleteQuery {
    title: Option<String>,
}

#[handler]
async fn clear_cache(req: &mut Request, res: &mut Response) {
    let query = req
        .parse_queries::<DeleteQuery>()
        .unwrap_or(DeleteQuery { title: None });
    let result = match query.title.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(title) => cache_service::delete_by_title(title).await,
        None => cache_service::clear().await,
    };
    match result {
        Ok(removed) => res.render(Json(RemovedResponse { removed })),
        Err(err) => render_service_error(res, err),
    }
}

#[handler]
async fn cleanup_cache(res: &mut Response) {
    match cache_service::purge_expired().await {
        Ok(removed) => res.render(Json(CleanupResponse {
            removed,
            ttl_ms: cache_service::ttl_ms(),
        })),
        Err(err) => render_service_error(res, err),
    }
}

/// 删除**当前播放歌曲**的歌词缓存；身份由服务端解析，调用方不传 bid/sid。
#[handler]
async fn delete_current_cache(res: &mut Response) {
    let snapshot = crate::service::lyric_service().await;
    let Some(song) = snapshot.get_now_song() else {
        res.render(Json(RemovedResponse { removed: 0 }));
        return;
    };
    let bid = song.bid as i32;
    let sid = song.sid as i32;
    match cache_service::delete_for_song(bid, sid).await {
        Ok(removed) => res.render(Json(RemovedResponse { removed })),
        Err(err) => render_service_error(res, err),
    }
}

pub fn get_cache_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_CACHE)
        .get(list_cache)
        .delete(clear_cache)
        .push(Router::with_path(CONFIG_ENDPOINT_CACHE_COUNT).get(count_cache))
        .push(Router::with_path(CONFIG_ENDPOINT_CACHE_CLEANUP).post(cleanup_cache))
        .push(Router::with_path(CONFIG_ENDPOINT_CACHE_CURRENT).delete(delete_current_cache))
        .push(Router::with_path("{bid}").delete(delete_cache))
}

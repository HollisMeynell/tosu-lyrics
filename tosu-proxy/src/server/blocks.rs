//! 黑名单 HTTP 管理接口（B-04）。
//!
//! - `GET    /api/blocks`        列出全部规则
//! - `POST   /api/blocks`        新增（幂等：同一 scope+value 重复添加只更新标题）
//! - `PATCH  /api/blocks/{id}`   修改展示用标题
//! - `DELETE /api/blocks/{id}`   删除单条（不存在返回 404 `not_found`）
//! - `DELETE /api/blocks`        清空全部（幂等，返回删除条数）
//!
//! 后端是唯一真相来源：这些接口写完状态后，副作用（清屏 / 恢复展示）由
//! `block_service` 统一施加，Controller 不需要自己再推一遍。

use crate::config::CONFIG_ENDPOINT_BLOCKS;
use crate::error::Error;
use crate::server::response::{
    CODE_INVALID_PARAM, CODE_NOT_FOUND, render_error, render_service_error,
};
use crate::service::block_service::{
    self, BlockError, BlockRuleInput, BlockRulePatch,
};
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::json;

fn render_block_error(res: &mut Response, err: BlockError) {
    match err {
        BlockError::InvalidParam(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message)
        }
        BlockError::NotFound(message) => {
            render_error(res, StatusCode::NOT_FOUND, CODE_NOT_FOUND, message)
        }
        BlockError::Internal(err) => render_service_error(res, err),
    }
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
async fn list_blocks(res: &mut Response) {
    match block_service::list().await {
        Ok(rules) => res.render(Json(json!({
            "total": rules.len(),
            "items": rules,
        }))),
        Err(err) => render_block_error(res, err),
    }
}

#[handler]
async fn add_block(req: &mut Request, res: &mut Response) {
    let input = match parse_body::<BlockRuleInput>(req).await {
        Ok(input) => input,
        Err(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message);
            return;
        }
    };
    match block_service::add(input).await {
        Ok(rule) => res.render(Json(rule)),
        Err(err) => render_block_error(res, err),
    }
}

#[handler]
async fn update_block(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<i32>("id") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "路径参数 id 必须是整数",
        );
        return;
    };
    // PATCH 允许空 body，视为"不改任何字段"
    let patch = match parse_body::<BlockRulePatch>(req).await {
        Ok(patch) => patch,
        Err(_) => BlockRulePatch {
            title: None,
            reason: None,
        },
    };
    match block_service::update(id, patch).await {
        Ok(rule) => res.render(Json(rule)),
        Err(err) => render_block_error(res, err),
    }
}

#[handler]
async fn delete_block(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<i32>("id") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "路径参数 id 必须是整数",
        );
        return;
    };
    match block_service::delete(id).await {
        Ok(Some(rule)) => res.render(Json(json!({ "removed": true, "rule": rule }))),
        Ok(None) => render_error(
            res,
            StatusCode::NOT_FOUND,
            CODE_NOT_FOUND,
            format!("黑名单规则 {id} 不存在"),
        ),
        Err(err) => render_block_error(res, err),
    }
}

#[handler]
async fn clear_blocks(res: &mut Response) {
    match block_service::clear_all().await {
        Ok(removed) => res.render(Json(json!({ "removed": removed }))),
        Err(err) => render_block_error(res, err),
    }
}

pub fn get_blocks_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_BLOCKS)
        .get(list_blocks)
        .post(add_block)
        .delete(clear_blocks)
        .push(
            Router::with_path("{id}")
                .patch(update_block)
                .delete(delete_block),
        )
}

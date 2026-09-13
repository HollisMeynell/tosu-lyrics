//! 展示设置 HTTP 接口（B-03）。
//!
//! - `GET   /api/settings` 读取完整设置（含默认值）
//! - `PATCH /api/settings` 局部更新；校验 + 落库成功后才广播，返回最终服务端状态

use crate::config::CONFIG_ENDPOINT_SETTINGS;
use crate::error::Error;
use crate::model::setting::LyricSettingsPatch;
use crate::server::response::{CODE_INVALID_PARAM, render_error, render_service_error};
use crate::service::{current_settings, patch_settings as apply_settings_patch};
use salvo::http::StatusCode;
use salvo::prelude::*;

#[handler]
async fn get_settings(res: &mut Response) {
    res.render(Json(current_settings().await));
}

#[handler]
async fn patch_settings(req: &mut Request, res: &mut Response) {
    let patch = match parse_patch(req).await {
        Ok(patch) => patch,
        Err(message) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, message);
            return;
        }
    };

    match apply_settings_patch(patch).await {
        // mutation 返回最终服务端状态，前端直接用它覆盖本地状态
        Ok((settings, _changed)) => res.render(Json(settings)),
        Err(err) => render_service_error(res, err),
    }
}

async fn parse_patch(req: &mut Request) -> Result<LyricSettingsPatch, String> {
    let payload = req
        .payload()
        .await
        .map_err(|e| format!("读取请求体失败: {e}"))?;
    let text = std::str::from_utf8(payload).map_err(|e| format!("请求体不是合法 UTF-8: {e}"))?;
    if text.trim().is_empty() {
        return Err("请求体为空".to_string());
    }
    crate::util::to_json::<LyricSettingsPatch>(text).map_err(|e: Error| format!("请求体格式错误: {e}"))
}

pub fn get_settings_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_SETTINGS)
        .get(get_settings)
        .patch(patch_settings)
}

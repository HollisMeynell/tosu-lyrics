use super::MAX_UPLOAD_BODY;
use crate::config::{CONFIG_ENDPOINT_FONT, CONFIG_ENDPOINT_FONT_LIST, CONFIG_ENDPOINT_FONT_UPLOAD};
use crate::error::Error;
use crate::model::http::font::{FontListResponse, UploadFontResponse};
use crate::server::response::{CODE_INVALID_PARAM, render_error, render_service_error};
use crate::service::font_service;
use salvo::http::StatusCode;
use salvo::http::header::CONTENT_TYPE;
use salvo::prelude::*;

fn font_content_type(bytes: &[u8]) -> &'static str {
    match bytes.get(..4) {
        Some(b"wOFF") => "font/woff",
        Some(b"wOF2") => "font/woff2",
        Some(b"OTTO") => "font/otf",
        _ => "font/ttf",
    }
}

/// GET /api/font/list — 返回所有可用字体（font/ 目录 + 内嵌默认字体）
#[handler]
async fn font_list(res: &mut Response) {
    let items = font_service::list_fonts();
    res.render(Json(FontListResponse { items }));
}

/// GET /api/font/:name — 按名称下载字体
#[handler]
async fn download_font(req: &mut Request, res: &mut Response) {
    let Some(name) = req.param::<String>("name") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "缺少字体名称参数",
        );
        return;
    };
    let Some(bytes) = font_service::load(&name).await else {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            "not_found",
            format!("字体 {name} 不存在"),
        );
        return;
    };

    let has_version = req.queries().contains_key("v");
    let header = res.headers_mut();
    let _ = header.insert(CONTENT_TYPE, font_content_type(&bytes).parse().unwrap());
    let cache_control = if has_version {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let _ = header.insert("cache-control", cache_control.parse().unwrap());
    res.status_code(StatusCode::OK);
    res.body(bytes);
}

/// POST /api/font/upload — 上传字体到字体库
#[handler]
async fn upload_font(req: &mut Request, res: &mut Response) {
    let form = match req.form_data_max_size(MAX_UPLOAD_BODY).await {
        Ok(form) => form,
        Err(err) => {
            render_error(
                res,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!(
                    "解析上传内容失败（上限 {} MB）: {err}",
                    MAX_UPLOAD_BODY / 1024 / 1024
                ),
            );
            return;
        }
    };
    let Some(file) = form.files.get("file") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "未收到字体文件（表单字段名应为 file）",
        );
        return;
    };
    let uploaded_path = file.path().to_path_buf();

    let bytes = match tokio::fs::read(&uploaded_path).await {
        Ok(b) => b,
        Err(err) => {
            render_error(
                res,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!("读取上传文件失败: {err}"),
            );
            return;
        }
    };

    match font_service::save(&bytes).await {
        Ok(font) => res.render(Json(UploadFontResponse { ok: true, font })),
        Err(Error::Runtime(message)) => {
            let (code, text) = match message.split_once(':') {
                Some(("invalid_param", rest)) => (CODE_INVALID_PARAM, rest),
                _ => ("internal", message.as_str()),
            };
            render_error(
                res,
                if code == CODE_INVALID_PARAM {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                },
                code,
                text,
            )
        }
        Err(err) => render_service_error(res, err),
    }
}

pub fn get_font_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_FONT)
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_LIST).get(font_list))
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_UPLOAD).post(upload_font))
        .push(Router::with_path("{name}").get(download_font))
}

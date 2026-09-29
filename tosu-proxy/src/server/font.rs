use crate::config::{
    CONFIG_ENDPOINT_FONT, CONFIG_ENDPOINT_FONT_DOWNLOAD, CONFIG_ENDPOINT_FONT_UPLOAD,
};
use crate::error::Error;
use crate::server::response::{CODE_INVALID_PARAM, render_error, render_service_error};
use crate::service::font_service::{self, FontKind};
use salvo::http::header::CONTENT_TYPE;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::json;

fn font_content_type(bytes: &[u8]) -> &'static str {
    match bytes.get(..4) {
        Some(b"wOFF") => "font/woff",
        Some(b"wOF2") => "font/woff2",
        Some(b"OTTO") => "font/otf",
        _ => "font/ttf",
    }
}

#[handler]
async fn font_info(res: &mut Response) {
    let items = font_service::all_info();
    res.render(Json(json!({ "items": items })));
}

#[handler]
async fn download_font(req: &mut Request, res: &mut Response) {
    let Some(kind) = req.param::<String>("kind").and_then(|k| FontKind::parse(&k).ok()) else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "字体种类只能是 main / sub",
        );
        return;
    };
    let Some(bytes) = font_service::load(kind).await else {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            "not_found",
            format!("{} 字体尚未上传", kind.as_str()),
        );
        return;
    };

    let version = font_service::version_of(&kind.path()).0;
    let has_version = req.queries().contains_key("v");
    // 注意：头必须插到 **Response** 上。旧实现插在 Request 上再交给
    // NamedFile::send，现在直接用 res.body()，插错对象等于没设。
    let header = res.headers_mut();
    let _ = header.insert(CONTENT_TYPE, font_content_type(&bytes).parse().unwrap());
    // 带版本参数时允许长缓存：版本变化会让 URL 变化，浏览器自然重新拉取。
    // 不带版本参数则禁止缓存，避免"覆盖后仍用旧字体"。
    let cache_control = if has_version {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let _ = header.insert("cache-control", cache_control.parse().unwrap());
    let _ = header.insert("etag", format!("\"{version}\"").parse().unwrap());

    res.status_code(StatusCode::OK);
    res.body(bytes);
}

#[handler]
async fn upload_font(req: &mut Request, res: &mut Response) {
    // 兼容旧入口 /api/font/upload：没有 kind 参数（或不是 main/sub）时按主字体处理
    let kind = req
        .param::<String>("kind")
        .and_then(|k| FontKind::parse(&k).ok())
        .unwrap_or(FontKind::Main);

    let Some(file) = req.first_file().await else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "未收到字体文件",
        );
        return;
    };
    let bytes = match tokio::fs::read(file.path()).await {
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

    match font_service::save(kind, &bytes).await {
        Ok(info) => res.render(Json(json!({ "ok": true, "font": info }))),
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

/// 旧下载入口：/api/font/download
#[handler]
async fn legacy_download(req: &mut Request, res: &mut Response) {
    let Some(bytes) = font_service::load(FontKind::Main).await else {
        render_error(res, StatusCode::NOT_FOUND, "not_found", "主字体尚未上传");
        return;
    };
    let header = res.headers_mut();
    let _ = header.insert(CONTENT_TYPE, font_content_type(&bytes).parse().unwrap());
    res.status_code(StatusCode::OK);
    res.body(bytes);
}

pub fn get_font_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_FONT)
        .push(Router::with_path("info").get(font_info))
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_UPLOAD).post(upload_font))
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_DOWNLOAD).get(legacy_download))
        .push(
            Router::with_path("{kind}")
                .get(download_font)
                .post(upload_font),
        )
}

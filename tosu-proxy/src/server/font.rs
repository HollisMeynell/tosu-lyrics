use super::MAX_UPLOAD_BODY;
use crate::config::{
    CONFIG_ENDPOINT_FONT, CONFIG_ENDPOINT_FONT_DOWNLOAD, CONFIG_ENDPOINT_FONT_UPLOAD,
};
use crate::error::Error;
use crate::model::http::font::{FontInfoResponse, UploadFontResponse};
use crate::server::response::{CODE_INVALID_PARAM, render_error, render_service_error};
use crate::service::font_service::{self, FontKind};
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

#[handler]
async fn font_info(res: &mut Response) {
    let items = font_service::all_info();
    res.render(Json(FontInfoResponse { items }));
}

#[handler]
async fn download_font(req: &mut Request, res: &mut Response) {
    let Some(kind) = req
        .param::<String>("kind")
        .and_then(|k| FontKind::parse(&k).ok())
    else {
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

    let version = font_service::version_of(&kind.upload_path()).0;
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
    let kind = req
        .param::<String>("kind")
        .and_then(|k| FontKind::parse(&k).ok())
        .unwrap_or(FontKind::Main);

    // 显式指定本次解析的上限，不依赖 request 级/全局的 secure_max_size：
    // salvo 默认上限只有 64KB，字体动辄 3~10MB，超限会在**响应发出前**中止读取，
    // 浏览器侧只看到 `TypeError: Failed to fetch`，拿不到任何错误体。
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
    // 先把路径取出来，避免继续持有对 req 的不可变借用
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

    match font_service::save(kind, &bytes).await {
        Ok(info) => res.render(Json(UploadFontResponse {
            ok: true,
            font: info,
        })),
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

/// 静态回退路径 `/lyrics/LRC.otf`。
///
/// **为什么需要单独处理**：前端在"还没上传过字体"时会请求这个路径
/// （src/utils/fonts.ts 的 STATIC_MAIN）。它本来落在 `StaticDir` 上，而那里配了
/// `fallback("index.html")` 供 SPA 路由使用 —— 副作用是**任何找不到的资源都会
/// 返回 index.html**。浏览器把这段 HTML 当字体解析，就会报
/// `OTS parsing error: invalid sfntVersion: 1008821359`（首四字节正是 `<!do`），
/// 并伴随 `Failed to decode downloaded font`。
///
/// 这里让字体路径由字体服务直接应答：有字体就返回字体、没有就 404，
/// **绝不会返回 HTML**。它注册在静态路由之前，因此优先生效。
#[handler]
async fn static_main_font(res: &mut Response) {
    serve_static_font(res, FontKind::Main, "主字体").await;
}

/// 静态回退路径 `/lyrics/tLRC.otf`，行为同上。
#[handler]
async fn static_sub_font(res: &mut Response) {
    serve_static_font(res, FontKind::Sub, "副字体").await;
}

/// 按静态资源的查找顺序读取**随包分发的默认字体**，找不到返回 `None`。
///
/// 顺序：`./static/lyrics` → `./static` → **内嵌程序资源**。
///
/// **为什么不再回落到 `./`**：工作目录里的 `LRC.otf` / `tLRC.otf` 在早期版本
/// 正是上传字体的落点（与默认字体同路径）—— 一旦回落到它，"默认字体"加载到的
/// 就是上传字体。上传字体现在写在 `uploaded/`（见 `FontKind::upload_path()`），
/// 而这最后一档取的是**内嵌在程序里的默认字体**，也就是真正"程序自带的默认字体"，
/// 单文件发行下必然存在。
async fn read_static_font(kind: FontKind) -> Option<(Vec<u8>, String)> {
    for dir in ["./static/lyrics", "./static"] {
        let path = std::path::Path::new(dir).join(kind.file_name());
        if let Ok(bytes) = tokio::fs::read(&path).await {
            return Some((bytes, font_service::version_of(&path).0));
        }
    }
    crate::server::embedded_resource(kind.file_name()).map(|bytes| {
        let len = bytes.len();
        (bytes, format!("embedded-{len}"))
    })
}

async fn serve_static_font(res: &mut Response, kind: FontKind, label: &str) {
    // 注意：这里**不再**读 `font_service::load()`。那个路径指向**上传字体**
    // （`uploaded/LRC.otf`）；如果用它，默认字体与上传字体就会是同一份字节，
    // family 拆开也白拆。
    let Some((bytes, version)) = read_static_font(kind).await else {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            "not_found",
            format!(
                "随包默认{label}不存在（应放在 static/{}）",
                kind.file_name()
            ),
        );
        return;
    };
    let header = res.headers_mut();
    let _ = header.insert(CONTENT_TYPE, font_content_type(&bytes).parse().unwrap());
    // 不带版本参数，且默认字体可能被替换，所以禁止缓存，避免浏览器继续用旧内容
    let _ = header.insert("cache-control", "no-cache".parse().unwrap());
    let _ = header.insert("etag", format!("\"{version}\"").parse().unwrap());
    res.status_code(StatusCode::OK);
    res.body(bytes);
}

/// 静态回退字体路由，必须注册在 `get_file_route()` **之前**。
pub fn get_static_font_route() -> Router {
    Router::with_path("lyrics")
        .push(Router::with_path(FontKind::Main.file_name()).get(static_main_font))
        .push(Router::with_path(FontKind::Sub.file_name()).get(static_sub_font))
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

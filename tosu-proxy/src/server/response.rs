use crate::error::Error;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::{Value, json};

pub const CODE_INVALID_PARAM: &str = "invalid_param";
pub const CODE_NO_SONG: &str = "no_song";
pub const CODE_NO_LYRIC: &str = "no_lyric";
pub const CODE_NOT_FOUND: &str = "not_found";
pub const CODE_WRITE_FAILED: &str = "write_failed";
pub const CODE_SONG_CHANGED: &str = "song_changed";
pub const CODE_SOURCE_FAILED: &str = "source_failed";
pub const CODE_INTERNAL: &str = "internal";

pub fn error_body(code: &str, message: impl Into<String>) -> Value {
    json!({ "error": { "code": code, "message": message.into() } })
}

pub fn render_error(
    res: &mut Response,
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) {
    res.status_code(status);
    res.render(Json(error_body(code, message)));
}

/// 按 `Error` 的类型挑选合适的错误码与状态码。
///
/// 目前约定：校验类错误（我们自己抛的 `Runtime`）算参数非法，
/// 数据库 / IO 失败算写入失败（5xx），其余一律 500。
pub fn render_service_error(res: &mut Response, err: Error) {
    let (status, code) = match &err {
        Error::Database(_) | Error::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, CODE_WRITE_FAILED),
        Error::Runtime(_) | Error::Static(_) => (StatusCode::BAD_REQUEST, CODE_INVALID_PARAM),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, CODE_INTERNAL),
    };
    render_error(res, status, code, err.to_string());
}

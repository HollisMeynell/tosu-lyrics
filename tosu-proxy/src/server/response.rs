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

pub fn render_service_error(res: &mut Response, err: Error) {
    let (status, code) = match &err {
        Error::Database(_) | Error::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, CODE_WRITE_FAILED),
        Error::Runtime(_) | Error::Static(_) => (StatusCode::BAD_REQUEST, CODE_INVALID_PARAM),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, CODE_INTERNAL),
    };
    render_error(res, status, code, err.to_string());
}

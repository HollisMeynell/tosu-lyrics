//! 管理 HTTP 的统一响应约定（B-00 的一部分）。
//!
//! 成功: 返回结构化 JSON（mutation 返回**最终服务端状态**）
//! 失败: `{"error":{"code":"...","message":"..."}}` + 有语义的 HTTP 状态码

use crate::error::Error;
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::{Value, json};

/// 错误码：参数非法
pub const CODE_INVALID_PARAM: &str = "invalid_param";
/// 错误码：没有当前歌曲
pub const CODE_NO_SONG: &str = "no_song";
/// 错误码：没有可用歌词
pub const CODE_NO_LYRIC: &str = "no_lyric";
/// 错误码：数据不存在
pub const CODE_NOT_FOUND: &str = "not_found";
/// 错误码：数据库 / 文件写入失败
pub const CODE_WRITE_FAILED: &str = "write_failed";
/// 错误码：请求目标歌曲已经不是当前歌曲（切歌竞态）
pub const CODE_SONG_CHANGED: &str = "song_changed";
/// 错误码：歌词源不存在或下载失败
pub const CODE_SOURCE_FAILED: &str = "source_failed";
/// 错误码：服务端内部错误
pub const CODE_INTERNAL: &str = "internal";

pub fn error_body(code: &str, message: impl Into<String>) -> Value {
    json!({ "error": { "code": code, "message": message.into() } })
}

pub fn render_error(res: &mut Response, status: StatusCode, code: &str, message: impl Into<String>) {
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

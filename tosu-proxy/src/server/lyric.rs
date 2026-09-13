//! LRC 上传（B-07）。
//!
//! - `POST /api/lyrics/upload`  上传 LRC 并绑定到当前播放歌曲
//!
//! 与旧实现的差别：
//! - 统一错误体 `{"error":{"code","message"}}`，不再返回纯文本
//! - 上传前校验编码与格式；**非法文件不替换旧歌词**
//! - 提交前做代际 + 身份校验：上传期间切歌则整份丢弃（409 `song_changed`）
//! - 有效歌词按当前播放进度立即刷新，不回到第 0 行

use crate::config::{CONFIG_ENDPOINT_LYRIC, CONFIG_ENDPOINT_LYRIC_UPLOAD};
use crate::error::Error;
use crate::lyric::Lyric;
use crate::server::response::{
    CODE_INVALID_PARAM, CODE_NO_LYRIC, CODE_NO_SONG, CODE_SONG_CHANGED, render_error,
};
use crate::service::{lyric_service, LyricService, STALE_REQUEST};
use salvo::http::StatusCode;
use salvo::prelude::*;
use serde_json::json;

/// 上传体积上限：1 MiB。歌词文本远超这个量级，超过基本可以确定传错了文件。
const MAX_UPLOAD_BYTES: usize = 1024 * 1024;

/// 取出上传内容：优先取 multipart 文件，其次取原始请求体。
async fn read_payload(req: &mut Request) -> Result<Vec<u8>, (StatusCode, &'static str, String)> {
    if let Some(file) = req.first_file().await {
        return tokio::fs::read(file.path()).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!("读取上传文件失败: {e}"),
            )
        });
    }
    let payload = req.payload().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            format!("读取请求体失败: {e}"),
        )
    })?;
    Ok(payload.to_vec())
}

/// 解码 + 解析。任何一步不合法都返回结构化错误，**不触碰当前歌词**。
fn parse_upload(bytes: &[u8]) -> Result<Lyric, (StatusCode, &'static str, String)> {
    if bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "上传内容为空".into(),
        ));
    }
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            CODE_INVALID_PARAM,
            format!("上传内容超过 {} 字节上限", MAX_UPLOAD_BYTES),
        ));
    }

    // 去 BOM：带 BOM 的 UTF-8 会让第一行的时间标签解析失败
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "歌词文件必须是 UTF-8 编码".to_string(),
        )
    })?;

    let lyric = Lyric::parse(text, None, None).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            CODE_NO_LYRIC,
            format!("歌词解析失败: {e}"),
        )
    })?;

    // 解析成功但一行时间标签都没有 —— 多半不是 LRC，不能拿它覆盖旧歌词
    if lyric.get_lyrics().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            CODE_NO_LYRIC,
            "文件里没有任何带时间标签的歌词行，不是有效的 LRC".into(),
        ));
    }

    Ok(lyric)
}

#[handler]
pub async fn upload_lyric(req: &mut Request, res: &mut Response) {
    // 0) 必须先有正在播放的歌
    let Some(ident) = LyricService::now_ident().await else {
        render_error(
            res,
            StatusCode::CONFLICT,
            CODE_NO_SONG,
            "当前没有播放中的歌曲，无法绑定歌词",
        );
        return;
    };

    // 1) 校验期不持锁：解析是纯 CPU 操作，没必要占着服务锁
    let bytes = match read_payload(req).await {
        Ok(b) => b,
        Err((status, code, message)) => {
            render_error(res, status, code, message);
            return;
        }
    };
    let lyric = match parse_upload(&bytes) {
        Ok(l) => l,
        Err((status, code, message)) => {
            render_error(res, status, code, message);
            return;
        }
    };
    let line_count = lyric.get_lyrics().len();

    // 2) 提交：换到服务锁后再校验一次代际与身份。
    //    上传（尤其是大文件 / 慢网络）期间完全可能切歌，那时整份丢弃。
    let mut service = lyric_service().await;
    match service.apply_uploaded_lyric(ident.sid, lyric).await {
        Ok(()) => res.render(Json(json!({
            "ok": true,
            "lines": line_count,
            "song": { "bid": ident.bid, "sid": ident.sid, "title": ident.title },
        }))),
        Err(err) if err.to_string().contains(STALE_REQUEST) => render_error(
            res,
            StatusCode::CONFLICT,
            CODE_SONG_CHANGED,
            "上传期间歌曲已切换，歌词已丢弃",
        ),
        Err(err) => render_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            err.to_string(),
        ),
    }
}

pub fn get_lyric_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_LYRIC)
        .push(Router::with_path(CONFIG_ENDPOINT_LYRIC_UPLOAD).post(upload_lyric))
}

/// 让 `Error` 在本模块可被构造（保留给后续扩展）
#[allow(dead_code)]
fn runtime_error(message: String) -> Error {
    Error::Runtime(message)
}

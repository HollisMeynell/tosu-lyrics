use crate::config::{CONFIG_ENDPOINT_LYRIC, CONFIG_ENDPOINT_LYRIC_UPLOAD};
use crate::service::LYRIC_SERVICE;
use salvo::http::StatusCode;
use salvo::prelude::*;

/// 上传歌词文件(.lrc / 文本), 取第一个文件, 绑定到当前播放歌曲
#[handler]
async fn upload_lyric(req: &mut Request, res: &mut Response) {
    let Some(file) = req.first_file().await else {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render("未收到歌词文件");
        return;
    };

    let bytes = match std::fs::read(file.path()) {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(format!("读取歌词文件失败: {e}"));
            return;
        }
    };

    let text = match String::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render("歌词文件编码必须是 UTF-8");
            return;
        }
    };

    let mut service = LYRIC_SERVICE.lock().await;
    match service.set_manual_lyric(&text).await {
        Ok(()) => {
            res.render("success");
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(format!("解析歌词失败: {e}"));
        }
    }
}

pub fn get_lyric_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_LYRIC)
        .push(Router::with_path(CONFIG_ENDPOINT_LYRIC_UPLOAD).post(upload_lyric))
}

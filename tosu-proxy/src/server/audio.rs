use crate::config::CONFIG_ENDPOINT_AUDIO_LEN;
use crate::model::http::audio::AudioLengthResponse;
use crate::server::response::{CODE_INTERNAL, CODE_INVALID_PARAM, render_error};
use crate::util::read_audio_length;
use salvo::http::StatusCode;
use salvo::prelude::*;

#[handler]
async fn get_audio_length(req: &mut Request, res: &mut Response) {
    let Some(path) = req.query::<String>("path") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "缺少查询参数 path",
        );
        return;
    };
    match read_audio_length(&path).await {
        Ok(length) => res.render(Json(AudioLengthResponse { length })),
        Err(err) => render_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            CODE_INTERNAL,
            format!("读取音频时长失败: {err}"),
        ),
    }
}

pub fn get_audio_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_AUDIO_LEN).get(get_audio_length)
}

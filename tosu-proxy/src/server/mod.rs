mod audio;
mod blocks;
mod cache;
mod clients;
mod file;
mod font;
mod lyric;
mod lyrics;
pub mod response;
mod settings;
mod status;
mod websocket;

use crate::config::GLOBAL_CONFIG;
use crate::error::*;
use crate::server::audio::get_audio_route;
use crate::server::blocks::get_blocks_route;
use crate::server::cache::get_cache_route;
use crate::server::clients::get_clients_route;
use crate::server::file::get_file_route;
use crate::server::font::get_font_route;
use crate::server::lyric::get_lyric_route;
use crate::server::lyrics::get_lyrics_route as get_content_lyrics_route;
use crate::server::settings::get_settings_route;
use crate::server::status::{get_display_route, get_status_route};
use crate::server::websocket::get_ws_route;
use salvo::prelude::Redirect;
use salvo::server::ServerHandle;
use salvo::{Response, handler};
use std::sync::OnceLock;
use tracing::info;

static SERVER_HANDLE: OnceLock<ServerHandle> = OnceLock::new();
pub use websocket::ALL_SESSIONS;

#[handler]
async fn root_redirect(res: &mut Response) {
    res.render(Redirect::permanent("/lyrics"));
}

/// 上传体积上限：32 MiB。
///
/// **为什么需要显式设置**：salvo 默认的请求体上限远小于字体文件
/// （实测 5 MB 的 LRC.otf 会在 448 KB 处被截断，服务端回 400、
/// 浏览器侧表现为 `TypeError: Failed to fetch`）。
/// 字体动辄几 MB，必须放宽；歌词文本用不到这么大，但多留余量没有副作用。
const MAX_UPLOAD_BODY: usize = 32 * 1024 * 1024;

pub async fn start_server() {
    use salvo::prelude::*;
    salvo::http::request::set_global_secure_max_size(MAX_UPLOAD_BODY);
    let api_router = Router::with_path("api")
        .push(get_font_route())
        .push(get_audio_route())
        .push(get_lyric_route())
        .push(get_status_route())
        .push(get_content_lyrics_route())
        .push(get_display_route())
        .push(get_settings_route())
        .push(get_blocks_route())
        .push(get_cache_route())
        .push(get_clients_route());
    let router = Router::new()
        .get(root_redirect)
        .push(get_ws_route())
        .push(get_file_route())
        .push(api_router);
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    info!("server start: http://127.0.0.1:{}", GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    let server = Server::new(acceptor);
    let handle = server.handle();
    SERVER_HANDLE
        .set(handle)
        .map_err(|_| Error::Impossible)
        .expect("?");
    info!("web 服务器初始化完成");
    server.serve(router).await;
}

pub fn close_server() {
    if let Some(handle) = SERVER_HANDLE.get() {
        handle.stop_graceful(None);
    }
}

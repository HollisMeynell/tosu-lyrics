mod audio;
mod file;
mod font;
mod websocket;

use crate::config::GLOBAL_CONFIG;
use crate::error::*;
use crate::server::audio::get_audio_route;
use crate::server::file::get_file_route;
use crate::server::font::get_font_route;
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

pub async fn start_server() {
    use salvo::prelude::*;
    let api_router = Router::with_path("api")
        .push(get_font_route())
        .push(get_audio_route());
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

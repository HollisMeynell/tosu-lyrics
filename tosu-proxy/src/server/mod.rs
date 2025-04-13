mod file;
mod websocket;

use crate::config::GLOBAL_CONFIG;
use crate::error::*;
use crate::server::file::get_file_route;
use crate::server::websocket::get_ws_route;
use salvo::server::ServerHandle;
use std::sync::OnceLock;
use salvo::{handler, Response};
use salvo::prelude::Redirect;
use tracing::info;

static SERVER_HANDLE: OnceLock<ServerHandle> = OnceLock::new();

#[handler]
async fn root_redirect(res: &mut Response) {
    res.render(Redirect::permanent("/lyrics"));
}

pub async fn start_server() {
    use salvo::prelude::*;
    let router = Router::new()
        .get(root_redirect)
        .push(get_ws_route())
        .push(get_file_route());
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    info!("server start: http://127.0.0.1:{}", GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    let server = Server::new(acceptor);
    let handle = server.handle();
    SERVER_HANDLE
        .set(handle)
        .map_err(|_| Error::Impossible)
        .expect("?");
    server.serve(router).await;
}

pub fn close_server() {
    if let Some(handle) = SERVER_HANDLE.get() {
        handle.stop_graceful(None);
    }
}

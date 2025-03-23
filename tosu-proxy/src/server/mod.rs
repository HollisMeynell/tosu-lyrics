mod websocket;

use crate::config::GLOBAL_CONFIG;
use crate::server::websocket::get_ws_route;

pub async fn start_server() {
    use salvo::prelude::*;
    let router = Router::new().push(get_ws_route());
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    Server::new(acceptor).serve(router).await;
}

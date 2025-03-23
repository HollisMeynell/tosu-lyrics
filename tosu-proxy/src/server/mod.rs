mod websocket;

use crate::config::GLOBAL_CONFIG;

async fn start_server() {
    use salvo::prelude::*;
    let router = Router::new();
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    Server::new(acceptor).serve(router).await;
}

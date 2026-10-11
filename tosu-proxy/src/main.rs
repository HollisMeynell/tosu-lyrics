use tosu_proxy::*;
use tracing::info;

#[tokio::main]
async fn main() -> error::Result<()> {
    let message = server::ensure_runtime_dir();
    config::GLOBAL_CONFIG.init_logger(message).await;
    database::init_database().await;
    setting::init_setting().await;
    service::init_service().await?;
    server::start_server().await;
    database::close().await;
    info!("bye~");
    Ok(())
}

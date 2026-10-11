use tosu_proxy::*;
use tracing::info;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 配置和数据库的相对路径必须以运行目录为基准。
    server::ensure_runtime_dir();
    config::GLOBAL_CONFIG.init_logger();
    database::init_database().await;
    setting::init_setting().await;
    service::init_service().await?;
    server::start_server().await;
    database::close().await;
    info!("bye~");
    Ok(())
}

use std::str::FromStr;
#[cfg(feature = "new")]
use tosu_proxy::*;
use tracing::log::Level;

#[cfg(feature = "new")]
#[tokio::main]
async fn main() -> error::Result<()> {
    use tracing::info;
    config::GLOBAL_CONFIG.init_logger();
    database::init_database().await;
    setting::init_setting().await;
    service::init_service().await?;
    server::start_server().await;
    database::close();
    info!("bye~");
    Ok(())
}

#[cfg(feature = "old")]
fn main() {
    println!("no compile")
}

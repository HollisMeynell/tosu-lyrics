use std::str::FromStr;
#[cfg(feature = "new")]
use tosu_proxy::*;
use tracing::log::Level;

#[cfg(feature = "new")]
async fn init_logger() {
    use tracing::Level;
    let level = if let Some(level) = &config::GLOBAL_CONFIG.log_level {
        Level::from_str(level)
            .expect("无法处理的配置 `log`, 仅支持 error, warn, info, debug, trace")
    } else if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_test_writer()
        .init();
}

#[cfg(feature = "new")]
async fn init_server() {
    server::start_server().await;
}

#[cfg(feature = "new")]
#[tokio::main]
async fn main() -> error::Result<()> {
    use tracing::info;
    init_logger().await;
    database::init_database().await;
    osu_source::init_osu_source().await?;
    init_server().await;
    database::close();
    info!("bye~");
    Ok(())
}

#[cfg(feature = "old")]
fn main() {
    println!("no compile")
}

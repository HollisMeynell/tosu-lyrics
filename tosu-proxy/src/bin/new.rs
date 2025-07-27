use std::str::FromStr;
#[cfg(feature = "new")]
use tosu_proxy::*;
use tracing::log::Level;

#[cfg(feature = "new")]
async fn init_server() {
    server::start_server().await;
}

#[cfg(feature = "new")]
#[tokio::main]
async fn main() -> error::Result<()> {
    use tracing::info;
    config::GLOBAL_CONFIG.init_logger();
    database::init_database().await;
    setting::init_setting().await;
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

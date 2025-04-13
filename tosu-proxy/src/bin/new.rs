#[cfg(feature = "new")]
use tosu_proxy::*;

#[cfg(feature = "new")]
async fn init_logger() {
    use tracing::Level;
    let level = if cfg!(debug_assertions) {
        Level::TRACE
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
    init_logger().await;
    database::init_database().await;
    lyric::init_lyric().await?;
    osu_source::init_osu_source().await?;
    init_server().await;
    database::close();
    println!("bye~");
    Ok(())
}

#[cfg(feature = "old")]
fn main() {
    println!("no compile")
}

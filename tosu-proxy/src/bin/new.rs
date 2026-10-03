use std::str::FromStr;
#[cfg(feature = "new")]
use tosu_proxy::*;
use tracing::log::Level;

#[cfg(feature = "new")]
#[tokio::main]
async fn main() -> error::Result<()> {
    use tracing::info;
    // 单文件发行：**必须最先**把运行目录切到 exe 同目录的 lyrics/，
    // 否则下面 init_logger / init_database（建库 + 迁移）会作用在旧 cwd，
    // 导致 lyrics/lyric.db 成为空库（no such table: lyric_cache 等）且首次启动需重启。
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

#[cfg(feature = "old")]
fn main() {
    println!("no compile")
}

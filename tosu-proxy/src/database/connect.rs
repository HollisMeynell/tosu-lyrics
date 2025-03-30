use crate::error::*;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend};
use std::sync::OnceLock;
use tracing::info;

static DATABASE_CONNECT: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init_database() {
    use crate::config::GLOBAL_CONFIG;
    let db_url = &GLOBAL_CONFIG.database;
    let mut option = ConnectOptions::new(db_url);
    option
        .max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);
    let connect = Database::connect(option)
        .await
        .expect("无法连接数据库, 请检查配置");
    connect.ping().await.expect("数据库检查失败");
    info!("数据库连接完成");
    DATABASE_CONNECT.set(connect).expect("无法初始化数据库");
}

pub fn database() -> &'static DatabaseConnection {
    DATABASE_CONNECT.get().expect("数据库连接池异常")
}

pub fn close() {
    let _ = database().clone().close();
}
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::OnceLock;
use tracing::info;

use super::migration::Migrator;

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

    // 运行 schema 迁移（建表、加列、建索引）
    Migrator::up(database(), None)
        .await
        .expect("数据库迁移失败");
    // 运行纯数据迁移（旧 lyric_config -> lyric_block 黑名单）
    super::entity::migrate_legacy_data()
        .await
        .expect("数据迁移失败");
}

pub fn database() -> &'static DatabaseConnection {
    DATABASE_CONNECT.get().expect("数据库连接池异常")
}

pub async fn close() {
    let _ = database().clone().close().await;
}

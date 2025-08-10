use crate::error::*;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
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
    use super::entity::init_all_table;
    init_all_table().await.expect("can not create all table");
}

pub fn database() -> &'static DatabaseConnection {
    DATABASE_CONNECT.get().expect("数据库连接池异常")
}

pub async  fn close() {
    let _ = database().clone().close().await;
}

pub async fn table_exists<T: AsRef<str>>(table_name: T) -> Result<bool> {
    let table_name = table_name.as_ref();
    let db = database();
    let backend = db.get_database_backend();
    let sql = match backend {
        DbBackend::Sqlite => {
            format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{table_name}'")
        }
        DbBackend::Postgres => {
            format!(
                "SELECT table_name FROM information_schema.tables WHERE table_schema='public' AND table_name='{table_name}'"
            )
        }
        DbBackend::MySql => {
            format!(
                "SELECT table_name FROM information_schema.tables WHERE table_name = '{table_name}'"
            )
        }
    };
    let stmt = Statement::from_sql_and_values(backend, sql, vec![]);
    Ok(db.query_one(stmt).await?.is_some())
}

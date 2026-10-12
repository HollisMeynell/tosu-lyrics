use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("lyric_cache", "updated_at").await? {
            let db = manager.get_connection();
            db.execute_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "ALTER TABLE lyric_cache ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
            ))
            .await?;
            tracing::info!("lyric_cache 已补充 updated_at 列");
        }

        let now_ms = sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis();
        let db = manager.get_connection();
        let backfilled = db
            .execute_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!(
                    "UPDATE lyric_cache SET updated_at = {} WHERE updated_at = 0",
                    now_ms
                ),
            ))
            .await?;
        if backfilled.rows_affected() > 0 {
            tracing::info!(
                "lyric_cache 回填 {} 条历史条目的 updated_at",
                backfilled.rows_affected()
            );
        }

        Ok(())
    }

    // 兼容不支持 DROP COLUMN 的旧 SQLite，保留新增列。
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

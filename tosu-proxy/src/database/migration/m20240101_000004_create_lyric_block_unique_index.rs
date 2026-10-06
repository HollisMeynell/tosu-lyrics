use sea_orm_migration::prelude::*;

/// 为 lyric_block(scope, value) 创建复合唯一索引
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_lyric_block_scope_value ON lyric_block(scope, value)",
        ))
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "DROP INDEX IF EXISTS idx_lyric_block_scope_value",
        ))
        .await?;
        Ok(())
    }
}

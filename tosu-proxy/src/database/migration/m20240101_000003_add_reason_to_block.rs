use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("lyric_block", "reason").await? {
            manager
                .get_connection()
                .execute_raw(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    "ALTER TABLE lyric_block ADD COLUMN reason TEXT NOT NULL DEFAULT ''",
                ))
                .await?;
            tracing::info!("lyric_block 已补充 reason 列");
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        create_table_if_not_exists(
            manager,
            "lyric_block",
            r#"CREATE TABLE IF NOT EXISTS "lyric_block" (
                "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
                "scope" text NOT NULL,
                "value" text NOT NULL,
                "title" text NOT NULL DEFAULT '',
                "sid" integer NOT NULL DEFAULT 0,
                "reason" text NOT NULL DEFAULT '',
                "created_at" integer NOT NULL DEFAULT 0
            )"#,
        )
        .await?;

        create_table_if_not_exists(
            manager,
            "lyric_cache",
            r#"CREATE TABLE IF NOT EXISTS "lyric_cache" (
                "sid" integer NOT NULL,
                "bid" integer NOT NULL PRIMARY KEY,
                "cache" blob NOT NULL,
                "title" text NOT NULL,
                "audio_length" integer NOT NULL,
                "updated_at" integer NOT NULL DEFAULT 0
            )"#,
        )
        .await?;

        create_table_if_not_exists(
            manager,
            "lyric_config",
            r#"CREATE TABLE IF NOT EXISTS "lyric_config" (
                "bid" integer NOT NULL PRIMARY KEY,
                "sid" integer NOT NULL,
                "title" text NOT NULL,
                "disable" boolean NOT NULL,
                "offset" integer NOT NULL
            )"#,
        )
        .await?;

        create_table_if_not_exists(
            manager,
            "config",
            r#"CREATE TABLE IF NOT EXISTS "config" (
                "key" text NOT NULL PRIMARY KEY,
                "setting" text NOT NULL
            )"#,
        )
        .await?;

        create_table_if_not_exists(
            manager,
            "lyric_binding",
            r#"CREATE TABLE IF NOT EXISTS "lyric_binding" (
                "sid" integer NOT NULL PRIMARY KEY,
                "source_type" text NOT NULL,
                "source_key" text NOT NULL,
                "title" text NOT NULL DEFAULT '',
                "artist" text NOT NULL DEFAULT '',
                "bid" integer NOT NULL DEFAULT 0,
                "created_at" integer NOT NULL DEFAULT 0
            )"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for table in [
            "lyric_binding",
            "config",
            "lyric_config",
            "lyric_cache",
            "lyric_block",
        ] {
            db.execute_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!("DROP TABLE IF EXISTS \"{}\"", table),
            ))
            .await?;
        }
        Ok(())
    }
}

async fn create_table_if_not_exists(
    manager: &SchemaManager<'_>,
    table: &str,
    sql: &str,
) -> Result<(), DbErr> {
    if !manager.has_table(table).await? {
        manager
            .get_connection()
            .execute_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                sql.to_string(),
            ))
            .await?;
    }
    Ok(())
}

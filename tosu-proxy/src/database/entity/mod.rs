pub mod lyric_binding;
pub mod lyric_block;
pub mod lyric_cache;
mod lyric_config;
mod setting;

use crate::database::table_exists;
use crate::error::Result;
use sea_orm::{ConnectionTrait, EntityName, Schema};

use super::database;
pub use lyric_binding::Entity as LyricBindingEntity;
pub use lyric_block::{Entity as LyricBlockEntity, SCOPE_BID, SCOPE_SID, SCOPE_TITLE};
pub use lyric_cache::Entity as LyricCacheEntity;
pub use lyric_config::Entity as LyricConfigEntity;
pub use setting::Entity as SettingEntity;

static DB_ERROR_MESSAGE: &str = "无法查询数据库";

macro_rules! init_entity {
    ($($fn_name:ident($entity:ident),)*) => {
        pub(super)async fn init_all_table() -> Result<()> {
            $($fn_name().await?;)*
            Ok(())
        }
        $(
        async fn $fn_name() -> Result<()> {
            let default = $entity::default();
            let table_name = EntityName::table_name(&default);
            if !table_exists(table_name).await? {
                let db = super::database();
                let backend = db.get_database_backend();
                let schema = Schema::new(backend)
                    .create_table_from_entity($entity);
                db.execute(backend.build(&schema)).await?;
            }
            Ok(())
        }
        )*
    };
}
init_entity! {
    init_setting(SettingEntity),
    init_lyric_cache(LyricCacheEntity),
    init_lyric_config(LyricConfigEntity),
    init_lyric_block(LyricBlockEntity),
    init_lyric_binding(LyricBindingEntity),
}

/// 复合唯一索引：同一 `(scope, value)` 只能有一行，是幂等 upsert 的依据。
/// DeriveEntityModel 的 `indexed` 只给单列非唯一索引，这里手工补。
async fn ensure_block_unique_index() -> Result<()> {
    let db = super::database();
    let backend = db.get_database_backend();
    let stmt = backend.build(
        sea_orm::sea_query::Index::create()
            .name("idx_lyric_block_scope_value")
            .table(LyricBlockEntity)
            .col(lyric_block::Column::Scope)
            .col(lyric_block::Column::Value)
            .unique()
            .if_not_exists(),
    );
    db.execute(stmt).await?;
    Ok(())
}

/// 建表 + 索引 + 旧数据迁移。**幂等**，每次启动都跑。
pub(super) async fn init_all_table_and_migrate() -> Result<()> {
    init_all_table().await?;
    ensure_block_unique_index().await?;
    migrate_legacy_block().await?;
    migrate_cache_updated_at().await?;
    migrate_block_reason().await?;
    Ok(())
}

/// B-06：给旧库的 `lyric_cache` 补 `updated_at` 列。
///
/// `init_entity!` 只负责"表不存在就建表"，**不会给已存在的表加列**，
/// 所以这里显式检查并 ALTER。旧行回填为迁移时刻，避免一启动就全部被判过期。
async fn migrate_cache_updated_at() -> Result<()> {
    let db = super::database();
    if !column_exists("lyric_cache", "updated_at").await? {
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE lyric_cache ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0"
                .to_string(),
        ))
        .await?;
        tracing::info!("lyric_cache 已补充 updated_at 列");
    }
    // 回填仍然是 0 的历史行（幂等：只影响 updated_at = 0 的行）
    let backfilled = db
        .execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!(
                "UPDATE lyric_cache SET updated_at = {} WHERE updated_at = 0",
                lyric_cache::now_ms()
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

/// B-10：给旧库的 `lyric_block` 补 `reason` 列（幂等）。
async fn migrate_block_reason() -> Result<()> {
    let db = super::database();
    if !column_exists("lyric_block", "reason").await? {
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE lyric_block ADD COLUMN reason TEXT NOT NULL DEFAULT ''".to_string(),
        ))
        .await?;
        tracing::info!("lyric_block 已补充 reason 列");
    }
    Ok(())
}

/// 表里是否存在某一列（SQLite: PRAGMA table_info）
async fn column_exists(table: &str, column: &str) -> Result<bool> {
    use sea_orm::ConnectionTrait;
    let db = super::database();
    let rows = db
        .query_all(sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("PRAGMA table_info({table})"),
        ))
        .await?;
    Ok(rows.iter().any(|row| {
        row.try_get::<String>("", "name")
            .map(|name| name == column)
            .unwrap_or(false)
    }))
}

/// 把旧 `lyric_config.disable = true` 的行迁成 `bid` 作用域的黑名单规则，
/// 然后清掉旧标志。可重复执行，不会产生重复规则。
async fn migrate_legacy_block() -> Result<()> {
    let legacy = LyricConfigEntity::legacy_disabled().await?;
    if legacy.is_empty() {
        return Ok(());
    }
    for (bid, sid, title) in &legacy {
        LyricBlockEntity::upsert(SCOPE_BID, &bid.to_string(), title, *sid, "").await?;
        LyricConfigEntity::clear_legacy_disable(*bid).await?;
    }
    let pruned = LyricConfigEntity::prune_empty().await?;
    tracing::info!(
        "黑名单迁移完成: {} 条旧屏蔽规则 -> lyric_block, 清理 {} 条空配置行",
        legacy.len(),
        pruned
    );
    Ok(())
}

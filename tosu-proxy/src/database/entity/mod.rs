pub mod lyric_binding;
pub mod lyric_block;
pub mod lyric_cache;
mod lyric_config;
mod setting;

pub(crate) static DB_ERROR_MESSAGE: &str = "无法查询数据库";
pub(crate) use crate::database::database;

pub use lyric_binding::Entity as LyricBindingEntity;
pub use lyric_block::{Entity as LyricBlockEntity, SCOPE_BID, SCOPE_SID, SCOPE_TITLE};
pub use lyric_cache::Entity as LyricCacheEntity;
pub use lyric_config::Entity as LyricConfigEntity;
pub use setting::Entity as SettingEntity;

/// 纯数据迁移：把旧 lyric_config.disable = true 的行迁成 bid 作用域的黑名单规则。
/// 使用 entity API 做跨表数据搬运，不涉及 schema 变更，保留在启动时调用。
/// 幂等，可重复执行。
pub(super) async fn migrate_legacy_data() -> crate::error::Result<()> {
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
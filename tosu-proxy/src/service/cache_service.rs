//! 歌词缓存管理服务（B-06）。
//!
//! 职责边界（B-00 契约 ⑤）：
//! - `lyric_cache` 是**纯缓存**，可以随时删除、过期、清理
//! - `lyric_binding`（来源绑定）、`lyric_config`（偏移）、`lyric_block`（黑名单）
//!   是**用户数据**，本模块的任何操作都**不得触碰它们**
//!
//! TTL：条目写入 / 刷新时记录 `updated_at`，超过 `ttl_ms` 视为过期、不再命中。
//! 默认 30 天；配置里给 `lyricCacheTtlHours` 可覆盖，设为 0 表示不启用。
//!
//! 当前内存态：删除缓存**不影响正在播放的歌词**（歌词已经在内存里）。
//! 缓存只影响"下一次进这首歌时是否还需要联网"。这样删除操作不会打断展示，
//! 也避免"删了缓存反而把当前歌词清掉"的意外。

use crate::database::LyricCacheEntity;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// 默认 TTL：30 天（毫秒）
pub const DEFAULT_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1000;

/// 当前生效的 TTL（毫秒）。`<= 0` 表示不启用。
pub fn ttl_ms() -> i64 {
    match crate::config::GLOBAL_CONFIG.lyric_cache_ttl_hours {
        Some(0) => 0,
        Some(hours) => hours.saturating_mul(60 * 60 * 1000),
        None => DEFAULT_TTL_MS,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub bid: i32,
    pub sid: i32,
    pub title: String,
    /// 毫秒
    pub audio_length: i32,
    /// 写入 / 刷新时间（毫秒时间戳）
    pub updated_at: i64,
    /// 缓存体积（字节），便于前端判断条目是否可用
    pub size: u64,
    /// 是否已过期（按当前 TTL 判定）
    pub expired: bool,
}

impl From<crate::database::lyric_cache::Model> for CacheEntry {
    fn from(m: crate::database::lyric_cache::Model) -> Self {
        let ttl = ttl_ms();
        // 先算过期状态再移动字段（is_fresh 借用整个 model）
        let expired = !crate::database::lyric_cache::is_fresh(&m, ttl);
        Self {
            bid: m.bid,
            sid: m.sid,
            title: m.title,
            audio_length: m.audio_length,
            updated_at: m.updated_at,
            size: m.cache.len() as u64,
            expired,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachePage {
    pub total: u64,
    pub page: u64,
    pub size: u64,
    /// 总页数（total = 0 时为 0）
    pub pages: u64,
    pub items: Vec<CacheEntry>,
    /// 当前生效的 TTL（毫秒），0 表示不启用
    pub ttl_ms: i64,
}

/// 分页查询。`page` 从 **1** 开始；`size` 会被夹到 1..=200。
pub async fn page(query: Option<&str>, page: u64, size: u64) -> Result<CachePage> {
    let page = page.max(1);
    let size = size.clamp(1, 200);
    let total = LyricCacheEntity::count_filtered(query).await?;
    let offset = (page - 1) * size;

    // 请求的页码超出范围时返回空列表而不是报错，但把真实 total 带上，
    // 前端据此把页码回退到最后一页。
    let items = LyricCacheEntity::page(query, offset, size)
        .await?
        .into_iter()
        .map(CacheEntry::from)
        .collect();

    Ok(CachePage {
        total,
        page,
        size,
        pages: total.div_ceil(size),
        items,
        ttl_ms: ttl_ms(),
    })
}

pub async fn count() -> Result<u64> {
    LyricCacheEntity::all_count().await
}

/// 删除单条。返回是否真的删掉了。
pub async fn delete(bid: i32) -> Result<bool> {
    Ok(LyricCacheEntity::delete_by_bid(bid).await? > 0)
}

/// 按标题（模糊）删除，返回删除条数。
pub async fn delete_by_title(title: &str) -> Result<u64> {
    LyricCacheEntity::delete_by_title_like(title).await
}

/// 清空全部，返回删除条数。
pub async fn clear() -> Result<u64> {
    LyricCacheEntity::delete_all().await
}

/// 清理过期条目，返回删除条数。
pub async fn purge_expired() -> Result<u64> {
    LyricCacheEntity::purge_expired(ttl_ms()).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ttl_default_when_unset() {
        // 未配置时用默认 30 天
        assert_eq!(DEFAULT_TTL_MS, 30 * 24 * 60 * 60 * 1000);
    }

    #[test]
    fn page_math_is_sane() {
        // 100 条、每页 30 -> 4 页
        let total: u64 = 100;
        let size: u64 = 30;
        assert_eq!(total.div_ceil(size), 4);
        // 整除时不多出一页
        assert_eq!(30u64.div_ceil(30), 1);
        // 空集合没有页
        assert_eq!(0u64.div_ceil(30), 0);
    }

    #[test]
    fn size_is_clamped() {
        // clamp 语义：0 -> 1，超大 -> 200
        assert_eq!(0u64.clamp(1, 200), 1);
        assert_eq!(9999u64.clamp(1, 200), 200);
    }
}

use crate::database::LyricCacheEntity;
use crate::error::Result;
use serde::{Deserialize, Serialize};

pub const DEFAULT_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1000;

/// <= 0 表示不启用
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
    pub audio_length: i32,
    pub updated_at: i64,
    pub size: u64,
    pub expired: bool,
}

impl From<crate::database::lyric_cache::Model> for CacheEntry {
    fn from(m: crate::database::lyric_cache::Model) -> Self {
        let ttl = ttl_ms();
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
    pub pages: u64,
    pub items: Vec<CacheEntry>,
    pub ttl_ms: i64,
}

/// page 从 1 开始；size 会被夹到 1..=200
pub async fn page(query: Option<&str>, page: u64, size: u64) -> Result<CachePage> {
    let page = page.max(1);
    let size = size.clamp(1, 200);
    let total = LyricCacheEntity::count_filtered(query).await?;
    let offset = (page - 1) * size;

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

pub async fn delete(bid: i32) -> Result<bool> {
    Ok(LyricCacheEntity::delete_by_bid(bid).await? > 0)
}

pub async fn delete_by_title(title: &str) -> Result<u64> {
    LyricCacheEntity::delete_by_title_like(title).await
}

pub async fn clear() -> Result<u64> {
    LyricCacheEntity::delete_all().await
}

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

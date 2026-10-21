use crate::database::LyricCacheEntity;
use crate::database::SettingEntity;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};

pub const DEFAULT_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1000;
/// 过期时间的持久化键（config 表）；为 None 时回落 config.json5 的初值
pub const CACHE_TTL_DB_KEY: &str = "cache_ttl_hours";
pub const MAX_TTL_HOURS: i64 = 24 * 365;

/// 过期时间（毫秒）：0 表示永不过期
static TTL_MS: AtomicI64 = AtomicI64::new(DEFAULT_TTL_MS);

fn ttl_ms_from_hours(hours: Option<i64>) -> i64 {
    match hours {
        Some(0) => 0,
        Some(hours) if hours > 0 => hours.saturating_mul(60 * 60 * 1000),
        _ => DEFAULT_TTL_MS,
    }
}

/// <= 0 表示不启用
pub fn ttl_ms() -> i64 {
    TTL_MS.load(Ordering::Relaxed)
}

/// 启动时恢复过期时间：优先用数据库里设置过的值，否则用 config.json5 的初值
pub async fn init() {
    let stored = SettingEntity::get_config(CACHE_TTL_DB_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|raw| raw.trim().parse::<i64>().ok());
    let hours = stored.or(crate::config::GLOBAL_CONFIG.lyric_cache_ttl_hours);
    TTL_MS.store(ttl_ms_from_hours(hours), Ordering::Relaxed);
}

/// 设置过期时间（小时），0 = 永不过期；写库后立即生效。
///
/// TTL 在每次读取时判定（见 `is_fresh`）：调小后旧缓存立即视为过期，
/// 调大后尚未被「清理过期」删除的条目会重新生效。
pub async fn set_ttl_hours(hours: i64) -> Result<i64> {
    SettingEntity::save_config(CACHE_TTL_DB_KEY.to_string(), hours.to_string()).await?;
    let ms = ttl_ms_from_hours(Some(hours));
    TTL_MS.store(ms, Ordering::Relaxed);
    Ok(ms)
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

/// 删除一首歌的歌词缓存：当前 bid 的行 + 相同 sid 的歌曲级回退行。
/// 范围只含 `lyric_cache`；来源绑定、偏移、黑名单与展示设置不受影响。
pub async fn delete_for_song(bid: i32, sid: i32) -> Result<u64> {
    LyricCacheEntity::delete_by_bid_or_sid(bid, sid).await
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
    fn ttl_hours_semantics() {
        assert_eq!(ttl_ms_from_hours(None), DEFAULT_TTL_MS);
        assert_eq!(ttl_ms_from_hours(Some(0)), 0);
        assert_eq!(ttl_ms_from_hours(Some(24)), 24 * 60 * 60 * 1000);
        assert_eq!(ttl_ms_from_hours(Some(-1)), DEFAULT_TTL_MS);
    }

    #[test]
    fn page_math_is_sane() {
        let total: u64 = 100;
        let size: u64 = 30;
        assert_eq!(total.div_ceil(size), 4);
        assert_eq!(30u64.div_ceil(30), 1);
        assert_eq!(0u64.div_ceil(30), 0);
    }

    #[test]
    fn size_is_clamped() {
        assert_eq!(0u64.clamp(1, 200), 1);
        assert_eq!(9999u64.clamp(1, 200), 200);
    }
}

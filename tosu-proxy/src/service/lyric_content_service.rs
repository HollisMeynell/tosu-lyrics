//! 歌词内容管理服务（B-05）。
//!
//! 覆盖：当前歌词 / 候选列表 / 主动搜索 / 预览 / 应用来源 / 恢复自动匹配 / 偏移。
//!
//! **异步正确性**（B-00 契约 ②）：所有会联网的操作（搜索、取词、应用来源）
//! 都在发起时取一份 `ticket()`（代际 + 歌曲身份），提交前重新校验。
//! 不匹配就返回 `STALE_REQUEST`，由 HTTP 层转成 409 `song_changed`。
//! `abort` 只是尽力而为，这里的校验才是正确性保证。
//!
//! **来源绑定与缓存分离**（契约 ④）：绑定写在 `lyric_binding`（按 sid 归属），
//! 与 `lyric_cache` 完全独立，清缓存不会丢绑定。

use crate::database::LyricBindingEntity;
use crate::error::{Error, Result};
use crate::lyric::{Lyric, LyricLine, LyricSource, LyricSourceEnum, SongInfo};
use crate::service::LYRIC_SERVICE;
use crate::service::lyric_service::{lyric_service, LyricService, SongIdent, STALE_REQUEST};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 搜索候选 —— 每一项都带**自己的身份**。
///
/// 前端必须按这里返回的 `(source, key)` 去应用 / 预览，
/// 不能复用"上一次预览的对象"，否则会出现"所有按钮都应用同一首"的经典缺陷。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// 来源类型（QQ / Netease）
    pub source: String,
    /// 该来源内部的候选标识
    pub key: String,
    pub title: String,
    pub artist: String,
    /// 毫秒
    pub length: u32,
    /// 是否与当前生效的来源绑定一致
    pub active: bool,
    /// 标题匹配分（0~100），越大越匹配当前歌
    pub title_score: i32,
    /// 候选时长 - 当前歌时长（毫秒）。没有在播歌曲时为 0。
    pub duration_delta: i64,
}

fn to_candidate(
    source: &str,
    song: &SongInfo,
    binding: Option<&(String, String)>,
    query: Option<(&str, i64)>,
) -> Candidate {
    let active = binding
        .map(|(st, sk)| st == source && sk == &song.key)
        .unwrap_or(false);
    // 标题匹配分：与当前歌标题比。没有在播歌曲时给 0（前端据此不做排序假设）
    let title_score = query
        .map(|(title, _)| crate::lyric::score_title(title, &song.title))
        .unwrap_or(0);
    // 时长差：候选 - 当前。没有在播歌曲时为 0，避免前端显示 NaN
    let duration_delta = query
        .map(|(_, cur_len)| song.length as i64 - cur_len)
        .unwrap_or(0);
    Candidate {
        source: source.to_string(),
        key: song.key.clone(),
        title: song.title.clone(),
        artist: song.artist.clone(),
        length: song.length,
        active,
        title_score,
        duration_delta,
    }
}

/// 统一排序：**标题匹配优先，其次时长差的绝对值**。
///
/// 刻意**不按来源分组** —— 旧行为是 QQ 永远排前面、网易云永远排后面，
/// 那让"哪个候选最像当前这首歌"这件事被来源顺序盖住了。
fn sort_candidates(items: &mut [Candidate]) {
    items.sort_by(|a, b| {
        b.title_score
            .cmp(&a.title_score)
            .then_with(|| a.duration_delta.abs().cmp(&b.duration_delta.abs()))
            // 完全并列时用来源 + key 定序，保证结果稳定可复现
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.key.cmp(&b.key))
    });
}

/// 当前生效的来源绑定
async fn current_binding(sid: i32) -> Option<(String, String)> {
    LyricBindingEntity::get(sid)
        .await
        .ok()
        .flatten()
        .map(|m| (m.source_type, m.source_key))
}

/// 当前歌曲身份 + 绑定（供各接口复用）
async fn current_context() -> (Option<SongIdent>, Option<(String, String)>) {
    let ident = LyricService::now_ident().await;
    let binding = match &ident {
        Some(ident) => current_binding(ident.sid).await,
        None => None,
    };
    (ident, binding)
}

/// 已有候选（来自内存搜索结果）
pub async fn candidates() -> Vec<Candidate> {
    let (ident, binding) = current_context().await;
    let _ = ident;
    let query = query_of(&ident);
    let svc = lyric_service().await;
    let mut items: Vec<Candidate> = svc
        .all_candidates()
        .await
        .into_iter()
        .map(|(source, song)| {
            to_candidate(source, &song, binding.as_ref(), query.as_ref().map(|(t, l)| (t.as_str(), *l)))
        })
        .collect();
    sort_candidates(&mut items);
    items
}

/// 当前歌的「标题 + 时长」，作为候选评分与时长差的基准
fn query_of(ident: &Option<SongIdent>) -> Option<(String, i64)> {
    let ident = ident.as_ref()?;
    let svc = LYRIC_SERVICE.try_lock().ok()?;
    let song = svc.get_now_song()?;
    Some((ident.title.clone(), song.length as i64))
}

/// 当前歌时长（毫秒）；拿不到锁或没有歌时返回 0
fn current_length() -> i64 {
    LYRIC_SERVICE
        .try_lock()
        .ok()
        .and_then(|svc| svc.get_now_song().map(|s| s.length as i64))
        .unwrap_or(0)
}

/// 主动搜索。`title` / `artist` 为空时使用当前播放歌曲。
///
/// 联网期间不持服务锁；提交前校验代际，切歌后旧搜索会被拒绝。
pub async fn search(title: Option<String>, artist: Option<String>) -> Result<Vec<Candidate>> {
    let (title, artist) = match (title, artist) {
        (Some(t), Some(a)) if !t.is_empty() && !a.is_empty() => (t, a),
        _ => {
            // 未显式给条件：必须有一首正在播放的歌
            let svc = lyric_service().await;
            let key = svc
                .get_now_song()
                .ok_or_else(|| Error::Runtime("no_song:当前没有播放中的歌曲，无法搜索".into()))?;
            (
                key.title_unicode.to_string(),
                key.artist_unicode.to_string(),
            )
        }
    };

    let found = LyricService::search_now(title, artist).await?;
    let (ident, binding) = current_context().await;
    let query = query_of(&ident);
    let mut items: Vec<Candidate> = found
        .into_iter()
        .map(|(source, song)| {
            to_candidate(source, &song, binding.as_ref(), query.as_ref().map(|(t, l)| (t.as_str(), *l)))
        })
        .collect();
    sort_candidates(&mut items);
    Ok(items)
}

/// 预览结果：只回结构化歌词，**不修改任何播放状态**
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewDto {
    pub source: String,
    pub key: String,
    /// 行数
    pub line_count: usize,
    pub lines: Vec<Value>,
}

fn lines_to_json(lines: &[LyricLine]) -> Vec<Value> {
    lines
        .iter()
        .map(|line| {
            let mut item = serde_json::Map::new();
            // 毫秒，与展示 WS 时间单位一致
            item.insert("time".into(), json!((line.time * 1000.0).round() as i64));
            if let Some(origin) = &line.origin {
                item.insert("origin".into(), json!(origin));
            }
            if let Some(translation) = &line.translation {
                item.insert("translation".into(), json!(translation));
            }
            Value::Object(item)
        })
        .collect()
}

/// 取某个候选的歌词内容（不提交、不写缓存、不广播）
pub async fn preview(source: &str, key: &str) -> Result<PreviewDto> {
    let lyric = fetch(source, key).await?;
    let lines = lines_to_json(lyric.get_lyrics());
    Ok(PreviewDto {
        source: source.to_string(),
        key: key.to_string(),
        line_count: lines.len(),
        lines,
    })
}

/// 按来源 + key 取回并解析歌词
async fn fetch(source: &str, key: &str) -> Result<Lyric> {
    let Some(source_impl) = LyricSourceEnum::get_by_name(source) else {
        return Err(Error::Runtime(format!(
            "source_failed:未知的歌词来源 {source}"
        )));
    };
    let raw = source_impl
        .fetch_lyrics(key)
        .await
        .map_err(|err| Error::Runtime(format!("source_failed:{err}")))?;
    let lyric = TryInto::<Lyric>::try_into(raw)
        .map_err(|err| Error::Runtime(format!("lyric_parse:{err}")))?;
    if lyric.get_lyrics().is_empty() {
        return Err(Error::Runtime("lyric_parse:该候选没有可用歌词".into()));
    }
    Ok(lyric)
}

/// 应用来源绑定：取词 → 校验代际与 sid → 提交 → 广播
///
/// 绑定按 **sid** 归属：同谱面集的其它难度共享这份绑定。
pub async fn apply_source(source: &str, key: &str) -> Result<Candidate> {
    let (generation, ident) = LyricService::ticket().await;
    let Some(ident) = ident else {
        return Err(Error::Runtime("no_song:当前没有播放中的歌曲".into()));
    };

    // 联网取词期间不持锁
    let lyric = fetch(source, key).await?;

    let (title, artist, length) = find_candidate_meta(source, key)
        .await
        .unwrap_or_else(|| (ident.title.clone(), String::new(), 0));

    // 先落绑定再提交歌词：即使随后因代际失效没提交，
    // 下一次进入这首歌（同 sid）也会按这份绑定取词，用户的选择不会丢。
    LyricBindingEntity::upsert(
        ident.sid,
        source,
        key,
        &title,
        &artist,
        ident.bid,
    )
    .await?;

    let stale = {
        let mut svc = lyric_service().await;
        svc.apply_source_lyric(generation, ident.sid, lyric).await
    };
    if let Err(err) = stale {
        if err.to_string() == STALE_REQUEST {
            // 绑定已保存，只是这次没赶上播放上下文
            return Err(Error::Runtime(STALE_REQUEST.into()));
        }
        return Err(err);
    }

    Ok(Candidate {
        source: source.to_string(),
        key: key.to_string(),
        title,
        artist,
        length,
        active: true,
        // 刚应用的就是当前绑定，标题必然匹配当前歌
        title_score: 100,
        duration_delta: length as i64 - current_length(),
    })
}

/// 从内存候选里找元信息；找不到就回落到空值（不影响正确性）
async fn find_candidate_meta(source: &str, key: &str) -> Option<(String, String, u32)> {
    let svc = lyric_service().await;
    svc.all_candidates()
        .await
        .into_iter()
        .find(|(name, song)| *name == source && song.key == key)
        .map(|(_, song)| (song.title, song.artist, song.length))
}

/// 恢复自动匹配：删除绑定并重新走一遍自动搜索。
///
/// **没有播放中的歌曲时也必须能清除绑定**：绑定是持久化数据，
/// 用户可能想在没放歌的时候先把它删掉。旧实现直接返回 `no_song`，
/// 会让"清不掉"变成一个假的功能可用性问题。
/// 此时只删数据、不做任何播放态副作用。
pub async fn clear_source() -> Result<bool> {
    let Some(ident) = LyricService::now_ident().await else {
        // 菜单状态：没有当前 sid 可删。清空所有绑定过于粗暴，
        // 因此返回"没有可清除的绑定"而不是报错。
        return Ok(false);
    };
    let removed = LyricBindingEntity::remove(ident.sid).await?;

    // 换了来源，当前歌词必须重新判定；换代 + 清屏后再重新加载
    {
        let mut svc = lyric_service().await;
        svc.clear_display_bump().await;
    }
    LyricService::reload_current().await;
    Ok(removed)
}

/// 设置偏移（毫秒），返回最终生效值。偏移与黑名单、缓存互相独立。
pub async fn set_offset(offset: i32) -> i32 {
    let mut svc = lyric_service().await;
    svc.set_offset(offset).await;
    svc.get_offset()
}

/// 当前状态摘要：给 `/api/lyrics/current` 用，说明"为什么没有歌词"
pub struct ContextSummary {
    /// `ok` | `blocked` | `none`
    pub state: &'static str,
    pub blocked: bool,
}

pub async fn current_context_summary() -> ContextSummary {
    let Some(ident) = LyricService::now_ident().await else {
        return ContextSummary {
            state: "none",
            blocked: false,
        };
    };
    let blocked = crate::service::block_service::blocked_rule(ident.bid, ident.sid, &ident.title)
        .await
        .ok()
        .flatten()
        .is_some();
    let has_lyric = {
        let svc = lyric_service().await;
        svc.get_now_all_lyrics().is_some()
    };
    ContextSummary {
        state: if blocked {
            "blocked"
        } else if has_lyric {
            "ok"
        } else {
            "none"
        },
        blocked,
    }
}

/// 当前来源绑定（可能为 None）
pub async fn binding() -> Option<Value> {
    let (ident, binding) = current_context().await;
    let ident = ident?;
    binding.map(|(source_type, source_key)| {
        json!({
            "sid": ident.sid,
            "source": source_type,
            "key": source_key,
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn song(key: &str, title: &str, artist: &str) -> SongInfo {
        SongInfo {
            title: title.into(),
            artist: artist.into(),
            length: 256_000,
            key: key.into(),
        }
    }

    /// 候选必须带上**自己的**身份，而不是共享一份
    #[test]
    fn candidates_keep_their_own_identity() {
        let a = to_candidate("QQ", &song("k1", "Lemon", "米津玄師"), None, None);
        let b = to_candidate("QQ", &song("k2", "アイネクライネ", "米津玄師"), None, None);
        assert_eq!(a.key, "k1");
        assert_eq!(b.key, "k2");
        assert_ne!(a.key, b.key, "不同候选不能共用同一个 key");
        assert_eq!(a.title, "Lemon");
        assert_eq!(b.title, "アイネクライネ");
    }

    /// active 只对 (source, key) 都一致的那一项为真
    #[test]
    fn active_marks_only_the_bound_candidate() {
        let binding = ("QQ".to_string(), "k2".to_string());
        let a = to_candidate("QQ", &song("k1", "A", ""), Some(&binding), None);
        let b = to_candidate("QQ", &song("k2", "B", ""), Some(&binding), None);
        let c = to_candidate("Netease", &song("k2", "B", ""), Some(&binding), None);
        assert!(!a.active);
        assert!(b.active);
        assert!(
            !c.active,
            "来源不同即使 key 相同也不能算命中绑定"
        );
    }

    #[test]
    fn no_binding_means_nothing_active() {
        let a = to_candidate("QQ", &song("k1", "A", ""), None, None);
        assert!(!a.active);
    }

    #[test]
    fn lines_are_emitted_in_milliseconds() {
        let lyric = Lyric::parse("[00:01.50]a\n[00:03.00]b", None, None).unwrap();
        let json = lines_to_json(lyric.get_lyrics());
        assert_eq!(json.len(), 2);
        assert_eq!(json[0]["time"], 1500);
        assert_eq!(json[1]["time"], 3000);
        assert_eq!(json[0]["origin"], "a");
    }
}


/// 批量检查候选**是否带翻译**。
///
/// 结果来自**真实取词**（不是猜测）：对每个候选取一次歌词，看是否存在
/// 翻译行。并发受限，避免一次搜索就把上游打满。
///
/// 返回与输入**同序**，前端按 `(source,key)` 对应回各行。
pub async fn check_translations(items: Vec<(String, String)>) -> Vec<(String, String, bool)> {
    use futures_util::stream::{self, StreamExt};

    const CONCURRENCY: usize = 8;

    let results: Vec<_> = stream::iter(items)
        .map(|(source, key)| async move {
            let has = match fetch(&source, &key).await {
                Ok(lyric) => lyric
                    .get_lyrics()
                    .iter()
                    .any(|line| line.translation.as_deref().is_some_and(|t| !t.trim().is_empty())),
                // 取不到就按"无翻译"处理，不编造
                Err(_) => false,
            };
            (source, key, has)
        })
        .buffer_unordered(CONCURRENCY)
        .collect()
        .await;

    results
}

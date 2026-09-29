use crate::database::LyricBindingEntity;
use crate::error::{Error, Result};
use crate::lyric::{Lyric, LyricLine, LyricSource, LyricSourceEnum, SongInfo};
use crate::service::LYRIC_SERVICE;
use crate::service::lyric_service::{lyric_service, LyricService, SongIdent, STALE_REQUEST};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 每一项都带自己的身份，前端必须按 (source, key) 去应用 / 预览
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub source: String,
    pub key: String,
    pub title: String,
    pub artist: String,
    pub length: u32,
    pub active: bool,
    pub title_score: i32,
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
    let title_score = query
        .map(|(title, _)| crate::lyric::title_score(title, &song.title))
        .unwrap_or(0);
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

/// 标题匹配优先，其次时长差的绝对值；不按来源分组
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

async fn current_binding(sid: i32) -> Option<(String, String)> {
    LyricBindingEntity::get(sid)
        .await
        .ok()
        .flatten()
        .map(|m| (m.source_type, m.source_key))
}

async fn current_context() -> (Option<SongIdent>, Option<(String, String)>) {
    let ident = LyricService::now_ident().await;
    let binding = match &ident {
        Some(ident) => current_binding(ident.sid).await,
        None => None,
    };
    (ident, binding)
}

pub async fn candidates() -> Vec<Candidate> {
    let (ident, binding) = current_context().await;
    let _ = ident;
    let query = query_of(&ident).await;
    let mut items: Vec<Candidate> = crate::service::lyric_candidates().await
        .into_iter()
        .map(|(source, song)| {
            to_candidate(source, &song, binding.as_ref(), query.as_ref().map(|(t, l)| (t.as_str(), *l)))
        })
        .collect();
    sort_candidates(&mut items);
    items
}

async fn query_of(ident: &Option<SongIdent>) -> Option<(String, i64)> {
    let ident = ident.as_ref()?;
    let svc = lyric_service().await;
    let song = svc.get_now_song()?;
    if song.bid as i32 != ident.bid { return None; }
    Some((ident.title.clone(), song.length as i64))
}

async fn current_length() -> i64 {
    lyric_service().await.get_now_song().map_or(0, |song| song.length as i64)
}

/// title / artist 为空时使用当前播放歌曲；联网期间不持服务锁
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
    let query = query_of(&ident).await;
    let mut items: Vec<Candidate> = found
        .into_iter()
        .map(|(source, song)| {
            to_candidate(source, &song, binding.as_ref(), query.as_ref().map(|(t, l)| (t.as_str(), *l)))
        })
        .collect();
    sort_candidates(&mut items);
    Ok(items)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewDto {
    pub source: String,
    pub key: String,
    pub line_count: usize,
    pub lines: Vec<Value>,
}

fn lines_to_json(lines: &[LyricLine]) -> Vec<Value> {
    lines
        .iter()
        .map(|line| {
            let mut item = serde_json::Map::new();
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

/// 绑定按 sid 归属：同谱面集的其它难度共享这份绑定
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

    // 先落绑定再提交歌词：即使代际失效没提交，下次进入同 sid 也会按这份绑定取词
    LyricBindingEntity::upsert(
        ident.sid,
        source,
        key,
        &title,
        &artist,
        ident.bid,
    )
    .await?;

    let sid = ident.sid;
    let stale = LYRIC_SERVICE.call(move |svc| Box::pin(async move {
        svc.apply_source_lyric(generation, sid, lyric).await
    })).await;
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
        duration_delta: length as i64 - current_length().await,
    })
}

async fn find_candidate_meta(source: &str, key: &str) -> Option<(String, String, u32)> {
    crate::service::lyric_candidates().await
        .into_iter()
        .find(|(name, song)| *name == source && song.key == key)
        .map(|(_, song)| (song.title, song.artist, song.length))
}

/// 没有播放中的歌曲时也能清除绑定（绑定是持久化数据）
pub async fn clear_source() -> Result<bool> {
    let Some(ident) = LyricService::now_ident().await else {
        return Ok(false);
    };
    let removed = LyricBindingEntity::remove(ident.sid).await?;

    LYRIC_SERVICE.call(|svc| Box::pin(async move { svc.clear_display_bump().await })).await;
    LyricService::reload_current().await;
    Ok(removed)
}

pub async fn set_offset(offset: i32) -> i32 {
    LYRIC_SERVICE.call(move |svc| Box::pin(async move {
        svc.set_offset(offset).await;
        svc.get_offset()
    })).await
}

pub struct ContextSummary {
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


/// 结果来自真实取词，不是猜测；并发受限避免把上游打满
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
                Err(_) => false,
            };
            (source, key, has)
        })
        .buffer_unordered(CONCURRENCY)
        .collect()
        .await;

    results
}

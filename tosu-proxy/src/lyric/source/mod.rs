mod netease;
mod qq;

use super::Lyric;
use crate::error::{Error, Result};
use async_trait::async_trait;
pub use netease::NeteaseLyricSource;
pub use qq::QQLyricSource;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::LazyLock;
use tracing::{debug, error};

const NO_LENGTH: u32 = 0;
/// 候选与音频时长的最大允许差(毫秒), 超过视为不同歌曲
const MAX_LENGTH_DIFF: u32 = 15000;
/// 时长非常接近(毫秒), 得满分
const GOOD_LENGTH_DIFF: u32 = 8000;
/// 标题有效字符数 <= 该值视为"短/常见"标题, 搜索时附加 artist
const SHORT_TITLE_CHARS: usize = 6;
/// 排名第一的候选标题+版本评分低于该值视为与目标歌曲明显无关, 不采用
const MIN_TITLE_SCORE: i32 = 45;
/// 标题达到该分数即视为"强匹配"(完全一致 / 候选含完整目标标题 / 只差一个附录),
/// 此时 artist 只参与排序, 不再能否决候选
const TITLE_STRONG_SCORE: i32 = 85;

const QQ_SEARCH_URL_PREFIX: &str =
    "https://c.y.qq.com/soso/fcgi-bin/client_search_cp?p=1&n=5&format=json&w=";

const NETEASE_SEARCH_URL_PREFIX: &str = "https://music.163.com/api/search/get?s=";

const NETEASE_SEARCH_URL_SUFFIX: &str = "&type=1&limit=5";

// 上游 URL 长度限制按 percent-encoding 后的整条 URL 计算。
const QQ_SEARCH_URL_MAX: usize = 8000;

const NETEASE_SEARCH_URL_MAX: usize = 7000;

fn search_url_limit(source: &str) -> Option<(&'static str, &'static str, usize)> {
    match source {
        "QQ" => Some((QQ_SEARCH_URL_PREFIX, "", QQ_SEARCH_URL_MAX)),
        "Netease" => Some((
            NETEASE_SEARCH_URL_PREFIX,
            NETEASE_SEARCH_URL_SUFFIX,
            NETEASE_SEARCH_URL_MAX,
        )),
        _ => None,
    }
}

fn encoded_search_url_len(prefix: &str, query: &str, suffix: &str) -> Option<usize> {
    reqwest::Url::parse(&format!("{prefix}{query}{suffix}"))
        .ok()
        .map(|u| u.as_str().len())
}

macro_rules! static_source {
    ($($name:ident : $t:ident),* $(,)?) => {
        $(
            pub static $name: LazyLock<$t> = LazyLock::new(<$t>::default);
        )*

        pub enum LyricSourceEnum {
            $($t(&'static $t)),*
        }

        #[async_trait]
        impl LyricSource for LyricSourceEnum {
            fn name(&self) -> &str {
                match self {
                    $(Self::$t(source) => source.name()),*
                }
            }

            async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>> {
                match self {
                    $(Self::$t(source) => source.search_music(title).await),*
                }
            }

            async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult> {
                match self {
                    $(Self::$t(source) => source.fetch_lyrics(song_id).await),*
                }
            }
        }

        impl LyricSourceEnum {
            pub fn get_by_name(name: &str) -> Option<Self> {
                let result = match name {
                    $(name if name == $name.name() => Self::$t(&*$name),)*
                    _ => return None,
                };
                Some(result)
            }
        }
    };
}

static_source! {
    QQ_LYRIC_SOURCE: QQLyricSource,
    NETEASE_LYRIC_SOURCE: NeteaseLyricSource,
}

// 词条使用词边界，不包含句点等非词字符。
static VERSION_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(remastered|remaster|remix|live|tv[ -]?size|tv[ -]?ver|short[ -]?ver|game[ -]?ver|full[ -]?ver|version|ver|instrumental|inst|off[ -]?vocal|acoustic|cover|edit|original|sped[ -]?up|slowed|nightcore)\b",
    )
    .unwrap()
});

fn normalize_title(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn core_title(s: &str) -> String {
    normalize_title(&VERSION_TAG_RE.replace_all(s, " "))
}

fn strip_bracket_content(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0usize;
    for ch in s.chars() {
        match ch {
            '(' | '（' | '[' | '【' | '{' | '｛' => depth += 1,
            ')' | '）' | ']' | '】' | '}' | '｝' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

fn version_tags(s: &str) -> Vec<String> {
    VERSION_TAG_RE
        .find_iter(s)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

fn bigram_similarity(a: &str, b: &str) -> f32 {
    if a.len() < 2 || b.len() < 2 {
        return 0.0;
    }
    let grams = |s: &str| -> HashSet<(char, char)> {
        let chars: Vec<char> = s.chars().collect();
        chars.windows(2).map(|w| (w[0], w[1])).collect()
    };
    let ga = grams(a);
    let gb = grams(b);
    if ga.is_empty() || gb.is_empty() {
        return 0.0;
    }
    let inter = ga.intersection(&gb).count();
    (2.0 * inter as f32) / (ga.len() + gb.len()) as f32
}

pub(crate) fn title_score(query: &str, candidate: &str) -> i32 {
    let q = normalize_title(query);
    let c = normalize_title(candidate);
    if q.is_empty() || c.is_empty() {
        return 0;
    }
    if q == c {
        return 100;
    }
    if c.contains(&q) {
        return 90;
    }
    let (qb, cb) = (
        normalize_title(&strip_bracket_content(query)),
        normalize_title(&strip_bracket_content(candidate)),
    );
    if !qb.is_empty() && qb == cb {
        return 85;
    }
    let (qc, cc) = (core_title(query), core_title(candidate));
    if !qc.is_empty() && qc == cc {
        return 85;
    }
    if q.contains(&c) {
        return 60;
    }
    20 + (bigram_similarity(&q, &c) * 50.0) as i32
}

const ARTIST_SEPARATORS: [char; 18] = [
    '&', ',', '/', '、', ';', '；', '(', ')', '（', '）', '[', ']', '【', '】', '×', '・', '·', '|',
];

// x 需两侧空格以免拆散名字；feat. 允许直接接中日文署名。
const ARTIST_JOINERS: [&str; 7] = [
    " feat.", " feat ", " vs.", " vs ", " x ", " with ", " meets ",
];

fn artist_parts(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|ch: char| ARTIST_SEPARATORS.contains(&ch))
        .flat_map(split_artist_joiners)
        .map(|s| s.trim().to_string())
        .filter(|t| t.chars().count() >= 2)
        .collect()
}

fn split_artist_joiners(part: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = part.trim().to_lowercase();
    loop {
        let hit = ARTIST_JOINERS
            .iter()
            .filter_map(|joiner| rest.find(joiner).map(|idx| (idx, joiner.len())))
            .min_by_key(|(idx, _)| *idx);
        let Some((idx, len)) = hit else { break };
        let head = rest[..idx].trim().to_string();
        if !head.is_empty() {
            out.push(head);
        }
        rest = rest[idx + len..].trim().to_string();
    }
    if !rest.is_empty() {
        out.push(rest);
    }
    out
}

fn artist_score(query: &str, candidate: &str) -> i32 {
    let parts = artist_parts(query);
    if parts.is_empty() {
        return 0;
    }
    let c = normalize_title(candidate);
    let hit = parts
        .iter()
        .filter(|t| {
            let normalized = normalize_title(t);
            !normalized.is_empty() && c.contains(&normalized)
        })
        .count();
    (hit * 20 / parts.len()) as i32
}

fn length_score(candidate: u32, audio: u32) -> i32 {
    if audio == NO_LENGTH {
        return 0;
    }
    let diff = candidate.abs_diff(audio);
    if diff <= GOOD_LENGTH_DIFF {
        20
    } else if diff <= MAX_LENGTH_DIFF {
        10
    } else {
        0
    }
}

fn version_adjust(query: &str, candidate: &str, title_s: i32) -> i32 {
    if title_s < 60 {
        return 0;
    }
    let qt = version_tags(query);
    let ct = version_tags(candidate);
    if qt.is_empty() && ct.is_empty() {
        return 0;
    }
    if qt.is_empty() {
        return 0;
    }
    if ct.is_empty() {
        return -10;
    }
    if qt.iter().any(|t| ct.contains(t)) {
        10
    } else {
        -10
    }
}

fn is_same_song(anchor: &SongInfo, candidate: &SongInfo) -> bool {
    if anchor.key == candidate.key {
        return true;
    }
    if title_score(&anchor.title, &candidate.title) < 90 {
        return false;
    }
    if artist_score(&anchor.artist, &candidate.artist) > 0 {
        return true;
    }
    normalize_title(&anchor.title) == normalize_title(&candidate.title)
        && (anchor.length == NO_LENGTH
            || candidate.length == NO_LENGTH
            || anchor.length.abs_diff(candidate.length) <= GOOD_LENGTH_DIFF)
}

fn is_generic_artist(artist: &str) -> bool {
    let a = artist.to_lowercase();
    a.contains("various artist") || a.contains("unknown artist") || a == "various" || a == "unknown"
}

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfoKey {
    #[serde(rename = "type")]
    pub source_type: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub length: u32,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricResult {
    pub lyric: Option<String>,
    pub trans: Option<String>,
}

impl LyricResult {
    fn new() -> Self {
        Self {
            lyric: None,
            trans: None,
        }
    }
    fn is_none(&self) -> bool {
        self.lyric.is_none() && self.trans.is_none()
    }
}

impl TryInto<Lyric> for LyricResult {
    type Error = Error;

    fn try_into(self) -> Result<Lyric> {
        match (self.lyric, self.trans) {
            (Some(lyric), Some(trans)) => Lyric::parse(&lyric, Some(&trans), None),
            (Some(lyric), None) => Lyric::parse(&lyric, None, None),
            (None, Some(trans)) => Lyric::parse(&trans, None, None),
            (None, None) => Err(Error::from("no lyric")),
        }
    }
}

#[async_trait]
#[allow(clippy::double_must_use)]
pub trait LyricSource: Send + Sync {
    fn name(&self) -> &str;
    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>>;
    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult>;
    fn preferred_song<'a>(
        songs: &'a [SongInfo],
        title: &str,
        length: u32,
        artist: &str,
    ) -> Vec<&'a SongInfo> {
        let query_tags = version_tags(title);
        let mut ranked: Vec<(&'a SongInfo, i32)> = songs
            .iter()
            .filter(|song| length == NO_LENGTH || song.length.abs_diff(length) <= MAX_LENGTH_DIFF)
            .filter(|song| {
                query_tags.is_empty()
                    || version_tags(&song.title)
                        .iter()
                        .any(|tag| query_tags.contains(tag))
            })
            .map(|song| {
                let title_s = title_score(title, &song.title);
                let mut score = title_s;
                score += artist_score(artist, &song.artist);
                score += length_score(song.length, length);
                score += version_adjust(title, &song.title, title_s);
                (song, score)
            })
            .collect();
        ranked.sort_unstable_by_key(|(_, score)| std::cmp::Reverse(*score));
        ranked.into_iter().map(|(song, _)| song).collect()
    }

    async fn search_all_music(&self, title: &str, artist: &str) -> Result<Vec<SongInfo>> {
        let artist_usable = !artist.trim().is_empty();
        let with_artist = if artist_usable {
            format!("{title} {artist}")
        } else {
            String::new()
        };

        if !artist_usable {
            debug!("[perf] (s) {} 分支=仅 title(artist 为空)", self.name());
            return self.search_music(title).await;
        }

        let within_limit = match search_url_limit(self.name()) {
            Some((prefix, suffix, max)) => {
                match encoded_search_url_len(prefix, &with_artist, suffix) {
                    Some(len) => len <= max,
                    None => false,
                }
            }
            None => false,
        };
        if !within_limit {
            debug!(
                "[perf] (s) {} 分支=超长跳过 title+artist(仅用 title) query_chars={}",
                self.name(),
                with_artist.chars().count(),
            );
            return self.search_music(title).await;
        }

        debug!(
            "[perf] (s) {} 分支=title+artist 并发 query_chars={}",
            self.name(),
            with_artist.chars().count(),
        );
        let (by_title, by_artist) =
            tokio::join!(self.search_music(title), self.search_music(&with_artist));
        let mut merged = by_title?;
        for song in by_artist? {
            if !merged.iter().any(|m| m.key == song.key) {
                merged.push(song);
            }
        }
        debug!(
            "[perf] (s) {} 分支结束=title+artist 并发 候选={}",
            self.name(),
            merged.len()
        );
        Ok(merged)
    }

    async fn search_lyrics(
        &self,
        song_all: &mut Vec<SongInfo>,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<Option<LyricResult>> {
        if song_all.is_empty() {
            return Ok(None);
        }
        let original_first_key = song_all[0].key.clone();

        let need_extra = !artist.trim().is_empty()
            && !is_generic_artist(artist)
            && !is_short_title(title)
            && Self::preferred_song(song_all, title, length, artist)
                .first()
                .map(|c| artist_score(artist, &c.artist) == 0)
                .unwrap_or(false);
        if need_extra {
            debug!(
                "[perf] (8b) {} need_extra 命中(候选池={}), title+artist 已并发合并, 不再发第二次搜索",
                self.name(),
                song_all.len(),
            );
        }

        let song = Self::preferred_song(song_all, title, length, artist);
        if song.is_empty() {
            return Ok(None);
        }

        let top = song.first().expect("ranked list is not empty");
        let title_s = title_score(title, &top.title);
        let version_s = version_adjust(title, &top.title, title_s);
        let artist_required = !artist.trim().is_empty() && !is_generic_artist(artist);
        let artist_ok = !artist_required || artist_score(artist, &top.artist) > 0;
        let title_strong = title_s >= TITLE_STRONG_SCORE;
        let exempt = length != NO_LENGTH
            && title_s < TITLE_STRONG_SCORE
            && top.key == original_first_key
            && top.length.abs_diff(length) <= GOOD_LENGTH_DIFF;
        if title_s + version_s < MIN_TITLE_SCORE {
            debug!(
                "首选候选与目标差异过大(标题{title_s} 版本{version_s}), 放弃该源: {}",
                top.title
            );
            return Ok(None);
        }
        if !artist_ok && !title_strong && !exempt {
            debug!(
                "首选候选不可靠(标题{title_s} 版本{version_s} 歌手匹配{artist_ok}), 放弃该源: {}",
                top.title
            );
            return Ok(None);
        }

        let mut attempt_count = 0usize;
        for info in &song {
            if is_same_song(top, info) {
                attempt_count += 1;
            }
        }
        debug!(
            "[perf] (10) {} 候选池={} 将串行尝试={} title_s={} version_s={} artist_ok={} title_strong={} exempt={}",
            self.name(),
            song.len(),
            attempt_count,
            title_s,
            version_s,
            artist_ok,
            title_strong,
            exempt,
        );

        for info in &song {
            if !is_same_song(top, info) {
                debug!("跳过与首选不同曲的候选: {}", info.title);
                continue;
            }
            let t0 = std::time::Instant::now();
            let lyrics = match self.fetch_lyrics(&info.key).await {
                Ok(lyrics) => lyrics,
                Err(err) => {
                    debug!(
                        "[perf] (10) {} 候选={} 请求失败 耗时={}ms: {err}",
                        self.name(),
                        info.key,
                        t0.elapsed().as_millis(),
                    );
                    error!("获取歌词失败(候选 {}): {}", info.key, err);
                    continue;
                }
            };
            if lyrics.is_none() {
                debug!(
                    "[perf] (10) {} 候选={} 无歌词 耗时={}ms",
                    self.name(),
                    info.key,
                    t0.elapsed().as_millis(),
                );
                continue;
            }
            match <LyricResult as TryInto<Lyric>>::try_into(lyrics.clone()) {
                Ok(lyric) if lyric.get_lyrics().len() > 4 => {
                    debug!(
                        "[perf] (10) {} 候选={} **采用** 耗时={}ms 行数={}",
                        self.name(),
                        info.key,
                        t0.elapsed().as_millis(),
                        lyric.get_lyrics().len(),
                    );
                    return Ok(Some(lyrics));
                }
                Ok(_) => debug!("歌词过短(<=4行), 跳过候选 {}", info.key),
                Err(err) => debug!("歌词解析失败, 跳过候选 {}: {}", info.key, err),
            }
        }

        Ok(None)
    }
}

fn is_short_title(title: &str) -> bool {
    let count = title.chars().filter(|c| c.is_alphanumeric()).count();
    count <= SHORT_TITLE_CHARS
}

#[cfg(test)]
mod tests {
    use super::*;

    const TITLE: &str = "Clair de lune";
    const ARTIST: &str = "Debussy";

    #[test]
    fn encoded_len_counts_percent_encoding_not_bytes() {
        let (prefix, suffix, _) = search_url_limit("QQ").unwrap();
        let base = prefix.len() + suffix.len();

        let ja = encoded_search_url_len(prefix, &"あ".repeat(10), suffix).unwrap();
        assert_eq!(ja - base, 90, "日文 10 字符应膨胀为 90 个 URL 字符");

        let ascii = encoded_search_url_len(prefix, &"a".repeat(10), suffix).unwrap();
        assert_eq!(ascii - base, 10);

        let real = encoded_search_url_len(
            prefix,
            "病み垢ステロイド かいりきベア feat. 初音ミク",
            suffix,
        )
        .unwrap();
        assert!(
            real < QQ_SEARCH_URL_MAX,
            "真实日文 query 不应被误判超长: {real}"
        );
    }

    #[test]
    fn limit_is_below_measured_failure_boundary() {
        let (qq_prefix, qq_suffix, qq_max) = search_url_limit("QQ").unwrap();
        let qq_fail = encoded_search_url_len(qq_prefix, &"a".repeat(9800), qq_suffix).unwrap();
        assert_eq!(qq_fail, 9872, "前缀长度变化会破坏与实测边界的对应关系");
        assert!(qq_fail > qq_max, "实测失败的长度必须被拒绝");

        let (ne_prefix, ne_suffix, ne_max) = search_url_limit("Netease").unwrap();
        let ne_fail = encoded_search_url_len(ne_prefix, &"a".repeat(8150), ne_suffix).unwrap();
        assert_eq!(ne_fail, 8204, "NetEase 的固定后缀必须计入长度");
        assert!(ne_fail > ne_max, "实测失败的长度必须被拒绝");

        assert_ne!(qq_max, ne_max);
    }

    #[test]
    fn unknown_source_is_treated_as_unsafe() {
        assert!(search_url_limit("QQ").is_some());
        assert!(search_url_limit("Netease").is_some());
        assert!(search_url_limit("SomeFutureSource").is_none());
    }

    #[test]
    fn special_characters_do_not_break_url_measurement() {
        for q in [
            "Chocolate Adventure feat. ななひら&すずしろ (Live Ver.) Neko Hacker",
            "中國話 S.H.E",
            "CANNIBALISM 北沢強兵 feat. flower",
            "A/B + C#D (E) & F",
            "하츠투하츠 FOCUS",
            "病み垢ステロイド かいりきベア feat. 初音ミク",
            "ロス ぐしゃぐしゃ",
            "佐々木 詩織 Prime Time Golden Hour Show",
        ] {
            let (qp, qs, qmax) = search_url_limit("QQ").unwrap();
            let (np, ns, nmax) = search_url_limit("Netease").unwrap();
            let qq = encoded_search_url_len(qp, q, qs);
            let ne = encoded_search_url_len(np, q, ns);
            assert!(qq.is_some(), "QQ URL 解析失败: {q}");
            assert!(ne.is_some(), "NetEase URL 解析失败: {q}");
            assert!(qq.unwrap() < qmax, "真实标题不应超限: {q}");
            assert!(ne.unwrap() < nmax, "真实标题不应超限: {q}");
        }
    }

    #[test]
    fn oversized_query_is_rejected_for_both_sources() {
        for q in ["a".repeat(20000), "あ".repeat(3000)] {
            let (qp, qs, qmax) = search_url_limit("QQ").unwrap();
            let (np, ns, nmax) = search_url_limit("Netease").unwrap();
            assert!(encoded_search_url_len(qp, &q, qs).unwrap() > qmax);
            assert!(encoded_search_url_len(np, &q, ns).unwrap() > nmax);
        }
    }

    #[test]
    fn title_exact_match_is_full_score() {
        assert_eq!(title_score("メクルメ", "メクルメ"), 100);
        assert_eq!(title_score("Yumemi Sunrise", "yumemi sunrise"), 100);
    }

    #[test]
    fn title_candidate_with_suffix_is_strong() {
        assert_eq!(title_score("メクルメ", "メクルメ (Game Ver.)"), 90);
        assert_eq!(title_score("Song", "Song (Remastered)"), 90);
    }

    #[test]
    fn title_differing_only_by_appendix_is_85() {
        assert_eq!(
            title_score("os-宇宙人(Asterisk Makina Remix)", "os-宇宙人"),
            85
        );
        assert_eq!(
            title_score(
                "World's End, Girl's Rondo (Asterisk DnB Remix)",
                "World's End, Girl's Rondo"
            ),
            85
        );
        assert_eq!(title_score("Song - Remastered", "Song"), 85);
    }

    #[test]
    fn version_tags_cover_reported_forms() {
        assert!(!version_tags("os-宇宙人(Asterisk Makina Remix)").is_empty());
        assert!(!version_tags("Song (Remastered)").is_empty());
        assert!(!version_tags("Song (TV Size)").is_empty());
        assert!(!version_tags("Song (Short Ver.)").is_empty());
        assert!(!version_tags("Song (Game Version)").is_empty());
        assert!(version_tags("メクルメ").is_empty());
        assert!(version_tags("Veronica").is_empty());
        assert!(version_tags("Instinct").is_empty());
    }

    #[test]
    fn title_trunk_helpers() {
        assert_eq!(
            strip_bracket_content("os-宇宙人(Asterisk Makina Remix)"),
            "os-宇宙人"
        );
        assert_eq!(strip_bracket_content("Song（TV Size）"), "Song");
        assert_eq!(core_title("Song - Remastered"), core_title("Song"));
    }

    #[test]
    fn unrelated_title_stays_low() {
        assert!(title_score("メクルメ", "夜に駆ける") < MIN_TITLE_SCORE);
        assert!(title_score("Yumemi Sunrise", "Wah Wah World") < MIN_TITLE_SCORE);
    }

    #[test]
    fn artist_cv_annotation_is_split() {
        let score = artist_score("篠澤広（CV: 川村玲奈）", "篠澤広");
        assert!(score > 0, "CV 标注应拆成独立片段, 实际得分 {score}");
    }

    #[test]
    fn artist_cooperation_forms_hit_partially() {
        assert!(artist_score("Xceon feat.森永真由美", "Xceon") > 0);
        assert!(artist_score("Xceon feat.森永真由美", "森永真由美") > 0);
        assert!(artist_score("Giga x Mitchie M", "Giga") > 0);
        assert!(artist_score("Giga x Mitchie M", "Mitchie M") > 0);
    }

    #[test]
    fn artist_joiners_keep_embedded_letters() {
        assert_eq!(artist_parts("Xceon"), vec!["xceon".to_string()]);
        assert_eq!(artist_score("Xceon", "Xceon"), 20);
    }

    #[test]
    fn artist_score_bounds() {
        assert_eq!(artist_score("歌組雪月花", "歌組雪月花"), 20);
        assert_eq!(artist_score("分島花音", "Someone Else"), 0);
        assert_eq!(artist_score("", "Anyone"), 0);
    }

    #[test]
    fn version_adjust_is_asymmetric() {
        assert_eq!(version_adjust("Song", "Song (Remastered)", 90), 0);
        assert_eq!(version_adjust("Song (Remix)", "Song", 85), -10);
        assert_eq!(version_adjust("Song (Remix)", "Song (Remix)", 100), 10);
        assert_eq!(version_adjust("Song (Remix)", "Song (Live)", 85), -10);
        assert_eq!(version_adjust("Song (Remix)", "Other (Live)", 30), 0);
    }

    #[test]
    fn same_song_tolerates_artist_wording() {
        let anchor = SongInfo {
            title: "メクルメ".into(),
            artist: "篠澤広（CV: 川村玲奈）".into(),
            length: 100_000,
            key: "a".into(),
        };
        let candidate = SongInfo {
            title: "メクルメ".into(),
            artist: "篠澤広".into(),
            length: 100_500,
            key: "b".into(),
        };
        assert!(is_same_song(&anchor, &candidate));

        let other = SongInfo {
            title: "別の曲".into(),
            artist: "篠澤広".into(),
            length: 100_000,
            key: "c".into(),
        };
        assert!(!is_same_song(&anchor, &other));
    }

    #[tokio::test]
    async fn test_qq_lyric_source() -> Result<()> {
        let song_info = QQ_LYRIC_SOURCE.search_all_music(TITLE, ARTIST).await?;
        let song = song_info.first().ok_or("not found song")?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = QQ_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_netease_lyric_source() -> Result<()> {
        let song_info = NETEASE_LYRIC_SOURCE.search_all_music(TITLE, ARTIST).await?;
        let song = song_info.first().ok_or("not found song")?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = NETEASE_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }
}

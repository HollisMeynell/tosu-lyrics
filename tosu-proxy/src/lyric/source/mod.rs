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

/// 版本标识关键词(remix/live/tv size/sped up 等), 用于区分同曲不同版本
static VERSION_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(remix|live|tv[ -]size|instrumental|acoustic|cover|edit|short ver|game ver|original|sped up|slowed|nightcore)\b",
    )
    .unwrap()
});

/// 标题归一化: 小写, 仅保留字母数字(含中日韩字符)
fn normalize_title(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// 提取标题中的版本标识
fn version_tags(s: &str) -> Vec<String> {
    VERSION_TAG_RE
        .find_iter(s)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

/// 2-gram Dice 相似度, 用于无法互相包含时的兜底比较
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

/// 标题相似度评分(0-100), 越高越可能是同一首歌
pub(crate) fn title_score(query: &str, candidate: &str) -> i32 {
    let q = normalize_title(query);
    let c = normalize_title(candidate);
    if q.is_empty() || c.is_empty() {
        return 0;
    }
    if q == c {
        return 100;
    }
    // 候选包含完整查询标题(候选带额外版本信息, 如查询是原版)
    if c.contains(&q) {
        return 90;
    }
    // 查询包含候选(候选缺少查询中的版本标签, 如 Remix 查到了原版)
    if q.contains(&c) {
        return 60;
    }
    // 字级别 bigram 相似度兜底
    20 + (bigram_similarity(&q, &c) * 50.0) as i32
}

/// artist 相似度软评分(0-20), 只用于排序, 不硬过滤
fn artist_score(query: &str, candidate: &str) -> i32 {
    let q = query.to_lowercase();
    if q.trim().is_empty() {
        return 0;
    }
    // 拆分 feat./vs. 与常见分隔符, 得到多个歌手片段, 任一命中即可得分
    let parts: Vec<&str> = q
        .split(|ch: char| {
            ch == '&'
                || ch == ','
                || ch == '/'
                || ch == '、'
                || ch == ';'
                || ch == '；'
                || ch == '('
                || ch == ')'
        })
        .flat_map(|part| {
            let mut segs: Vec<&str> = Vec::new();
            let mut rest = part.trim();
            while let Some(idx) = rest.find(" feat.") {
                let (head, tail) = rest.split_at(idx);
                if !head.trim().is_empty() {
                    segs.push(head.trim());
                }
                rest = tail[" feat.".len()..].trim();
            }
            if let Some(idx) = rest.find(" vs.") {
                let (head, tail) = rest.split_at(idx);
                if !head.trim().is_empty() {
                    segs.push(head.trim());
                }
                rest = tail[" vs.".len()..].trim();
            }
            if !rest.is_empty() {
                segs.push(rest);
            }
            segs
        })
        .filter(|t| t.chars().count() >= 2)
        .collect();
    if parts.is_empty() {
        return 0;
    }
    let c = normalize_title(candidate);
    let hit = parts
        .iter()
        .filter(|t| c.contains(&normalize_title(t)))
        .count();
    (hit * 20 / parts.len()) as i32
}

/// 时长接近程度评分(0-20); 时长未知时不参与评分
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

/// 版本标识一致性调整(-10..10):
/// 只有确认候选与目标歌曲本身相关(标题相等/包含)时版本信息才参与评分,
/// 避免完全无关的歌曲因为词面含 "remix" 等关键词而获得加分
fn version_adjust(query: &str, candidate: &str, title_s: i32) -> i32 {
    if title_s < 60 {
        return 0;
    }
    let qt = version_tags(query);
    let ct = version_tags(candidate);
    if qt.is_empty() && ct.is_empty() {
        return 0;
    }
    if qt.iter().any(|t| ct.contains(t)) {
        10
    } else {
        -10
    }
}

/// 判断候选是否与首选属于同一首歌(抓词失败后的回退范围):
/// 标题高度一致(相等或候选包含首选标题)且歌手相关;
/// 同名但歌手完全无关的候选不算同一首歌
fn is_same_song(anchor: &SongInfo, candidate: &SongInfo) -> bool {
    if anchor.key == candidate.key {
        return true;
    }
    title_score(&anchor.title, &candidate.title) >= 90
        && artist_score(&anchor.artist, &candidate.artist) > 0
}

/// 通用署名(合集/未知), 此类 artist 不参与补充搜索与 artist 筛选,
/// 避免误伤 "Various Artists" 等合集曲目
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
    /// 毫秒
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
pub trait LyricSource: Send + Sync {
    fn name(&self) -> &str;
    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>>;
    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult>;
    /// 候选排序: 标题相似度为主, artist/时长/版本标识为辅, 只排序不硬过滤
    /// 硬过滤仅两类: 时长差超过 MAX_LENGTH_DIFF 视为不同歌曲;
    /// 目标标题带版本标记(Remix/Live/Instrumental/TV Size 等)时,
    /// 只接受带相同标记的候选, 不回退到普通原版
    fn preferred_song<'a>(
        songs: &'a [SongInfo],
        title: &str,
        length: u32,
        artist: &str,
    ) -> Vec<&'a SongInfo> {
        let query_tags = version_tags(title);
        let mut ranked: Vec<(&'a SongInfo, i32)> = songs
            .iter()
            // 时长硬过滤(时长未知时跳过, 由标题评分兜底)
            .filter(|song| length == NO_LENGTH || song.length.abs_diff(length) <= MAX_LENGTH_DIFF)
            // 版本硬过滤: 目标带版本标记时, 候选必须带相同标记
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
        // 分数降序; 排序稳定, 同分保持搜索结果原始顺序
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.into_iter().map(|(song, _)| song).collect()
    }

    /// 搜索策略:
    /// - 标题足够独特: 只搜 title(与旧版一致), 无结果时用 title+artist 兜底
    /// - 标题过短/常见: 同时发出纯 title 与 title+artist 两条查询,
    ///   合并去重(纯 title 结果在前), artist 仅作为辅助信息
    /// `length` 使用 毫秒数
    async fn search_all_music(&self, title: &str, artist: &str) -> Result<Vec<SongInfo>> {
        let artist_usable = !artist.trim().is_empty();
        let with_artist = if artist_usable {
            format!("{title} {artist}")
        } else {
            String::new()
        };

        if artist_usable && is_short_title(title) {
            let mut merged = self.search_music(title).await?;
            let with = self.search_music(&with_artist).await?;
            for song in with {
                if !merged.iter().any(|m| m.key == song.key) {
                    merged.push(song);
                }
            }
            return Ok(merged);
        }

        let song_all = self.search_music(title).await?;
        if !song_all.is_empty() {
            return Ok(song_all);
        }
        if artist_usable {
            return self.search_music(&with_artist).await;
        }
        Ok(song_all)
    }

    /// 按排序后的候选逐个尝试取词; length 为 0 表示时长未知
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
        // 豁免判断的参照: 补充搜索前的原始第一名
        let original_first_key = song_all[0].key.clone();

        // 首选候选 artist 完全无关时, 追加 title+artist 搜索扩大候选池,
        // 让纯标题搜索没查到的正确歌曲有机会进入(结果去重, 重新排序)
        let need_extra = !artist.trim().is_empty()
            && !is_generic_artist(artist)
            && !is_short_title(title)
            && Self::preferred_song(song_all, title, length, artist)
                .first()
                .map(|c| artist_score(artist, &c.artist) == 0)
                .unwrap_or(false);
        if need_extra {
            if let Ok(extra) = self.search_music(&format!("{title} {artist}")).await {
                for s in extra {
                    if !song_all.iter().any(|m| m.key == s.key) {
                        song_all.push(s);
                    }
                }
            }
        }

        let song = Self::preferred_song(song_all, title, length, artist);
        if song.is_empty() {
            return Ok(None);
        }

        // 首选候选须可靠才采用:
        // - 标题+版本评分过低(与目标歌曲明显无关)不采用
        // - artist 完全无关的候选不采用(避免同名歌冒充; 通用署名除外)
        // 豁免: 候选是补充搜索前的原始第一名、时长差 <=8s 且标题非完全一致,
        // 兼容跨写法等平台命名差异(同名歌不在豁免范围内)
        let top = song.first().expect("ranked list is not empty");
        let title_s = title_score(title, &top.title);
        let version_s = version_adjust(title, &top.title, title_s);
        let artist_required = !artist.trim().is_empty() && !is_generic_artist(artist);
        let artist_ok = !artist_required || artist_score(artist, &top.artist) > 0;
        let exempt = length != NO_LENGTH
            && title_s < 90
            && top.key == original_first_key
            && top.length.abs_diff(length) <= GOOD_LENGTH_DIFF;
        if (title_s + version_s < MIN_TITLE_SCORE || !artist_ok) && !exempt {
            debug!(
                "首选候选不可靠(标题{title_s} 版本{version_s} 歌手匹配{artist_ok}), 放弃该源: {}",
                top.title
            );
            return Ok(None);
        }

        for info in &song {
            // 回退范围: 只允许尝试与首选同曲的候选(标题高度一致且歌手相关),
            // 不因同名/标题部分相似/时长接近而跨到其他歌曲
            if !is_same_song(top, info) {
                debug!("跳过与首选不同曲的候选: {}", info.title);
                continue;
            }
            // 单个候选获取失败不中断, 继续尝试同曲的下一个候选
            let lyrics = match self.fetch_lyrics(&info.key).await {
                Ok(lyrics) => lyrics,
                Err(err) => {
                    error!("获取歌词失败(候选 {}): {}", info.key, err);
                    continue;
                }
            };
            if lyrics.is_none() {
                continue;
            }
            // 质量门槛: <=4 行视为残缺/不可用歌词, 换下一个同曲候选
            match <LyricResult as TryInto<Lyric>>::try_into(lyrics.clone()) {
                Ok(lyric) if lyric.get_lyrics().len() > 4 => return Ok(Some(lyrics)),
                Ok(_) => debug!("歌词过短(<=4行), 跳过候选 {}", info.key),
                Err(err) => debug!("歌词解析失败, 跳过候选 {}: {}", info.key, err),
            }
        }

        Ok(None)
    }
}

/// 标题有效字符数过少视为"短/常见"标题
fn is_short_title(title: &str) -> bool {
    let count = title.chars().filter(|c| c.is_alphanumeric()).count();
    count <= SHORT_TITLE_CHARS
}

#[cfg(test)]
mod tests {
    use super::*;

    const TITLE: &str = "Clair de lune";
    const ARTIST: &str = "Debussy";

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

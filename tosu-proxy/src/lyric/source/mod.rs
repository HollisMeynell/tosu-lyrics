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

// ---------------------------------------------------------------------------
// 搜索 query 长度安全检查
//
// 背景: `title + artist` 会随 `search_music` 直接拼进各家搜索接口的 URL
// (见 `qq.rs::search_url` / `netease.rs::search_url`)。两个源都是字符串拼接,
// 代码里**没有任何长度上限**, 而超长 query 的服务端行为是"静默失败":
// QQ 直接返回 400, NetEase 返回 HTTP 200 + body `{"code":400}`
// (后者会被现有代码当成"无结果", 完全无声)。
//
// 因此这里在**发出请求之前**先判断长度, 超长就直接退化为"只搜 title"。
// 注意长度必须是 **percent-encoding 之后**的 URL 字符数, 而不是
// `String::len()`(UTF-8 字节数): 日文/中文/韩文一个字符 3 字节,
// 编码后膨胀成 9 个 URL 字符(实测同一 encoded 长度下 ASCII 与日文行为一致)。
// ---------------------------------------------------------------------------

/// QQ 搜索 URL 的固定前缀 —— 与 `qq.rs::search_url` 逐字一致, 仅用于测量。
const QQ_SEARCH_URL_PREFIX: &str =
    "https://c.y.qq.com/soso/fcgi-bin/client_search_cp?p=1&n=5&format=json&w=";

/// NetEase 搜索 URL 的固定前缀 —— 与 `netease.rs::search_url` 逐字一致, 仅用于测量。
const NETEASE_SEARCH_URL_PREFIX: &str = "https://music.163.com/api/search/get?s=";

/// NetEase 搜索 URL 里跟在 query **之后**的固定部分。
/// 必须计入长度, 否则测量结果会比重测边界小 15 个字符。
const NETEASE_SEARCH_URL_SUFFIX: &str = "&type=1&limit=5";

/// QQ 搜索接口可接受的 URL 长度安全上限(**含固定前缀/后缀的整条 URL**)。
///
/// 实测依据(真实请求, 纯 ASCII query, 在边界附近逐 100 字符扫描):
/// ```text
/// URL 9772 字符  → HTTP 200, 正常 JSON
/// URL 9872 字符  → HTTP 400(服务端拒绝)
/// URL 18085 字符 → HTTP 414 Request-URI Too Large
/// ```
/// 取 8000 作为保守上限: 比实测失败边界低约 18%, 给网关/负载变化留余量。
const QQ_SEARCH_URL_MAX: usize = 8000;

/// NetEase 搜索接口可接受的 URL 长度安全上限(**含固定前缀/后缀的整条 URL**)。
///
/// 实测依据(真实请求, 纯 ASCII query):
/// ```text
/// URL 8154 字符 → HTTP 200 且业务正常(code=200, 返回歌曲列表)
/// URL 8204 字符 → HTTP 200 但 body = {"code":400,"message":"请求解析失败!"}
/// ```
/// 注意 NetEase **不返回 4xx**, 而是用 HTTP 200 + 业务错误码表达失败,
/// 现有代码会把它当"无结果", 所以超长 query 是**静默**退化的。
/// 取 7000 作为保守上限: 比实测失败边界低约 14%。
const NETEASE_SEARCH_URL_MAX: usize = 7000;

/// 按来源名返回 (URL 前缀, URL 后缀, 安全上限)。未知来源返回 `None`,
/// 调用方必须按"不安全"处理(即不发 title+artist)。
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

/// 用与真实请求**完全相同**的方式测量整条 URL 的字符数(query 已按 URL 规则编码)。
///
/// 直接借项目已有的 `reqwest::Url` 解析, 因此与真实请求走同一套 url crate 的
/// percent-encoding 规则(不用手写编码器, 避免规则不一致)。
/// 解析失败(例如 query 含控制字符)返回 `None` ⇒ 调用方按不安全处理。
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

/// 版本标识关键词(remix/remaster/live/tv size/version 等), 用于区分同曲不同版本。
///
/// 词表收边统一用 `\b`, 因此**不要把 `\.` 这类非词字符写进词条** —— `ver.`
/// 这种写法由 `ver` 命中即可; 把 `\.` 放进 `\b...\b` 之间反而会因为句点后
/// 不再是词边界而永远匹配不上。
static VERSION_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(remastered|remaster|remix|live|tv[ -]?size|tv[ -]?ver|short[ -]?ver|game[ -]?ver|full[ -]?ver|version|ver|instrumental|inst|off[ -]?vocal|acoustic|cover|edit|original|sped[ -]?up|slowed|nightcore)\b",
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

/// 剥掉版本标识后的"核心标题"(已归一化)。
///
/// 用来判断"同一首歌的两种版本标注": 核心标题一致时, 版本差异交给
/// `version_adjust` 决定排序, 而不是让候选因为版本词字面不同直接出局。
fn core_title(s: &str) -> String {
    normalize_title(&VERSION_TAG_RE.replace_all(s, " "))
}

/// 去掉标题里成对括号及其内容(版本 / 混音 / 出处等附录), 保留主干。
///
/// `os-宇宙人(Asterisk Makina Remix)` -> `os-宇宙人`
///
/// 为什么不能只靠 `core_title`: 括号里常带**混音师名**(如 `Asterisk Makina`),
/// 那不是版本词, 剥版本词后主干仍与目标不同, 于是候选被压到 60 分。
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
    // 只有附录/版本标注不同: 核心标题一致即视为同一首歌并进入评分, 具体
    // 版本差异交给 version_adjust 排序。不给 100 是为了让标注完全一致的候选
    // 仍然优先。
    //
    // 两种情况都要覆盖:
    // - 括号附录不同: `os-宇宙人(Asterisk Makina Remix)` 对 `os-宇宙人`
    //   (括号里常常只是混音师名, 剥版本词剥不掉)
    // - 没有括号、只是版本词不同: `Song - Remastered` 对 `Song`
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
    // 查询包含候选(候选缺少查询中的版本标签, 如 Remix 查到了原版)
    if q.contains(&c) {
        return 60;
    }
    // 字级别 bigram 相似度兜底
    20 + (bigram_similarity(&q, &c) * 50.0) as i32
}

/// artist 串里视为"分隔"的标点。
///
/// 全角括号/方括号必须在内: osu 的 `ArtistUnicode` 常用
/// `篠澤広（CV: 川村玲奈）` 这种写法, 不拆开就会要求候选 artist 完整包含
/// `篠澤広cv川村玲奈`, 命中率几乎为 0。
const ARTIST_SEPARATORS: [char; 18] = [
    '&', ',', '/', '、', ';', '；', '(', ')', '（', '）', '[', ']', '【', '】', '×', '・', '·', '|',
];

/// 视为"合作连接词"的整词, 只要求**前导**空格。
///
/// 两点原因:
/// - 不拆裸 `x`/`X`: `Xceon`、`xi` 这类名字里的字母会被误伤, 只有两侧带空格的
///   ` x ` 才是 "A x B" 式的合作署名
/// - 不要求尾随空格: `Xceon feat.森永真由美` 这种中日文写法里 `feat.` 后面直接
///   接名字, 要求尾空格会完全匹配不上(旧实现用的就是 `find(" feat.")`)
const ARTIST_JOINERS: [&str; 7] = [
    " feat.", " feat ", " vs.", " vs ", " x ", " with ", " meets ",
];

/// 把 artist 串切成多个可比片段(任一命中即可得分)。
fn artist_parts(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|ch: char| ARTIST_SEPARATORS.contains(&ch))
        .flat_map(split_artist_joiners)
        .map(|s| s.trim().to_string())
        .filter(|t| t.chars().count() >= 2)
        .collect()
}

/// 拆分 ` feat. ` / ` x ` / ` with ` 这类合作连接词。
///
/// 先把片段统一小写再取索引: `find` 返回的字节下标必须与切片的字符串一致
/// (`to_lowercase` 有可能改变字节长度, 用原串下标会切在字符中间而 panic)。
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

/// artist 相似度软评分(0-20): 命中片段比例越高分越高, 不做任何硬性判定
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
/// 只有确认候选与目标歌曲本身相关(标题相等/包含/核心标题一致)时版本信息才参与评分,
/// 避免完全无关的歌曲因为词面含 "remix" 等关键词而获得加分。
///
/// 三种不对称情况需要分开处理:
/// - 目标无版本词、候选有(平台常自行标注 Remastered / TV Size 等): 不能据此判定
///   "版本不同", 保持中性; 版本差异已由 `title_score` 的 100 与 90 之分体现
/// - 目标要特定版本、候选没有: 候选很可能是原版, 轻度降权让版本一致的候选优先
/// - 双方都有版本词: 有交集加分, 无交集(如 Remix 对上 Live)降权
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

/// 判断候选是否与首选属于同一首歌(抓词失败后的回退范围):
/// 标题高度一致(相等或候选包含首选标题)且歌手相关;
/// 同名但歌手完全无关的候选不算同一首歌。
///
/// 例外: 标题归一化后完全一致、且时长不冲突时, 即使 artist 写法不同
/// (CV 标注 / 罗马字 / 只写了合作者之一)也视为同一首 —— 否则同曲的其它候选
/// 会因为 artist 的表述差异被全部跳过, 正确歌词失去回退机会。
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
        ranked.sort_unstable_by_key(|(_, score)| std::cmp::Reverse(*score));
        ranked.into_iter().map(|(song, _)| song).collect()
    }

    /// 搜索策略:
    /// - artist 可用: **并发**发出「纯 title」与「title + artist」两条查询,
    ///   合并去重(纯 title 结果在前, 与原先"补充搜索结果追加到末尾"的顺序一致)
    /// - artist 不可用: 只搜 title
    /// - `title + artist` 超过该源 URL 长度安全上限时: **完全不发**这条查询,
    ///   只用 title(上限与依据见文件顶部的 `QQ_SEARCH_URL_MAX` / `NETEASE_SEARCH_URL_MAX`)
    /// `length` 使用 毫秒数
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

        // ---- 安全检查: title+artist 是否长到不能发给这个源 ----
        //
        // 超长时**完全不发**这条查询: 不截断 title、不截断 artist、
        // 也不用"先发出去等服务端报错再 fallback"的做法(那会白多一次失败请求)。
        let within_limit = match search_url_limit(self.name()) {
            Some((prefix, suffix, max)) => {
                match encoded_search_url_len(prefix, &with_artist, suffix) {
                    Some(len) => len <= max,
                    None => false, // 无法测量 ⇒ 保守地不发
                }
            }
            None => false, // 未知来源 ⇒ 保守地不发
        };
        if !within_limit {
            debug!(
                "[perf] (s) {} 分支=超长跳过 title+artist(仅用 title) query_chars={}",
                self.name(),
                with_artist.chars().count(),
            );
            return self.search_music(title).await;
        }

        // ---- title 与 title+artist **并发**发出 ----
        //
        // 原先只有"短标题"走并发; 非短标题先只搜 title, 返回空才**串行**兜底搜
        // title+artist, 而 `search_lyrics` 里的 `need_extra` 还会再**串行**补一次。
        // 单源搜索实测约 2.2~3.4 秒(QQ 侧), 那条串行往返会让墙钟直接翻倍
        // (实测 CANNIBALISM: 2769ms + 2928ms + 104ms = 5805ms)。
        //
        // 这里两条查询的字符串与原先兜底/need_extra 用的**完全相同**,
        // 合并顺序(纯 title 结果在前)也相同, 所以候选集合与排序结果不变,
        // 只是省掉了一次串行网络往返。
        debug!(
            "[perf] (s) {} 分支=title+artist 并发 query_chars={}",
            self.name(),
            with_artist.chars().count(),
        );
        let (by_title, by_artist) = tokio::join!(
            self.search_music(title),
            self.search_music(&with_artist)
        );
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
            // `title + artist` 的结果**已经**由 `search_all_music` 并发取回并合并进
            // `song_all`(查询串、去重规则、合并顺序都相同), 所以这里不再发第二次请求。
            //
            // 原先这里会**串行**补搜一次 title+artist —— 单源实测 2.2~3.4 秒(QQ 侧),
            // 正是"3 秒变 6 秒"的另一半(实测 CANNIBALISM 的 2928ms)。
            // 该判定保留, 但只用于日志可观测性。
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

        // 首选候选须可靠才采用:
        // - 标题+版本评分过低(与目标歌曲明显无关)不采用
        // - 标题不够强时才要求 artist 命中(避免同名歌冒充; 通用署名除外)
        // 豁免: 候选是补充搜索前的原始第一名、时长差 <=8s 且标题非完全一致,
        // 兼容跨写法等平台命名差异(同名歌不在豁免范围内)
        let top = song.first().expect("ranked list is not empty");
        let title_s = title_score(title, &top.title);
        let version_s = version_adjust(title, &top.title, title_s);
        let artist_required = !artist.trim().is_empty() && !is_generic_artist(artist);
        let artist_ok = !artist_required || artist_score(artist, &top.artist) > 0;
        // 标题强匹配(完全一致 / 候选含完整目标标题 / 只差一个附录)时 artist 只参与
        // 排序、不再否决: `篠澤広（CV: 川村玲奈）` 这类 CV 标注、罗马字/假名差异、
        // 平台只写合作者之一的情况很常见, 不应该因为 artist 字面不同就把标题已经
        // 对上的候选丢掉。
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

        // [perf] (10) 纯日志：准入判定结果 + 接下来会串行发几个取词请求
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
            // 回退范围: 只允许尝试与首选同曲的候选(标题高度一致且歌手相关),
            // 不因同名/标题部分相似/时长接近而跨到其他歌曲
            if !is_same_song(top, info) {
                debug!("跳过与首选不同曲的候选: {}", info.title);
                continue;
            }
            // 单个候选获取失败不中断, 继续尝试同曲的下一个候选
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
            // 质量门槛: <=4 行视为残缺/不可用歌词, 换下一个同曲候选
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

    // ---------- 搜索 query 长度安全检查（优化 2, 纯逻辑, 不联网） ----------
    //
    // 上限的实测依据见文件顶部 `QQ_SEARCH_URL_MAX` / `NETEASE_SEARCH_URL_MAX`。

    /// 长度必须按 **percent-encoding 之后**的整条 URL 字符数计算,
    /// 而不是 `String::len()`（UTF-8 字节数）。
    #[test]
    fn encoded_len_counts_percent_encoding_not_bytes() {
        let (prefix, suffix, _) = search_url_limit("QQ").unwrap();
        let base = prefix.len() + suffix.len();

        // 10 个日文字符 = 30 字节 → percent-encoding 后 90 个 URL 字符
        let ja = encoded_search_url_len(prefix, &"あ".repeat(10), suffix).unwrap();
        assert_eq!(ja - base, 90, "日文 10 字符应膨胀为 90 个 URL 字符");

        // 纯 ASCII 不膨胀
        let ascii = encoded_search_url_len(prefix, &"a".repeat(10), suffix).unwrap();
        assert_eq!(ascii - base, 10);

        // 真实日文歌曲的 query 远低于上限, 不能被误判成超长
        let real = encoded_search_url_len(
            prefix,
            "病み垢ステロイド かいりきベア feat. 初音ミク",
            suffix,
        )
        .unwrap();
        assert!(real < QQ_SEARCH_URL_MAX, "真实日文 query 不应被误判超长: {real}");
    }

    /// 实测**失败**的长度必须落在上限之外；上限必须严格低于实测失败边界。
    #[test]
    fn limit_is_below_measured_failure_boundary() {
        // QQ 实测: URL 9772 成功 / 9872 → HTTP 400
        let (qq_prefix, qq_suffix, qq_max) = search_url_limit("QQ").unwrap();
        let qq_fail =
            encoded_search_url_len(qq_prefix, &"a".repeat(9800), qq_suffix).unwrap();
        assert_eq!(qq_fail, 9872, "前缀长度变化会破坏与实测边界的对应关系");
        assert!(qq_fail > qq_max, "实测失败的长度必须被拒绝");

        // NetEase 实测: URL 8154 成功 / 8204 → HTTP 200 + body {{\"code\":400}}
        let (ne_prefix, ne_suffix, ne_max) = search_url_limit("Netease").unwrap();
        let ne_fail =
            encoded_search_url_len(ne_prefix, &"a".repeat(8150), ne_suffix).unwrap();
        assert_eq!(ne_fail, 8204, "NetEase 的固定后缀必须计入长度");
        assert!(ne_fail > ne_max, "实测失败的长度必须被拒绝");

        // 两个源的上限必须分别存在且互不混用
        assert_ne!(qq_max, ne_max);
    }

    /// 未知来源必须按"不安全"处理, 不能因为查不到上限就放行。
    #[test]
    fn unknown_source_is_treated_as_unsafe() {
        assert!(search_url_limit("QQ").is_some());
        assert!(search_url_limit("Netease").is_some());
        assert!(search_url_limit("SomeFutureSource").is_none());
    }

    /// 含特殊字符的真实标题不得让 URL 解析失败
    /// （解析失败会被保守地判为"不许发 title+artist"）。
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

    /// 极端超长的 query 必须被判为超限（两种字符集都要测）。
    #[test]
    fn oversized_query_is_rejected_for_both_sources() {
        for q in ["a".repeat(20000), "あ".repeat(3000)] {
            let (qp, qs, qmax) = search_url_limit("QQ").unwrap();
            let (np, ns, nmax) = search_url_limit("Netease").unwrap();
            assert!(encoded_search_url_len(qp, &q, qs).unwrap() > qmax);
            assert!(encoded_search_url_len(np, &q, ns).unwrap() > nmax);
        }
    }

    // ---------- 评分算法（纯逻辑, 不联网） ----------
    //
    // 覆盖"标题/artist 写法差异导致正确歌词被丢掉"的几类真实输入,
    // 其中带曲名的用例取自实际反馈的谱面 (TitleUnicode / ArtistUnicode)。

    /// 标题完全一致 → 满分
    #[test]
    fn title_exact_match_is_full_score() {
        assert_eq!(title_score("メクルメ", "メクルメ"), 100);
        assert_eq!(title_score("Yumemi Sunrise", "yumemi sunrise"), 100);
    }

    /// 候选带平台自行追加的说明 → 90, 仍属"强匹配"
    #[test]
    fn title_candidate_with_suffix_is_strong() {
        assert_eq!(title_score("メクルメ", "メクルメ (Game Ver.)"), 90);
        assert_eq!(title_score("Song", "Song (Remastered)"), 90);
    }

    /// 只有附录/版本标注不同 → 85: 进入评分, 不再被压到 60
    #[test]
    fn title_differing_only_by_appendix_is_85() {
        // 括号里是混音师名(不是版本词), 只能靠"剥括号"识别
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
        // 没有括号、只是版本词不同
        assert_eq!(title_score("Song - Remastered", "Song"), 85);
    }

    /// 版本词表的覆盖面: 反馈里出现的几类写法都要识别
    #[test]
    fn version_tags_cover_reported_forms() {
        assert!(!version_tags("os-宇宙人(Asterisk Makina Remix)").is_empty());
        assert!(!version_tags("Song (Remastered)").is_empty());
        assert!(!version_tags("Song (TV Size)").is_empty());
        assert!(!version_tags("Song (Short Ver.)").is_empty());
        assert!(!version_tags("Song (Game Version)").is_empty());
        assert!(version_tags("メクルメ").is_empty());
        // 普通单词里含 ver/inst 字母不应被误判
        assert!(version_tags("Veronica").is_empty());
        assert!(version_tags("Instinct").is_empty());
    }

    /// 主干标题: 剥括号 / 剥版本词
    #[test]
    fn title_trunk_helpers() {
        assert_eq!(
            strip_bracket_content("os-宇宙人(Asterisk Makina Remix)"),
            "os-宇宙人"
        );
        assert_eq!(strip_bracket_content("Song（TV Size）"), "Song");
        assert_eq!(core_title("Song - Remastered"), core_title("Song"));
    }

    /// 无关标题仍应低分(放宽后不能误匹配)
    #[test]
    fn unrelated_title_stays_low() {
        assert!(title_score("メクルメ", "夜に駆ける") < MIN_TITLE_SCORE);
        assert!(title_score("Yumemi Sunrise", "Wah Wah World") < MIN_TITLE_SCORE);
    }

    /// 全角括号的 CV 标注必须拆开: 否则候选只写主 artist 时命中率恒为 0
    #[test]
    fn artist_cv_annotation_is_split() {
        let score = artist_score("篠澤広（CV: 川村玲奈）", "篠澤広");
        assert!(score > 0, "CV 标注应拆成独立片段, 实际得分 {score}");
    }

    /// ` feat.` / ` feat ` / ` x ` 形式的合作署名要能部分命中
    #[test]
    fn artist_cooperation_forms_hit_partially() {
        assert!(artist_score("Xceon feat.森永真由美", "Xceon") > 0);
        assert!(artist_score("Xceon feat.森永真由美", "森永真由美") > 0);
        assert!(artist_score("Giga x Mitchie M", "Giga") > 0);
        assert!(artist_score("Giga x Mitchie M", "Mitchie M") > 0);
    }

    /// 连接词只在独立成词时拆分, 不能拆散 `Xceon`
    #[test]
    fn artist_joiners_keep_embedded_letters() {
        assert_eq!(artist_parts("Xceon"), vec!["xceon".to_string()]);
        assert_eq!(artist_score("Xceon", "Xceon"), 20);
    }

    /// artist 分数边界
    #[test]
    fn artist_score_bounds() {
        assert_eq!(artist_score("歌組雪月花", "歌組雪月花"), 20);
        assert_eq!(artist_score("分島花音", "Someone Else"), 0);
        assert_eq!(artist_score("", "Anyone"), 0);
    }

    /// 版本调整的几种不对称情况
    #[test]
    fn version_adjust_is_asymmetric() {
        // 目标没有版本词、候选有(平台自行标注): 中性, 不再扣 10
        assert_eq!(version_adjust("Song", "Song (Remastered)", 90), 0);
        // 目标要特定版本、候选没有: 降权
        assert_eq!(version_adjust("Song (Remix)", "Song", 85), -10);
        // 双方都有且一致: 加分
        assert_eq!(version_adjust("Song (Remix)", "Song (Remix)", 100), 10);
        // 双方都有但冲突: 降权
        assert_eq!(version_adjust("Song (Remix)", "Song (Live)", 85), -10);
        // 标题本身就不相关时不参与版本评分
        assert_eq!(version_adjust("Song (Remix)", "Other (Live)", 30), 0);
    }

    /// 抓词回退: artist 写法不同但标题完全一致时仍视为同一首
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

        // 标题不同则不算同一首
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

use super::{CLIENT, LyricResult, LyricSource, SongInfo};
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::Deserialize;

const NETEASE_API_BASE: &str = "https://music.163.com/api";

pub struct NeteaseLyricSource;

// 歌曲搜索
#[derive(Deserialize)]
struct NeteaseResponse {
    code: i32,
    result: NeteaseSearchResult,
}

#[derive(Deserialize)]
struct NeteaseSearchResult {
    #[serde(default)]
    songs: Option<Vec<NeteaseSong>>,
}

#[derive(Deserialize)]
struct NeteaseSong {
    id: i64,
    name: String,
    artists: Vec<NeteaseArtist>,
    duration: u32,
}

#[derive(Deserialize)]
struct NeteaseArtist {
    name: String,
}

// 歌词搜索
#[derive(Deserialize)]
struct NeteaseLyricResponse {
    #[serde(default)]
    #[serde(rename = "pureMusic")]
    pure_music: bool,
    code: i32,
    #[serde(default)]
    lrc: Option<LyricItem>,
    #[serde(default)]
    tlyric: Option<LyricItem>,
}

#[derive(Deserialize)]
struct LyricItem {
    lyric: String,
}

impl Default for NeteaseLyricSource {
    fn default() -> Self {
        Self
    }
}

impl NeteaseLyricSource {
    /// 构建根据标题搜索歌曲的 URL。
    fn search_url(title: &str) -> String {
        // TODO: 标题可能包含特殊字符（如 &、? 等），应进行 URL 编码
        // use url::form_urlencoded;
        // let encoded_title = form_urlencoded::byte_serialize(title.as_bytes()).collect::<String>();
        format!(
            "{}/search/get?s={}&type=1&limit=5",
            NETEASE_API_BASE,
            title // 如果编码就要用 encoded_title
        )
    }

    /// 构建根据歌曲 ID 获取歌词的 URL。
    fn lyric_url(song_id: &str) -> String {
        format!(
            "{}/song/lyric?id={}&lv=1&kv=1&tv=-1",
            NETEASE_API_BASE, song_id
        )
    }
}

#[async_trait]
impl LyricSource for NeteaseLyricSource {
    fn name(&self) -> &str {
        "Netease"
    }

    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>> {
        let url = Self::search_url(title);
        let response = CLIENT.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let result: NeteaseResponse = response.json().await?;

        // 处理错误代码
        if result.code != 200 {
            return Ok(vec![]);
        }

        let song_list = result
            .result
            .songs
            .unwrap_or_default()
            .into_iter()
            .map(|song| SongInfo {
                title: song.name,
                artist: song
                    .artists
                    .into_iter()
                    .map(|a| a.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                length: song.duration,
                key: song.id.to_string(),
            })
            .collect();
        Ok(song_list)
    }

    /// 获取指定网易云歌曲 ID 的原文和翻译歌词。
    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult> {
        let url = Self::lyric_url(song_id);
        let response = CLIENT.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::Runtime(format!(
                "请求失败: {} {}",
                response.status(),
                response.text().await?,
            )));
        }

        let result: NeteaseLyricResponse = response.json().await?;

        if result.code != 200 {
            return Err(Error::Runtime(format!("Netease API 错误: {}", result.code)));
        }

        if result.pure_music {
            return Ok(LyricResult {
                lyric: None,
                trans: None,
            });
        }

        let lyric = result
            .lrc
            .and_then(|item| Some(item.lyric))
            .filter(|s| !s.is_empty());

        let trans = result
            .tlyric
            .and_then(|item| Some(item.lyric))
            .filter(|s| !s.is_empty());
        Ok(LyricResult { lyric, trans })
    }
}

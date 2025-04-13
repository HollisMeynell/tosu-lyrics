use super::{CLIENT, LyricResult, LyricSource, SongInfo};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct NeteaseLyricSource;

// 歌曲搜索
#[derive(Deserialize)]
struct NeteaseResponse {
    code: i32,
    result: NeteaseResult,
}

#[derive(Deserialize)]
struct NeteaseResult {
    songs: Vec<NeteaseSong>,
}

#[derive(Deserialize)]
struct NeteaseSong {
    name: String,
    artists: Vec<NeteaseArtist>,
    duration: u32,
    id: i64,
}

#[derive(Deserialize)]
struct NeteaseArtist {
    name: String,
}

// 歌词搜索
#[derive(Deserialize)]
struct NeteaseLyricResponse {
    lrc: LyricItem,
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
    fn search_url(title: &str) -> String {
        format!(
            "https://music.163.com/api/search/get?s={}&type=1&limit=5",
            title
        )
    }

    fn lyric_url(song_id: &str) -> String {
        format!(
            "https://music.163.com/api/song/lyric?id={}&lv=1&kv=1&tv=-1",
            song_id
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
        let result: NeteaseResponse = CLIENT.get(&url).send().await?.json().await?;

        if result.code != 200 {
            return Ok(vec![]);
        }

        let song_list = result
            .result
            .songs
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

    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult> {
        let url = Self::lyric_url(song_id);
        let result: NeteaseLyricResponse = CLIENT.get(&url).send().await?.json().await?;

        let lyric = if result.lrc.lyric.is_empty() {
            None
        } else {
            Some(result.lrc.lyric)
        };
        let trans = match result.tlyric {
            None => None,
            Some(l) => {
                if l.lyric.is_empty() {
                    None
                } else {
                    Some(l.lyric)
                }
            }
        };
        Ok(LyricResult { lyric, trans })
    }
}

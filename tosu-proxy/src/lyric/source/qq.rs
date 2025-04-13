use super::{CLIENT, LyricResult, LyricSource, SongInfo};
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct QQLyricSource;

// 歌曲搜索
#[derive(Deserialize)]
struct QQResponse {
    code: i32,
    data: QQData,
}

#[derive(Deserialize)]
struct QQData {
    song: SongList,
}

#[derive(Deserialize)]
struct SongList {
    list: Vec<QQSong>,
}

#[derive(Deserialize)]
struct QQSong {
    songname: String,
    singer: Vec<Singer>,
    interval: u32,
    songmid: String,
}

#[derive(Deserialize)]
struct Singer {
    name: String,
}

// 歌词搜索
#[derive(Deserialize)]
struct QQLyricResponse {
    code: i32,
    lyric: String,
    trans: String,
}

impl Default for QQLyricSource {
    fn default() -> Self {
        Self
    }
}

impl QQLyricSource {
    fn search_url(title: &str) -> String {
        format!(
            "https://c.y.qq.com/soso/fcgi-bin/client_search_cp?p=1&n=10&format=json&w={}",
            title
        )
    }

    fn lyric_url(song_id: &str) -> String {
        format!(
            "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg?songmid={}&format=json&nobase64=1",
            song_id
        )
    }
}

#[async_trait]
impl LyricSource for QQLyricSource {
    fn name(&self) -> &str {
        "QQ"
    }

    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>> {
        let url = Self::search_url(title);
        let result = CLIENT
            .get(&url)
            .header("Referer", "https://y.qq.com/portal/player.html")
            .send()
            .await?
            .json::<QQResponse>()
            .await?;

        if result.code != 0 {
            return Ok(vec![]);
        }

        let song_list = result
            .data
            .song
            .list
            .into_iter()
            .map(|song| SongInfo {
                title: song.songname,
                artist: song
                    .singer
                    .into_iter()
                    .map(|s| s.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                // 这里腾讯给的是秒数, 转换一下
                length: song.interval * 1000,
                key: song.songmid,
            })
            .collect();
        Ok(song_list)
    }

    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult> {
        let url = Self::lyric_url(song_id);
        let result: QQLyricResponse = CLIENT
            .get(&url)
            .header("Referer", "https://y.qq.com/portal/player.html")
            .send()
            .await?
            .json()
            .await?;
        // 腾讯你是不是有点大病
        let lyric = if result.lyric.is_empty() || result.lyric.contains("此歌曲为没有填词的纯音乐")
        {
            None
        } else {
            Some(result.lyric)
        };
        let trans = if result.trans.is_empty() {
            None
        } else {
            Some(result.trans)
        };

        Ok(LyricResult { lyric, trans })
    }
}

mod netease;
mod qq;

use super::Lyric;
use crate::error::{Error, Result};
use async_trait::async_trait;
pub use netease::NeteaseLyricSource;
pub use qq::QQLyricSource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::LazyLock;

const NO_LENGTH: u32 = 0;
const ALLOW_OFFSET: u32 = 15000;

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
    fn preferred_song<'a>(
        songs: &'a [SongInfo],
        title: &str,
        length: u32,
        artist: &str,
    ) -> Vec<&'a SongInfo> {
        songs
            .iter()
            .filter(|&s| Self::song_filter_length(s, length))
            .filter(|&s| Self::song_filter_title(s, title))
            .filter(|&s| Self::song_filter_artist(s, artist))
            .collect()
    }

    /// 使用策略: 先搜 title artist, 搜不到转 只搜 title
    /// `length` 使用 毫秒数
    async fn search_all_music(&self, title: &str, artist: &str) -> Result<Vec<SongInfo>> {
        let mut song_all = self.search_music(&format!("{title} {artist}")).await?;
        if song_all.is_empty() {
            let other = self.search_music(title).await?;
            song_all.extend(other);
        }
        Ok(song_all)
    }

    /// length 设为 0, 不应用过滤
    async fn search_lyrics(
        &self,
        song_all: &[SongInfo],
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<Option<LyricResult>> {
        if song_all.is_empty() {
            return Ok(None);
        }
        let song = Self::preferred_song(song_all, title, length, artist);
        if song.is_empty() {
            return Ok(None);
        }

        for info in song {
            let lyrics = self.fetch_lyrics(&info.key).await?;
            if !lyrics.is_none() {
                return Ok(Some(lyrics));
            }
        }

        Ok(None)
    }

    fn song_filter_length(song: &SongInfo, length: u32) -> bool {
        if length == NO_LENGTH {
            return true;
        }
        let diff = song.length.abs_diff(length);
        diff <= ALLOW_OFFSET
    }

    fn song_filter_title(song: &SongInfo, title: &str) -> bool {
        // todo: 要用 nlp 来处理字符串相似度吗?
        true
    }

    fn song_filter_artist(song: &SongInfo, artist: &str) -> bool {
        // todo: 同上
        true
    }
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

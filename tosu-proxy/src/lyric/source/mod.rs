mod netease;
mod qq;

use crate::error::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use salvo::async_trait;
pub use netease::NeteaseLyricSource;
pub use qq::QQLyricSource;

pub static QQ_LYRIC_SOURCE: LazyLock<QQLyricSource> = LazyLock::new(|| QQLyricSource::default());

pub static NETEASE_LYRIC_SOURCE: LazyLock<NeteaseLyricSource> =
    LazyLock::new(|| NeteaseLyricSource::default());

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
});

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

#[async_trait]
pub trait LyricSource {
    fn name(&self) -> &str;
    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>>;
    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult>;
    fn preferred_song<'a>(
        songs: &'a Vec<SongInfo>,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Option<&'a SongInfo> {
        // todo: 选择合适的曲子, 目前只是第一个
        songs.first()
    }
    /// `length` 使用 毫秒数
    async fn search_lyrics(
        &self,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<Option<LyricResult>> {
        let song_all = self.search_music(&format!("{title}-{artist}")).await?;
        if song_all.is_empty() {
            return Ok(None);
        }
        let song = Self::preferred_song(&song_all, title, length, artist);
        if song.is_none() {
            return Ok(None);
        }
        let info = song.unwrap();
        let lyrics = self.fetch_lyrics(&info.key).await?;
        if lyrics.lyric.is_none() {
            Ok(None)
        } else {
            Ok(Some(lyrics))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qq_lyric_source() -> Result<()> {
        let song_info = QQ_LYRIC_SOURCE.search_music("Clair de lune").await?;
        let song = song_info.first().ok_or::<Error>("not found song".into())?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = QQ_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_netease_lyric_source() -> Result<()> {
        let song_info = NETEASE_LYRIC_SOURCE.search_music("Clair de lune").await?;
        let song = song_info.first().ok_or::<Error>("not found song".into())?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = NETEASE_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }
}

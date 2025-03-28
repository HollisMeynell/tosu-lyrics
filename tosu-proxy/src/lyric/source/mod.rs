mod netease;
mod qq;

use crate::error::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

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
    pub length: u64,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricResult {
    pub lyric: Option<String>,
    pub trans: Option<String>,
}

pub trait LyricSource {
    fn name(&self) -> &str;
    async fn search_music(&self, title: &str) -> Result<Vec<SongInfo>>;
    async fn fetch_lyrics(&self, song_id: &str) -> Result<LyricResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qq_lyric_source() -> Result<()> {
        let song_info = QQ_LYRIC_SOURCE.search_music("月光鸣奏").await?;
        let song = song_info.first().ok_or::<Error>("not found song".into())?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = QQ_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_netease_lyric_source() -> Result<()> {
        let song_info = NETEASE_LYRIC_SOURCE.search_music("稻香").await?;
        let song = song_info.first().ok_or::<Error>("not found song".into())?;
        let song_name = &song.title;
        let key = &song.key;
        let lyric = NETEASE_LYRIC_SOURCE.fetch_lyrics(key).await?;
        println!("{song:?}\n{lyric:?}");
        Ok(())
    }
}

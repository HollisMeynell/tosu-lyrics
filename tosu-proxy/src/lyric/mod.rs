mod lyric;
mod source;

use crate::config::Settings;
use crate::error::Result;
pub use lyric::*;
pub use source::*;

pub struct LyricService {}

impl LyricService {
    /// 时间单位为毫秒
    pub async fn time_next(&mut self, t: i32) -> Result<()> {
        Ok(())
    }
    pub async fn song_change(&mut self, title: &str) -> Result<()> {
        Ok(())
    }
}

pub async fn init_lyric() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;
    #[tokio::test]
    async fn test_parse_lyric() -> Result<()> {
        let title = "稻香";
        let mut lyric = QQ_LYRIC_SOURCE
            .search_lyrics(title, 203000, "周杰伦")
            .await?
            .ok_or::<Error>("没找到对应歌曲".into())?;
        let l1 = &lyric.lyric.unwrap();
        let l2 = lyric.trans.as_deref();
        let l = Lyric::parse(&l1, l2, Some(title))?;
        println!("{l:?}");
        Ok(())
    }
}

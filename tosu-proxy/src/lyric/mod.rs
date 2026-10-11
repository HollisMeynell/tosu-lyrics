mod lyric_source;
mod source;

use crate::error::{Error, Result};
pub use lyric_source::*;
use sea_orm::EntityTrait;
pub use source::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;
    #[tokio::test]
    async fn test_parse_lyric() -> Result<()> {
        let title = "稻香";
        let mut all_music = QQLyricSource.search_all_music(title, "周杰伦").await?;
        let mut lyric = QQ_LYRIC_SOURCE
            .search_lyrics(&mut all_music, title, 203000, "周杰伦")
            .await?
            .ok_or::<Error>("没找到对应歌曲".into())?;
        let l1 = &lyric.lyric.unwrap();
        let l2 = lyric.trans.as_deref();
        let l = Lyric::parse(l1, l2, Some(title))?;
        println!("{l:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_netease_lyric_parse() -> Result<()> {
        let lyric = NETEASE_LYRIC_SOURCE.fetch_lyrics("3340114785").await?;
        println!("{lyric:?}");
        let l1 = match &lyric.lyric {
            Some(lyric) => lyric,
            None => {
                println!("no lyric");
                return Ok(());
            }
        };
        let l2 = lyric.trans.as_deref();
        let l = Lyric::parse(l1, l2, Some("111"))?;
        println!("parsed lyric: {l:?}");
        assert!(
            l.get_lyrics().len() > 3,
            "应解析到正文歌词，而不只是作词作曲信息"
        );
        assert!(
            l.get_lyrics().iter().any(|line| line.translation.is_some()),
            "应解析并匹配翻译歌词"
        );
        Ok(())
    }
}

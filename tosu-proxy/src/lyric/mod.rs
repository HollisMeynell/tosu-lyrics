mod lyric;
mod source;

pub use lyric::*;
pub use source::*;

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

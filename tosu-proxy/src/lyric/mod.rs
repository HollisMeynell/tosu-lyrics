mod lyric;
mod source;

use crate::error::Result;
pub use lyric::*;
pub use source::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::info;

pub struct LyricService {
    // 当前歌词的下标
    now_index: usize,
    now_lyric: Option<Lyric>,
    // 偏移值, 毫秒
    offset: i32,
    music_cache: Arc<Mutex<HashMap<&'static str, Vec<SongInfo>>>>,
    wait_tasks: Mutex<Option<JoinSet<&'static str>>>,
}

impl LyricService {
    pub fn new() -> Self {
        let mut cache = HashMap::with_capacity(2);
        cache.insert(QQ_LYRIC_SOURCE.name(), Vec::with_capacity(10));
        cache.insert(NETEASE_LYRIC_SOURCE.name(), Vec::with_capacity(10));
        Self {
            now_index: 0,
            now_lyric: None,
            offset: 0,
            // 使用 Arc 和 Mutex 包装缓存
            music_cache: Arc::new(Mutex::new(cache)),
            wait_tasks: Mutex::new(None),
        }
    }

    pub async fn song_change(&mut self, title: &str, artist: &str, length: i32) -> Result<()> {
        // todo: 实现 offset 偏移, 从数据库中 存储/读取 缓存
        self.clear_cache().await;

        let length = if length < 0 { 0u32 } else { length as u32 };

        let mut join_set = JoinSet::new();
        Self::spawn_search_task(
            &mut join_set,
            &*QQ_LYRIC_SOURCE,
            title,
            artist,
            Arc::clone(&self.music_cache),
        );

        Self::spawn_search_task(
            &mut join_set,
            &*NETEASE_LYRIC_SOURCE,
            title,
            artist,
            Arc::clone(&self.music_cache),
        );

        macro_rules! search_and_set {
            (>$t:ident) => {
                if self
                    .search_and_set_lyric(&*$t, title, length, artist)
                    .await?
                {
                    return Ok(());
                }
            };
            (:$t:ident) => {
                if self
                    .search_and_set_lyric(&*$t, title, length, artist)
                    .await?
                {
                    let mut tasks = self.wait_tasks.lock().await;
                    *tasks = Some(join_set);
                    drop(tasks);
                    break;
                }
            };
            ($name:ident:$t:ident) => {
                if $name == $t.name() {
                    search_and_set!(:$t)
                }
            }
        }

        while let Some(source_name) = join_set.join_next().await {
            if source_name.is_err() {
                continue;
            }
            let source_name = source_name.unwrap();
            search_and_set!(source_name:NETEASE_LYRIC_SOURCE);
            search_and_set!(source_name:QQ_LYRIC_SOURCE);
        }

        search_and_set!(>NETEASE_LYRIC_SOURCE);
        search_and_set!(>QQ_LYRIC_SOURCE);

        self.now_lyric = None;

        Ok(())
    }

    /// 时间单位为毫秒
    pub async fn time_next(&mut self, t: i32) -> Result<()> {
        if let Some(lyric) = &mut self.now_lyric {
            let mut t = if t < 0 { 0 } else { t };
            t += self.offset;
            if let Some((index, lyric_line)) = lyric.find_line(t as f32 / 1000f32) {
                if self.now_index == index {
                    return Ok(());
                }

                self.now_index = index;
                info!("lyric change({}): {:?}", index, lyric_line);
            }
        }
        Ok(())
    }

    // 清理缓存
    async fn clear_cache(&mut self) {
        let mut tasks = self.wait_tasks.lock().await;
        *tasks = None;
        drop(tasks);

        let mut cache = self.music_cache.lock().await;
        if let Some(qq_cache) = cache.get_mut(QQ_LYRIC_SOURCE.name()) {
            qq_cache.clear();
        }
        if let Some(netease_cache) = cache.get_mut(NETEASE_LYRIC_SOURCE.name()) {
            netease_cache.clear();
        }
    }

    // 获取歌词存到 self.music_cache
    async fn search_and_set_lyric<S: LyricSource + 'static>(
        &mut self,
        source: &'static S,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<bool> {
        let cache = self.music_cache.lock().await;
        let cache_vec = cache.get(source.name());

        let songs_to_search = cache_vec.unwrap();

        if songs_to_search.is_empty() {
            return Ok(false);
        }

        let lyric_result = source
            .search_lyrics(&songs_to_search, title, length, artist)
            .await?;

        if let Some(lyric) = lyric_result {
            self.now_lyric = Some(lyric.try_into()?);
            return Ok(true);
        }

        Ok(false)
    }

    fn spawn_search_task<S: LyricSource + 'static>(
        tasks: &mut JoinSet<&'static str>,
        source: &'static S,
        title: &str,
        artist: &str,
        music_cache: Arc<Mutex<HashMap<&'static str, Vec<SongInfo>>>>,
    ) {
        let title = title.to_string();
        let artist = artist.to_string();
        tasks.spawn(async move {
            if let Ok(musics) = source.search_all_music(&title, &artist).await {
                let mut cache = music_cache.lock().await;
                if let Some(cache_vec) = cache.get_mut(source.name()) {
                    cache_vec.extend(musics);
                }
            }
            source.name()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;
    #[tokio::test]
    async fn test_parse_lyric() -> Result<()> {
        let title = "稻香";
        let all_music = QQLyricSource.search_all_music(title, "周杰伦").await?;
        let mut lyric = QQ_LYRIC_SOURCE
            .search_lyrics(&all_music, title, 203000, "周杰伦")
            .await?
            .ok_or::<Error>("没找到对应歌曲".into())?;
        let l1 = &lyric.lyric.unwrap();
        let l2 = lyric.trans.as_deref();
        let l = Lyric::parse(&l1, l2, Some(title))?;
        println!("{l:?}");
        Ok(())
    }
}

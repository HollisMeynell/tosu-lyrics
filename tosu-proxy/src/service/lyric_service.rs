use crate::database::{LyricCacheEntity, LyricConfigEntity};
use crate::error::{Error, Result};
use crate::lyric::{
    Lyric, LyricLine, LyricResult, LyricSource, LyricSourceEnum, NETEASE_LYRIC_SOURCE,
    QQ_LYRIC_SOURCE, SongInfo, SongInfoKey,
};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::lyric::{LyricPayload, SequenceType};
use crate::model::websocket::setting::block::BlockItem;
use crate::osu_source::OsuSongInfo;
use crate::server::ALL_SESSIONS;
use sea_orm::EntityTrait;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{debug, error};

pub static LYRIC_SERVICE: LazyLock<Mutex<LyricService>> =
    LazyLock::new(|| Mutex::new(LyricService::default()));

pub struct LyricService {
    // 当前歌词的下标
    now_index: usize,
    now_lyric: Option<Lyric>,
    now_save_cache: Option<OsuSongInfo>,
    // 偏移值, 毫秒
    offset: i32,

    // 当前歌词的起始/终止时间 毫秒
    current_lyric_start_time: i32,
    current_lyric_end_time: i32,

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
            now_save_cache: None,
            offset: 0,
            current_lyric_start_time: -1,
            current_lyric_end_time: -1,
            // 使用 Arc 和 Mutex 包装缓存
            music_cache: Arc::new(Mutex::new(cache)),
            wait_tasks: Mutex::new(None),
        }
    }

    pub async fn song_change(&mut self, song: OsuSongInfo) -> Result<()> {
        self.clear_cache().await;

        let title = song.title_unicode.to_string();
        let artist = song.artist_unicode.to_string();

        let bid = song.bid as i32;
        let sid = song.sid as i32;
        let length = if song.length < 0 {
            0u32
        } else {
            song.length as u32
        };

        self.now_save_cache = Some(song);

        let (disable, offset) = LyricConfigEntity::find_setting(bid, sid, &title).await;

        if disable {
            return Ok(());
        } else {
            self.offset = offset;
        }

        // 先查询缓存
        let cache = match LyricCacheEntity::find_by_bid(bid).await {
            Some(v) => Some(v),
            None => LyricCacheEntity::find_by_sid(sid).await,
        };

        if let Some(cache) = cache {
            match Lyric::from_json_cache(cache.cache.as_slice()) {
                Ok(lyric) => {
                    self.now_lyric = Some(lyric);
                    debug!("通过缓存加载 {title}");
                    return Ok(());
                }
                Err(err) => {
                    LyricCacheEntity::delete_by_id(bid);
                    error!("缓存失效(已移除): {}", err);
                }
            }
        }

        // 后台任务: 同时查询 网易云 | QQ, 用于当其中一个查到结果, 另一个还在进行时允许另一个在后台继续执行
        let mut join_set = JoinSet::new();
        Self::spawn_search_task(
            &mut join_set,
            &*QQ_LYRIC_SOURCE,
            &title,
            &artist,
            Arc::clone(&self.music_cache),
        );

        Self::spawn_search_task(
            &mut join_set,
            &*NETEASE_LYRIC_SOURCE,
            &title,
            &artist,
            Arc::clone(&self.music_cache),
        );
        macro_rules! search_and_set {
            (>$t:ident) => {
                let search_success = self.search_and_set_lyric(&*$t, &title, length, &artist).await?;
                if search_success
                {
                    debug!("通过网络加载 {title}");
                    let Some(lyric) = &self.now_lyric else { return Ok(()); };
                    let Some(save_key) = &self.now_save_cache else { return Ok(()); };
                    match Self::save_lyric(save_key, lyric).await {
                        Ok(_) | Err(Error::LyricParse(_)) => {
                            debug!("记录到缓存 {title}");
                        }
                        Err(err) => {
                            error!("存储缓存异常: {}", err);
                        }
                    };
                    return Ok(());
                }
            };
            (:$t:ident) => {
                if self
                    .search_and_set_lyric(&*$t, &title, length, &artist)
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
            let Ok(source_name) = source_name else {
                continue;
            };
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
        if self.now_lyric.is_none() {
            return Ok(());
        }

        let Some(lyric) = &mut self.now_lyric else {
            return Ok(());
        };

        let mut t = if t < 0 { 0 } else { t };
        t += self.offset;

        if t >= self.current_lyric_start_time && t <= self.current_lyric_end_time {
            // 时间没变
            return Ok(());
        }

        let mut ws_lyric = LyricPayload::default();
        {
            let current = lyric.find_line(t as f32 / 1000f32);

            if current.is_none() {
                return Ok(());
            }

            let (index, lyric_line) = current.ok_or(Error::Impossible)?;

            // 记录, 将秒转为毫秒
            self.current_lyric_start_time = (lyric_line.time * 1000f32) as i32;
            match lyric.get_line_by_index(index + 1) {
                None => self.current_lyric_end_time = i32::MAX,
                Some(l) => self.current_lyric_end_time = (l.time * 1000f32) as i32,
            };

            if self.now_index == index {
                return Ok(());
            }

            self.now_index = index;

            ws_lyric.sequence = if self.now_index < index {
                SequenceType::Down
            } else {
                SequenceType::Up
            };

            ws_lyric.next_time = if self.current_lyric_end_time == i32::MAX {
                0
            } else {
                self.current_lyric_end_time - self.current_lyric_start_time
            };

            ws_lyric.set_current_lyric(lyric_line).await;
        }

        'previous: {
            if self.now_index == 0 {
                break 'previous;
            }
            let previous = lyric.get_line_by_index(self.now_index - 1);
            if previous.is_none() {
                break 'previous;
            }
            ws_lyric
                .set_previous_lyric(previous.ok_or(Error::Impossible)?)
                .await;
        }

        'next: {
            let next = lyric.get_line_by_index(self.now_index - 1);
            if next.is_none() {
                break 'next;
            }
            let next = next.ok_or(Error::Impossible)?;
            let new_time = (next.time * 1000f32) as i32;
            ws_lyric.next_time = new_time - ws_lyric.next_time;
            ws_lyric.set_next_lyric(next).await;
        }
        let message: WebSocketMessage = ws_lyric.into();
        ALL_SESSIONS.send_to_all_client(message.into()).await;
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
            .search_lyrics(songs_to_search, title, length, artist)
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

    pub async fn get_search_result(&self) -> Value {
        use serde_json::map::Map;
        let mut result: Map<String, Value> = Map::new();
        let cache = Arc::clone(&self.music_cache);
        let cache_map = cache.lock().await;

        let qq_key = QQ_LYRIC_SOURCE.name().to_string();
        let netease_key = NETEASE_LYRIC_SOURCE.name().to_string();
        Self::search_result_to_json(&cache_map, qq_key, &mut result).await;
        Self::search_result_to_json(&cache_map, netease_key, &mut result).await;
        Value::Object(result)
    }

    async fn search_result_to_json(
        data: &HashMap<&'static str, Vec<SongInfo>>,
        key: String,
        result: &mut serde_json::map::Map<String, Value>,
    ) {
        let data_vec = if let Some(result) = data.get(key.as_str()) {
            result
        } else {
            return;
        };
        let data_vec = data_vec
            .iter()
            .map(serde_json::to_value)
            .filter_map(std::result::Result::ok)
            .collect::<Vec<Value>>();
        result.insert(key, Value::Array(data_vec));
    }

    pub async fn set_song_by_key(&mut self, key_info: &SongInfoKey) -> Result<()> {
        let Some(source) = LyricSourceEnum::get_by_name(key_info.source_type.as_ref()) else {
            return Err(format!("no source type is {}", key_info.source_type).into());
        };
        let lyric = source.fetch_lyrics(key_info.key.as_ref()).await?;
        let lyric = TryInto::<Lyric>::try_into(lyric)?;
        if let Some(save_key) = &self.now_save_cache {
            Self::save_lyric(save_key, &lyric)
                .await
                .inspect_err(|err| error!("存储缓存异常: {}", err))?;
        }
        self.now_lyric = Some(lyric);
        _ = self.time_next(0);
        Ok(())
    }

    pub fn get_now_all_lyrics(&self) -> Option<&[LyricLine]> {
        self.now_lyric.as_ref().map(Lyric::get_lyrics)
    }

    pub async fn set_block(&mut self, block: bool) -> Result<()> {
        let Some(key) = self.now_save_cache.take() else {
            return Err("no save cache is set".into());
        };

        let (is_block, offset) =
            if let Some((block, offset)) = LyricConfigEntity::get_by_bid(key.bid as i32).await {
                (block, offset)
            } else {
                (false, 0)
            };

        if is_block == block {
            return Ok(());
        }

        if block {
            LyricConfigEntity::save_setting(
                key.bid as i32,
                key.sid as i32,
                &key.title,
                true,
                offset,
            )
            .await;
            self.now_lyric.take();
            self.now_save_cache = Some(key);
        } else {
            if offset == 0 {
                LyricConfigEntity::delete_by_bid(key.bid as i32).await;
            } else {
                LyricConfigEntity::save_setting(
                    key.bid as i32,
                    key.sid as i32,
                    &key.title,
                    false,
                    offset,
                )
                .await;
            }
            self.song_change(key).await?;
        }
        Ok(())
    }

    pub async fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
        let Some(key) = self.now_save_cache.as_ref() else {
            return;
        };
        let config = LyricConfigEntity::get_by_bid(key.bid as i32).await;
        match config {
            None | Some((false, _)) if offset == 0 => {
                LyricConfigEntity::delete_by_bid(key.bid as i32).await;
            }
            Some((disable, _)) => {
                LyricConfigEntity::save_setting(
                    key.bid as i32,
                    key.sid as i32,
                    &key.title,
                    disable,
                    offset,
                )
                .await;
            }
            None => {
                LyricConfigEntity::save_setting(
                    key.bid as i32,
                    key.sid as i32,
                    &key.title,
                    false,
                    offset,
                )
                .await;
            }
        }
    }

    pub fn get_offset(&self) -> i32 {
        self.offset
    }

    async fn save_lyric(this: &OsuSongInfo, lyric: &Lyric) -> Result<()> {
        LyricCacheEntity::save(
            this.sid as i32,
            this.bid as i32,
            this.title.as_ref(),
            this.length,
            lyric,
        )
        .await
    }

    pub async fn get_all_block_list(&self) -> Vec<BlockItem> {
        LyricConfigEntity::get_all_disable()
            .await
            .into_iter()
            .map(|(sid, bid, title)| {
                let sid = sid as u32;
                let bid = bid as u32;
                BlockItem { bid, sid, title }
            })
            .collect()
    }
}

impl Default for LyricService {
    fn default() -> Self {
        Self::new()
    }
}

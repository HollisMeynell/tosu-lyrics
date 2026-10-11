use crate::database::{
    LyricBlockEntity, LyricCacheEntity, LyricConfigEntity, SCOPE_BID, SCOPE_SID,
};
use crate::error::{Error, Result};
use crate::lyric::{
    Lyric, LyricLine, LyricSource, LyricSourceEnum, NETEASE_LYRIC_SOURCE, QQ_LYRIC_SOURCE,
    SongInfo, SongInfoKey,
};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::lyric::{LyricLinePayload, LyricPayload, SequenceType};
use crate::model::websocket::setting::SettingPayload;
use crate::model::websocket::setting::block::BlockItem;
use crate::osu_source::OsuSongInfo;
use crate::server::ALL_SESSIONS;
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use serde_json::Value;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, LazyLock};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;
use tracing::{debug, error};

type ServiceJob = Box<dyn for<'a> FnOnce(&'a mut LyricService) -> BoxFuture<'a, ()> + Send>;

pub struct LyricServiceHandle {
    sender: mpsc::Sender<ServiceJob>,
}

impl LyricServiceHandle {
    fn start() -> Self {
        let (sender, mut receiver) = mpsc::channel::<ServiceJob>(64);
        tokio::spawn(async move {
            let mut service = LyricService::new();
            while let Some(job) = receiver.recv().await {
                job(&mut service).await;
            }
        });
        Self { sender }
    }

    pub(crate) async fn call<R, F>(&self, operation: F) -> R
    where
        R: Send + 'static,
        F: for<'a> FnOnce(&'a mut LyricService) -> BoxFuture<'a, R> + Send + 'static,
    {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(Box::new(move |service| {
                Box::pin(async move {
                    if !reply.is_closed() {
                        let result = operation(service).await;
                        let _ = reply.send(result);
                    }
                })
            }))
            .await
            .expect("lyric service task stopped");
        response
            .await
            .expect("lyric service task stopped before replying")
    }
}

pub static LYRIC_SERVICE: LazyLock<LyricServiceHandle> = LazyLock::new(LyricServiceHandle::start);

pub struct LyricSnapshot {
    song: Option<OsuSongInfo>,
    lyric: Option<Arc<Lyric>>,
    offset: i32,
    frame: Option<FrameTime>,
}

impl LyricSnapshot {
    pub fn get_now_song(&self) -> Option<&OsuSongInfo> {
        self.song.as_ref()
    }
    pub fn get_now_all_lyrics(&self) -> Option<&[LyricLine]> {
        self.lyric.as_ref().map(|lyric| lyric.get_lyrics())
    }
    pub fn get_offset(&self) -> i32 {
        self.offset
    }
    pub fn current_frame(&self) -> Option<FrameTime> {
        self.frame
    }
}

pub async fn lyric_service() -> LyricSnapshot {
    LYRIC_SERVICE
        .call(|service| {
            Box::pin(async move {
                LyricSnapshot {
                    song: service.now_save_cache.clone(),
                    lyric: service.now_lyric.clone(),
                    offset: service.offset,
                    frame: service.current_frame(),
                }
            })
        })
        .await
}

pub async fn lyric_candidates() -> Vec<(&'static str, SongInfo)> {
    LYRIC_SERVICE
        .call(|service| Box::pin(async move { service.all_candidates().await }))
        .await
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongIdent {
    pub bid: i32,
    pub sid: i32,
    pub title: String,
}

impl SongIdent {
    pub fn from_song(song: &OsuSongInfo) -> Self {
        Self {
            bid: song.bid as i32,
            sid: song.sid as i32,
            title: song.title_unicode.to_string(),
        }
    }
}

pub const STALE_REQUEST: &str = "stale_request";

const BEFORE_FIRST_LINE: i32 = i32::MIN;

#[inline]
fn to_ms(seconds: f32) -> i32 {
    (seconds * 1000f32) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTime {
    pub current: i32,
    pub next_time: i32,
    window_start: i32,
    window_end: i32,
    in_line: bool,
}

pub struct LyricService {
    now_index: usize,
    now_lyric: Option<Arc<Lyric>>,
    now_save_cache: Option<OsuSongInfo>,
    offset: i32,
    now_time: i32,
    current_lyric_start_time: i32,
    current_lyric_end_time: i32,
    music_cache: HashMap<&'static str, Vec<SongInfo>>,
    is_song_changed: bool,

    generation: u64,

    now_ident: Option<SongIdent>,
    pending_song: Option<tokio::task::AbortHandle>,
}

struct SearchPlan {
    ident: SongIdent,
    artist: String,
    length: u32,
    generation: u64,
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
            now_time: 0,
            current_lyric_start_time: -1,
            current_lyric_end_time: -1,
            music_cache: cache,
            is_song_changed: true,
            generation: 0,
            now_ident: None,
            pending_song: None,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn generation_matches(&self, generation: u64) -> bool {
        self.generation == generation
    }

    pub fn invalidate_async(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub(crate) async fn observe_song(&mut self, song: OsuSongInfo) {
        if song.sid < 0 && song.artist == "nekodex" {
            return;
        }
        let flow_start = std::time::Instant::now();
        debug!(
            "[perf] (1) 换歌事件 bid={} sid={} len={}ms gen={} title={:?} artist={:?}",
            song.bid, song.sid, song.length, self.generation, song.title, song.artist,
        );
        self.cancel_pending_song();
        self.clear_state();
        self.invalidate_async();
        Self::broadcast_clear().await;
        Self::broadcast_loading(true).await;
        let generation = self.generation;
        let task = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            debug!(
                "[perf] (3) debounce 结束 +{}ms",
                flow_start.elapsed().as_millis()
            );
            let plan = LYRIC_SERVICE
                .call(move |svc| {
                    Box::pin(async move {
                        if !svc.generation_matches(generation) {
                            return Ok(None);
                        }
                        svc.begin_song(song).await
                    })
                })
                .await;
            match plan {
                Ok(plan) => {
                    if let Err(err) = Self::load_plan(plan).await {
                        error!("song update error: {err}");
                    }
                }
                Err(err) => error!("song update error: {err}"),
            }
            debug!(
                "[perf] (13) 换歌链路结束 总耗时={}ms",
                flow_start.elapsed().as_millis()
            );
        });
        self.pending_song = Some(task.abort_handle());
    }

    fn cancel_pending_song(&mut self) {
        if let Some(task) = self.pending_song.take() {
            task.abort();
        }
    }

    #[cfg(test)]
    fn clear_display_sync(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
    }

    pub async fn now_ident() -> Option<SongIdent> {
        LYRIC_SERVICE
            .call(|svc| Box::pin(async move { svc.now_ident.clone() }))
            .await
    }

    pub async fn song_change(song: OsuSongInfo) -> Result<()> {
        let plan = LYRIC_SERVICE
            .call(move |svc| Box::pin(async move { svc.begin_song(song).await }))
            .await?;
        Self::load_plan(plan).await
    }

    async fn load_plan(plan: Option<SearchPlan>) -> Result<()> {
        let result: Result<()> = async {
            let Some(plan) = plan else {
                return Ok(());
            };

            let search_start = std::time::Instant::now();
            let mut results: HashMap<&'static str, Vec<SongInfo>> = HashMap::with_capacity(2);
            let mut tasks = Self::spawn_sources(&plan.ident.title, &plan.artist);
            let mut lyric: Option<Lyric> = None;
            let mut netease_fetch_done = false;

            while let Some(joined) = tasks.join_next().await {
                let (name, songs) = match joined {
                    Ok(Some(v)) => v,
                    Ok(None) => continue,
                    Err(err) => {
                        error!("歌词源搜索任务异常: {err}");
                        continue;
                    }
                };
                results.insert(name, songs);
                Self::publish_candidates(&plan, results.clone()).await?;

                if name == NETEASE_LYRIC_SOURCE.name() && !netease_fetch_done {
                    netease_fetch_done = true;
                    if let Some(songs) = results.get_mut(NETEASE_LYRIC_SOURCE.name()) {
                        let t0 = std::time::Instant::now();
                        let hit = NETEASE_LYRIC_SOURCE
                            .search_lyrics(songs, &plan.ident.title, plan.length, &plan.artist)
                            .await?;
                        debug!(
                            "[perf] (8) NetEase 取词 候选数={} 耗时={}ms 命中={}",
                            songs.len(),
                            t0.elapsed().as_millis(),
                            hit.is_some(),
                        );
                        if let Some(l) = hit {
                            lyric = Some(l.try_into()?);
                            break;
                        }
                    }
                }
            }

            debug!(
                "[perf] (7) 搜索阶段结束(NetEase-first) 总耗时={}ms 候选数 netease={} qq={}",
                search_start.elapsed().as_millis(),
                results
                    .get(NETEASE_LYRIC_SOURCE.name())
                    .map_or(0, |v| v.len()),
                results.get(QQ_LYRIC_SOURCE.name()).map_or(0, |v| v.len()),
            );

            let fetch_start = std::time::Instant::now();
            if lyric.is_none() {
                while let Some(joined) = tasks.join_next().await {
                    let (name, songs) = match joined {
                        Ok(Some(v)) => v,
                        Ok(None) => continue,
                        Err(err) => {
                            error!("歌词源搜索任务异常: {err}");
                            continue;
                        }
                    };
                    results.insert(name, songs);
                    Self::publish_candidates(&plan, results.clone()).await?;
                }
                if let Some(songs) = results.get_mut(QQ_LYRIC_SOURCE.name()) {
                    let t0 = std::time::Instant::now();
                    let hit = QQ_LYRIC_SOURCE
                        .search_lyrics(songs, &plan.ident.title, plan.length, &plan.artist)
                        .await?;
                    debug!(
                        "[perf] (9) QQ 取词 候选数={} 耗时={}ms 命中={}",
                        songs.len(),
                        t0.elapsed().as_millis(),
                        hit.is_some(),
                    );
                    if let Some(l) = hit {
                        lyric = Some(l.try_into()?);
                    }
                }
            }
            debug!(
                "[perf] (12) 取词阶段结束 耗时={}ms 是否取到歌词={}",
                fetch_start.elapsed().as_millis(),
                lyric.is_some(),
            );
            LYRIC_SERVICE
                .call(move |svc| {
                    Box::pin(async move { svc.commit_search(plan, results, lyric).await })
                })
                .await
        }
        .await;
        Self::broadcast_loading(false).await;
        result
    }

    async fn fetch_search_lyric(
        plan: &SearchPlan,
        results: &mut HashMap<&'static str, Vec<SongInfo>>,
    ) -> Result<Option<Lyric>> {
        if let Some(songs) = results.get_mut(NETEASE_LYRIC_SOURCE.name()) {
            let t0 = std::time::Instant::now();
            let hit = NETEASE_LYRIC_SOURCE
                .search_lyrics(songs, &plan.ident.title, plan.length, &plan.artist)
                .await?;
            debug!(
                "[perf] (8) NetEase 取词 候选数={} 耗时={}ms 命中={}",
                songs.len(),
                t0.elapsed().as_millis(),
                hit.is_some(),
            );
            if let Some(lyric) = hit {
                return Ok(Some(lyric.try_into()?));
            }
        }
        if let Some(songs) = results.get_mut(QQ_LYRIC_SOURCE.name()) {
            let t0 = std::time::Instant::now();
            let hit = QQ_LYRIC_SOURCE
                .search_lyrics(songs, &plan.ident.title, plan.length, &plan.artist)
                .await?;
            debug!(
                "[perf] (9) QQ 取词 候选数={} 耗时={}ms 命中={}",
                songs.len(),
                t0.elapsed().as_millis(),
                hit.is_some(),
            );
            if let Some(lyric) = hit {
                return Ok(Some(lyric.try_into()?));
            }
        }
        Ok(None)
    }

    async fn begin_song(&mut self, song: OsuSongInfo) -> Result<Option<SearchPlan>> {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;

        self.clear_state();
        Self::broadcast_clear().await;
        self.clear_cache();
        self.now_time = song.now.max(0);

        let ident = SongIdent::from_song(&song);
        let title = &ident.title;
        let artist = song.artist_unicode.to_string();
        let bid = ident.bid;
        let sid = ident.sid;
        let length = song.length.max(0) as u32;

        self.now_ident = Some(ident.clone());
        self.now_save_cache = Some(song);

        self.offset = LyricConfigEntity::get_offset(bid).await?;

        let blocked = crate::service::block_service::blocked_rule(bid, sid, title)
            .await
            .map_err(|err| Error::Runtime(err.to_string()))?
            .is_some();
        if blocked {
            debug!("{title} 命中黑名单, 不加载歌词");
            return Ok(None);
        }

        let ttl = crate::service::cache_service::ttl_ms();
        let cache = match LyricCacheEntity::find_by_bid(bid, ttl).await? {
            Some(v) => Some(v),
            None => LyricCacheEntity::find_by_sid(sid, ttl).await?,
        };

        if let Some(cache) = cache {
            match Lyric::from_json_cache(cache.cache.as_slice()) {
                Ok(lyric) => {
                    self.now_lyric = Some(Arc::new(lyric));
                    debug!("通过缓存加载 {title}");
                    self.push_now(self.now_time).await;
                    return Ok(None);
                }
                Err(err) => {
                    let removed = LyricCacheEntity::delete_by_bid(cache.bid).await;
                    error!(
                        "缓存失效(已移除 bid={}): {} ({:?})",
                        cache.bid, err, removed
                    );
                }
            }
        }

        Ok(Some(SearchPlan {
            ident,
            artist,
            length,
            generation,
        }))
    }

    async fn search_sources(plan: &SearchPlan) -> HashMap<&'static str, Vec<SongInfo>> {
        Self::search_raw(&plan.ident.title, &plan.artist).await
    }

    async fn publish_candidates(
        plan: &SearchPlan,
        results: HashMap<&'static str, Vec<SongInfo>>,
    ) -> Result<()> {
        let ident = plan.ident.clone();
        let generation = plan.generation;
        let title = plan.ident.title.clone();
        LYRIC_SERVICE
            .call(move |svc| {
                Box::pin(async move {
                    if !svc.generation_matches(generation) {
                        debug!(
                            "丢弃过期候选: {} 的代次 {} != 当前 {}",
                            title, generation, svc.generation
                        );
                        return Ok(());
                    }
                    match &svc.now_ident {
                        Some(cur) if cur == &ident => {}
                        _ => {
                            debug!("丢弃无主候选: {}", title);
                            return Ok(());
                        }
                    }
                    let cache = &mut svc.music_cache;
                    for (name, musics) in results {
                        if let Some(slot) = cache.get_mut(name) {
                            slot.extend(musics);
                        }
                    }
                    Ok(())
                })
            })
            .await
    }
    async fn commit_search(
        &mut self,
        plan: SearchPlan,
        results: HashMap<&'static str, Vec<SongInfo>>,
        lyric: Option<Lyric>,
    ) -> Result<()> {
        let generation = plan.generation;
        if !self.generation_matches(generation) {
            debug!(
                "丢弃过期搜索结果: {} 的代际 {} != 当前 {}",
                plan.ident.title, generation, self.generation
            );
            return Ok(());
        }
        match &self.now_ident {
            Some(ident) if ident == &plan.ident => {}
            _ => {
                debug!("丢弃无主搜索结果: {}", plan.ident.title);
                return Ok(());
            }
        }

        if let Some(lyric) = lyric {
            if let Some(save_key) = self.now_save_cache.as_ref()
                && let Err(err) = Self::save_lyric(save_key, &lyric).await
            {
                error!("存储缓存异常: {err}");
            }
            self.now_lyric = Some(Arc::new(lyric));
            self.push_now(self.now_time).await;
        } else {
            self.now_lyric = None;
        }
        Ok(())
    }

    pub async fn apply_uploaded_lyric(&mut self, expect_sid: i32, lyric: Lyric) -> Result<()> {
        match self.now_ident.as_ref() {
            Some(ident) if ident.sid == expect_sid => {}
            _ => return Err(Error::Runtime(STALE_REQUEST.into())),
        }

        if let Some(save_key) = self.now_save_cache.as_ref() {
            Self::save_lyric(save_key, &lyric)
                .await
                .inspect_err(|err| error!("存储缓存异常: {err}"))
                .ok();
        }

        self.now_lyric = Some(Arc::new(lyric));
        let now = self.now_time;
        self.push_now(now).await;
        Ok(())
    }

    pub async fn reload_current() {
        let plan = LYRIC_SERVICE
            .call(|svc| {
                Box::pin(async move {
                    let Some(song) = svc.now_save_cache.clone() else {
                        return Ok(None);
                    };
                    svc.begin_song(song).await
                })
            })
            .await;
        match plan {
            Ok(plan) => {
                if let Err(err) = Self::load_plan(plan).await {
                    error!("重新加载当前歌失败: {err}");
                }
            }
            Err(err) => error!("重新加载当前歌失败: {err}"),
        }
    }

    async fn try_source_lyric<S: LyricSource + 'static>(
        &mut self,
        source: &'static S,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<bool> {
        let search_success = self
            .search_and_set_lyric(source, title, length, artist)
            .await?;
        if !search_success {
            return Ok(false);
        }

        debug!("通过网络加载 {title}");

        {
            let Some(lyric) = &self.now_lyric else {
                return Ok(false);
            };
            let Some(save_key) = &self.now_save_cache else {
                return Ok(false);
            };
            match Self::save_lyric(save_key, lyric).await {
                Ok(_) | Err(Error::LyricParse(_)) => debug!("记录到缓存 {title}"),
                Err(err) => error!("存储缓存异常: {}", err),
            }
        }

        self.push_now(self.now_time).await;
        Ok(true)
    }

    pub fn clear_state(&mut self) {
        self.now_lyric = None;
        self.now_index = usize::MAX;
        self.current_lyric_start_time = -1;
        self.current_lyric_end_time = -1;
        self.is_song_changed = true;
        self.now_time = 0;
    }

    pub fn current_frame(&self) -> Option<FrameTime> {
        let lyric = self.now_lyric.as_ref()?;
        if lyric.get_lyrics().is_empty() {
            return None;
        }
        let t = self.now_time.saturating_add(self.offset);
        Some(Self::frame_at(lyric, t))
    }

    pub fn get_now_song(&self) -> Option<&OsuSongInfo> {
        self.now_save_cache.as_ref()
    }

    pub async fn clear_display(&mut self) {
        self.cancel_pending_song();
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
        Self::broadcast_clear().await;
        Self::broadcast_loading(false).await;
    }

    pub async fn song_clean(&mut self) {
        self.cancel_pending_song();
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
        self.now_save_cache = None;
        self.now_ident = None;
        Self::broadcast_clear().await;
        Self::broadcast_loading(false).await;
    }

    pub async fn broadcast_clear() {
        let clean_message = SettingPayload::new("setClear".to_string());
        ALL_SESSIONS
            .send_to_all_client(Into::<WebSocketMessage>::into(clean_message).into())
            .await;
    }

    pub async fn broadcast_loading(loading: bool) {
        let mut message = SettingPayload::new("setLyricLoading".to_string());
        message.value = Some(Value::Bool(loading));
        ALL_SESSIONS
            .send_to_all_client(Into::<WebSocketMessage>::into(message).into())
            .await;
    }

    fn frame_at(lyric: &Lyric, t: i32) -> FrameTime {
        let t_sec = t as f32 / 1000f32;
        match lyric.find_line(t_sec) {
            Some((index, line)) => {
                let line_start = to_ms(line.time);
                match lyric.get_line_by_index(index + 1) {
                    Some(next) => {
                        let next_start = to_ms(next.time);
                        FrameTime {
                            current: index as i32,
                            next_time: next_start.saturating_sub(t),
                            window_start: line_start,
                            window_end: next_start,
                            in_line: true,
                        }
                    }
                    None => FrameTime {
                        current: index as i32,
                        next_time: -1,
                        window_start: line_start,
                        window_end: i32::MAX,
                        in_line: true,
                    },
                }
            }
            None => {
                let first_start = lyric
                    .get_line_by_index(0)
                    .map(|l| to_ms(l.time))
                    .unwrap_or(0);
                FrameTime {
                    current: 0,
                    next_time: first_start.saturating_sub(t),
                    window_start: BEFORE_FIRST_LINE,
                    window_end: first_start,
                    in_line: false,
                }
            }
        }
    }

    fn apply_frame(&mut self, frame: &FrameTime) {
        self.now_index = if frame.in_line {
            frame.current as usize
        } else {
            usize::MAX
        };
        self.current_lyric_start_time = frame.window_start;
        self.current_lyric_end_time = frame.window_end;
        self.is_song_changed = false;
    }

    async fn push_now(&mut self, now_ms: i32) {
        let Some(lyric) = self.now_lyric.as_ref() else {
            return;
        };
        let lyrics: Vec<LyricLinePayload> = lyric
            .get_lyrics()
            .iter()
            .map(|line| LyricLinePayload {
                origin: line.origin.clone(),
                translation: line.translation.clone(),
            })
            .collect();
        if lyrics.is_empty() {
            return;
        }

        let t = now_ms.max(0).saturating_add(self.offset);
        let frame = Self::frame_at(lyric, t);
        self.apply_frame(&frame);

        let ws_lyric = LyricPayload {
            lyric: Some(Arc::from(lyrics)),
            current: frame.current,
            next_time: frame.next_time,
            sequence: SequenceType::Up,
        };

        let message: WebSocketMessage = ws_lyric.into();
        ALL_SESSIONS.send_to_all_client(message.into()).await;
    }

    pub fn get_snapshot(&self) -> Option<WebSocketMessage> {
        let lyric = self.now_lyric.as_ref()?;
        if lyric.get_lyrics().is_empty() {
            return None;
        }
        let t = self.now_time.saturating_add(self.offset);
        let frame = Self::frame_at(lyric, t);
        let lyrics: Vec<LyricLinePayload> = lyric
            .get_lyrics()
            .iter()
            .map(|line| LyricLinePayload {
                origin: line.origin.clone(),
                translation: line.translation.clone(),
            })
            .collect();
        let snapshot = LyricPayload {
            lyric: Some(Arc::from(lyrics)),
            current: frame.current,
            next_time: frame.next_time,
            sequence: SequenceType::Up,
        };
        Some(snapshot.into())
    }

    pub async fn time_next(&mut self, t: i32) -> Result<()> {
        self.now_time = t.max(0);

        let Some(lyric) = self.now_lyric.as_ref() else {
            return Ok(());
        };

        let t = self.now_time.saturating_add(self.offset);

        if t >= self.current_lyric_start_time && t < self.current_lyric_end_time {
            return Ok(());
        }

        let mut ws_lyric = LyricPayload::default();
        if self.is_song_changed {
            let lyrics: Vec<LyricLinePayload> = lyric
                .get_lyrics()
                .iter()
                .map(|lyric| LyricLinePayload {
                    origin: lyric.origin.clone(),
                    translation: lyric.translation.clone(),
                })
                .collect();
            ws_lyric.lyric = Some(Arc::from(lyrics))
        }

        let frame = Self::frame_at(lyric, t);
        let prev_index = self.now_index;
        let target_index = if frame.in_line {
            frame.current as usize
        } else {
            usize::MAX
        };
        let should_push = prev_index != target_index || ws_lyric.lyric.is_some();
        self.apply_frame(&frame);

        if !should_push {
            return Ok(());
        }

        ws_lyric.sequence = if prev_index < target_index {
            SequenceType::Down
        } else {
            SequenceType::Up
        };
        ws_lyric.current = frame.current;
        ws_lyric.next_time = frame.next_time;

        let message: WebSocketMessage = ws_lyric.into();
        ALL_SESSIONS.send_to_all_client(message.into()).await;
        Ok(())
    }

    fn clear_cache(&mut self) {
        self.is_song_changed = true;
        for songs in self.music_cache.values_mut() {
            songs.clear();
        }
    }

    async fn search_and_set_lyric<S: LyricSource + 'static>(
        &mut self,
        source: &'static S,
        title: &str,
        length: u32,
        artist: &str,
    ) -> Result<bool> {
        let cache = &mut self.music_cache;
        let Some(songs_to_search) = cache.get_mut(source.name()) else {
            return Ok(false);
        };

        if songs_to_search.is_empty() {
            return Ok(false);
        }

        let lyric_result = source
            .search_lyrics(songs_to_search, title, length, artist)
            .await?;

        if let Some(lyric) = lyric_result {
            self.now_lyric = Some(Arc::new(lyric.try_into()?));
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn get_search_result(&self) -> Value {
        use serde_json::map::Map;
        let mut result: Map<String, Value> = Map::new();
        let cache_map = &self.music_cache;

        let qq_key = QQ_LYRIC_SOURCE.name().to_string();
        let netease_key = NETEASE_LYRIC_SOURCE.name().to_string();
        Self::search_result_to_json(cache_map, qq_key, &mut result);
        Self::search_result_to_json(cache_map, netease_key, &mut result);
        Value::Object(result)
    }

    fn search_result_to_json(
        data: &HashMap<&'static str, Vec<SongInfo>>,
        key: String,
        result: &mut serde_json::map::Map<String, Value>,
    ) {
        let Some(data_vec) = data.get(key.as_str()) else {
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
                .inspect_err(|err| error!("存储缓存异常: {}", err))
                .ok();
        }
        self.now_lyric = Some(Arc::new(lyric));
        self.push_now(self.now_time).await;
        Ok(())
    }

    pub async fn set_manual_lyric(&mut self, lyric_text: &str) -> Result<()> {
        let Some(key) = self.now_save_cache.as_ref() else {
            return Err("当前没有播放歌曲, 无法上传歌词".into());
        };

        let lyric = Lyric::parse(lyric_text, None, None)?;

        Self::save_lyric(key, &lyric)
            .await
            .inspect_err(|err| error!("存储缓存异常: {}", err))
            .ok();

        self.now_lyric = Some(Arc::new(lyric));
        self.push_now(self.now_time).await;
        Ok(())
    }

    pub fn get_now_all_lyrics(&self) -> Option<&[LyricLine]> {
        self.now_lyric.as_ref().map(|lyric| lyric.get_lyrics())
    }

    pub async fn set_block(&mut self, block: bool) -> Result<()> {
        let Some(key) = self.now_save_cache.as_ref() else {
            return Err("no save cache is set".into());
        };
        let bid = key.bid as i32;
        let sid = key.sid as i32;
        let title = &key.title_unicode;

        if block {
            LyricBlockEntity::upsert(SCOPE_BID, &bid.to_string(), title, sid, "").await?;
            self.clear_state();
            Self::broadcast_clear().await;
        } else {
            LyricBlockEntity::delete_by_scope_value(SCOPE_BID, &bid.to_string()).await?;
        }
        Ok(())
    }

    pub async fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
        self.current_lyric_start_time = -1;
        self.current_lyric_end_time = -1;

        if let Some(key) = self.now_save_cache.as_ref() {
            let bid = key.bid as i32;
            let sid = key.sid as i32;
            let title = &key.title_unicode;
            if let Err(err) = LyricConfigEntity::save_offset(bid, sid, title, offset).await {
                error!("保存偏移失败: {err}");
            }
        }

        let now = self.now_time;
        self.push_now(now).await;
    }

    pub fn get_offset(&self) -> i32 {
        self.offset
    }

    async fn save_lyric(this: &OsuSongInfo, lyric: &Lyric) -> Result<()> {
        LyricCacheEntity::save(
            this.sid as i32,
            this.bid as i32,
            this.title_unicode.as_ref(),
            this.length,
            lyric,
        )
        .await
    }

    pub async fn get_all_block_list(&self) -> Result<Vec<BlockItem>> {
        Ok(LyricBlockEntity::list_all()
            .await?
            .into_iter()
            .map(|rule| {
                let numeric = rule.value.parse::<i64>().unwrap_or(0).max(0) as u32;
                let (bid, sid) = match rule.scope.as_str() {
                    SCOPE_BID => (numeric, rule.sid.max(0) as u32),
                    SCOPE_SID => (0, numeric),
                    _ => (0, 0),
                };
                BlockItem {
                    bid,
                    sid,
                    title: rule.title,
                }
            })
            .collect())
    }

    pub async fn ticket() -> (u64, Option<SongIdent>) {
        LYRIC_SERVICE
            .call(|svc| Box::pin(async move { (svc.generation, svc.now_ident.clone()) }))
            .await
    }

    pub async fn all_candidates(&self) -> Vec<(&'static str, SongInfo)> {
        let cache = &self.music_cache;
        let mut out = Vec::new();
        for (name, list) in cache.iter() {
            for song in list {
                out.push((*name, song.clone()));
            }
        }
        out
    }

    pub async fn search_now(
        title: String,
        artist: String,
    ) -> Result<Vec<(&'static str, SongInfo)>> {
        let generation = LYRIC_SERVICE
            .call(|svc| Box::pin(async move { svc.generation() }))
            .await;
        let results = Self::search_raw(&title, &artist).await;

        LYRIC_SERVICE
            .call(move |svc| {
                Box::pin(async move {
                    if !svc.generation_matches(generation) {
                        return Err(Error::Runtime(STALE_REQUEST.into()));
                    }
                    {
                        let cache = &mut svc.music_cache;
                        for (name, musics) in results {
                            if let Some(slot) = cache.get_mut(name) {
                                slot.clear();
                                slot.extend(musics);
                            }
                        }
                    }
                    Ok(svc.all_candidates().await)
                })
            })
            .await
    }

    fn spawn_sources(title: &str, artist: &str) -> JoinSet<Option<(&'static str, Vec<SongInfo>)>> {
        let mut tasks: JoinSet<Option<(&'static str, Vec<SongInfo>)>> = JoinSet::new();

        let (qq_title, qq_artist) = (title.to_string(), artist.to_string());
        tasks.spawn(async move {
            let name = QQ_LYRIC_SOURCE.name();
            let t0 = std::time::Instant::now();
            match AssertUnwindSafe(QQ_LYRIC_SOURCE.search_all_music(&qq_title, &qq_artist))
                .catch_unwind()
                .await
            {
                Ok(Ok(songs)) => {
                    debug!(
                        "[perf] (2) {} 搜索完成 耗时={}ms 候选={}",
                        name,
                        t0.elapsed().as_millis(),
                        songs.len()
                    );
                    Some((name, songs))
                }
                Ok(Err(err)) => {
                    debug!(
                        "[perf] (2) {} 搜索失败 耗时={}ms: {err}",
                        name,
                        t0.elapsed().as_millis()
                    );
                    None
                }
                Err(_) => {
                    error!("{name} 搜索发生 panic，忽略该源");
                    None
                }
            }
        });

        let (netease_title, netease_artist) = (title.to_string(), artist.to_string());
        tasks.spawn(async move {
            let name = NETEASE_LYRIC_SOURCE.name();
            let t0 = std::time::Instant::now();
            match AssertUnwindSafe(
                NETEASE_LYRIC_SOURCE.search_all_music(&netease_title, &netease_artist),
            )
            .catch_unwind()
            .await
            {
                Ok(Ok(songs)) => {
                    debug!(
                        "[perf] (2) {} 搜索完成 耗时={}ms 候选={}",
                        name,
                        t0.elapsed().as_millis(),
                        songs.len()
                    );
                    Some((name, songs))
                }
                Ok(Err(err)) => {
                    debug!(
                        "[perf] (2) {} 搜索失败 耗时={}ms: {err}",
                        name,
                        t0.elapsed().as_millis()
                    );
                    None
                }
                Err(_) => {
                    error!("{name} 搜索发生 panic，忽略该源");
                    None
                }
            }
        });

        tasks
    }

    async fn search_raw(title: &str, artist: &str) -> HashMap<&'static str, Vec<SongInfo>> {
        let mut tasks = Self::spawn_sources(title, artist);
        let mut out = HashMap::with_capacity(2);
        while let Some(res) = tasks.join_next().await {
            match res {
                Ok(Some((name, songs))) => {
                    out.insert(name, songs);
                }
                Ok(None) => {}
                Err(err) => error!("歌词源搜索任务异常: {err}"),
            }
        }
        out
    }

    pub async fn apply_source_lyric(
        &mut self,
        generation: u64,
        expect_sid: i32,
        lyric: Lyric,
    ) -> Result<()> {
        if !self.generation_matches(generation) {
            return Err(Error::Runtime(STALE_REQUEST.into()));
        }
        match self.now_ident.as_ref() {
            Some(ident) if ident.sid == expect_sid => {}
            _ => return Err(Error::Runtime(STALE_REQUEST.into())),
        }

        if let Some(save_key) = self.now_save_cache.as_ref() {
            Self::save_lyric(save_key, &lyric)
                .await
                .inspect_err(|err| error!("存储缓存异常: {err}"))
                .ok();
        }
        self.now_lyric = Some(Arc::new(lyric));
        let now = self.now_time;
        self.push_now(now).await;
        Ok(())
    }

    // 换源会立即重载，不能设置持续清屏抑制。
    pub async fn clear_display_bump(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
        Self::broadcast_clear().await;
    }

    pub async fn get_cache_count(&self) -> Result<u64> {
        LyricCacheEntity::all_count().await
    }

    pub async fn clear_all_cache(&self) -> Result<u64> {
        LyricCacheEntity::delete_all().await
    }
}

impl Default for LyricService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn rejected_search_does_not_modify_candidates() {
        for stale_generation in [false, true] {
            let mut service = LyricService::new();
            service.now_ident = Some(SongIdent::from_song(&song(1, 100, "current")));
            let plan = SearchPlan {
                ident: SongIdent::from_song(&song(2, 200, "other")),
                artist: String::new(),
                length: 0,
                generation: if stale_generation { 1 } else { 0 },
            };
            let results = HashMap::from([(
                QQ_LYRIC_SOURCE.name(),
                vec![SongInfo {
                    title: "stale".into(),
                    artist: String::new(),
                    length: 0,
                    key: "stale".into(),
                }],
            )]);
            service
                .commit_search(plan, results, Some(lyric_3()))
                .await
                .unwrap();
            assert!(service.all_candidates().await.is_empty());
            assert!(service.now_lyric.is_none());
        }
    }

    #[tokio::test]
    async fn cleared_candidates_keep_both_source_keys() {
        let mut service = LyricService::new();
        service
            .music_cache
            .get_mut(QQ_LYRIC_SOURCE.name())
            .unwrap()
            .push(SongInfo {
                title: "candidate".into(),
                artist: String::new(),
                length: 0,
                key: "candidate".into(),
            });
        assert_eq!(service.all_candidates().await.len(), 1);
        service.clear_cache();
        let result = service.get_search_result().await;
        assert_eq!(result.as_object().unwrap().len(), 2);
        for source in [QQ_LYRIC_SOURCE.name(), NETEASE_LYRIC_SOURCE.name()] {
            assert_eq!(result[source], serde_json::json!([]));
        }
    }

    fn lyric_3() -> Lyric {
        Lyric::parse("[00:01.00]a\n[00:03.00]b\n[00:05.00]c", None, None).expect("测试歌词应能解析")
    }

    fn in_window(frame: &FrameTime, t: i32) -> bool {
        t >= frame.window_start && t < frame.window_end
    }

    fn target_index(frame: &FrameTime) -> usize {
        if frame.in_line {
            frame.current as usize
        } else {
            usize::MAX
        }
    }

    #[test]
    fn before_first_line() {
        let lyric = lyric_3();
        let f = LyricService::frame_at(&lyric, 500);
        assert_eq!(f.current, 0);
        assert!(!f.in_line, "首行之前不应锁定下标");
        assert_eq!(f.next_time, 500, "next_time 应为距离首行的剩余时长");
        assert_eq!(f.window_start, BEFORE_FIRST_LINE);
        assert_eq!(f.window_end, 1000);
    }

    #[test]
    fn before_first_line_window_does_not_swallow_first_line() {
        let lyric = lyric_3();
        let before = LyricService::frame_at(&lyric, 500);
        assert!(in_window(&before, 500));
        assert!(
            !in_window(&before, 1000),
            "到达首行起点时必须离开窗口, 才能重新推送"
        );
    }

    #[test]
    fn entering_first_line_counts_as_change() {
        let lyric = lyric_3();
        let mut service = LyricService::new();
        service.now_lyric = Some(Arc::new(lyric));

        let before = {
            let lyric = service.now_lyric.as_ref().unwrap();
            LyricService::frame_at(lyric, 500)
        };
        service.apply_frame(&before);
        assert_eq!(service.now_index, usize::MAX);
        assert!(!service.is_song_changed, "apply_frame 应清掉待下发标志");

        let at_first = {
            let lyric = service.now_lyric.as_ref().unwrap();
            LyricService::frame_at(lyric, 1000)
        };
        assert_eq!(at_first.current, 0);
        assert_eq!(at_first.next_time, 2000);
        assert_ne!(
            service.now_index,
            target_index(&at_first),
            "进入首行必须被识别为状态变化"
        );
    }

    #[test]
    fn next_time_is_remaining_ms() {
        let lyric = lyric_3();
        let on_start = LyricService::frame_at(&lyric, 1000);
        assert_eq!(on_start.current, 0);
        assert_eq!(on_start.next_time, 2000);

        let mid_line = LyricService::frame_at(&lyric, 2200);
        assert_eq!(mid_line.current, 0);
        assert_eq!(mid_line.next_time, 800, "行中应返回剩余时长");
    }

    #[test]
    fn window_boundary_is_half_open() {
        let lyric = lyric_3();
        let prev = LyricService::frame_at(&lyric, 2999);
        assert_eq!(prev.current, 0);
        assert!(
            !in_window(&prev, 3000),
            "t == window_end 必须离开窗口, 否则边界会被吞掉"
        );

        let at_boundary = LyricService::frame_at(&lyric, 3000);
        assert_eq!(at_boundary.current, 1);
    }

    #[test]
    fn seek_forward_backward_and_last_line() {
        let lyric = lyric_3();

        let back = LyricService::frame_at(&lyric, 1500);
        assert_eq!(back.current, 0);
        assert_eq!(back.next_time, 1500);

        let last = LyricService::frame_at(&lyric, 6000);
        assert_eq!(last.current, 2);
        assert_eq!(last.next_time, -1);
        assert_eq!(last.window_end, i32::MAX);
        assert!(in_window(&last, 6000));
    }

    #[test]
    fn clear_state_invalidates_window() {
        let mut service = LyricService::new();
        service.now_lyric = Some(Arc::new(lyric_3()));
        let f = LyricService::frame_at(service.now_lyric.as_ref().unwrap(), 3000);
        service.apply_frame(&f);

        service.clear_state();
        assert_eq!(service.now_index, usize::MAX);
        assert!(service.is_song_changed);
        assert!(service.now_lyric.is_none());
        assert_eq!(service.now_time, 0);
        assert!(
            !in_window(
                &FrameTime {
                    current: 0,
                    next_time: 0,
                    window_start: service.current_lyric_start_time,
                    window_end: service.current_lyric_end_time,
                    in_line: false,
                },
                0
            ),
            "清空后的窗口不应包含任何播放进度"
        );
    }

    #[test]
    fn refresh_uses_latest_progress() {
        let lyric = lyric_3();
        let f = LyricService::frame_at(&lyric, 3200);
        assert_eq!(f.current, 1);
        assert_eq!(f.next_time, 1800);
    }

    fn song(bid: i64, sid: i64, title: &str) -> OsuSongInfo {
        OsuSongInfo {
            bid,
            sid,
            length: 100_000,
            now: 0,
            artist: "artist".into(),
            artist_unicode: "artist".into(),
            title: title.into(),
            title_unicode: title.into(),
        }
    }

    #[test]
    fn identity_is_bid_not_title() {
        let a = SongIdent::from_song(&song(1, 100, "Lemon"));
        let b = SongIdent::from_song(&song(2, 200, "Lemon"));
        assert_ne!(a, b, "标题相同不能判定为同一首歌");

        let a2 = SongIdent::from_song(&song(1, 100, "Lemon"));
        assert_eq!(a, a2, "bid 相同即同一首歌");
    }

    #[test]
    fn identity_ignores_artist_and_length() {
        let mut other = song(1, 100, "Lemon");
        other.artist = "别的艺术家".into();
        other.length = 999;
        assert_eq!(
            SongIdent::from_song(&song(1, 100, "Lemon")),
            SongIdent::from_song(&other)
        );
    }

    #[test]
    fn generation_advances_on_every_state_change() {
        let mut service = LyricService::new();
        let g0 = service.generation();
        service.clear_display_sync();
        assert!(service.generation() > g0, "清屏必须换代");

        let g1 = service.generation();
        service.clear_state();
        assert_eq!(
            service.generation(),
            g1,
            "clear_state 只是内部重置, 不应自行换代(否则 begin_song 的校验会自相矛盾)"
        );
    }

    #[test]
    fn stale_generation_is_rejected() {
        let mut service = LyricService::new();
        service.clear_display_sync();
        let stale = service.generation();
        service.clear_display_sync();

        assert_ne!(stale, service.generation());
        assert!(!service.generation_matches(stale), "旧代际必须被判为过期");
    }

    #[test]
    fn generation_is_monotonic() {
        let mut service = LyricService::new();
        let mut prev = service.generation();
        for _ in 0..5 {
            service.clear_display_sync();
            assert!(service.generation() > prev);
            prev = service.generation();
        }
    }
}

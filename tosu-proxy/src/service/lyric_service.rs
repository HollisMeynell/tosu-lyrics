use crate::database::{
    LyricBlockEntity, LyricCacheEntity, LyricConfigEntity, SCOPE_BID, SCOPE_SID,
};
use crate::error::{Error, Result};
use crate::lyric::{
    Lyric, LyricLine, LyricResult, LyricSource, LyricSourceEnum, NETEASE_LYRIC_SOURCE,
    QQ_LYRIC_SOURCE, SongInfo, SongInfoKey,
};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::lyric::{LyricLinePayload, LyricPayload, SequenceType};
use crate::model::websocket::setting::SettingPayload;
use crate::model::websocket::setting::block::BlockItem;
use crate::osu_source::OsuSongInfo;
use crate::server::ALL_SESSIONS;
use sea_orm::EntityTrait;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinSet;
use tracing::{debug, error};

pub static LYRIC_SERVICE: LazyLock<Mutex<LyricService>> =
    LazyLock::new(|| Mutex::new(LyricService::default()));

/// 取服务锁（供其它服务模块使用）
pub async fn lyric_service() -> tokio::sync::MutexGuard<'static, LyricService> {
    LYRIC_SERVICE.lock().await
}

/// 歌曲身份（B-00 契约）。
///
/// **判定"同一首歌"只认 `bid`**。`title` 仅用于展示与"标题级黑名单规则"，
/// 绝不用于判等 —— 标题相同的两首歌是两首歌。
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

/// 陈旧请求的错误标记。HTTP 层据此返回 409 `song_changed`。
///
/// 用独立常量而不是任意字符串，避免"解析错误文本判断类型"。
pub const STALE_REQUEST: &str = "stale_request";

/// "首行之前"这一特殊窗口的左端点, 小于任何合法播放进度
const BEFORE_FIRST_LINE: i32 = i32::MIN;

/// 歌词时间(秒, f32) -> 毫秒
#[inline]
fn to_ms(seconds: f32) -> i32 {
    (seconds * 1000f32) as i32
}

/// 一帧歌词帧的时间信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTime {
    /// 当前行下标; 首行之前为 0
    pub current: i32,
    /// 距离下一行开始的剩余毫秒; 末行为 -1
    pub next_time: i32,
    /// 当前时间窗口 `[window_start, window_end)`
    window_start: i32,
    window_end: i32,
    /// 是否已进入某一行。`false` 表示"首行之前", 此时不锁定下标,
    /// 以便播放头进入首行时仍会推送一帧正确的 next_time
    in_line: bool,
}

pub struct LyricService {
    // 当前歌词的下标
    now_index: usize,
    now_lyric: Option<Lyric>,
    now_save_cache: Option<OsuSongInfo>,
    // 偏移值, 毫秒
    offset: i32,

    // 最近一次收到的播放进度, 毫秒(不含偏移)
    now_time: i32,

    // 当前歌词的起始/终止时间 毫秒
    current_lyric_start_time: i32,
    current_lyric_end_time: i32,

    music_cache: Arc<Mutex<HashMap<&'static str, Vec<SongInfo>>>>,

    // 任务取消通道
    cancel_tx: broadcast::Sender<()>,
    cancel_rx: Mutex<Option<broadcast::Receiver<()>>>,

    // 当前歌曲是否切换的状态
    is_song_changed: bool,

    /// 歌曲代际（B-00 契约）。
    ///
    /// 每次换歌 / 回菜单 / 清空展示都自增。所有异步结果（搜索、取词、上传、
    /// 换源）在**提交前**必须比对代际：不相等就丢弃。
    ///
    /// 这是正确性保证，不是优化 —— `abort` 只是尽力而为，任务可能在
    /// abort 生效前就已经算完并在等锁。
    generation: u64,

    /// 当前歌曲身份（换歌立即更新，回菜单清空）
    now_ident: Option<SongIdent>,
}

/// 一次换歌需要在阶段 2/3 之间携带的全部信息。
///
/// 显式携带而不是回头读全局状态，是为了让"提交时的世界"和"发起时的世界"
/// 可以被直接比对 —— 这是代际校验之外的第二道保险。
struct SearchPlan {
    ident: SongIdent,
    title: String,
    artist: String,
    length: u32,
    #[allow(dead_code)]
    bid: i32,
    #[allow(dead_code)]
    sid: i32,
    generation: u64,
}

impl LyricService {
    pub fn new() -> Self {
        let mut cache = HashMap::with_capacity(2);
        cache.insert(QQ_LYRIC_SOURCE.name(), Vec::with_capacity(10));
        cache.insert(NETEASE_LYRIC_SOURCE.name(), Vec::with_capacity(10));
        // 创建任务取消通道
        let (cancel_tx, cancel_rx) = broadcast::channel(1);
        Self {
            now_index: 0,
            now_lyric: None,
            now_save_cache: None,
            offset: 0,
            now_time: 0,
            current_lyric_start_time: -1,
            current_lyric_end_time: -1,
            // 使用 Arc 和 Mutex 包装缓存
            music_cache: Arc::new(Mutex::new(cache)),
            cancel_tx,
            cancel_rx: Mutex::new(Some(cancel_rx)),
            is_song_changed: true,
            generation: 0,
            now_ident: None,
        }
    }

    /// 当前代际（测试与提交校验用）
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 异步结果提交前的唯一判据：代际是否仍然有效
    pub fn generation_matches(&self, generation: u64) -> bool {
        self.generation == generation
    }

    /// 立即作废所有在途异步操作。
    ///
    /// **必须在"观察到歌曲变化"的那一刻调用**，而不是等防抖结束、
    /// `song_change` 真正开始时才换代。否则中间这 100ms 里，
    /// 一个已经发出的搜索/取词仍然会拿旧代际通过校验并写入新歌状态。
    pub fn invalidate_async(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// 仅用于测试：同步换代，避免在单测里 await 广播
    #[cfg(test)]
    fn clear_display_sync(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
    }

    /// 读取当前歌曲身份（短暂持锁，供黑名单等外部服务使用）
    pub async fn now_ident() -> Option<SongIdent> {
        lyric_service().await.now_ident.clone()
    }

    /// 换歌入口（B-00 重构）。
    ///
    /// **不再全程持锁**：只有"换代 + 读本地状态"和"提交结果"两段短暂持锁，
    /// 联网搜索阶段完全不持锁。这样快速切歌时旧歌的搜索不会阻塞新歌，
    /// 而过期结果靠在提交前比对 `generation` 丢弃 —— 不是只靠 abort。
    pub async fn song_change(song: OsuSongInfo) -> Result<()> {
        // ---- 阶段 1: 短暂持锁, 换代并读本地状态 ----
        let (generation, plan) = {
            let mut svc = lyric_service().await;
            svc.begin_song(song).await?
        };

        let Some(plan) = plan else {
            return Ok(()); // 命中黑名单 / 命中缓存, 阶段 1 内已完成
        };

        // ---- 阶段 2: 不持锁, 联网搜索 ----
        let results = Self::search_sources(&plan).await;

        // ---- 阶段 3: 校验代际后提交 ----
        let mut svc = lyric_service().await;
        svc.commit_search(generation, plan, results).await
    }

    /// 阶段 1：换代、记录身份、清屏、查黑名单 / 偏移 / 缓存。
    ///
    /// 返回 `None` 表示已经处理完毕（被屏蔽或命中缓存），调用方不必联网。
    async fn begin_song(&mut self, song: OsuSongInfo) -> Result<(u64, Option<SearchPlan>)> {
        // 换代：此后所有更早发出的异步结果都会在提交时被判为过期
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;

        // 换歌时先重置歌词展示状态并清空展示, 避免旧歌词残留/旧窗口吞掉新歌推送
        self.clear_state();
        Self::broadcast_clear().await;
        self.clear_cache().await;
        // 以 tosu 上报的新歌进度为准, 搜索期间由后续时间事件继续刷新
        self.now_time = if song.now < 0 { 0 } else { song.now };

        let ident = SongIdent::from_song(&song);
        let title = song.title_unicode.to_string();
        let artist = song.artist_unicode.to_string();
        let bid = ident.bid;
        let sid = ident.sid;
        let length = if song.length < 0 {
            0u32
        } else {
            song.length as u32
        };

        self.now_ident = Some(ident.clone());
        self.now_save_cache = Some(song);

        // 黑名单：只看这个人自己声明的作用域, 不回退到同名标题
        self.offset = LyricConfigEntity::get_offset(bid).await?;

        // 黑名单**现查**而不是缓存：规则可能在歌曲播放中途被增删，
        // 缓存一个 is_blocked 字段就会在那种时刻变成过期真相。
        let blocked = crate::service::block_service::blocked_rule(bid, sid, &title)
            .await
            .map_err(|err| Error::Runtime(err.to_string()))?
            .is_some();
        if blocked {
            debug!("{title} 命中黑名单, 不加载歌词");
            return Ok((generation, None));
        }

        // 先查询缓存（带 TTL：过期条目视为未命中）
        let ttl = crate::service::cache_service::ttl_ms();
        let cache = match LyricCacheEntity::find_by_bid(bid, ttl).await? {
            Some(v) => Some(v),
            None => LyricCacheEntity::find_by_sid(sid, ttl).await?,
        };

        if let Some(cache) = cache {
            match Lyric::from_json_cache(cache.cache.as_slice()) {
                Ok(lyric) => {
                    self.now_lyric = Some(lyric);
                    debug!("通过缓存加载 {title}");
                    // 用最新进度而不是进歌消息里的位置, 查询缓存期间播放头可能已前进
                    self.push_now(self.now_time).await;
                    return Ok((generation, None));
                }
                Err(err) => {
                    // R9：这里原本构造了 delete 却没 await，失效条目永远不会被清掉
                    let removed = LyricCacheEntity::delete_by_bid(cache.bid).await;
                    error!("缓存失效(已移除 bid={}): {} ({:?})", cache.bid, err, removed);
                }
            }
        }

        Ok((
            generation,
            Some(SearchPlan {
                ident,
                title,
                artist,
                length,
                bid,
                sid,
                generation,
            }),
        ))
    }

    /// 阶段 2：并发查询各歌词源。**不持锁**，只把结果返回给调用方。
    async fn search_sources(plan: &SearchPlan) -> HashMap<&'static str, Vec<SongInfo>> {
        let mut join_set = JoinSet::new();
        Self::spawn_search_task(
            &mut join_set,
            &*QQ_LYRIC_SOURCE,
            plan.title.clone(),
            plan.artist.clone(),
        );
        Self::spawn_search_task(
            &mut join_set,
            &*NETEASE_LYRIC_SOURCE,
            plan.title.clone(),
            plan.artist.clone(),
        );

        let mut out = HashMap::with_capacity(2);
        while let Some(res) = join_set.join_next().await {
            if let Ok(Some((name, musics))) = res {
                out.insert(name, musics);
            }
        }
        out
    }

    /// 阶段 3：校验代际 / 身份后才允许提交搜索结果。
    async fn commit_search(
        &mut self,
        generation: u64,
        plan: SearchPlan,
        results: HashMap<&'static str, Vec<SongInfo>>,
    ) -> Result<()> {
        if !self.generation_matches(generation) {
            debug!(
                "丢弃过期搜索结果: {} 的代际 {} != 当前 {}",
                plan.title, generation, self.generation
            );
            return Ok(());
        }
        // 双重校验：代际相同但身份不同（理论上不该发生）同样丢弃
        match &self.now_save_cache {
            Some(key) if SongIdent::from_song(key) == plan.ident => {}
            _ => {
                debug!("丢弃无主搜索结果: {}", plan.title);
                return Ok(());
            }
        }

        // 结果写入共享缓存, 供 B-05 的"搜索结果"接口读取
        {
            let mut cache = self.music_cache.lock().await;
            for (name, musics) in results {
                if let Some(slot) = cache.get_mut(name) {
                    slot.extend(musics);
                }
            }
        }

        // 与旧版一致: 网易云优先, QQ 兜底
        if self
            .try_source_lyric(&*NETEASE_LYRIC_SOURCE, &plan.title, plan.length, &plan.artist)
            .await?
        {
            return Ok(());
        }
        if self
            .try_source_lyric(&*QQ_LYRIC_SOURCE, &plan.title, plan.length, &plan.artist)
            .await?
        {
            return Ok(());
        }

        self.now_lyric = None;
        Ok(())
    }


    /// 应用上传的歌词（B-07）。
    ///
    /// `expect_sid` 是**发起上传时**的歌曲归属。提交前重新校验：
    /// 当前歌还在、且 sid 没变。上传期间切歌则整份丢弃。
    ///
    /// 归属用 sid 而不是 bid：同谱面集的其它难度可以共享这份手工歌词。
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

        self.now_lyric = Some(lyric);
        // 按当前播放位置立即下发，而不是回到第 0 行
        let now = self.now_time;
        self.push_now(now).await;
        Ok(())
    }

    /// 重新加载当前歌（黑名单解除 / 外部要求刷新时使用）。
    ///
    /// 走完整的换歌流程，因此同样受代际保护；代价是可能重新联网搜索。
    pub async fn reload_current() {
        let song = {
            let svc = lyric_service().await;
            svc.now_save_cache.clone()
        };
        let Some(song) = song else { return };
        if let Err(err) = Self::song_change(song).await {
            error!("重新加载当前歌失败: {err}");
        }
    }

    /// 从指定源取词并缓存, 成功后立即下发首帧
    /// 返回该源是否成功取到歌词
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

        // 立即下发首帧; 搜索耗时期间播放头已经前进, 因此用最新进度而不是搜索开始时的位置
        self.push_now(self.now_time).await;
        Ok(true)
    }

    /// 重置歌词展示状态(换歌/清空时调用)
    pub fn clear_state(&mut self) {
        self.now_lyric = None;
        self.now_index = usize::MAX;
        self.current_lyric_start_time = -1;
        self.current_lyric_end_time = -1;
        self.is_song_changed = true;
        self.now_time = 0;
    }

    /// 当前展示帧(按最近播放进度计算), 无歌词时返回 `None`
    pub fn current_frame(&self) -> Option<FrameTime> {
        let lyric = self.now_lyric.as_ref()?;
        if lyric.get_lyrics().is_empty() {
            return None;
        }
        let t = self.now_time.saturating_add(self.offset);
        Some(Self::frame_at(lyric, t))
    }

    /// 当前歌曲信息(切歌后立即更新, 回到菜单后清空)
    pub fn get_now_song(&self) -> Option<&OsuSongInfo> {
        self.now_save_cache.as_ref()
    }

    /// 清空展示: 重置歌词状态并广播清屏, **保留当前歌曲信息**。
    ///
    /// 语义为"持续清屏", 直到切歌 / 换源 / 上传歌词才会重新出现。
    pub async fn clear_display(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
        Self::broadcast_clear().await;
    }

    /// 回到菜单 / 无歌曲: 清空展示并忘掉当前歌曲
    pub async fn song_clean(&mut self) {
        // 换代：在途的搜索结果不能再落到"已经是菜单"的状态上
        self.generation = self.generation.wrapping_add(1);
        self.clear_state();
        self.now_save_cache = None;
        self.now_ident = None;
        Self::broadcast_clear().await;
    }

    /// 向所有歌词页广播"清空歌词"
    pub async fn broadcast_clear() {
        let clean_message = SettingPayload::new("setClear".to_string());
        ALL_SESSIONS
            .send_to_all_client(Into::<WebSocketMessage>::into(clean_message).into())
            .await;
    }

    /// 按播放进度(毫秒, 已含偏移)计算当前展示帧的时间信息。
    ///
    /// 纯函数, 不依赖全局状态, 便于单测覆盖首行前 / 行中 / 跳转 / 末行等边界。
    fn frame_at(lyric: &Lyric, t: i32) -> FrameTime {
        let t_sec = t as f32 / 1000f32;
        match lyric.find_line(t_sec) {
            Some((index, line)) => {
                let line_start = to_ms(line.time);
                match lyric.get_line_by_index(index + 1) {
                    // next_time 统一为"距离下一行开始的剩余毫秒"
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
                    // 末行: 没有下一行
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
                // 在首行之前: 展示第 0 行, next_time 为距离首行的剩余毫秒。
                // `in_line = false` 表示不锁定下标, 播放头进入首行时会重新推送
                // 一帧带正确 next_time 的帧。
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

    /// 把帧的时间信息写入服务状态, 保证下一帧的窗口判断基于最新位置
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

    /// 歌词加载完成后立即推送首帧(完整歌词 + 当前行), 不必等待下一次时间事件
    ///
    /// `now_ms` 为**不含偏移**的播放进度(毫秒), 内部与 `time_next` 统一加上偏移,
    /// 保证换源 / 上传歌词后按实际播放位置刷新, 而不是固定回到第 0 行。
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

        let t = (if now_ms < 0 { 0 } else { now_ms }).saturating_add(self.offset);
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

    /// 生成当前歌词状态快照, 用于歌词页接入(含 OBS 刷新 / 断线重连)时立即下发。
    ///
    /// 按**当前播放进度**重新计算下标与剩余时长, 而不是重放上一帧:
    /// 晚加入的展示端拿到的 nextTime 才是它真正需要的剩余时间。
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

    /// 时间单位为毫秒
    pub async fn time_next(&mut self, t: i32) -> Result<()> {
        // 记录最近进度: 换源 / 上传歌词 / 重连快照都按这个位置刷新
        self.now_time = if t < 0 { 0 } else { t };

        let Some(lyric) = self.now_lyric.as_ref() else {
            return Ok(());
        };

        let t = self.now_time.saturating_add(self.offset);

        // 时间未越过当前行窗口(左闭右开): 无需推送。
        // 右端取开区间, 保证播放头到达下一行起点时一定重新计算, 也保证
        // "首行之前"的窗口在到达首行时能正常结束。
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
        // 下标变化、或刚要下发完整歌词、或"首行之前"这一状态本身发生变化时才推送
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

    // 清理缓存
    async fn clear_cache(&mut self) {
        self.is_song_changed = true;

        // Step 1: 广播取消信号给所有活动任务
        let _ = self.cancel_tx.send(());

        // Step 2: 重置取消通道接收器，为下一批任务做准备
        let new_rx = self.cancel_tx.subscribe();
        *self.cancel_rx.lock().await = Some(new_rx);

        // Step 3: 清空音乐缓存
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
        let mut cache = self.music_cache.lock().await;
        let songs_to_search = cache.get_mut(source.name()).unwrap();

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

    /// 阶段 2 的并发单元：只做网络搜索并把结果**返回**，绝不写共享状态。
    ///
    /// 旧实现在任务里直接往 `music_cache` 写，于是一个已经过期的任务也能污染
    /// 新歌的搜索结果。现在写入统一由阶段 3 在代际校验之后完成。
    fn spawn_search_task<S: LyricSource + 'static>(
        tasks: &mut JoinSet<Option<(&'static str, Vec<SongInfo>)>>,
        source: &'static S,
        title: String,
        artist: String,
    ) {
        tasks.spawn(async move {
            match source.search_all_music(&title, &artist).await {
                Ok(musics) => Some((source.name(), musics)),
                Err(err) => {
                    debug!("{} 搜索失败: {err}", source.name());
                    None
                }
            }
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
        // 缓存写入失败不应阻止换源生效
        if let Some(save_key) = &self.now_save_cache {
            Self::save_lyric(save_key, &lyric)
                .await
                .inspect_err(|err| error!("存储缓存异常: {}", err))
                .ok();
        }
        self.now_lyric = Some(lyric);
        // 按当前播放位置立即下发完整帧, 不等下一次时间事件
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

        self.now_lyric = Some(lyric);
        // 按当前播放位置立即下发完整帧, 不等下一次时间事件
        self.push_now(self.now_time).await;
        Ok(())
    }

    pub fn get_now_all_lyrics(&self) -> Option<&[LyricLine]> {
        self.now_lyric.as_ref().map(Lyric::get_lyrics)
    }

    /// 屏蔽当前播放的歌（旧 WS 管理命令入口，必须保留）。
    ///
    /// 写入 `bid` 作用域规则；**不再动 offset 那一行** —— 两者现在分表存储，
    /// 拉黑不会丢失偏移，取消拉黑也不会复原成旧偏移。
    ///
    /// 调用方持有服务锁，因此这里只做 DB 写 + 清屏；
    /// "解除后重新加载"由调用方在**释放锁之后**调用 `LyricService::reload_current()`。
    pub async fn set_block(&mut self, block: bool) -> Result<()> {
        let Some(key) = self.now_save_cache.as_ref() else {
            return Err("no save cache is set".into());
        };
        let bid = key.bid as i32;
        let sid = key.sid as i32;
        let title = key.title_unicode.to_string();

        if block {
            LyricBlockEntity::upsert(SCOPE_BID, &bid.to_string(), &title, sid, "").await?;
            // 同步清空展示, 避免拉黑后旧歌词残留在所有展示端
            self.clear_state();
            Self::broadcast_clear().await;
        } else {
            // 只删 bid 作用域的规则；sid/title 规则是用户显式声明的更大范围，不动
            LyricBlockEntity::delete_by_scope_value(SCOPE_BID, &bid.to_string()).await?;
        }
        Ok(())
    }

    /// 设置偏移（毫秒）。立即生效并推帧，不等下一次时间事件。
    ///
    /// 与黑名单完全解耦：`offset == 0` 只删除偏移行，不会碰任何屏蔽规则。
    pub async fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
        // 重置当前时间窗口，强制 time_next 重新计算
        // 防止 offset 变化后用旧的时间窗口判断导致错误跳行
        self.current_lyric_start_time = -1;
        self.current_lyric_end_time = -1;

        if let Some(key) = self.now_save_cache.as_ref() {
            let bid = key.bid as i32;
            let sid = key.sid as i32;
            let title = key.title_unicode.to_string();
            if let Err(err) = LyricConfigEntity::save_offset(bid, sid, &title, offset).await {
                error!("保存偏移失败: {err}");
            }
        }

        // 偏移变了就立刻按新偏移重算并下发当前帧
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
            this.title.as_ref(),
            this.length,
            lyric,
        )
        .await
    }

    /// 旧 WS 协议的屏蔽列表形态。
    ///
    /// R6 回归点：旧实现从 `get_all_disable()` 拿到的是 `(bid, sid, title)`，
    /// 却按 `(sid, bid, title)` 解构，导致 bid/sid 互换。现在按作用域显式映射，
    /// 不再依赖元组顺序。
    pub async fn get_all_block_list(&self) -> Result<Vec<BlockItem>> {
        Ok(LyricBlockEntity::list_all()
            .await?
            .into_iter()
            .map(|rule| {
                let numeric = rule.value.parse::<i64>().unwrap_or(0).max(0) as u32;
                let (bid, sid) = match rule.scope.as_str() {
                    SCOPE_BID => (numeric, rule.sid.max(0) as u32),
                    SCOPE_SID => (0, numeric),
                    // title 规则没有数字身份，用 0 表示"非数字作用域"
                    _ => (0, 0),
                };
                BlockItem { bid, sid, title: rule.title }
            })
            .collect())
    }


    // ---------------- B-05：内容管理需要的公开能力 ----------------

    /// 当前代际 + 当前歌曲身份的快照。
    ///
    /// 异步操作（主动搜索 / 预览 / 应用来源 / 上传）在**发起时**取一份，
    /// 提交前用它比对；不一致就说明期间切了歌或清了屏，结果必须丢弃。
    pub async fn ticket() -> (u64, Option<SongIdent>) {
        let svc = lyric_service().await;
        (svc.generation, svc.now_ident.clone())
    }

    /// 当前内存里的全部搜索候选（各来源合并）
    pub async fn all_candidates(&self) -> Vec<(&'static str, SongInfo)> {
        let cache = self.music_cache.lock().await;
        let mut out = Vec::new();
        for (name, list) in cache.iter() {
            for song in list {
                out.push((*name, song.clone()));
            }
        }
        out
    }

    /// 主动搜索：给定条件并发查询各来源，结果写入内存候选。
    ///
    /// 联网阶段**不持锁**；提交前校验代际，过期则返回 `Err` 由调用方转成 409。
    pub async fn search_now(title: String, artist: String) -> Result<Vec<(&'static str, SongInfo)>> {
        let (generation, _ident) = Self::ticket().await;
        let results = Self::search_raw(&title, &artist).await;

        let svc = lyric_service().await;
        if !svc.generation_matches(generation) {
            return Err(Error::Runtime(STALE_REQUEST.into()));
        }
        {
            let mut cache = svc.music_cache.lock().await;
            for (name, musics) in results {
                if let Some(slot) = cache.get_mut(name) {
                    slot.clear();
                    slot.extend(musics);
                }
            }
        }
        Ok(svc.all_candidates().await)
    }

    /// 并发查询各来源，返回结果但**不写任何共享状态**
    async fn search_raw(
        title: &str,
        artist: &str,
    ) -> HashMap<&'static str, Vec<SongInfo>> {
        let mut join_set = JoinSet::new();
        Self::spawn_search_task(
            &mut join_set,
            &*QQ_LYRIC_SOURCE,
            title.to_string(),
            artist.to_string(),
        );
        Self::spawn_search_task(
            &mut join_set,
            &*NETEASE_LYRIC_SOURCE,
            title.to_string(),
            artist.to_string(),
        );
        let mut out = HashMap::with_capacity(2);
        while let Some(res) = join_set.join_next().await {
            if let Ok(Some((name, musics))) = res {
                out.insert(name, musics);
            }
        }
        out
    }

    /// 应用来源绑定后的歌词并在校验通过时提交（B-05）。
    ///
    /// `generation` 是**发起取词时**的快照。这里做三重校验：
    /// 代际没变、当前歌仍在、sid 仍是同一个 —— 任一不满足就丢弃，
    /// 不写入 `now_lyric`、不写缓存、不广播。
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
        self.now_lyric = Some(lyric);
        let now = self.now_time;
        self.push_now(now).await;
        Ok(())
    }

    /// 换代 + 清空展示，但**不建立清屏抑制**。
    ///
    /// 供"换源 / 恢复自动匹配"这类**马上要重新加载**的场景使用。
    /// 不能用 `clear_display()` —— 那会记住当前歌并抑制它，
    /// 紧接着的重新加载就会被自己挡住（这确实发生过一次）。
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

    /// 三行歌词: 1s / 3s / 5s
    fn lyric_3() -> Lyric {
        Lyric::parse("[00:01.00]a\n[00:03.00]b\n[00:05.00]c", None, None)
            .expect("测试歌词应能解析")
    }

    /// `time_next` 使用的窗口判断: 左闭右开
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

    /// R1/R5 回归: 首行之前的窗口必须在首行起点处结束,
    /// 否则 `time_next` 会因"仍在窗口内"提前返回, 首行永远拿不到正确的 next_time
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

    /// R1 回归: 首行之前不锁定下标, 进入首行时必须被识别为状态变化
    #[test]
    fn entering_first_line_counts_as_change() {
        let lyric = lyric_3();
        let mut service = LyricService::new();
        service.now_lyric = Some(lyric);

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

    /// R5: next_time 是"剩余毫秒", 行中进入不是整行时长
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

    /// R5: 行边界是左闭右开, t == 下一行起点必须落到下一行
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

    /// 跳转: 前向 / 反向 / 末行
    #[test]
    fn seek_forward_backward_and_last_line() {
        let lyric = lyric_3();

        // 反向跳回第一行
        let back = LyricService::frame_at(&lyric, 1500);
        assert_eq!(back.current, 0);
        assert_eq!(back.next_time, 1500);

        // 跳到末行: 没有下一行
        let last = LyricService::frame_at(&lyric, 6000);
        assert_eq!(last.current, 2);
        assert_eq!(last.next_time, -1);
        assert_eq!(last.window_end, i32::MAX);
        assert!(in_window(&last, 6000));
    }

    /// 清空后窗口必须失效, 保证新歌第一帧不被旧窗口挡住
    #[test]
    fn clear_state_invalidates_window() {
        let mut service = LyricService::new();
        service.now_lyric = Some(lyric_3());
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

    /// 修正 `now_time` 语义: 换源 / 上传按最近进度刷新, 而不是固定 0
    #[test]
    fn refresh_uses_latest_progress() {
        let lyric = lyric_3();
        // now_time = 3.2s 时刷新, 应落在第二行而不是第一行
        let f = LyricService::frame_at(&lyric, 3200);
        assert_eq!(f.current, 1);
        assert_eq!(f.next_time, 1800);
    }

    // ---------- B-00: 歌曲身份与代际 ----------

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

    /// 身份判定只看 bid: 标题相同但 bid 不同 = 两首歌
    #[test]
    fn identity_is_bid_not_title() {
        let a = SongIdent::from_song(&song(1, 100, "Lemon"));
        let b = SongIdent::from_song(&song(2, 200, "Lemon"));
        assert_ne!(a, b, "标题相同不能判定为同一首歌");

        let a2 = SongIdent::from_song(&song(1, 100, "Lemon"));
        assert_eq!(a, a2, "bid 相同即同一首歌");
    }

    /// 同一首歌换了标题(例如 Unicod/ASCII 变化)仍视为同一首
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

    /// 换歌必须换代; 回菜单 / 清屏同样换代
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

    /// 过期代际的提交必须被丢弃
    #[test]
    fn stale_generation_is_rejected() {
        let mut service = LyricService::new();
        service.clear_display_sync();
        let stale = service.generation();
        service.clear_display_sync(); // 又换了一代

        assert_ne!(stale, service.generation());
        assert!(
            !service.generation_matches(stale),
            "旧代际必须被判为过期"
        );
    }

    /// 幂等: 同一首歌重复进入不应破坏代际单调性
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

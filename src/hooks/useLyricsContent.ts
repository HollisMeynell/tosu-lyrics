import { createSignal } from "solid-js";
import {
    Candidate,
    CurrentLyricDto,
    PreviewDto,
    applySource,
    clearSource,
    checkTranslations,
    fetchCurrentLyric,
    fetchCandidates,
    previewCandidate,
    searchLyrics,
    setOffset as setOffsetApi,
} from "@/services/lyricsContentService";
import { ApiError } from "@/services/settingsService";
import { clearDisplay } from "@/services/statusService";

const message = (err: unknown) =>
    err instanceof ApiError ? err.message : String(err);

/**
 * 关键约束：每个候选按钮必须携带自己的身份 (source, key)，
 * 而不是依赖"上一次预览的那一个"——那会导致所有应用按钮都在用最后一首。
 */
/**
 * "当前歌曲自动候选"的**模块级**缓存与在途请求复用。
 *
 * **为什么放模块级**：页面重新挂载（组件实例重建）不应丢掉缓存，否则每次进页面
 * 都要重新 POST 一次搜索。
 *
 * **key**：用当前歌曲的稳定身份（sid + bid + title + artist + length）拼接。任一
 * 字段变化都算不同歌，因此不会错误复用旧结果 —— 只用 sid 是不够的，同一 sid 也
 * 可能换难度/换标题。
 *
 * **value 缓存 Promise**：多个组件/页面同时要同一首歌时只发出一次 HTTP 请求。
 * 请求失败不写缓存，避免一次网络错误导致之后一直拿不到候选。
 */
const AUTO_CANDIDATE_TTL_MS = 5 * 60 * 1000;
const autoCandidateCache = new Map<
    string,
    { at: number; items: Candidate[] }
>();
const autoCandidateInflight = new Map<string, Promise<Candidate[]>>();

/** 让某首歌的自动候选缓存失效（例如换了来源绑定后需要拿到新的候选状态） */
const invalidateAutoCandidates = () => {
    autoCandidateCache.clear();
};

export function createLyricsContentController() {
    const [current, setCurrent] = createSignal<CurrentLyricDto | null>(null);
    const [noSong, setNoSong] = createSignal(false);
    const [candidates, setCandidates] = createSignal<Candidate[]>([]);
    /**
     * 当前展示的候选是否来自**用户主动搜索**。
     *
     * - false：展示的是"当前歌曲候选"（自动加载），切歌时应自动刷新为新歌候选；
     * - true ：展示的是用户手动搜索结果，切歌时**保持不动**，不被自动候选顶掉。
     */
    const [manualSearch, setManualSearch] = createSignal(false);
    /** 最近一次"当前歌曲候选"，与展示用的 candidates 分开保存 */
    const [songCandidates, setSongCandidates] = createSignal<Candidate[]>([]);
    /**
     * 候选请求代次：只有最新一次请求的结果允许写入状态。
     * 防止旧歌曲的请求晚于新歌曲返回时覆盖新结果。
     */
    let candidatesGeneration = 0;
    /** 预览结果按 `source|key` 缓存，保证每行显示的是**它自己**的预览 */
    const [previews, setPreviews] = createSignal<Record<string, PreviewDto>>({});
    const [previewing, setPreviewing] = createSignal<string | null>(null);
    /** 「加载列表歌词」批量加载状态 */
    const [listLoading, setListLoading] = createSignal(false);
    /** 本页预览区当前展示的候选 id（null = 展示正在使用的当前歌词）。 */
    const [previewTarget, setPreviewTarget] = createSignal<string | null>(null);
    const [loading, setLoading] = createSignal(false);
    const [searching, setSearching] = createSignal(false);
    const [saving, setSaving] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);
    /** `source|key` -> 是否有翻译；未知的键不在表里（显示"读取中"） */
    const [translations, setTranslations] = createSignal<Record<string, boolean>>({});

    const candidateId = (c: { source: string; key: string }) =>
        `${c.source}|${c.key}`;

    const loadCurrent = async () => {
        setLoading(true);
        setError(null);
        try {
            setCurrent(await fetchCurrentLyric());
            setNoSong(false);
            return true;
        } catch (err) {
            // "没有播放中的歌曲"是正常状态，不是错误
            if (err instanceof ApiError && err.code === "no_song") {
                setCurrent(null);
                setNoSong(true);
                return true;
            }
            setError(message(err));
            return false;
        } finally {
            setLoading(false);
        }
    };

    /**
     * 自动加载**当前播放歌曲**的候选。
     *
     * 走搜索接口（不带条件 = 按当前歌曲搜索），而不是读 `GET /search-results`
     * 缓存 —— 后者只是读服务端候选缓存，切歌瞬间还没为新歌生成，会读到空列表并卡住。
     *
     * 在此基础上加了"请求复用"避免每次进页面 / 切歌回来都重新搜索：
     * - 先拿当前歌曲身份拼 key（拿不到就跳过缓存，仍执行搜索）；
     * - 命中未过期的结果缓存 → 直接用，**不发请求**；
     * - 同一首歌有在途请求 → 复用同一个 Promise，**不发第二个请求**；
     * - 只有成功才写结果缓存，失败不写。
     *
     * 若当前展示的是用户手动搜索结果，只更新内部 songCandidates，不动 candidates。
     *
     * @param force 忽略缓存强制重新搜索（换来源绑定后需要反映新的绑定状态）
     */
    /**
     * 候选加载状态：显式区分「尚未加载」与「正在读取」。
     * idle 未加载(需加载) / loading 加载中 / loaded 已加载 / error 加载失败(可重试)
     */
    const [candidateState, setCandidateState] = createSignal<
        "idle" | "loading" | "loaded" | "error"
    >("idle");

    /**
     * **只读后端已有候选**（GET /api/lyrics/search-results）：不发起任何耗时搜索。
     *
     * 后端 commit_search() 时已把候选写进 music_cache，这里直接复用；没有结果就保持
     * idle（这一轮后端确实没有候选）。onMount 与 lyricLoading 由 true 变 false 之后调用本函数。
     *
     * 防串歌：读取前记下当前 sid，读完后若 sid 已变化则丢弃结果，
     * 避免把上一首歌的候选显示到新歌上。
     */
    /**
     * 切歌瞬间**同步清空**候选，避免上一首的候选短暂闪现。
     * 只动候选相关状态，不碰主歌词 / WS / current。
     */
    const clearCandidates = () => {
        setSongCandidates([]);
        setCandidates([]);
        setCandidateState("idle");
    };
    // 「候选尽早显示」的读取代次：新歌 / loading 变化后旧读取立即失效
    let candidateReadToken = 0;

    /**
     * 清空旧候选并开始**有限次**读取后端候选（`loading=true` 时调用）。
     *
     * 后端 `search_sources()` 一完成就写 `music_cache`，而此刻 `loading` 仍为 true，
     * 所以这里在 loading 期间轻量轮询 `GET /api/lyrics/search-results`
     * （**纯读缓存，不是搜索**）：立即读一次，为空则每 500ms 再读。
     *
     * 终止条件：**读到非空** / 切歌换代（`candidateReadToken` 失效）/ `attempt>=60` 安全上限。
     * **注意：`lyricLoading=false` 不作为终止条件** —— 否则当搜索恰好落在两次轮询之间时，
     * loading 期间的读取会全部为空、只剩 `loading=false` 的兜底读取，
     * 表现就是"候选要等主歌词结束才出现"。
     */
    const startCandidateRead = () => {
        clearCandidates();
        const token = ++candidateReadToken;
        const tick = (attempt: number) => {
            if (token !== candidateReadToken) return;
            void loadStoredCandidates().then(() => {
                if (token !== candidateReadToken) return;
                if (songCandidates().length > 0) return; // 已读到候选 → 停止
                if (attempt >= 60) return;               // 安全上限（≈30s）
                window.setTimeout(() => tick(attempt + 1), 500);
            });
        };
        tick(0);
    };
    const loadStoredCandidates = async () => {
        const sid = current()?.song?.sid;
        if (sid === undefined || sid === null) return;
        // 与 loadCandidates() 共用同一代次：任何更新的候选读取/搜索都会让它作废
        const generation = ++candidatesGeneration;
        setCandidateState("loading");
        try {
            const result = await fetchCandidates();
            // 旧请求（已被更新的候选读取取代）或已切歌 → 丢弃，绝不覆盖新歌候选
            if (generation !== candidatesGeneration) return;
            if (current()?.song?.sid !== sid) return;
            const items = result.items ?? [];
            setSongCandidates(items);
            if (!manualSearch()) setCandidates(items);
            setCandidateState(items.length > 0 ? "loaded" : "idle");
        } catch {
            // 旧请求（已被更新的候选读取取代）或已切歌 → 丢弃，绝不覆盖新歌候选
            if (generation !== candidatesGeneration) return;
            if (current()?.song?.sid !== sid) return;
            setCandidateState("idle");
        }
    };

    const loadCandidates = async (force = false) => {
        // 直接读当前歌曲身份来拼缓存 key。**这里不再补拉 /api/lyrics/current** ——
        // 那会给"开始搜索"平白加一次串行往返，把请求启动时间推后（旧版本没有这一步）。
        // 调用方负责先确保 current() 已就绪，见 Content 的 onMount。
        const song = current()?.song;
        if (!song) return; // 歌曲身份未知：不发请求
        const key = [
            song.sid,
            song.bid,
            song.title,
            song.artist,
            song.length,
        ].join("|");

        if (!force) {
            const cached = autoCandidateCache.get(key);
            if (cached && Date.now() - cached.at < AUTO_CANDIDATE_TTL_MS) {
                setSongCandidates(cached.items);
                if (!manualSearch()) setCandidates(cached.items);
                return;
            }
        }

        const generation = ++candidatesGeneration;
        try {
            let pending = autoCandidateInflight.get(key);
            if (!pending) {
                pending = searchLyrics().then((result) => result.items);
                autoCandidateInflight.set(key, pending);
                // 无论成功失败都清掉在途记录；失败不会进入结果缓存
                pending
                    .catch(() => undefined)
                    .finally(() => autoCandidateInflight.delete(key));
            }
            const items = await pending;
            if (generation !== candidatesGeneration) return; // 已有更新的请求
            autoCandidateCache.set(key, { at: Date.now(), items });
            setSongCandidates(items);
            if (!manualSearch()) setCandidates(items);
        } catch {
            // 自动候选失败（例如歌曲刚切换、当前无歌曲）不弹错误，保持原状
        }
    };

    // 搜索完成后在后台补齐翻译标记，不阻塞用户
    const fillTranslations = async (items: Candidate[]) => {
        if (items.length === 0) return;
        try {
            const results = await checkTranslations(
                items.map((c) => ({ source: c.source, key: c.key }))
            );
            setTranslations((prev) => {
                const next = { ...prev };
                for (const r of results) next[`${r.source}|${r.key}`] = r.hasTranslation;
                return next;
            });
        } catch {
            // 补标记失败不影响搜索本身
        }
    };

    /** 主动搜索。切歌导致的 409 是正确结果，单独提示。 */
    const search = async (title?: string, artist?: string) => {
        // 不带条件 = "按当前播放歌曲搜索"，语义上属于自动候选，不算手动搜索；
        // 否则用户清空输入框搜索后，会被永久钉在"手动搜索"模式上。
        const isAuto = title == null && artist == null;
        // 让在途的自动候选请求失效，避免它晚回来把手动结果覆盖掉
        const generation = ++candidatesGeneration;
        setSearching(true);
        setError(null);
        setNotice(null);
        try {
            const result = await searchLyrics(title, artist);
            if (generation !== candidatesGeneration) return false; // 已有更新的请求
            if (isAuto) {
                setSongCandidates(result.items);
                setManualSearch(false);
            } else {
                setManualSearch(true);
            }
            setCandidates(result.items);
            setTranslations({});
            void fillTranslations(result.items);
            if (result.items.length === 0) setNotice("没有找到候选");
            return true;
        } catch (err) {
            if (err instanceof ApiError && err.code === "song_changed") {
                setNotice("搜索期间歌曲已切换，结果已丢弃");
                return false;
            }
            setError(message(err));
            return false;
        } finally {
            setSearching(false);
        }
    };

    /**
     * 批量加载当前候选列表中所有候选的歌词（「加载列表歌词」按钮）。
     *
     * - 复用现有单候选机制 `preview(candidate)` → `previewCandidate(source,key)`，
     *   按 source+key 直接取词，**不重新搜索 QQ/网易云**；
     * - **已有 `previews[id]` 的候选直接跳过**，绝不重复请求；
     * - 并发上限 3，避免请求洪峰；
     * - 只写 `previews` 缓存：**不改候选顺序、不 apply 任何候选、不影响自动匹配结果**。
     */
    const loadAllCandidateLyrics = async () => {
        if (listLoading()) return;
        const pending = candidates().filter((item) => !previews()[candidateId(item)]);
        if (pending.length === 0) return;
        setListLoading(true);
        const queue = [...pending];
        const worker = async () => {
            for (;;) {
                const next = queue.shift();
                if (!next) return;
                try {
                    await preview(next, true);
                } catch {
                    // 单个候选失败不阻断其余候选
                }
            }
        };
        try {
            await Promise.all([worker(), worker(), worker()]);
        } finally {
            setListLoading(false);
        }
    };
    const preview = async (candidate: Candidate, silent = false) => {
        const id = candidateId(candidate);
        setPreviewing(id);
        setError(null);
        try {
            const result = await previewCandidate(candidate.source, candidate.key);
            setPreviews((prev) => ({ ...prev, [id]: result }));
            // 预览顺带拿到了真实数据，可以顺手确定"有没有翻译"
            setTranslations((prev) => ({
                ...prev,
                [id]: result.lines.some((l) => !!l.translation?.trim()),
            }));
            return true;
        } catch (err) {
            // 批量加载(silent=true)时，单个候选「无歌词 no lyric」只算该候选失败，
            // 不得写入全局 error，否则会弹出「操作失败：no lyric」横幅。
            if (!silent) setError(message(err));
            return false;
        } finally {
            setPreviewing(null);
        }
    };

    /**
     * 让本页预览区显示某个搜索结果的歌词。
     *
     * 只改"本页看什么"：不写 lyricStore、不影响展示端、不触发来源切换、不碰缓存。
     * 数据复用已有的 `previews` 缓存，只有没拉取过时才调用 `preview`。
     */
    const showPreview = async (candidate: Candidate) => {
        const id = candidateId(candidate);
        setPreviewTarget(id);
        if (!previews()[id]) {
            await preview(candidate);
        }
    };

    /** 关闭预览，预览区回到"当前正在使用的歌词"。 */
    const clearPreview = () => setPreviewTarget(null);

    /** 应用某个候选作为来源绑定。用的是**该候选自己的**身份。 */
    const apply = async (candidate: Candidate) => {
        setSaving(true);
        setError(null);
        setNotice(null);
        try {
            await applySource(candidate.source, candidate.key);
            setNotice(`已应用来源：${candidate.title}（${candidate.source}）`);
            // 绑定变了，候选的"当前使用"标记也变了 → 忽略缓存强制重取一次
            invalidateAutoCandidates();
            await Promise.all([loadCurrent(), loadCandidates(true)]);
            return true;
        } catch (err) {
            if (err instanceof ApiError && err.code === "song_changed") {
                setNotice("应用期间歌曲已切换，未生效");
                return false;
            }
            setError(message(err));
            return false;
        } finally {
            setSaving(false);
        }
    };

    const revertToAuto = async () => {
        setSaving(true);
        setError(null);
        setNotice(null);
        try {
            await clearSource();
            setNotice("已恢复自动匹配");
            // 同上：来源绑定变了，强制重取
            invalidateAutoCandidates();
            await Promise.all([loadCurrent(), loadCandidates(true)]);
            return true;
        } catch (err) {
            setError(message(err));
            return false;
        } finally {
            setSaving(false);
        }
    };

    /** 清空所有展示端；语义是持续清屏，直到换歌 / 换源 / 上传歌词 */
    const clear = async () => {
        setSaving(true);
        setError(null);
        setNotice(null);
        try {
            await clearDisplay();
            setNotice("已清屏（换歌 / 换源 / 上传歌词后恢复）");
            await loadCurrent();
            return true;
        } catch (err) {
            setError(message(err));
            return false;
        } finally {
            setSaving(false);
        }
    };

    const updateOffset = async (offset: number) => {
        setSaving(true);
        setError(null);
        try {
            const effective = await setOffsetApi(offset);
            setCurrent((prev) => (prev ? { ...prev, offset: effective } : prev));
            setNotice(`偏移已设为 ${effective} ms`);
            return true;
        } catch (err) {
            setError(message(err));
            return false;
        } finally {
            setSaving(false);
        }
    };

    return {
        current,
        noSong,
        candidates,
        manualSearch,
        songCandidates,
        listLoading,
        loadAllCandidateLyrics,
        previews,
        translations,
        previewing,
        previewTarget,
        loading,
        searching,
        saving,
        error,
        notice,
        candidateId,
        loadCurrent,
        clearCandidates,
        candidateState,
        startCandidateRead,
        loadStoredCandidates,
        loadCandidates,
        search,
        preview,
        showPreview,
        clearPreview,
        apply,
        revertToAuto,
        updateOffset,
        clear,
        clearError: () => setError(null),
        clearNotice: () => setNotice(null),
        setError,
    };
}

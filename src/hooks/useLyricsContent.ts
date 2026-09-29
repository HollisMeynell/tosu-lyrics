import { createSignal } from "solid-js";
import {
    Candidate,
    CurrentLyricDto,
    PreviewDto,
    applySource,
    clearSource,
    checkTranslations,
    fetchCandidates,
    fetchCurrentLyric,
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
export function createLyricsContentController() {
    const [current, setCurrent] = createSignal<CurrentLyricDto | null>(null);
    const [noSong, setNoSong] = createSignal(false);
    const [candidates, setCandidates] = createSignal<Candidate[]>([]);
    /** 预览结果按 `source|key` 缓存，保证每行显示的是**它自己**的预览 */
    const [previews, setPreviews] = createSignal<Record<string, PreviewDto>>({});
    const [previewing, setPreviewing] = createSignal<string | null>(null);
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

    const loadCandidates = async () => {
        try {
            setCandidates((await fetchCandidates()).items);
        } catch (err) {
            setError(message(err));
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
        setSearching(true);
        setError(null);
        setNotice(null);
        try {
            const result = await searchLyrics(title, artist);
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

    const preview = async (candidate: Candidate) => {
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
            setError(message(err));
            return false;
        } finally {
            setPreviewing(null);
        }
    };

    /** 应用某个候选作为来源绑定。用的是**该候选自己的**身份。 */
    const apply = async (candidate: Candidate) => {
        setSaving(true);
        setError(null);
        setNotice(null);
        try {
            await applySource(candidate.source, candidate.key);
            setNotice(`已应用来源：${candidate.title}（${candidate.source}）`);
            await Promise.all([loadCurrent(), loadCandidates()]);
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
            await Promise.all([loadCurrent(), loadCandidates()]);
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
        previews,
        translations,
        previewing,
        loading,
        searching,
        saving,
        error,
        notice,
        candidateId,
        loadCurrent,
        loadCandidates,
        search,
        preview,
        apply,
        revertToAuto,
        updateOffset,
        clear,
        clearError: () => setError(null),
        clearNotice: () => setNotice(null),
        setError,
    };
}

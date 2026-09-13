import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

/**
 * 歌词内容管理 HTTP 客户端（B-05）。
 *
 * 后端是唯一真相：搜索候选、预览、来源绑定、偏移全部走这里。
 * 前端不再通过旧 WS 向对端查询歌词。
 */

export interface LyricLineDto {
    /** 毫秒 */
    time: number;
    origin?: string;
    translation?: string;
}

export interface SongDto {
    bid: number;
    sid: number;
    title: string;
    artist: string;
    /** 毫秒；-1 表示未读到音频时长 */
    length: number;
}

/** 来源绑定（按 sid 归属） */
export interface SourceBinding {
    sid: number;
    source: string;
    key: string;
}

export interface CurrentLyricDto {
    song: SongDto;
    offset: number;
    /** 当前行下标；无歌词为 -1 */
    current: number;
    nextTime: number;
    /** 无可用歌词时为 null */
    lyric: LyricLineDto[] | null;
    /** ok | blocked | none —— 说明"为什么没有歌词" */
    lyricState: "ok" | "blocked" | "none";
    blocked: boolean;
    source: SourceBinding | null;
}

export interface Candidate {
    source: string;
    key: string;
    title: string;
    artist: string;
    /** 毫秒 */
    length: number;
    /** 是否与当前生效的来源绑定一致 */
    active: boolean;
    /** 标题匹配分（0~100），越大越匹配当前歌 */
    titleScore: number;
    /** 候选时长 - 当前歌时长（毫秒）；没有在播歌曲时为 0 */
    durationDelta: number;
}

export interface CandidateListDto {
    total: number;
    items: Candidate[];
}

export interface PreviewDto {
    source: string;
    key: string;
    lineCount: number;
    lines: LyricLineDto[];
}

const LYRICS_URL = `${BACKEND_API_BASE}/lyrics`;

async function toApiError(res: Response): Promise<ApiError> {
    let code = "unknown";
    let message = `HTTP ${res.status}`;
    try {
        const body = await res.json();
        if (body?.error?.code) {
            code = body.error.code;
            message = body.error.message ?? message;
        }
    } catch {
        // 非 JSON 错误体时保留默认信息
    }
    return new ApiError(code, message, res.status);
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as T;
}

/**
 * 当前歌词。
 *
 * **没有播放中的歌曲**时后端返回 404，这里抛出 `ApiError(code="no_song")`；
 * "有歌但暂时没歌词"是 200 + `lyric: null`，**不是错误** —— 调用方应看
 * `lyricState` 而不是把 null 当异常。
 */
export async function fetchCurrentLyric(): Promise<CurrentLyricDto> {
    return jsonOrThrow<CurrentLyricDto>(await fetch(`${LYRICS_URL}/current`));
}

export async function fetchCandidates(): Promise<CandidateListDto> {
    return jsonOrThrow<CandidateListDto>(
        await fetch(`${LYRICS_URL}/search-results`)
    );
}

/**
 * 主动搜索。不传条件时后端按当前播放歌曲搜索。
 *
 * 搜索期间若切了歌，后端返回 409 `song_changed` —— 这是**正确结果**，
 * 不是错误，调用方应提示"歌曲已切换"而不是重试。
 */
export async function searchLyrics(
    title?: string,
    artist?: string
): Promise<CandidateListDto> {
    return jsonOrThrow<CandidateListDto>(
        await fetch(`${LYRICS_URL}/search`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ title, artist }),
        })
    );
}

/** 预览某候选。不会修改当前播放。 */
export async function previewCandidate(
    source: string,
    key: string
): Promise<PreviewDto> {
    const query = new URLSearchParams({ source, key });
    return jsonOrThrow<PreviewDto>(
        await fetch(`${LYRICS_URL}/preview?${query}`)
    );
}

/** 应用来源绑定。绑定按 sid 归属，同谱面集的其它难度会共享。 */
export async function applySource(
    source: string,
    key: string
): Promise<void> {
    await jsonOrThrow(
        await fetch(`${LYRICS_URL}/source`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ source, key }),
        })
    );
}

/** 恢复自动匹配（删除来源绑定） */
export async function clearSource(): Promise<void> {
    await jsonOrThrow(await fetch(`${LYRICS_URL}/source`, { method: "DELETE" }));
}

/** 设置偏移（毫秒），返回后端最终生效值 */
export async function setOffset(offset: number): Promise<number> {
    const body = await jsonOrThrow<{ offset: number }>(
        await fetch(`${LYRICS_URL}/offset`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ offset }),
        })
    );
    return body.offset;
}

/**
 * 批量询问这些候选**有没有翻译**。
 *
 * 结果来自后端**真实取词**，不是猜测；取不到的一律按"无翻译"返回。
 */
export async function checkTranslations(
    items: { source: string; key: string }[]
): Promise<{ source: string; key: string; hasTranslation: boolean }[]> {
    if (items.length === 0) return [];
    const res = await fetch(`${LYRICS_URL}/translation-check`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ items }),
    });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as {
        items: { source: string; key: string; hasTranslation: boolean }[];
    };
    return body.items ?? [];
}

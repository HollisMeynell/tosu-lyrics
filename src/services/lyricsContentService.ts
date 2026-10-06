import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export interface LyricLineDto {
    time: number;
    origin?: string;
    translation?: string;
}

export interface SongDto {
    bid: number;
    sid: number;
    title: string;
    artist: string;
    length: number;
}

export interface SourceBinding {
    sid: number;
    source: string;
    key: string;
}

export interface CurrentLyricDto {
    song: SongDto;
    offset: number;
    current: number;
    nextTime: number;
    lyric: LyricLineDto[] | null;
    lyricState: "ok" | "blocked" | "none";
    blocked: boolean;
    source: SourceBinding | null;
}

export interface Candidate {
    source: string;
    key: string;
    title: string;
    artist: string;
    length: number;
    active: boolean;
    titleScore: number;
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
    } catch {}
    return new ApiError(code, message, res.status);
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as T;
}

export async function fetchCurrentLyric(): Promise<CurrentLyricDto> {
    return jsonOrThrow<CurrentLyricDto>(await fetch(`${LYRICS_URL}/current`));
}

export async function fetchCandidates(): Promise<CandidateListDto> {
    return jsonOrThrow<CandidateListDto>(
        await fetch(`${LYRICS_URL}/search-results`)
    );
}

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

export async function previewCandidate(
    source: string,
    key: string
): Promise<PreviewDto> {
    const query = new URLSearchParams({ source, key });
    return jsonOrThrow<PreviewDto>(
        await fetch(`${LYRICS_URL}/preview?${query}`)
    );
}

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

export async function clearSource(): Promise<void> {
    await jsonOrThrow(await fetch(`${LYRICS_URL}/source`, { method: "DELETE" }));
}

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

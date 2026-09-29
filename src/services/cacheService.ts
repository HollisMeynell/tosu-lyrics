import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export interface CacheEntry {
    bid: number;
    sid: number;
    title: string;
    audioLength: number;
    updatedAt: number;
    size: number;
    expired: boolean;
}

export interface CachePageDto {
    total: number;
    page: number;
    size: number;
    pages: number;
    items: CacheEntry[];
    ttlMs: number;
}

const CACHE_URL = `${BACKEND_API_BASE}/cache`;

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

export async function fetchCachePage(
    page: number,
    size: number,
    query?: string
): Promise<CachePageDto> {
    const params = new URLSearchParams({
        page: String(page),
        size: String(size),
    });
    if (query?.trim()) params.set("q", query.trim());
    return jsonOrThrow<CachePageDto>(await fetch(`${CACHE_URL}?${params}`));
}

export async function fetchCacheCount(): Promise<number> {
    const body = await jsonOrThrow<{ total: number }>(
        await fetch(`${CACHE_URL}/count`)
    );
    return body.total;
}

export async function deleteCacheItem(bid: number): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}/${bid}`, { method: "DELETE" })
    );
    return body.removed;
}

export async function deleteCacheByTitle(title: string): Promise<number> {
    const params = new URLSearchParams({ title });
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}?${params}`, { method: "DELETE" })
    );
    return body.removed;
}

export async function clearCache(): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(CACHE_URL, { method: "DELETE" })
    );
    return body.removed;
}

export async function cleanupCache(): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}/cleanup`, { method: "POST" })
    );
    return body.removed;
}

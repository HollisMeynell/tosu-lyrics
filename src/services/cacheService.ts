import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

/**
 * 歌词缓存管理 HTTP 客户端（B-06）。
 *
 * 缓存是**纯缓存**：这里的任何操作都不会触碰来源绑定 / 偏移 / 黑名单。
 */

export interface CacheEntry {
    bid: number;
    sid: number;
    title: string;
    /** 毫秒 */
    audioLength: number;
    updatedAt: number;
    /** 字节 */
    size: number;
    expired: boolean;
}

export interface CachePageDto {
    total: number;
    /** 从 1 开始 */
    page: number;
    size: number;
    pages: number;
    items: CacheEntry[];
    /** 当前生效 TTL（毫秒），0 表示不启用 */
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
    } catch {
        // 非 JSON 错误体时保留默认信息
    }
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

/** 删除单条，返回实际删除条数（0 表示本来就不存在） */
export async function deleteCacheItem(bid: number): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}/${bid}`, { method: "DELETE" })
    );
    return body.removed;
}

/** 按标题模糊删除，返回删除条数 */
export async function deleteCacheByTitle(title: string): Promise<number> {
    const params = new URLSearchParams({ title });
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}?${params}`, { method: "DELETE" })
    );
    return body.removed;
}

/** 清空全部，返回删除条数 */
export async function clearCache(): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(CACHE_URL, { method: "DELETE" })
    );
    return body.removed;
}

/** 清理过期条目，返回删除条数 */
export async function cleanupCache(): Promise<number> {
    const body = await jsonOrThrow<{ removed: number }>(
        await fetch(`${CACHE_URL}/cleanup`, { method: "POST" })
    );
    return body.removed;
}

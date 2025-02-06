// 功能: 解析 URL 中清除缓存参数

import Cache from "@/utils/cache.ts";

// 解析 URL 参数
export function parseUrlParams(url: string): URLSearchParams {
    const urlObj = new URL(url);
    return new URLSearchParams(urlObj.search);
}

// 清除缓存
function clearCache(): void {
    try {
        Cache.clearLyricsCache();
    } catch (e) {
        console.error("Failed to clear IndexedDB cache:", e);
    }
}

// 解析参数
export function paramParse(params: URLSearchParams): void {
    if (params.get("clear-cache")) {
        clearCache();
    }
}

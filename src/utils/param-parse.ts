// 功能: 解析 URL 参数，包括 CORS 参数和清除缓存参数

import { createSignal } from "solid-js";
import Cache from "@/utils/cache.ts";

export const [consoleEnabled, setConsoleEnabled] = createSignal(false);
export const [showController, setShowController] = createSignal(true);

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
    if (params.get("cors")) {
        window.ignoreCORS = true;
    }
    if (params.get("clear-cache")) {
        clearCache();
    }
    const controllerParam = params.get("controller");
    setShowController(controllerParam === "true");
    setConsoleEnabled(!!params.get("console"));
}

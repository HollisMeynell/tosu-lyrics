import cache from "@/utils/cache";
import {
    lyricsStore,
    setCurrentLyrics,
    setTextColor,
    setUseTranslationAsMain,
    setShowSecond,
    setAlignment,
} from "@/stores/lyricsStore";
import { lyricUL, lyricBlink } from "@/pages/LyricsBox";
import { paramParse } from "@/utils/param-parse";
import { wsService } from "@/services/WebSocketService";
import { configService } from "@/services/ConfigService";

export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    try {
        // 初始化存储适配器
        const adapter = await cache.getStorageAdapter();
        cache.storageAdapter = adapter;

        // 注册缓存处理器
        wsService.registerQueryHandler("query-cache-list", async () => {
            const allKeys = await cache.storageAdapter?.getLyricsList();
            return allKeys || [];
        });
        wsService.registerHandler("remove-cache-item", (params) => {
            const { bid } = params as { bid: number };
            cache.storageAdapter?.clearLyrics(bid);
        });
        wsService.registerHandler("remove-all-cache", () =>
            cache.storageAdapter?.clearLyrics()
        );

        // 解析 URL 参数
        const params = paramParse();
        if (params["clear-cache"]) {
            // 清除缓存
            try {
                cache.clearLyricsCache();
            } catch (e) {
                console.error("Failed to clear IndexedDB cache:", e);
            }
        }

        // 加载存储配置
        const config = await configService.fetchConfig();
        lyricsStore.parseSettings(config);

        // 注册设置处理器
        wsService.registerHandler("currentLyrics", setCurrentLyrics);
        wsService.registerHandler("textColor", setTextColor);
        wsService.registerHandler(
            "useTranslationAsMain",
            setUseTranslationAsMain
        );
        wsService.registerHandler("showSecond", setShowSecond);
        wsService.registerHandler("alignment", setAlignment);

        // 注册歌词闪烁处理器
        wsService.registerHandler("blink-lyric", () => lyricBlink(lyricUL));
    } catch (error) {
        console.error("Failed to initialize:", error);
    }
};

import cache from "@/utils/cache";
import {
    setAlignment,
    setShowSecond,
    setTextColor,
    setUseTranslationAsMain,
} from "@/stores/settingsStore";
import store from "@/stores/indexStore";
import {
    addTitleBlackListItem,
    deleteTitleBlackListItem,
} from "@/stores/blackListStore";
import { lyricBlink } from "@/pages/LyricsBox";
import { paramParse } from "@/utils/parseParams";
import { MessageHandler, wsService } from "@/services/webSocketService";
import { configService } from "@/services/configService";
import {
    changeOrigin,
    getLyricsByKey,
    getMusicQueryResult,
    getNowLyrics,
    getNowTitle,
} from "@/services/managers/tosuManager";

export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    try {
        // 初始化存储适配器
        cache.storageAdapter = await cache.getStorageAdapter();

        // 注册缓存处理器
        wsService.registerQueryHandler("query-cache-list", async () => {
            const allKeys = await cache.storageAdapter?.getLyricsList();
            return allKeys || [];
        });
        wsService.registerHandler("remove-cache-item", (params) => {
            const { key } = params as {
                key: string | number;
            };
            cache.storageAdapter?.clearLyrics(key);
        });
        wsService.registerHandler("remove-all-cache", () =>
            cache.storageAdapter?.clearLyrics()
        );
        wsService.registerHandler("change-lyric", changeOrigin);
        wsService.registerQueryHandler("get-now-title", async () =>
            getNowTitle()
        );
        wsService.registerQueryHandler("query-now-lyrics", async () =>
            getNowLyrics()
        );
        wsService.registerQueryHandler("query-now-music-info", async () =>
            getMusicQueryResult()
        );
        wsService.registerQueryHandler(
            "query-lyrics-by-key",
            async (params) => {
                const { adapter, key } = params as {
                    adapter: string;
                    key: string | number;
                };
                return getLyricsByKey(adapter, key);
            }
        );

        // 解析 URL 参数
        const params = paramParse();
        if (params["clear-cache"]) {
            // 清除缓存
            try {
                await cache.clearLyricsCache();
            } catch (e) {
                console.error("Failed to clear IndexedDB cache:", e);
            }
        }

        // 加载存储配置
        const config = await configService.fetchConfig();
        store.parseSettings(config);

        // 注册设置处理器
        wsService.registerHandler("text-color", setTextColor);
        wsService.registerHandler(
            "use-main-translation",
            setUseTranslationAsMain
        );
        wsService.registerHandler(
            "add-black-list",
            addTitleBlackListItem as MessageHandler
        );
        wsService.registerHandler(
            "delete-black-list",
            deleteTitleBlackListItem as MessageHandler
        );
        wsService.registerHandler("showSecond", setShowSecond);
        wsService.registerHandler("alignment", setAlignment);

        // 注册歌词闪烁处理器
        wsService.registerHandler("blink-lyric", lyricBlink);
    } catch (error) {
        console.error("Failed to initialize:", error);
    }
};

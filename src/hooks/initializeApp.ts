import cache from "@/utils/cache";
import {
    DEFAULT_TEXT_COLOR,
    setAlignment,
    setFont,
    setShowSecond,
    setTextColor,
    setUseTranslationAsMain,
} from "@/stores/settingsStore";
import store from "@/stores/indexStore";
import { blacklistStore } from "@/stores/blacklistStore";
import { paramParse } from "@/utils/parseParams";
import type { MessageHandler } from "@/services/webSocketService";
import { configService } from "@/services/configService";
import {
    changeOrigin,
    getLyricsByKey,
    getMusicQueryResult,
    getNowLyrics,
    getNowTitle,
} from "@/services/managers/tosuManager";
import { Websocket } from "@/api/websocket.ts";
import { BaseLyricSetter, WebsocketSetting } from "@/api/model.ts";
import { applyLyricEvent, clearLyrics } from "@/stores/lyricStore";

/**
 * 后端设置广播 -> 展示页状态
 */
const handleSettingBroadcast = (data: WebsocketSetting) => {
    switch (data.key) {
        case "setColor": {
            const value = data.value as BaseLyricSetter | undefined;
            setTextColor({
                first: value?.first ?? DEFAULT_TEXT_COLOR.first,
                second: value?.second ?? DEFAULT_TEXT_COLOR.second,
            });
            break;
        }
        case "setAlignment": {
            const value = data.value as BaseLyricSetter | undefined;
            const align = value?.first ?? value?.second;
            if (align === "left" || align === "center" || align === "right") {
                setAlignment(align);
            }
            break;
        }
        case "setFont": {
            const value = data.value as BaseLyricSetter | undefined;
            const font = value?.first ?? value?.second;
            if (font != null) {
                setFont(font);
            }
            break;
        }
        case "setTranslationMain": {
            if (typeof data.value === "boolean") {
                setUseTranslationAsMain(data.value);
            }
            break;
        }
        case "setSecondShow": {
            if (typeof data.value === "boolean") {
                setShowSecond(data.value);
            }
            break;
        }
        case "setClear": {
            clearLyrics();
            break;
        }
        default:
            break;
    }
};

/**
 * 歌词展示页: 连接新版后端 WebSocket, 歌词推送写入 lyricStore, 设置广播写入 settingsStore
 */
const initializeLyricClient = () => {
    const ws = new Websocket();
    ws.setLyricHandler(applyLyricEvent);
    ws.setSettingHandler(handleSettingBroadcast);
};

/**
 * 控制面板: 沿用旧版 peer-to-peer 逻辑, 后续阶段接入新版协议后移除
 */
const initializeLegacy = async () => {
    const [{ wsService }, { lyricBlink }] = await Promise.all([
        import("@/services/webSocketService"),
        import("@/pages/LyricsBox"),
    ]);

    try {
        // 初始化存储适配器
        cache.storageAdapter = await cache.getStorageAdapter();

        // 注册缓存处理器
        wsService.registerQueryHandler("query-cache-list", async (params) => {
            const { page = 0, size = 50 } = params as {
                page?: number;
                size?: number;
            };

            const allKeys = await cache.getLyricsCacheList(page, size);

            return allKeys ?? [];
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
            blacklistStore.add as MessageHandler
        );
        wsService.registerHandler(
            "delete-black-list",
            blacklistStore.remove as MessageHandler
        );
        wsService.registerHandler("showSecond", setShowSecond);
        wsService.registerHandler("alignment", setAlignment);

        // 注册歌词闪烁处理器
        wsService.registerHandler("blink-lyric", lyricBlink);
    } catch (error) {
        console.error("Failed to initialize:", error);
    }
};

export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    // 歌词展示页接入新版后端 WebSocket, 控制面板暂沿用旧逻辑
    if (!window.location.pathname.startsWith("/lyrics/controller")) {
        initializeLyricClient();
        return;
    }

    await initializeLegacy();
};

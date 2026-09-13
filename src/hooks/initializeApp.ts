import {
    DEFAULT_SHADOW,
    setAlignment,
    setFont,
    setFontSize,
    setSecondFont,
    setShadow,
    setShowSecond,
    setTextColor,
    setUseTranslationAsMain,
    DEFAULT_TEXT_COLOR,
} from "@/stores/settingsStore";
import { Websocket } from "@/api/websocket.ts";
import { BaseLyricSetter, WebsocketSetting } from "@/api/model.ts";
import type { Shadow } from "@/types/globalTypes";
import { applyLyricEvent, clearLyrics } from "@/stores/lyricStore";
import { lyricBlink } from "@/pages/LyricsBox";
import { initializeDarkMode } from "@/stores/settingsStore";

/**
 * 后端设置广播 → 展示页状态。
 *
 * 这是**展示端**消费 WS 事件的唯一入口：后端在管理 HTTP 操作成功后会广播
 * `setColor` / `setFont` / `setFontSize` / `setAlignment` /
 * `setTranslationMain` / `setSecondShow` / `setShadow` / `setClear`。
 *
 * 注意：这里的 key 是**后端主动推给展示端的事件**，不是管理请求。
 * 前端不通过 WS 发送任何管理命令（B-11 / F-10 之后管理一律走 HTTP）。
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
            // 主副字体各自独立；缺一个时用另一个兜底
            const first = value?.first ?? value?.second;
            const second = value?.second ?? value?.first;
            if (first != null) setFont(first);
            if (second != null) setSecondFont(second);
            break;
        }
        case "setFontSize": {
            const value = data.value as
                | { first?: number; second?: number }
                | undefined;
            const first = Number(value?.first);
            const second = Number(value?.second);
            if (Number.isFinite(first) && Number.isFinite(second)) {
                setFontSize({ first, second });
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
        case "setShadow": {
            const value = data.value as
                | { first?: Shadow; second?: Shadow }
                | undefined;
            setShadow({
                first: value?.first ?? DEFAULT_SHADOW,
                second: value?.second ?? DEFAULT_SHADOW,
            });
            break;
        }
        case "setBlink": {
            // 后端定向闪烁（B-08 / 本轮修复）。
            //
            // 旧实现通过 `wsService.registerHandler("blink-lyric", lyricBlink)`
            // 接这个事件；F-09 删掉旧链后**没有再补上**，于是"点了测试按钮但
            // 展示端毫无反应" —— 请求发出去了、后端也单播了，前端却静默忽略。
            lyricBlink();
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
 * 接入新版后端 WebSocket。
 *
 * 只做两件事：接收歌词推送、接收展示设置广播。
 * **不发送任何管理请求** —— 管理页面全部走 HTTP。
 */
const connectBackend = () => {
    const ws = new Websocket();
    ws.setLyricHandler(applyLyricEvent);
    ws.setSettingHandler(handleSettingBroadcast);
};

/**
 * 应用初始化。
 *
 * 改造后不再区分"展示页 / 控制台"两套完全不同的初始化：
 * - 展示页（LyricsBox）：接入后端 WS 接收推送
 * - 控制台（Controller）：不接 WS，管理数据全部来自 HTTP
 *
 * 旧的 `initializeLegacy`（浏览器直连 tosu、IndexedDB 歌词缓存、
 * 旧 `/api/config`、旧 WS 管理处理器注册）已随 F-09 / F-10 整体删除。
 */
export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    // 夜间模式两端都需要
    initializeDarkMode();

    // 控制台不消费展示推送，因此不建立 WS 连接
    if (window.location.pathname.startsWith("/lyrics/controller")) {
        return;
    }

    connectBackend();
};

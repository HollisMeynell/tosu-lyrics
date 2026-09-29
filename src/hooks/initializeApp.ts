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

/** 后端设置广播 → 展示页状态（展示端消费 WS 事件的唯一入口）。 */
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

const connectBackend = () => {
    const ws = new Websocket();
    ws.setLyricHandler(applyLyricEvent);
    ws.setSettingHandler(handleSettingBroadcast);
};

/** 展示页接入 WS；控制台不接 WS，管理数据全部来自 HTTP。 */
export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    initializeDarkMode();

    // 控制台不消费展示推送，不建立 WS 连接
    if (window.location.pathname.startsWith("/lyrics/controller")) {
        return;
    }

    connectBackend();
};

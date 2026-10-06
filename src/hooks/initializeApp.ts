import {
    DEFAULT_SHADOW,
    setAlignment,
    setFont,
    setFontSize,
    setSecondFont,
    setShadow,
    setShowSecond,
    setLyricLines,
    setTextColor,
    setUseTranslationAsMain,
    DEFAULT_TEXT_COLOR,
} from "@/stores/settingsStore";
import { Websocket } from "@/api/websocket.ts";
import { BaseLyricSetter, WebsocketSetting } from "@/api/model.ts";
import type { Shadow } from "@/types/globalTypes";
import { applyLyricEvent, clearLyrics, setLyricLoading } from "@/stores/lyricStore";
import { lyricBlink } from "@/pages/LyricsBox";
import { initializeDarkMode } from "@/stores/settingsStore";
import { normalizeLyricLines } from "@/utils/lyricLines";

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
        case "setLyricLines": {
            // 与 HTTP 快照同一条归一化路径：缺失 / 偶数 / 越界都收敛成合法奇数
            setLyricLines(normalizeLyricLines(data.value));
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
        case "setLyricLoading": {
            // 后端在换歌/搜索期间置 true, 加载结束(含"没有歌词")置 false
            if (typeof data.value === "boolean") {
                setLyricLoading(data.value);
            }
            break;
        }
        default:
            break;
    }
};

const connectBackend = (identity?: string) => {
    const ws = new Websocket(identity);
    ws.setLyricHandler(applyLyricEvent);
    ws.setSettingHandler(handleSettingBroadcast);
};

/**
 * 接入展示端 WS。
 *
 * 控制台的管理数据仍然全部来自 HTTP，但它顶部的歌词预览与加载动画必须与
 * `/lyrics` 完全一致，所以同样订阅展示端推送 —— 两边共用同一份 `lyricStore`
 * 与 `lyricLoading`，动画自然同步，不存在第二套实现。
 */
export const initializeApp = async () => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    initializeDarkMode();

    // 控制台顶部要显示与 /lyrics 完全一致的歌词与加载动画（同一份 lyricStore /
    // lyricLoading），所以也接入展示端推送；用 id 自报身份，便于在"在线展示端"
    // 列表里区分出这是控制台而不是一个展示端。
    const isController =
        window.location.pathname.startsWith("/lyrics/controller");
    connectBackend(isController ? "controller" : undefined);
};

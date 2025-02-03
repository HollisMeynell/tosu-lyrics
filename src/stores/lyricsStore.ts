import { createSignal } from "solid-js";
import { LyricLine } from "@/types/config-global";
import { wsService } from "@/services/WebSocketService";
import { configService } from "@/services/ConfigService";
import { AlignType, Settings } from "@/types/config-global";

const [currentLyrics, setCurrentLyrics] = createSignal<LyricLine[] | undefined>(
    undefined,
);

const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};
const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
const [useTranslationAsMain, setUseTranslationAsMain] = createSignal(false);
const [showSecond, setShowSecond] = createSignal(true);
const [alignment, setAlignment] = createSignal<AlignType>("center");

// Initialize WebSocket handlers
(() => {
    wsService.registerHandler("currentLyrics", setCurrentLyrics);
    wsService.registerHandler("textColor", setTextColor);
    wsService.registerHandler("useTranslationAsMain", setUseTranslationAsMain);
    wsService.registerHandler("showSecond", setShowSecond);
    wsService.registerHandler("alignment", setAlignment);
})();

// Store
export const lyricsStore = {
    get getState() {
        return {
            currentLyrics: currentLyrics(),
            textColor: textColor(),
            useTranslationAsMain: useTranslationAsMain(),
            showSecond: showSecond(),
            alignment: alignment(),
        };
    },

    parseSettings: (config: Settings) => {
        const setValue = (val: unknown | undefined, setter: (val: unknown) => void)=> {
            if (val != null) {
                setter(val);
            }
        }
        if (config) {
            // 兼容不同版本导致的部分配置缺失
            setValue(config.currentLyrics, setCurrentLyrics);
            setValue(config.useTranslationAsMain, setUseTranslationAsMain);
            setValue(config.showSecond, setShowSecond);
            setValue(config.alignment, setAlignment);

            if (config.textColor) {
                setTextColor({
                    ...DEFAULT_TEXT_COLOR,
                    ...config.textColor,
                });
            }
        }
    },

    updateCurrentLyrics(lyrics: LyricLine[] | undefined) {
        setCurrentLyrics(lyrics);
        wsService.pushSetting("currentLyrics", lyrics);
    },

    /**
     * 分段处理, 当设置颜色时实时显示在歌词上
     */
    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        // 应该当 color 被应用时再触发保存

        /*
        这里我认为没有必要在拖动颜色选择器时就应用到歌词上
        首先控制台可能是在手机或者另一个屏幕的浏览器上
        控制台界面不会显示位于顶部的动态歌词, 以避免重复的查询导致 api 超限
        另外也没有必要连 tosu 来同步时间轴
        所以仅使用 onChange 足够了, 也就不需要拆分 setTextColor 与 sendColorConfig
        */
        configService.saveConfig(this.getState);
    },

    /**
     * 分段处理, 当颜色设置被确定时发送广播
     */
    sendColorConfig() {
        const nowColor = textColor();
        wsService.pushSetting("textColor", nowColor);
    },

    setShowSecond(show: boolean) {
        setShowSecond(show);
        wsService.pushSetting("showSecond", show);
        configService.saveConfig(this.getState);
    },

    setUseTranslationAsMain(use: boolean) {
        setUseTranslationAsMain(use);
        wsService.pushSetting("useTranslationAsMain", use);
        configService.saveConfig(this.getState);
    },

    setAlignment(align: AlignType) {
        setAlignment(align);
        wsService.pushSetting("alignment", align);
        configService.saveConfig(this.getState);
    },
};

export default lyricsStore;

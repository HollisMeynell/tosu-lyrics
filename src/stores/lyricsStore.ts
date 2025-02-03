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

    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        // 应该当 color 被应用时再触发保存
        wsService.pushSetting("textColor", newTextColor);
        configService.saveConfig(this.getState);
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

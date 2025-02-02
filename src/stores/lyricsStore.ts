import { createSignal } from "solid-js";
import { LyricLine } from "@/types/config-global";
import { wsService } from "@/services/WebSocketService";
import { configService } from "@/services/ConfigService";
import { AlignType, Settings } from "@/types/config-global";

const [currentLyrics, setCurrentLyrics] = createSignal<LyricLine[] | undefined>(
    undefined
);
const [textColor, setTextColor] = createSignal({
    first: "#ffffff",
    second: "#a0a0a0",
});
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
        if (config) {
            setCurrentLyrics(config.currentLyrics);
            setTextColor(config.textColor);
            setUseTranslationAsMain(config.useTranslationAsMain);
            setShowSecond(config.showSecond);
            setAlignment(config.alignment);
        }
    },

    updateCurrentLyrics(lyrics: LyricLine[] | undefined) {
        setCurrentLyrics(lyrics);
        wsService.pushSetting("currentLyrics", lyrics);
    },

    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        wsService.pushSetting("textColor", newTextColor);
        configService.saveConfig(this.getState); // 触发防抖保存
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

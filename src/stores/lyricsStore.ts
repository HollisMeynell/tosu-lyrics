import { createSignal } from "solid-js";
import { LyricLine } from "@/common/music-api.ts";
import { AlignType, Settings } from "@/common/config-global.ts";
import { pushSettingMessage, registerSetterHandle } from "@/common/config-async.ts";


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

const init = (
    () => {
        registerSetterHandle({key: "currentLyrics", handle: setCurrentLyrics})
        registerSetterHandle({key: "textColor", handle: setTextColor})
        registerSetterHandle({key: "useTranslationAsMain", handle: setUseTranslationAsMain})
        registerSetterHandle({key: "showSecond", handle: setShowSecond})
        registerSetterHandle({key: "alignment", handle: setAlignment})
        return true;
    }
)()

console.log(`init: ${init}`);

// Store
export const lyricsStore = {
    get state() {
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

    updateCurrentLyrics: (lyrics: LyricLine[] | undefined) => {
        setCurrentLyrics(lyrics);
        pushSettingMessage("currentLyrics", lyrics);
    },

    setTextColor: (order: "first" | "second", color: string) => {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        pushSettingMessage("textColor", newTextColor);
    },

    setShowSecond: (show: boolean) => {
        setShowSecond(show);
        pushSettingMessage("showSecond", show);
    },

    setUseTranslationAsMain: (use: boolean) => {
        setUseTranslationAsMain(use);
        pushSettingMessage("useTranslationAsMain", use);
    },

    setAlignment: (align: AlignType) => {
        setAlignment(align);
        pushSettingMessage("alignment", align);
    },
};

export default lyricsStore;

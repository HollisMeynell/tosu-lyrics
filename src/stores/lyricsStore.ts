import { createSignal } from "solid-js";
import { LyricLine } from "@/types/config-global";
import { wsService } from "@/services/WebSocketService";
import { configService } from "@/services/ConfigService";
import { AlignType, Settings } from "@/types/config-global";

const [currentLyrics, setCurrentLyrics] = createSignal<LyricLine[] | undefined>(
    undefined
);

const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};
const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
const [useTranslationAsMain, setUseTranslationAsMain] = createSignal(false);
const [showSecond, setShowSecond] = createSignal(true);
const [alignment, setAlignment] = createSignal<AlignType>("center");
export const [darkMode, setDarkMode] = createSignal(
    localStorage.getItem("darkMode") === "true"
);

// Initialize WebSocket handlers
(() => {
    wsService.registerHandler("currentLyrics", setCurrentLyrics);
    wsService.registerHandler("textColor", setTextColor);
    wsService.registerHandler("useTranslationAsMain", setUseTranslationAsMain);
    wsService.registerHandler("showSecond", setShowSecond);
    wsService.registerHandler("alignment", setAlignment);
})();

// 防抖函数
function debounce<T extends (...args: unknown[]) => void>(
    func: T,
    wait: number
): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout> | null = null;

    return function (this: unknown, ...args: Parameters<T>): void {
        if (timeout !== null) {
            clearTimeout(timeout);
        }
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
}

// Store
export const lyricsStore = {
    get getState() {
        return {
            currentLyrics: currentLyrics(),
            textColor: textColor(),
            useTranslationAsMain: useTranslationAsMain(),
            showSecond: showSecond(),
            alignment: alignment(),
            darkMode: darkMode(),
        };
    },

    parseSettings: (config: Settings) => {
        const setValue = (
            val: unknown | undefined,
            setter: (val: unknown) => void
        ) => {
            if (val != null) {
                setter(val);
            }
        };
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
        lyricsStore.initializeDarkMode();
    },

    updateCurrentLyrics(lyrics: LyricLine[] | undefined) {
        setCurrentLyrics(lyrics);
        wsService.pushSetting("currentLyrics", lyrics);
    },

    saveConfigDebounced: debounce(() => {
        wsService.pushSetting("textColor", textColor());
        configService.saveConfig(lyricsStore.getState);
    }, 500),

    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        this.saveConfigDebounced();
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

    initializeDarkMode() {
        const darkModeMemo = localStorage.getItem("darkMode") === "true";
        console.log("darkModeMemo", darkModeMemo);
        setDarkMode(darkModeMemo);
        if (darkModeMemo) {
            document.documentElement.classList.add("dark");
        } else {
            document.documentElement.classList.remove("dark");
        }
    },

    toggleDarkMode() {
        if (!document.startViewTransition) {
            lyricsStore.executeDarkModeToggle();
            return;
        }
        document.startViewTransition(() => lyricsStore.executeDarkModeToggle());
    },

    executeDarkModeToggle() {
        setDarkMode(!darkMode());
        document.documentElement.classList.toggle("dark");

        if (document.documentElement.classList.contains("dark")) {
            localStorage.setItem("darkMode", "true");
        } else {
            localStorage.removeItem("darkMode");
        }
    },
};

export default lyricsStore;

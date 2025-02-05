import { createSignal } from "solid-js";
import { LyricLine, Shadow } from "@/types/config-global";
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

const DEFAULT_SHADOW :Shadow= {
    enable: false,
    inset: false,
    color: "#000000",
    type: undefined,
}

const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
// todo: 阴影控制
const [shadow, setShadow] = createSignal<Shadow>(DEFAULT_SHADOW);
const [useTranslationAsMain, setUseTranslationAsMain] = createSignal(false);
const [showSecond, setShowSecond] = createSignal(true);
const [alignment, setAlignment] = createSignal<AlignType>("center");

// 这两个只在当前端用, 不要同步
export const [darkMode, setDarkMode] = createSignal(
    localStorage.getItem("darkMode") === "true"
);
const [showController, setShowController] = createSignal(
    localStorage.getItem("showController") === "true"
);

// 监听 ctrl + alt + t 点击次数 这个有什么用?
window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.altKey && e.key === "t") {
        setShowController(!showController());
        localStorage.setItem(
            "showController",
            showController() ? "true" : "false"
        );
    }
});

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
    get getState():Settings {
        // showController 跟 darkMode 不同步
        return {
            shadow: shadow(),
            currentLyrics: currentLyrics(),
            textColor: textColor(),
            useTranslationAsMain: useTranslationAsMain(),
            showSecond: showSecond(),
            alignment: alignment(),
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
        lyricsStore.initializeState();
    },

    updateCurrentLyrics(lyrics: LyricLine[] | undefined) {
        setCurrentLyrics(lyrics);
        // wsService.pushSetting("currentLyrics", lyrics);
    },
    setShadow(shadow: Shadow) {
        setShadow(shadow);
    },

    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
    },

    sendColorConfig() {
        const nowColor = textColor();
        wsService.pushSetting("textColor", nowColor);
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

    initializeState() {
        let darkModeMemo = localStorage.getItem("darkMode");
        if (darkModeMemo === null) {
            localStorage.setItem("darkMode", "true");
            document.documentElement.classList.add("dark");
            darkModeMemo = "true";
        } else if (darkModeMemo === "true") {
            document.documentElement.classList.add("dark");
        } else if (darkModeMemo === "false") {
            document.documentElement.classList.remove("dark");
        }
        setDarkMode(darkModeMemo === "true");

        let showControllerMemo = localStorage.getItem("showController");
        if (showControllerMemo === null) {
            localStorage.setItem("showController", "false");
            setShowController(false);
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
        const mode = !darkMode();
        setDarkMode(mode);
        document.documentElement.classList.toggle("dark");

        if (document.documentElement.classList.contains("dark")) {
            localStorage.setItem("darkMode", "" + mode);
        } else {
            localStorage.setItem("darkMode", "" + mode);
        }
    },
};

export default lyricsStore;

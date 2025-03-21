import {
    DEFAULT_TEXT_COLOR,
    font,
    shadow,
    textColor,
    useTranslationAsMain,
    showSecond,
    darkMode,
    alignment,
    setDarkMode,
    setFont,
    setAlignment,
    setShadow,
    setShowSecond,
    setTextColor,
    setUseTranslationAsMain,
} from "@/stores/settingsStore";
import { blacklistStore } from "./blacklistStore";
import { BlacklistItem, Shadow } from "@/types/globalTypes";
import { wsService } from "@/services/webSocketService";
import { configService } from "@/services/configService";
import { AlignType, Config } from "@/types/globalTypes";

// Store
const lyricsStore = {
    get getState(): Config {
        return {
            settings: {
                font: font(),
                shadow: shadow(),
                textColor: textColor(),
                useTranslationAsMain: useTranslationAsMain(),
                showSecond: showSecond(),
                alignment: alignment(),
            },
            titleBlackList: blacklistStore.items,
        };
    },

    parseSettings: (config: Config) => {
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
            setValue(config.settings.font, setFont);
            setValue(
                config.settings.useTranslationAsMain,
                setUseTranslationAsMain
            );
            setValue(config.settings.showSecond, setShowSecond);
            setValue(config.settings.alignment, setAlignment);
            setValue(config.titleBlackList, (list) => {
                console.log(list);
                blacklistStore.restore(list as BlacklistItem[]);
            });

            if (config.settings.textColor) {
                setTextColor({
                    ...DEFAULT_TEXT_COLOR,
                    ...config.settings.textColor,
                });
            }
        }
        lyricsStore.initializeDarkMode();
    },

    setShadow(shadow: Shadow) {
        setShadow(shadow);
    },

    setFont(font: string) {
        setFont(font);
        wsService.pushSetting("font", font);
        void configService.saveConfig(this.getState);
    },

    setTextColor(order: "first" | "second", color: string) {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
    },

    sendColorConfig() {
        const nowColor = textColor();
        wsService.pushSetting("text-color", nowColor);
        void configService.saveConfig(this.getState);
    },

    setShowSecond(show: boolean) {
        setShowSecond(show);
        wsService.pushSetting("showSecond", show);
        void configService.saveConfig(this.getState);
    },

    setUseTranslationAsMain(use: boolean) {
        setUseTranslationAsMain(use);
        wsService.pushSetting("use-main-translation", use);
        void configService.saveConfig(this.getState);
    },

    addTitleToBlackList(title: string) {
        wsService.pushSetting("add-black-list", title);
        blacklistStore.add({
            id: title,
            name: title,
            reason: "Added by user",
        });
    },

    deleteTitleBlackList(title: string) {
        wsService.pushSetting("delete-black-list", title);
        blacklistStore.remove(title);
    },

    syncTitleBlackList() {
        void configService.saveConfig(this.getState);
    },

    setAlignment(align: string) {
        setAlignment(align as AlignType);
        wsService.pushSetting("alignment", align);
        void configService.saveConfig(this.getState);
    },

    initializeDarkMode() {
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
        localStorage.setItem("darkMode", String(mode));
    },
};

export default lyricsStore;

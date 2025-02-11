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

import {
    addTitleBlackListItem,
    deleteTitleBlackListItem,
    resetTitleBlackList,
    titleBlackList,
} from "@/stores/blackListStore";

import { Shadow } from "@/types/globalTypes";
import { wsService } from "@/services/webSocketService";
import { configService } from "@/services/configService";
import { AlignType, Settings } from "@/types/globalTypes";

// Store
const lyricsStore = {
    get getState(): Settings {
        return {
            font: font(),
            shadow: shadow(),
            textColor: textColor(),
            useTranslationAsMain: useTranslationAsMain(),
            showSecond: showSecond(),
            alignment: alignment(),
            titleBlackList: titleBlackList.list,
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
            setValue(config.font, setFont);
            setValue(config.useTranslationAsMain, setUseTranslationAsMain);
            setValue(config.showSecond, setShowSecond);
            setValue(config.alignment, setAlignment);
            setValue(config.titleBlackList, (list) =>
                resetTitleBlackList(list as string[])
            );

            if (config.textColor) {
                setTextColor({
                    ...DEFAULT_TEXT_COLOR,
                    ...config.textColor,
                });
            }
        }
        lyricsStore.initializeState();
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

    addTitleBlackList(title: string) {
        wsService.pushSetting("add-black-list", title);
        addTitleBlackListItem(title);
    },

    deleteTitleBlackList(title: string) {
        wsService.pushSetting("delete-black-list", title);
        deleteTitleBlackListItem(title);
    },

    asyncTitleBlackList() {
        void configService.saveConfig(this.getState);
    },

    setAlignment(align: string) {
        setAlignment(align as AlignType);
        wsService.pushSetting("alignment", align);
        void configService.saveConfig(this.getState);
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

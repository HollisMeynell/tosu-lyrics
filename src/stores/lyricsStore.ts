import { createSignal } from "solid-js";
import { Shadow } from "@/types/globalTypes";
import { wsService } from "@/services/webSocketService";
import { configService } from "@/services/configService";
import { AlignType, Settings, alignmentOptions } from "@/types/globalTypes";
import { createStore, produce } from "solid-js/store";

const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};

const DEFAULT_SHADOW: Shadow = {
    enable: false,
    inset: false,
    color: "#000000",
    type: undefined,
};

// 同步信息
export const [font, setFont] = createSignal("");
export const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
export const [shadow, setShadow] = createSignal<Shadow>(DEFAULT_SHADOW);
export const [useTranslationAsMain, setUseTranslationAsMain] =
    createSignal(true);
export const [showSecond, setShowSecond] = createSignal(true);
export const [alignment, setAlignment] = createSignal<AlignType>(
    alignmentOptions[1].value
);
const [titleBlackList, setTitleBlackList] = createStore({
    list: [] as string[],
    set: new Set<string>(),
});

export const inTitleBlackList = (title: string) =>
    titleBlackList.set.has(title);

export const getTitleBlackList = () => titleBlackList.list;

export const resetTitleBlackList = (data?: string[]) => {
    if (data) {
        setTitleBlackList({
            list: data,
            set: new Set(data),
        });
    } else {
        setTitleBlackList({
            list: [],
            set: new Set(),
        });
    }
};

export const addTitleBlackListItem = (title: string) => {
    // if (titleBlackList.set.has(title)) return;
    if (titleBlackList.list.indexOf(title) >= 0) return;
    setTitleBlackList(
        "set",
        produce((set) => {
            set.add(title);
        })
    );
    setTitleBlackList(
        "list",
        produce((list) => {
            list.unshift(title);
        })
    );
};

export const deleteTitleBlackListItem = (title: string) => {
    // if (!titleBlackList.set.has(title)) return;
    if (titleBlackList.list.indexOf(title) < 0) return;
    setTitleBlackList(
        "set",
        produce((set) => {
            set.delete(title);
        })
    );
    setTitleBlackList(
        "list",
        produce((list) => {
            const index = list.indexOf(title);
            list.splice(index, 1);
        })
    );
};

// 非同步信息
export const [darkMode, setDarkMode] = createSignal(
    localStorage.getItem("darkMode") === "true"
);
export const [showController, setShowController] = createSignal(
    localStorage.getItem("showController") === "true"
);

function toggleController() {
    const show = !showController();
    setShowController(show);
    localStorage.setItem("showController", String(show));
}

// 触发 Controller
window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.altKey && e.key === "t") {
        toggleController();
    }
});

// 触发 Controller (Mobile)
window.addEventListener("touchstart", (e) => {
    if (e.touches.length === 3) {
        toggleController();
        e.preventDefault();
    }
});

// Store
export const lyricsStore = {
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
        wsService.pushSetting("textColor", nowColor);
        void configService.saveConfig(this.getState);
    },

    setShowSecond(show: boolean) {
        setShowSecond(show);
        wsService.pushSetting("showSecond", show);
        void configService.saveConfig(this.getState);
    },

    setUseTranslationAsMain(use: boolean) {
        setUseTranslationAsMain(use);
        wsService.pushSetting("useTranslationAsMain", use);
        void configService.saveConfig(this.getState);
    },

    addTitleBlackList(title: string) {
        wsService.pushSetting("addBlackList", title);
        addTitleBlackListItem(title);
    },

    deleteTitleBlackList(title: string) {
        wsService.pushSetting("deleteBlackList", title);
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
        localStorage.setItem("darkMode", String(mode));
    },
};

export default lyricsStore;

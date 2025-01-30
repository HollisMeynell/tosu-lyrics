import { createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import { LyricLine } from "@/common/music-api.ts";

// 定义对齐方式类型
type AlignType = "left" | "center" | "right";

// Store 状态接口
interface LyricsState {
    textColor: {
        first: string;
        second: string;
    };
    alignment: AlignType;
    currentLyrics: LyricLine[] | undefined;
}

const getInitialTextColor = () => {
    try {
        const storedValue = localStorage.getItem("textColor");
        if (storedValue) {
            const parsed = JSON.parse(storedValue);
            return {
                first: parsed.first,
                second: parsed.second,
            };
        }
    } catch (error) {
        console.error("Error parsing stored text color:", error);
    }
    return {
        first: "#ffffff",
        second: "#a0a0a0",
    };
};

const [currentLyrics, setCurrentLyrics] = createSignal<LyricLine[] | undefined>(
    undefined
);
const [textColorSignal, setTextColorSignal] = createSignal(getInitialTextColor());

// 创建初始状态
const [state, setState] = createStore<LyricsState>({
    textColor: textColorSignal(),
    alignment: "center",
    currentLyrics: undefined,
});

// 监听 localStorage 变化
window.addEventListener("storage", (e) => {
    if (e.key === "textColor" && e.newValue) {
        try {
            const newColor = JSON.parse(e.newValue);
            setTextColorSignal(newColor);
            setState("textColor", newColor);
        } catch (error) {
            console.error("Error handling storage event:", error);
        }
    }
});

// Store 操作方法
export const lyricsStore = {
    // 获取状态
    getState: () => state,

    // 获取当前歌词
    currentLyrics,

    // 设置文字颜色
    setTextColor: (order: "first" | "second", color: string) => {
        const newTextColor = {
            ...state.textColor,
            [order]: color,
        };
        try {
            localStorage.setItem("textColor", JSON.stringify(newTextColor)); // 更新 localStorage
            setTextColorSignal(newTextColor); // 更新信号
            setState("textColor", newTextColor); // 更新 store 状态
            // 手动触发 storage 事件以确保跨窗口同步
            window.dispatchEvent(
                new StorageEvent("storage", {
                    key: "textColor",
                    newValue: JSON.stringify(newTextColor),
                    storageArea: localStorage,
                })
            );
        } catch (error) {
            console.error("Error updating text color:", error);
        }
    },

    // 设置对齐方式
    setAlignment: (alignment: AlignType) => {
        setState("alignment", alignment);
    },

    // 更新当前歌词
    updateCurrentLyrics: (lyrics: LyricLine[] | undefined) => {
        setCurrentLyrics(lyrics);
        setState("currentLyrics", lyrics);
    },
};

export const useTextColor = () => textColorSignal;

export default lyricsStore;

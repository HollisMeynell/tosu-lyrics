import { createSignal, onMount } from "solid-js";
import { LyricLine } from "@/common/music-api.ts";

type AlignType = "left" | "center" | "right";

type TextColorValue = { first: string; second: string };
type StateUpdate =
    | { keyToChange: "textColor"; value: TextColorValue }
    | { keyToChange: "useTranslationAsMain"; value: boolean }
    | { keyToChange: "showSecond"; value: boolean }
    | { keyToChange: "currentLyrics"; value: LyricLine[] | undefined }
    | { keyToChange: "alignment"; value: AlignType };

interface Config {
    textColor: TextColorValue;
    useTranslationAsMain: boolean;
    showSecond: boolean;
    currentLyrics: LyricLine[] | undefined;
    alignment: AlignType;
}

interface ConfigMessage {
    command: string;
    echo: string | null;
}

const connect_ws = new WebSocket("ws://localhost:41280/api/ws");

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

// WebSocket event handlers
connect_ws.onmessage = (event) => {
    const data: StateUpdate = event.data;
    console.log("Received state update:", data);

    switch (data.keyToChange) {
        case "textColor":
            setTextColor(data.value);
            break;
        case "useTranslationAsMain":
            setUseTranslationAsMain(data.value);
            break;
        case "showSecond":
            setShowSecond(data.value);
            break;
        case "currentLyrics":
            setCurrentLyrics(data.value);
            break;
        case "alignment":
            setAlignment(data.value);
            break;
    }
};

connect_ws.onerror = (error) => {
    console.error("WebSocket error:", error);
};

connect_ws.onclose = () => {
    console.log("WebSocket connection closed");
};

async function saveConfig(config: Config) {
    try {
        const response = await fetch('/api/config', {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(config), // 将配置对象转换为 JSON 字符串
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        console.log('Config saved successfully:', result);
    } catch (error) {
        console.error('Failed to save config:', error);
    }
}

// 加载配置
async function loadConfig() {
    try {
        const response = await fetch('/api/config', {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
            },
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const config = await response.json();
        console.log('Config loaded successfully:', config);
        return config;
    } catch (error) {
        console.error('Failed to load config:', error);
        return null;
    }
}

// Update state and send over WebSocket
const updateState = (newState: StateUpdate) => {
    if (connect_ws.readyState === WebSocket.OPEN) {
        const configMessage: ConfigMessage = {
            command: JSON.stringify(newState),
            echo: null,
        };
        connect_ws.send(JSON.stringify(configMessage));
        saveConfig(lyricsStore.state);
    } else {
        console.error("WebSocket is not open.");
    }
};

// Load config
const loadConfigAndSetState = async () => {
    const config = await loadConfig();
    if (config) {
        setCurrentLyrics(config.currentLyrics);
        setTextColor(config.textColor);
        setUseTranslationAsMain(config.useTranslationAsMain);
        setShowSecond(config.showSecond);
        setAlignment(config.alignment);
    }
};

onMount(loadConfigAndSetState);

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

    updateCurrentLyrics: (lyrics: LyricLine[] | undefined) => {
        setCurrentLyrics(lyrics);
        updateState({ keyToChange: "currentLyrics", value: lyrics });
    },

    setTextColor: (order: "first" | "second", color: string) => {
        const newTextColor = { ...textColor(), [order]: color };
        setTextColor(newTextColor);
        updateState({ keyToChange: "textColor", value: newTextColor });
    },

    setShowSecond: (show: boolean) => {
        setShowSecond(show);
        updateState({ keyToChange: "showSecond", value: show });
    },

    setUseTranslationAsMain: (use: boolean) => {
        setUseTranslationAsMain(use);
        updateState({ keyToChange: "useTranslationAsMain", value: use });
    },

    setAlignment: (align: AlignType) => {
        setAlignment(align);
        updateState({ keyToChange: "alignment", value: align });
    },
};

export default lyricsStore;

import type { Shadow } from "@/types/globalTypes";
interface WebsocketMessage {
    type: string;
}

function isWebsocketMessage(obj: unknown): obj is WebsocketMessage {
    return (
        obj !== null &&
        typeof obj === "object" &&
        "type" in obj &&
        typeof obj.type === "string"
    );
}

function isLyric(obj: WebsocketMessage): obj is WebsocketLyric {
    return obj.type === "lyric";
}

function isSetting(obj: WebsocketMessage): obj is WebsocketSetting {
    return obj.type === "setting";
}

type WebsocketLyric = WebsocketMessage & {
    type: "lyric";
    lyric?: LyricLine[];
    current: number;
    nextTime: number;
    sequence: "up" | "down";
};

type WebsocketSetting<
    K extends keyof WebsocketSettingTypeMap = keyof WebsocketSettingTypeMap,
> = WebsocketMessage & {
    type: "setting";
    key: K;
    value?: WebsocketSettingTypeMap[K];
    echo?: string;
};

interface WebsocketSettingTypeMap {
    setClear: null;
    setFont: BaseLyricSetter;
    setFontSize: BaseLyricSetter;
    setAlignment: BaseLyricSetter;
    setColor: BaseLyricSetter;
    setTranslationMain: boolean;
    setSecondShow: boolean;
    setBlink: null;
    setShadow: { first?: Shadow; second?: Shadow };
}

interface LyricLine {
    origin?: string;
    translation?: string;
}

interface BaseLyricSetter {
    first?: string;
    second?: string;
}


interface SongInfo {
    title: string;
    artist: string;
    length: string;
    key: string;
}


export { isWebsocketMessage, isLyric, isSetting };

export type {
    WebsocketMessage,
    WebsocketLyric,
    WebsocketSetting,
    WebsocketSettingTypeMap,
    BaseLyricSetter,
    SongInfo,
    LyricLine,
};

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
    /**
     * 管理请求的关联键。前端已不再通过 WS 发管理请求（管理走 HTTP），
     * 因此这个字段只作为**兼容读取**保留：后端不会再产生它。
     */
    echo?: string;
};

/**
 * 展示端会收到的设置事件（F-10）。
 *
 * 改造前这张表同时描述了**管理请求**（`getFont` / `setBlock` / `setCacheClean`
 * / `getCacheCount` …）和它们的返回值。管理走 HTTP 之后，这里只保留
 * **后端主动推给展示端**的事件 —— 前端不再发送任何 setting 消息。
 */
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

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
    error?: string;
    echo?: string;
};

interface WebsocketSettingTypeMap {
    setClear: null;
    setFont: BaseLyricSetter;
    getFont: BaseLyricSetter;
    setFontSize: BaseLyricSetter;
    getFontSize: BaseLyricSetter;
    setAlignment: BaseLyricSetter;
    getAlignment: BaseLyricSetter;
    setColor: BaseLyricSetter;
    getColor: BaseLyricSetter;
    setTranslationMain: boolean;
    getTranslationMain: boolean;
    setSecondShow: boolean;
    getSecondShow: boolean;
    setLyricSource: SongInfoKey;
    getLyricList: SongInfoList;
    getAllLyric: LyricLine[];
    setBlock: null;
    setUnblock: null;
    getBlockList: BlockItem[];
    getCacheCount: number;
    setCacheClean: null;
    getLyricOffset: number;
    setLyricOffset: number;
}

interface BaseLyricSetter {
    first?: string;
    second?: string;
}

interface SongInfoKey {
    type: "QQ" | "Netease";
    key: string;
}

interface SongInfoList {
    QQ: SongInfo[];
    Netease: SongInfo[];
}

interface SongInfo {
    title: string;
    artist: string;
    length: string;
    key: string;
}

type LyricLine = BaseLyricSetter;

interface BlockItem {
    bid?: number;
    sid?: number;
    title?: string;
}

export { isWebsocketMessage, isLyric, isSetting };

export type {
    WebsocketMessage,
    WebsocketLyric,
    WebsocketSetting,
    WebsocketSettingTypeMap,
    BaseLyricSetter,
    SongInfoKey,
    SongInfoList,
    SongInfo,
    LyricLine,
    BlockItem,
};

export type LyricRawLine = {
    time: number;
    first: string;
    second?: string;
};

export type LyricLine = {
    main: string;
    origin?: string;
};

export type MusicInfo = {
    title: string;
    artist: string;
    length: number;
    key: number | string;
};

export type UnifiedLyricResult = {
    lyric: string; // 原版歌词
    trans?: string; // 翻译歌词
};

export interface MusicQueryInfoData {
    [key: string]: MusicInfo[];
}

export interface MusicQueryInfo {
    title: string;
    data: MusicQueryInfoData;
}

import {getLyricUrl, getMusicInfoUrl, parseReg, splitReg} from "./constant.ts";
import { doRequest } from "../do-request.ts";
import {
    AdaptorStatus,
    Lyric,
    LyricAdaptor,
    MusicInfo,
} from "../music-api.ts";
import { MAX_TIME } from "../constant.ts";

type ArtistDetail = {
    id: number;
    name: string;
};

type SongDetail = {
    id: number;
    name: string;
    artists: ArtistDetail[];
    duration: number;
};

type SongSearchResult = {
    code: number;
    result: {
        songs: SongDetail[];
        songCount: number;
    };
};

type LyricResult = {
    // 原版
    lrc: LyricItem;
    // ?
    klyric: LyricItem;
    // 翻译
    tlyric: LyricItem;
    code: number;
};

type LyricItem = {
    version: number;
    lyric: string;
};

function checkText(text: string) {
    // 剔除类似 "翻译: xxx" 等无关信息
    return !text.match(/^..\s?:/);
}

function parse(match: RegExpMatchArray) {
    const minutes = parseInt(match[1]);
    const seconds = parseFloat(match[2]);
    const lyric = match[3];
    const time = minutes * 60 + seconds;
    const text = lyric.trim();
    return checkText(text) ? { time, text } : { time: NaN, text: "" };
}

async function searchMusic(title: string): Promise<MusicInfo[]> {
    try {
        const url = getMusicInfoUrl(title);
        const response = await doRequest({ url });
        const data:SongSearchResult =  JSON.parse(response.body);
        if (data.code !== 200 || !data.result?.songCount) {
            return [];
        }
        return data.result?.songs.map((x) => {
            return {
                title: x.name,
                artist: x.artists.map((x) => x.name).join(", ") || "Unknown",
                length: Math.round(x.duration / 1000),
                key: x.id,
            };
        }) || [];
    } catch (error) {
        console.error("[网易云]Failed to search music:", error);
        throw new Error("[网易云]Failed to search music");
    }
}

async function getLyrics(songID: number | string): Promise<Lyric> {
    const url = getLyricUrl(songID);
    const response = await doRequest({ url });
    const lyric: LyricResult = JSON.parse(response.body);

    const result = new Lyric();

    // 解析原版歌词
    if (lyric.lrc?.lyric?.length > 0) {
        const lines = lyric.lrc.lyric.split(splitReg);
        for (const line of lines) {
            const match = line.match(parseReg);
            if (!match) continue;
            const { time, text } = parse(match);
            result.insert(time, text);
        }
    }
    // 解析翻译歌词
    if (lyric.tlyric?.lyric?.length > 0) {
        const lines = lyric.tlyric.lyric.split(splitReg);
        let n = 0; // 初始化指针 n
        for (const line of lines) {
            const match = line.match(parseReg);
            if (!match) continue;
            const { time, text } = parse(match);
            n = result.insert(time, text, n);
        }
    }

    if (result.lyrics.length === 0) {
        throw new Error("No lyrics found");
    }

    return result;
}

class NeteastLyricAdaptor implements LyricAdaptor {
    name = "网易云";
    status = AdaptorStatus.Pending;
    result: MusicInfo[] = [];

    musicIdCache: number = 0;



    async hasLyrics(title: string, length: number): Promise<boolean> {
        this.status = AdaptorStatus.Loading;
        const songs = await searchMusic(title);
        this.result = songs.filter(song => Math.abs(song.length - length) < MAX_TIME);
        if (this.result.length > 0) {
            this.status = AdaptorStatus.Pending;
            return true;
        } else {
            this.status = AdaptorStatus.NotFound;
            return false;
        }
    }

    async getLyrics(): Promise<Lyric> {
        this.status = AdaptorStatus.Loading;
        try {
            const result = await getLyrics(this.result[0].key);
            this.status = AdaptorStatus.Pending;
            return result;
        } catch (error) {
            this.status = AdaptorStatus.NoAccept;
            throw error;
        }
    }

    async getLyricsByKey(key: string): Promise<Lyric> {
        return await getLyrics(key);
    }
}

export default new NeteastLyricAdaptor();

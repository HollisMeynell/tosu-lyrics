import {getLyricUrl, getMusicInfoUrl} from "./constant.ts";
import {doRequest} from "../do-request.ts";
import {AdaptorStatus, getAudioLength, Lyric, LyricAdaptor, MusicInfo} from "../music-api.ts";
import {MAX_TIME} from "../constant.ts";

type ArtistDetail = {
    id: number;
    name: string;
}

type SongDetail = {
    id: number;
    name: string;
    artists: ArtistDetail[];
    duration: number;
}

type SongSearchResult = {
    code: number;
    result: {
        songs: SongDetail[];
        songCount: number;
    }
}

type SongLyric = {
    // 原版
    lrc: LyricItem;
    // ?
    klyric: LyricItem;
    // 翻译
    tlyric: LyricItem;
    code: number;
}

type LyricItem = {
    version: number;
    lyric: string;
}

function checkText(text: string) {
    // 剔除类似 "翻译: xxx" 等无关信息
    if (text.match(/^..\s?:/)) return false;
    return true;
}

function parse(match: RegExpMatchArray) {
    const minutes = parseInt(match[1]);
    const seconds = parseFloat(match[2]);
    const lyric = match[3];
    const time = minutes * 60 + seconds;
    const text = lyric.trim();
    if (checkText(text)) {
        return {time, text}
    } else {
        return {time: NaN, text: ""}
    }
}

async function searchMusic(title: string): Promise<SongSearchResult> {
    const url = getMusicInfoUrl(title);
    const response = await doRequest({url})
    return JSON.parse(response.body)
}


async function getLyrics(songID: number | string): Promise<Lyric> {
    const url = getLyricUrl(songID);
    const response = await doRequest({url})

    const lyric: SongLyric = JSON.parse(response.body)

    const result = new Lyric()

    let lyricText: string;

    if (lyric.lrc?.lyric?.length > 0) {
        lyricText = lyric.lrc.lyric;
    } else if (lyric.tlyric?.lyric?.length > 0) {
        const temp = lyric.tlyric;
        lyric.tlyric = lyric.lrc;
        lyric.lrc = temp;
        lyricText = lyric.lrc.lyric;
    } else {
        throw new Error("无歌词");
    }

    const splitReg = /(?=\[\d+:\d+\.\d+])/;
    const timeReg = /\[(\d+):(\d+\.\d+)](.*)/;
    let lines = lyricText.split(splitReg);
    for (const line of lines) {
        const match = line.match(timeReg);
        if (!match) continue;
        const {time, text} = parse(match);
        result.insert(time, text);
    }
    if (!(lyric.tlyric?.lyric?.length > 0)) {
        return result;
    }
    lyricText = lyric.tlyric.lyric;

    let n = 0;
    lines = lyricText.split(splitReg);
    for (const line of lines) {
        const match = line.match(timeReg);
        if (!match) {
            continue;
        }
        const {time, text} = parse(match);

        n = result.insert(time, text, n);
    }
    return result;
}

class NeteastLyricAdaptor implements LyricAdaptor {
    name = "网易云";
    status = AdaptorStatus.Pending;
    result: MusicInfo[] = [];

    musicIdCache: number = 0;

    async hasLyrics(): Promise<boolean> {
        // 目前只有这一个适配器，所以直接返回 true
        return true;
    }

    async getLyrics(title: string): Promise<Lyric> {
        this.status = AdaptorStatus.Loading;
        const [musicInfo, length] = await Promise.all([searchMusic(title), getAudioLength()]);
        if (musicInfo.code !== 200 || musicInfo.result?.songCount === 0) {
            this.status = AdaptorStatus.NotFound;
        }
        this.result = musicInfo.result?.songs.map((x) => {
            return {
                title: x.name,
                artist: x.artists.map((x) => x.name).join(", "),
                length: Math.round(x.duration / 1000),
                key: x.id.toString(),
            }
        }) || []
        const songDetail = musicInfo.result
            ?.songs
            ?.find((x) => Math.abs(x.duration - length) < MAX_TIME)
        if (songDetail === undefined) {
            this.status = AdaptorStatus.NoAccept;
            throw new Error("未找到歌曲");
        }
        return await getLyrics(songDetail.id);
    }

    async getLyricsByKey(key: string): Promise<Lyric> {
        return await getLyrics(key);
    }
}

export default new NeteastLyricAdaptor();

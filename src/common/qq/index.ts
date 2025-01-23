import {AdaptorStatus, Lyric, LyricAdaptor, MusicInfo} from "../music-api.ts";
import {getLyricUrl, getMusicInfoUrl, LyricUrlHeader} from "./constant.ts";
import {doRequest, RequestProp} from "../do-request.ts";
import {MAX_TIME} from "../constant.ts";
import LrcParse from "../lrc-parse.ts";

type ArtistDetail = {
    id: number;
    name: string;
}

type SongDetail = {
    albumid: number;
    albummid: string;
    albumname: string;

    songid: number;
    songmid: string;
    songname: string;
    interval: number; // seconds

    singer: ArtistDetail[];
}

type SongSearchResult = {
    code: number; // success: 0 ???wtf
    message: string;
    data: {
        keyword: string;
        song: {
            curnum: number;
            curpage: number;
            list: SongDetail[];
            totalnum: number;
        }
    }
}

// qq 音乐的结构比网易云好一点
type LyricResult = {
    code: number;
    subcode: number;
    lyric: string;
    trans: string;
}

async function searchMusic(title: string): Promise<MusicInfo[]> {
    try {
        const url = getMusicInfoUrl(title);
        const result = await doRequest({url});
        const data: SongSearchResult = JSON.parse(result.body);
        if (data.code !== 0) {
            return [];
        }
        return data.data.song.list.map(song => {
            const songInfo: MusicInfo = {
                title: song.songname,
                artist: song.singer.map(s => s.name).join(", ") || "Unknown",
                length: song.interval * 1000,
                key: song.songmid
            }
            return songInfo
        });
    } catch (error) {
        console.error("[QQ]Failed to search music:", error);
        throw new Error("[QQ]Failed to search music");
    }
}

async function getLyrics(songID: number | string): Promise<Lyric> {
    const url = getLyricUrl(songID);
    const requestProp: RequestProp = {
        url,
        header: LyricUrlHeader,
    }
    const response = await doRequest(requestProp);

    const lyrics: LyricResult = JSON.parse(response.body);

    const result = LrcParse(lyrics.lyric, lyrics.trans);

    if (result.lyrics.length === 0) {
        throw new Error("No lyrics found");
    }

    return result;
}

class QQLyricAdaptor implements LyricAdaptor {
    name = "QQ";
    status = AdaptorStatus.Pending;
    result: MusicInfo[] = [];

    async hasLyrics(title: string, length: number): Promise<boolean> {
        const songs = await searchMusic(title);
        if (length <= 0) {
            this.result = songs;
        } else {
            this.result = songs.filter(song => Math.abs(song.length - length) < MAX_TIME);
        }
        if (this.result.length > 0) {
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

export default new QQLyricAdaptor();

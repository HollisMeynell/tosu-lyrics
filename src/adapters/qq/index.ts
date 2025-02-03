import { SEARCH_MUSIC_URL, GET_LYRIC_URL } from "@/config/constants";
import { LyricAdaptor } from "@/adapters/lyric-adaptor.ts";
import { doRequest } from "@/utils/request.ts";

type ArtistDetail = {
    id: number;
    name: string;
};

type SongDetail = {
    albumid: number;
    albummid: string;
    albumname: string;

    songid: number;
    songmid: string;
    songname: string;
    interval: number; // seconds

    singer: ArtistDetail[];
};

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
        };
    };
};

// qq 音乐的结构比网易云好一点
type LyricResult = {
    code: number;
    subcode: number;
    lyric: string;
    trans: string;
};

export class QQLyricAdaptor extends LyricAdaptor {
    constructor() {
        super("QQ");
    }

    async searchMusic(title: string) {
        try {
            const url = SEARCH_MUSIC_URL("QQ", title);
            const result = await doRequest({ url });
            const data: SongSearchResult = JSON.parse(result.body);
            if (data.code !== 0) {
                return [];
            }
            return data.data.song.list.map((song) => {
                const songInfo = {
                    title: song.songname,
                    artist: song.singer.map((s) => s.name).join(", ") || "Unknown",
                    length: song.interval * 1000,
                    key: song.songmid,
                };
                return songInfo;
            });
        } catch (error) {
            console.error("[QQ]Failed to search music:", error);
            throw new Error("[QQ]Failed to search music");
        }
    }

    async fetchLyrics(songID: number | string) {
        const url = GET_LYRIC_URL(this.name, songID);
        const LyricUrlHeader = {
            Referer: "https://y.qq.com/portal/player.html",
        };
        const requestProp = {
            url,
            header: LyricUrlHeader,
        };
        const response = await doRequest(requestProp);

        const lyrics: LyricResult = JSON.parse(response.body);

        // 转换为 UnifiedLyricResult
        return {
            lyric: lyrics.lyric,
            trans: lyrics.trans,
        };
    }
}

export default new QQLyricAdaptor();

import { SEARCH_MUSIC_URL, GET_LYRIC_URL } from "@/config/constants";
import { doRequest } from "@/utils/request.ts";
import { LyricAdaptor } from "@/adapters/lyric-adaptor.ts";

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

type GETLyricResult = {
    lrc: LyricItem; // 原歌词
    klyric: LyricItem; // karaoke 歌词
    tlyric: LyricItem; // 翻译歌词
    code: number; // 状态码
};

type LyricItem = {
    version: number; // 版本
    lyric: string; // 歌词文本
};

// 网易云音乐适配器
export class NeteaseLyricAdaptor extends LyricAdaptor {
    constructor() {
        super("网易云");
    }

    musicIdCache: number = 0;

    async searchMusic(title: string) {
        try {
            const url = SEARCH_MUSIC_URL("Netease", title);
            const response = await doRequest({ url });
            const data: SongSearchResult = JSON.parse(response.body);
            if (data.code !== 200 || !data.result?.songCount) {
                return [];
            }
            return (
                data.result?.songs.map((x) => {
                    return {
                        title: x.name,
                        artist: x.artists.map((x) => x.name).join(", ") || "Unknown",
                        length: Math.round(x.duration),
                        key: x.id,
                    };
                }) || []
            );
        } catch (error) {
            console.error("[网易云]Failed to search music:", error);
            throw error;
        }
    }

    async fetchLyrics(songID: number | string) {
        const url = GET_LYRIC_URL("Netease", songID);
        const response = await doRequest({ url });
        const lyric: GETLyricResult = JSON.parse(response.body);

        // 转换为 UnifiedLyricResult
        return {
            lyric: lyric.lrc?.lyric || "", // 原版歌词
            trans: lyric.tlyric?.lyric, // 翻译歌词
        };
    }
}

export default new NeteaseLyricAdaptor();

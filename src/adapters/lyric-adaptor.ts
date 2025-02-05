import { TIME_DIFF_FILTER } from "@/config/constants.ts";

export type MusicInfo = {
    title: string;
    artist: string;
    length: number;
    key: number | string;
};

export type AdaptorStatus = "Pending" | "NotFound" | "NoAccept" | "Loading";

export type UnifiedLyricResult = {
    lyric: string; // 原版歌词
    trans?: string; // 翻译歌词
};

export abstract class LyricAdaptor {
    name: string;
    status: AdaptorStatus;
    result: MusicInfo[] = [];

    protected constructor(name: string) {
        this.name = name;
        this.status = "Pending";
    }

    // 根据歌曲名搜索歌曲
    abstract searchMusic(title: string): Promise<MusicInfo[]>;

    // 根据ID获取歌词
    abstract fetchLyrics(songID: number | string): Promise<UnifiedLyricResult>;

    // 检索是否有歌词
    async hasLyrics(title: string, length: number): Promise<boolean> {
        this.status = "Loading";
        let songs: MusicInfo[];
        try {
            songs = await this.searchMusic(title);
            console.log(this.name, songs);
        } catch {
            this.status = "NotFound";
            return false;
        }
        if (length <= 0) {
            this.result = songs;
        } else {
            this.result = songs.filter((song) => TIME_DIFF_FILTER(song.length, length));
        }
        if (this.result.length > 0) {
            return true;
        } else {
            this.status = "NotFound";
            return false;
        }
    }

    // 尝试从搜索结果中的每一首歌获取歌词
    async getLyricsFromResult(): Promise<UnifiedLyricResult> {
        this.status = "Loading";
        for (const song of this.result) {
            try {
                const result = await this.fetchLyrics(song.key);
                this.status = "Pending";
                return result;
            } catch (error) {
                console.error(error);
            }
        }
        this.status = "NoAccept";
        throw Error(`${this.name}: No lyrics found`);
    }

    // 通过 key 获取歌词
    async getLyricsByKey(key: string): Promise<UnifiedLyricResult> {
        return await this.fetchLyrics(key);
    }
}

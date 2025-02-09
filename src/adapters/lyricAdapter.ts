import { TIME_DIFF_FILTER } from "@/config/constants.ts";
import { MusicInfo, UnifiedLyricResult } from "@/types/lyricTypes.ts";

export type AdapterStatus = "Pending" | "NotFound" | "NoAccept" | "Loading";

export abstract class LyricAdapter {
    name: string;
    status: AdapterStatus;
    result: MusicInfo[] = [];
    // 所有的搜索结果, 用于发送到控制台来选择
    allResult: MusicInfo[] = [];
    // 当前正使用的歌词
    current: number | string | undefined;

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
            this.allResult = songs;
            console.log(this.name, songs);
        } catch {
            this.status = "NotFound";
            return false;
        }
        if (length <= 0) {
            this.result = songs;
        } else {
            this.result = songs.filter((song) =>
                TIME_DIFF_FILTER(song.length, length)
            );
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
    async getLyricsByKey(key: string | number): Promise<UnifiedLyricResult> {
        return await this.fetchLyrics(key);
    }
}

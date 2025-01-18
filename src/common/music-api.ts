import { AUDIO_URL } from "./constant.ts";

type LyricLine = {
    time: number;
    first: string;
    second?: string;
};

export enum AdaptorStatus {
    "Pending",
    "NotFound",
    "NoAccept",
    "Loading",
}

export type MusicInfo = {
    title: string;
    artist: string;
    length: number;
    key: string;
};

export interface LyricAdaptor {
    name: string;
    status: AdaptorStatus;
    result: MusicInfo[];

    // 检索是否有歌词
    hasLyrics(title: string, length: number): Promise<boolean>;

    // 获取歌词
    getLyrics(title: string, length: number): Promise<Lyric>;

    // 通过 key 获取歌词
    getLyricsByKey(key: string): Promise<Lyric>;
}

export class Lyric {
    lyrics: LyricLine[];
    endTime: number;
    cursor: number;

    constructor() {
        this.lyrics = [];
        this.endTime = -1;
        this.cursor = 0;
    }

    // 插入歌词
    insert(time: number, t: string): void {
        const text = t.trim();
        if (isNaN(time) || time < 0 || text.length === 0) return;

        let low = 0;
        let high = this.lyrics.length - 1;

        while (low <= high) {
            const mid = Math.floor((low + high) / 2);
            const midTime = this.lyrics[mid].time;

            // 判断时间戳是否接近（差值小于 1e-2）
            if (Math.abs(midTime - time) < 1e-2) {
                // 时间戳接近，更新已有歌词
                this.lyrics[mid].second = this.lyrics[mid].first;
                this.lyrics[mid].first = text;
                return;
            } else if (midTime < time) {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        // 时间戳不接近，插入新歌词
        this.lyrics.splice(low, 0, { time, first: text });

        // 更新 endTime
        if (time > this.endTime) {
            this.endTime = time;
        }
    }

    nextTime(): number {
        if (this.cursor >= this.lyrics.length - 1) return 0;
        return this.lyrics[this.cursor + 1].time - this.lyrics[this.cursor].time;
    }

    jump(time: number) {
        if (this.lyrics.length === 0) return;
        if (time < 1.5) {
            this.cursor = 0;
            return;
        }

        let low = 0;
        let high = this.lyrics.length - 1;

        while (low <= high) {
            const mid = Math.floor((low + high) / 2);
            if (this.lyrics[mid].time <= time) {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        this.cursor = high;
    }
}

export async function getLyrics(title: string): Promise<Lyric> {
    const { default: NeteastLyricAdaptor } = await import("./neteast");
    return NeteastLyricAdaptor.getLyrics(title);
}

export async function getAudioLength(): Promise<number> {
    return new Promise((resolve, reject) => {
        const audio = new Audio(AUDIO_URL);

        // 成功加载元数据后获取时长
        audio.onloadedmetadata = () => {
            resolve(audio.duration * 1000);
        };

        // 处理加载失败的情况
        audio.onerror = () => {
            reject(new Error("Failed to load audio"));
        };

        // 开始加载音频
        audio.load();
    });
}

import {AUDIO_URL} from "./constant.ts";


type LyricLine = {
    time: number;
    first: string;
    second?: string,
}

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
}

export interface LyricAdaptor {
    name: string;
    status: AdaptorStatus;
    result: MusicInfo[];

    // 检索是否有歌词
    hasLyrics(title: string, length: number): Promise<boolean>;

    // 获取歌词
    getLyrics(title: string, length: number): Promise<Lyric>;

    getLyricsByKey(key: string): Promise<Lyric>;
}

export class Lyric {
    lyrics: LyricLine[];
    endTime: number;
    cursor: number;

    constructor() {
        this.lyrics = [];
        this.endTime = -1;
        this.cursor = 0

    }

    insert(time: number, t: string, n: number = 0): number {
        const text = t.trim();
        if (isNaN(time) || time < 0 || text.length == 0) return n + 1;

        if (time > this.endTime) {
            this.endTime = time;
            this.lyrics.push({time: time, first: text})
        } else {
            if (!this.lyrics[n]) {
                this.lyrics.push({time: time, first: text})
            } else if (Math.abs(this.lyrics[n].time - time) < 1e-2) {
                this.lyrics[n].second = this.lyrics[n].first;
                this.lyrics[n].first = text;
            } else if (this.lyrics[n].time > time + 1e-2) {
                if (n == 0) {
                    this.lyrics.splice(n, 0, {time, first: text})
                } else {
                    while (n >= 0 && this.lyrics[n].time >= time + 1e-2) {
                        n--;
                    }
                    if (Math.abs(this.lyrics[n].time - time) < 1e-2) {
                        this.lyrics[n].second = this.lyrics[n].first;
                        this.lyrics[n].first = text;
                    } else {
                        this.lyrics.splice(n, 0, {time, first: text})
                    }
                }
            } else {
                if (n + 1 >= this.lyrics.length) {
                    this.lyrics.push({time: time, first: text})
                } else {
                    while (n < this.lyrics.length && this.lyrics[n + 1].time < time - 1e-2) {
                        n++;
                    }
                    if (Math.abs(this.lyrics[n + 1].time - time) < 1e-2) {
                        this.lyrics[n + 1].second = this.lyrics[n + 1].first
                        this.lyrics[n + 1].first = text
                    } else {
                        this.lyrics.splice(n, 0, {time, first: text})
                    }
                }
            }
        }
        return n + 1;
    }

    nextTime():number {
        if (this.cursor >= this.lyrics.length - 1) return 0;
        return this.lyrics[this.cursor + 1].time - this.lyrics[this.cursor].time;
    }

    jump(time: number) {
        if (this.lyrics.length == 0) return;
        if (time < 1.5) {
            this.cursor = 0;
            return;
        }

        const n = this.cursor;
        if (this.lyrics[n].time < time) {
            const nextTime = this.lyrics[n + 1]?.time || 99999;
            if (nextTime > time) {
                return;
            }
            const nextTime2 = this.lyrics[n + 2]?.time || 99999;
            if (nextTime2 > time) {
                this.cursor = n + 1;
                return;
            }
        }

        let low = 0;
        let high = this.lyrics.length - 1;

        if (this.lyrics[low].time > time) {
            this.cursor = low;
            return;
        }

        if (this.lyrics[high].time < time) {
            this.cursor = high;
            return;
        }

        while (low < high) {
            const mid = Math.floor((low + high) / 2);
            if (mid == low) {
                this.cursor = mid;
                break;
            }
            if (this.lyrics[mid].time < time) {
                low = mid;
            } else {
                high = mid;
            }
        }
    }
}


export async function getLyrics(title: string): Promise<Lyric> {
    const {default: NeteastLyricAdaptor} = await import('./neteast');
    return NeteastLyricAdaptor.getLyrics(title);
}

export async function getAudioLength(): Promise<number> {
    return new Promise((resolve) => {
        const audio = new Audio(AUDIO_URL);
        audio.load();
        const check = () => {
            if (audio.readyState > 0) {
                if (audio.duration === Infinity) {
                    audio.currentTime = 1e6;
                    setInterval(check, 50);
                    return;
                }
                resolve(audio.duration * 1000);
            } else {
                setInterval(check, 50);
            }
        }
        setInterval(check, 50);
    })
}

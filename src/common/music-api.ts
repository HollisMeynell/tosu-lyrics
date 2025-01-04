import {proxyRequest} from "./proxy-request.ts";
import {AUDIO_URL, getLyricUrl, getMusicInfoUrl} from "./constant.ts";

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

export async function searchMusic(title: string): Promise<SongSearchResult> {
    const url = getMusicInfoUrl(title);
    const response = await proxyRequest({url})
    return JSON.parse(response.body)
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

type LyricLine = {
    time: number;
    text: string;
    translate?: string,
}

function checkText(text: string) {
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

export class Lyric {
    lyrics: LyricLine[];
    cursor: number;

    constructor(lyric?: SongLyric) {
        this.lyrics = [];
        this.cursor = 0

        if (!lyric) return;

        let lyricText: string;
        if (lyric.lrc?.lyric?.length > 0) {
            lyricText = lyric.lrc.lyric;
        } else if (lyric.tlyric?.lyric?.length > 0) {
            const temp = lyric.tlyric;
            lyric.tlyric = lyric.lrc;
            lyric.lrc = temp;
            lyricText = lyric.lrc.lyric;
        } else {
            return;
        }
        let lines = lyricText.split("\n");
        for (const line of lines) {
            const match = line.match(/\[([0-9]+):([0-9]+\.[0-9]+)\](.*)/);
            if (match) {
                const {time, text} = parse(match);
                if (isNaN(time) || text.length == 0) {
                    continue;
                }
                this.lyrics.push({time: time, text: text})
            }
        }
        if (!(lyric.tlyric?.lyric?.length > 0)) {
            return;
        }
        lyricText = lyric.tlyric.lyric;

        let n = 0;

        lines = lyricText.split("\n");
        for (const line of lines) {
            const match = line.match(/\[([0-9]+):([0-9]+\.[0-9]+)\](.*)/);
            if (!match) {
                continue;
            }
            const {time, text} = parse(match);
            if (isNaN(time) || text.length == 0) {
                continue;
            }

            if (!this.lyrics[n]) {
                this.lyrics.push({time: time, text: text})
            } else if (Math.abs(this.lyrics[n].time - time) < 1e-2) {
                this.lyrics[n].translate = text
            } else if (this.lyrics[n].time > time + 1e-2) {
                this.lyrics.splice(n, 0, {time, text})
            } else {
                if (n + 1 >= this.lyrics.length) {
                    this.lyrics.push({time: time, text: text})
                } else {
                    while (n < this.lyrics.length && this.lyrics[n + 1].time < time - 1e-2) {
                        n++;
                    }
                    if (Math.abs(this.lyrics[n + 1].time - time) < 1e-2) {
                        this.lyrics[n + 1].translate = text
                    } else {
                        this.lyrics.splice(n, 0, {time, text})
                    }
                }
            }
            n++;
        }

        console.log("", this.lyrics)
    }

    nextTime() {
        if (this.cursor >= this.lyrics.length - 1) return 3;
        return this.lyrics[this.cursor + 1].time - this.lyrics[this.cursor].time;
    }

    jump(time: number) {
        debugger
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

export async function getLyrics(songID: number): Promise<Lyric> {
    const url = getLyricUrl(songID);
    const response = await proxyRequest({url})
    const data: SongLyric = JSON.parse(response.body)
    return new Lyric(data)
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

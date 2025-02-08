import { parseLyricText } from "@/utils/lrc-parse";
import { LyricRawLine } from "@/types/config-global";

export class Lyric {
    lyrics: LyricRawLine[];
    endTime: number;
    cursor: number;

    constructor() {
        this.lyrics = [];
        this.endTime = -1;
        this.cursor = 0;
    }

    // 插入单句歌词
    insert(time: number, t: string, n: number = 0): number {
        const text = t.trim();

        // 检查时间戳和歌词文本是否有效
        if (isNaN(time) || time < 0 || text.length == 0) return n + 1;

        // 时间戳大于当前最大时间戳，直接插入到末尾
        if (time > this.endTime) {
            this.endTime = time;
            this.lyrics.push({ time: time, first: text });
            return n + 1;
        }

        // 时间戳小于或等于当前最大时间戳，从指针 n 开始查找插入位置
        if (!this.lyrics[n]) {
            // 指针 n 超出数组范围，直接插入到末尾
            this.lyrics.push({ time: time, first: text });
        } else if (Math.abs(this.lyrics[n].time - time) < 1e-2) {
            // 时间戳接近（差值小于 1e-2），更新已有歌词 (翻译在前)
            this.lyrics[n].second = this.lyrics[n].first;
            this.lyrics[n].first = text;
        } else if (this.lyrics[n].time > time + 1e-2) {
            // 时间戳小于指针 n 的时间戳，向前查找插入位置
            if (n == 0) {
                // 直接插入到数组开头
                this.lyrics.splice(n, 0, { time, first: text });
            } else {
                // 否则，向前查找合适的位置
                while (n >= 0 && this.lyrics[n].time >= time + 1e-2) {
                    n--;
                }
                if (Math.abs(this.lyrics[n].time - time) < 1e-2) {
                    this.lyrics[n].second = this.lyrics[n].first;
                    this.lyrics[n].first = text;
                } else {
                    this.lyrics.splice(n, 0, { time, first: text });
                }
            }
        } else {
            // 时间戳大于指针 n 的时间戳，向后查找插入位置
            if (n + 1 >= this.lyrics.length) {
                // n + 1 超出数组范围，直接插入到末尾
                this.lyrics.push({ time: time, first: text });
            } else {
                // 否则，向后查找合适的位置
                while (
                    n < this.lyrics.length &&
                    this.lyrics[n + 1].time < time - 1e-2
                ) {
                    n++;
                }
                if (Math.abs(this.lyrics[n + 1].time - time) < 1e-2) {
                    this.lyrics[n + 1].second = this.lyrics[n + 1].first;
                    this.lyrics[n + 1].first = text;
                } else {
                    this.lyrics.splice(n, 0, { time, first: text });
                }
            }
        }
        // 返回新的指针位置
        return n + 1;
    }

    insertAll(lyric: string, trans?: string, title?: string) {
        if (!lyric?.trim()?.length) {
            throw Error("Empty lyric");
        }

        try {
            // 解析原版歌词
            const lyricLines = parseLyricText(lyric);

            if (title && lyricLines.length > 0 && lyricLines[0].time !== 0) {
                this.insert(0, title); // 插入标题，时间戳为 0
            }

            for (const { time, text } of lyricLines) {
                if (!this.insert(time, text)) {
                    console.warn(`Failed to insert lyric at ${time}: ${text}`);
                }
            }

            // 解析翻译歌词
            if (trans?.trim()?.length) {
                const transLines = parseLyricText(trans);
                let cursor = 0;
                for (const { time, text } of transLines) {
                    cursor = this.insert(time, text, cursor);
                    if (cursor === -1) {
                        console.warn(
                            `Failed to insert translation at ${time}: ${text}`
                        );
                        break;
                    }
                }
            }
        } catch (error) {
            console.error("Failed to parse lyric:", error);
            throw error;
        }
    }

    // 获取当前歌词
    nextTime(): number {
        if (this.cursor >= this.lyrics.length - 1) return 0;
        return (
            this.lyrics[this.cursor + 1].time - this.lyrics[this.cursor].time
        );
    }

    // 跳转到指定时间
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
            const smaller = this.lyrics[mid].time <= time;
            const bigger = this.lyrics[mid + 1].time > time;
            if (smaller && bigger) {
                this.cursor = mid;
                return;
            } else if (smaller) {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        this.cursor = low;
    }
}

import { createSignal } from "solid-js";
import { WebsocketLyric } from "@/api/model.ts";
import { LyricLine } from "@/types/lyricTypes.ts";

// 歌词展示页的同步状态, 由后端 lyric 事件直接写入

// 展示用歌词行: main 为翻译, origin 为原歌词
export const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
// 当前歌词下标
export const [cursor, setCursor] = createSignal(0);
// 距离下一行开始的毫秒数, 末行为 -1(展示时按 0 处理)
export const [nextTime, setNextTime] = createSignal(0);

// 后端歌词行 -> 展示歌词行
const toDisplayLine = (line: {
    origin?: string;
    translation?: string;
}): LyricLine => {
    const { origin, translation } = line;
    if (translation != null) {
        return origin != null
            ? { main: translation, origin }
            : { main: translation };
    }
    return { main: origin ?? "" };
};

/**
 * 处理后端下发的歌词事件
 */
export const applyLyricEvent = (data: WebsocketLyric) => {
    // lyric 数组仅在换歌后首次下发
    if (data.lyric != null) {
        setLyrics(data.lyric.map(toDisplayLine));
    }
    // current 为 -1 表示不需要显示
    if (data.current >= 0) {
        setCursor(data.current);
    }
    setNextTime(data.nextTime > 0 ? data.nextTime : 0);
};

/**
 * 清空当前歌词 (setClear 广播)
 */
export const clearLyrics = () => {
    setLyrics([]);
    setCursor(0);
    setNextTime(0);
};

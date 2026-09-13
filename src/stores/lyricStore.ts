import { batch, createSignal } from "solid-js";
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
 *
 * WS 回调不是 Solid 的事件处理器, 不会自动批处理。若不显式 batch,
 * lyrics / cursor / nextTime 会各自触发一次滚动副作用, 造成一次错误滚动。
 */
export const applyLyricEvent = (data: WebsocketLyric) => {
    batch(() => {
        // lyric 数组仅在换歌 / 换源 / 上传后首次下发
        const hasFullList = data.lyric != null;
        const nextLyrics = hasFullList ? data.lyric!.map(toDisplayLine) : lyrics();
        // current 为 -1 表示不需要显示; 下标越界时保持现状, 避免滚动到不存在的行
        const cursorInRange =
            data.current >= 0 && data.current < nextLyrics.length;

        if (hasFullList) setLyrics(nextLyrics);
        if (cursorInRange) setCursor(data.current);
        // 末行 nextTime 为 -1, 展示层按 0(立即完成)处理
        setNextTime(data.nextTime > 0 ? data.nextTime : 0);
    });
};

/**
 * 清空当前歌词 (setClear 广播)
 */
export const clearLyrics = () => {
    batch(() => {
        setLyrics([]);
        setCursor(0);
        setNextTime(0);
    });
};

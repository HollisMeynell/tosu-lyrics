import { batch, createSignal } from "solid-js";
import { WebsocketLyric } from "@/api/model.ts";
import { LyricLine } from "@/types/lyricTypes.ts";

export const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
export const [cursor, setCursor] = createSignal(0);
// 距离下一行开始的毫秒数, 末行为 -1(展示时按 0 处理)
export const [nextTime, setNextTime] = createSignal(0);

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

export const applyLyricEvent = (data: WebsocketLyric) => {
    batch(() => {
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

export const clearLyrics = () => {
    batch(() => {
        setLyrics([]);
        setCursor(0);
        setNextTime(0);
    });
};

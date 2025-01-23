// parse LRC

import {Lyric} from "./music-api.ts";

const regLineStart = /(?=\[.+:.+])/
const regLyricLine = /\[(\d+):(\d+\.\d+)](.+)/

function parseLyric(match: RegExpMatchArray) {
    const minutes = parseInt(match[1]);
    const seconds = parseFloat(match[2]);
    const lyric = match[3];
    const time = minutes * 60 + seconds;
    const text = lyric.trim();
    return { time, text }
}

export default function (lyric: string, trans?: string) {
    if (!lyric?.trim()?.length) {
        throw Error("Empty lyric");
    }
    const result = new Lyric();

    // 解析原版歌词
    let lines = lyric.split(regLineStart);
    for (const line of lines) {
        const lyricMatcher = line.match(regLyricLine);
        if (!lyricMatcher) {
            continue;
        }
        const { time, text } = parseLyric(lyricMatcher);
        result.insert(time, text);
    }

    if (!trans?.trim()?.length) {
        return result;
    }

    // 解析翻译歌词
    lines = trans?.split(regLineStart);
    let cursor = 0;
    for (const line of lines) {
        const lyricMatcher = line.match(regLyricLine);
        if (!lyricMatcher) {
            continue;
        }
        const { time, text } = parseLyric(lyricMatcher);
        cursor = result.insert(time, text, cursor);
    }

    return result;
}

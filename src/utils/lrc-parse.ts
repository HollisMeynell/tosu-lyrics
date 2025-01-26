// Purpose: Parse LRC lyric text into Lyric object.

type ParsedLyricLine = {
    time: number;
    text: string;
};

const regLineStart = /(?=\[.+:.+])/;
const regLyricLine = /\[(\d+):(\d+\.\d+)](.+)/;

// 解析歌词行
function parseLyric(match: RegExpMatchArray): ParsedLyricLine {
    const minutes = parseInt(match[1]);
    const seconds = parseFloat(match[2]);
    const lyric = match[3];
    const time = minutes * 60 + seconds;
    const text = lyric.trim();
    return { time, text };
}

// 解析歌词文本
export function parseLyricText(lyricText: string): ParsedLyricLine[] {
    const lines = lyricText.split(regLineStart);
    const result: ParsedLyricLine[] = [];

    for (const line of lines) {
        const lyricMatcher = line.match(regLyricLine);
        if (!lyricMatcher) continue; // 跳过无效行
        result.push(parseLyric(lyricMatcher));
    }

    return result;
}

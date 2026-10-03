// 功能: 面板-文字样式（歌词行数）
import { Select } from "@/components/ui";
import { lyricLines } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";
import {
    DEFAULT_LYRIC_LINES,
    LYRIC_LINE_OPTIONS,
    normalizeLyricLines,
} from "@/utils/lyricLines";

interface LyricLinesProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

/**
 * 只给出合法奇数，界面上不可能选出偶数。
 * 后端 / 广播回来的值仍会再走一次 `normalizeLyricLines`（见 settingsStore）。
 */
const options = LYRIC_LINE_OPTIONS.map((value) => ({
    code: String(value),
    name: value === DEFAULT_LYRIC_LINES ? `${value} 行（默认）` : `${value} 行`,
}));

export default function LyricLines(props: LyricLinesProps) {
    const commit = (raw: string) => {
        const value = normalizeLyricLines(raw);
        if (value === lyricLines()) return;
        void props.update({ lyricLines: value });
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-9">
            <h2 class="text-xl font-normal">歌词行数</h2>
            <Select
                class="min-w-[10rem]"
                clearable={false}
                options={options}
                value={String(lyricLines())}
                disabled={props.disabled}
                onChange={commit}
            />
        </div>
    );
}

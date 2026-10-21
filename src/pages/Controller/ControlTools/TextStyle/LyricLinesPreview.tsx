// 功能: 面板-文字样式（歌词行数预览）
import { For, createMemo } from "solid-js";
import { lyricLines } from "@/stores/settingsStore";
import {
    lineRowHeight,
    lyricBoxHeight,
    normalizeLyricLines,
} from "@/utils/lyricLines";

/** 预览缩放：与展示端共用同一套行高计算，只按比例缩小 */
const PREVIEW_SCALE = 0.35;

/**
 * 歌词行数的实际排布预览（行数 > 3 时展示在右侧）。
 *
 * 行高直接取 `lineRowHeight` / `lyricBoxHeight` —— 与 `/lyrics` 展示端同一份
 * 计算，中间 3 行完整间距、上下外侧行紧凑，调整行数时这里同步变化。
 */
export default function LyricLinesPreview() {
    const lines = createMemo(() => normalizeLyricLines(lyricLines()));
    const rows = createMemo(() => {
        const mid = (lines() - 1) / 2;
        return Array.from({ length: lines() }, (_, i) =>
            lineRowHeight(Math.abs(i - mid))
        );
    });

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl font-medium inline">行数预览</h2>
                <p class="text-sm inline text-gray-500">
                    共 {lines()} 行 · 中间 3 行保持完整间距，上下外侧行紧凑
                </p>
            </div>
            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <div class="flex justify-center">
                <div class="w-full max-w-md flex flex-col items-center rounded-lg border border-gray-200 dark:border-gray-700 px-6 py-4 overflow-hidden">
                    <For each={rows()}>
                        {(height, index) => {
                            const active = index() === (lines() - 1) / 2;
                            return (
                                <div
                                    class="w-full flex items-center justify-center transition-[height] duration-300"
                                    style={{
                                        height: `${height * PREVIEW_SCALE}px`,
                                    }}
                                >
                                    <div
                                        class={
                                            active
                                                ? "w-1/2 h-2 rounded-full bg-[#ec4899]"
                                                : "w-1/3 h-1.5 rounded-full bg-gray-300 dark:bg-gray-600"
                                        }
                                    />
                                </div>
                            );
                        }}
                    </For>
                </div>
            </div>
            <p class="text-xs text-gray-500">
                展示端窗口高度约 {lyricBoxHeight(lines())} px，随行数变化。
            </p>
        </div>
    );
}

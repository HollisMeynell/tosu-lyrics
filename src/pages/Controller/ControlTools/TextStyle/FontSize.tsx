// 功能: 面板-文字样式（字号）
import { fontSize } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";

interface FontSizeProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

/** 与后端 `MIN_FONT_SIZE` / `MAX_FONT_SIZE` 保持一致 */
const MIN = 0.5;
const MAX = 12;

const inputClass =
    "w-20 p-2 pl-3 border border-[#cbd5e1] rounded-lg shadow-xs bg-white " +
    "focus:outline-hidden focus:ring-1 focus:ring-[#eb4898] focus:border-[#eb4898] " +
    "hover:border-[#94a3b8] transition-colors dark:bg-[#020616] dark:border-[#475569] " +
    "dark:text-white disabled:opacity-50 disabled:cursor-not-allowed";

export default function FontSize(props: FontSizeProps) {
    const commit = (order: "first" | "second", raw: string) => {
        const value = Number(raw);
        if (!Number.isFinite(value) || value < MIN || value > MAX) return;
        const current = fontSize();
        if (current[order] === value) return;
        void props.update({ fontSize: { ...current, [order]: value } });
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-9">
            <h2 class="text-xl font-normal">字号</h2>
            <div class="flex flex-row items-center gap-4">
                <p>主歌词:</p>
                <input
                    type="number"
                    class={inputClass}
                    min={MIN}
                    max={MAX}
                    step="0.5"
                    disabled={props.disabled}
                    value={fontSize().first}
                    onChange={(e) => commit("first", e.currentTarget.value)}
                />
                <p>副歌词:</p>
                <input
                    type="number"
                    class={inputClass}
                    min={MIN}
                    max={MAX}
                    step="0.5"
                    disabled={props.disabled}
                    value={fontSize().second}
                    onChange={(e) => commit("second", e.currentTarget.value)}
                />
            </div>
        </div>
    );
}

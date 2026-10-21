// 功能: 面板-文字样式（颜色、字体、显示效果）
import { Component, createSignal } from "solid-js";
import { ColorSelector } from "@/components/ui";
import { textColor } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";
import { parseColor, withAlpha } from "@/utils/color";

interface TextColorProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

/** 不透明度滑杆：拖动只更新本地预览，松手（change）才提交 */
const AlphaSlider: Component<{
    value: number;
    disabled?: boolean;
    onCommit: (alpha: number) => void;
}> = (props) => {
    const [preview, setPreview] = createSignal<number | undefined>(undefined);
    const shown = () => preview() ?? props.value;

    return (
        <>
            <input
                type="range"
                min="0"
                max="100"
                step="1"
                class="w-24 md:w-32 accent-[#ec4899] cursor-pointer disabled:cursor-not-allowed"
                title="文字不透明度：100% 完全不透明，0% 完全透明"
                value={Math.round(shown() * 100)}
                disabled={props.disabled}
                onInput={(e) => setPreview(Number(e.currentTarget.value) / 100)}
                onChange={(e) => {
                    const alpha = Number(e.currentTarget.value) / 100;
                    setPreview(undefined);
                    props.onCommit(alpha);
                }}
            />
            <span class="text-sm text-gray-500 w-11 tabular-nums">
                {Math.round(shown() * 100)}%
            </span>
        </>
    );
};

export default function TextColor(props: TextColorProps) {
    const commit = (order: "first" | "second", value: string) => {
        void props.update({
            textColor: { ...textColor(), [order]: value },
        });
    };

    const commitAlpha = (order: "first" | "second", alpha: number) => {
        void props.update({
            textColor: {
                ...textColor(),
                [order]: withAlpha(textColor()[order], alpha),
            },
        });
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-9">
            <h2 class="text-xl font-normal">文字颜色</h2>
            <div class="flex flex-row items-center gap-4 flex-wrap">
                <div class="flex flex-row items-center gap-2">
                    <p>主歌词:</p>
                    <ColorSelector
                        class="min-w-6"
                        value={textColor().first}
                        disabled={props.disabled}
                        onCommit={(value) => commit("first", value)}
                    />
                    <p class="text-sm text-gray-500">不透明度</p>
                    <AlphaSlider
                        value={parseColor(textColor().first).alpha}
                        disabled={props.disabled}
                        onCommit={(alpha) => commitAlpha("first", alpha)}
                    />
                </div>
                <div class="flex flex-row items-center gap-2">
                    <p>副歌词:</p>
                    <ColorSelector
                        class="min-w-6"
                        value={textColor().second}
                        disabled={props.disabled}
                        onCommit={(value) => commit("second", value)}
                    />
                    <p class="text-sm text-gray-500">不透明度</p>
                    <AlphaSlider
                        value={parseColor(textColor().second).alpha}
                        disabled={props.disabled}
                        onCommit={(alpha) => commitAlpha("second", alpha)}
                    />
                </div>
            </div>
        </div>
    );
}

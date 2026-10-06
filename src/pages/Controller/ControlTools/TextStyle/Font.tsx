// 功能: 面板-文字样式（主 / 副歌词字体选择）
import { Show, onMount } from "solid-js";
import { FontPicker } from "@/components/ui";
import { fontMode } from "@/stores/fontModeStore";
import { font, secondFont } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";
import { fontEntries, fetchFontList } from "@/utils/fonts";

interface FontProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

/**
 * 系统字体：与项目原有列表保持一致，继续作为额外选项保留。
 * cSpell: ignore msyh simsun fangsong kaiti
 */
const SYSTEM_FONTS = [
    { code: "msyh", name: "微软雅黑" },
    { code: "simsun", name: "宋体" },
    { code: "fangsong", name: "仿宋" },
    { code: "kaiti", name: "楷体" },
    { code: "arial", name: "Arial" },
];

export default function Controller(props: FontProps) {
    const shared = () => fontMode() === "shared";

    // 首次挂载时加载字体列表
    onMount(() => {
        void fetchFontList();
    });

    /**
     * 选项列表 = 默认字体 + 字体库中所有字体 + 系统字体。
     *
     * 默认字体永远是第一个选项（空串表示使用系统默认）。
     * 字体库中的字体按名称显示。
     */
    const options = () => {
        const opts = [{ code: "", name: "默认字体" }];
        // 添加字体库中的字体
        for (const entry of fontEntries()) {
            opts.push({ code: entry.name, name: entry.name });
        }
        // 添加系统字体
        return [...opts, ...SYSTEM_FONTS];
    };

    // SettingsPatch 里 font 必须是完整的一对，所以每次都带上另一侧的原值。
    const applyMain = (value: string) => {
        void props.update({
            // 共用模式下改主字体要同时写两侧
            font: { first: value, second: shared() ? value : secondFont() },
        });
    };

    const applySub = (value: string) => {
        void props.update({ font: { first: font(), second: value } });
    };

    return (
        <div class="flex flex-col items-start gap-4">
            <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                <h2 class="text-xl font-normal">主歌词字体</h2>
                <FontPicker
                    class="w-56"
                    options={options()}
                    value={font()}
                    disabled={props.disabled}
                    onChange={applyMain}
                />
            </div>

            <Show
                when={!shared()}
                fallback={
                    <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                        <h2 class="text-xl font-normal">副歌词字体</h2>
                        <span class="text-sm text-gray-500">
                            共用字体模式：副歌词跟随主字体
                        </span>
                    </div>
                }
            >
                <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                    <h2 class="text-xl font-normal">副歌词字体</h2>
                    <FontPicker
                        class="w-56"
                        options={options()}
                        value={secondFont()}
                        disabled={props.disabled}
                        onChange={applySub}
                    />
                </div>
            </Show>
        </div>
    );
}
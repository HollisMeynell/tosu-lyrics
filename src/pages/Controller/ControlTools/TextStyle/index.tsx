// 功能: 面板-文字样式（颜色、字体、显示效果）
//
// B-03 纵向切片：设置全部通过 `/api/settings`（HTTP）读写，不再走旧 Config / wsService。
// 保存成功用服务端返回值覆盖本地状态；失败时保留原值并显示错误。
import { Show, onMount } from "solid-js";
import { Button, ToggleSwitch, ToggleNSwitch } from "@/components/ui";
import TextColor from "./TextColor";
import Font from "./Font";
import FontSize from "./FontSize";
import { AlignType, alignmentOptions } from "@/types/globalTypes";
import {
    alignment,
    showSecond,
    useTranslationAsMain,
} from "@/stores/settingsStore";
import { createSettingsController } from "@/hooks/useSettings";
import { DEFAULT_SHADOW } from "@/stores/settingsStore";
import { SettingsPatch } from "@/types/globalTypes";

/**
 * 原项目（tosu-lyrics-origin）默认文字样式。
 *
 * 逐项核对过 origin 的 `stores/settingsStore.ts` 与 `pages/LyricsBox/index.tsx`：
 * 颜色 `#ffffff` / `#e0e0e0`，字号 3em / 2em（active），字体名为空，
 * 居中对齐，翻译优先开启，显示副歌词开启，阴影关闭。
 *
 * **只覆盖文字样式相关字段**，不碰其它状态。
 */
const DEFAULT_TEXT_STYLE: SettingsPatch = {
    textColor: { first: "#ffffff", second: "#e0e0e0" },
    fontSize: { first: 3, second: 2 },
    font: { first: "", second: "" },
    alignment: "center",
    translationMain: true,
    secondShow: true,
    shadow: { first: DEFAULT_SHADOW, second: DEFAULT_SHADOW },
};

export default function TextStyle() {
    const settings = createSettingsController();
    onMount(() => {
        void settings.load();
    });

    return (
        <div class="flex flex-col gap-4">
            <h1 class="text-2xl font-medium">文字样式</h1>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />

            <Show when={settings.error()}>
                <p class="text-sm text-red-500">
                    设置未保存（已保留原值）：{settings.error()}
                </p>
            </Show>

            <div class="flex flex-row items-center gap-3">
                <Button
                    class="px-4 py-1"
                    onClick={() => settings.update(DEFAULT_TEXT_STYLE)}
                    disabled={settings.saving()}
                >
                    恢复默认样式
                </Button>
                <span class="text-xs text-gray-500">
                    还原为原项目默认（颜色 / 字号 / 字体 / 对齐 / 翻译优先 / 副歌词 / 阴影）
                </span>
            </div>

            <Show
                when={!settings.loading()}
                fallback={<p class="text-gray-500">正在读取设置…</p>}
            >
                <div class="flex flex-col items-start gap-8 md:gap-4">
                    {/* 对齐方式 */}
                    <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-6">
                        <h2 class="text-2xl font-normal">对齐方式</h2>
                        <ToggleNSwitch
                            options={alignmentOptions}
                            class="min-w-[15rem]"
                            disabled={settings.saving()}
                            selectedValue={alignment()}
                            onUpdateSelectedValue={(value) =>
                                void settings.update({
                                    alignment: value as AlignType,
                                })
                            }
                        />
                    </div>
                    {/* 字体颜色 */}
                    <TextColor
                        update={settings.update}
                        disabled={settings.saving()}
                    />
                    {/* 字体 */}
                    <Font
                        update={settings.update}
                        disabled={settings.saving()}
                    />
                    {/* 字号 */}
                    <FontSize
                        update={settings.update}
                        disabled={settings.saving()}
                    />
                    {/* 显示 */}
                    <div class="flex flex-row items-center gap-3">
                        <ToggleSwitch
                            disabled={settings.saving()}
                            modelValue={useTranslationAsMain()}
                            onUpdateModelValue={(value) =>
                                void settings.update({
                                    translationMain: value,
                                })
                            }
                        />
                        <h2 class="text-2xl font-normal">以翻译歌词为主</h2>
                    </div>
                    <div class="flex flex-row items-center gap-3">
                        <ToggleSwitch
                            disabled={settings.saving()}
                            modelValue={showSecond()}
                            onUpdateModelValue={(value) =>
                                void settings.update({ secondShow: value })
                            }
                        />
                        <h2 class="text-2xl font-normal">显示副歌词</h2>
                    </div>
                </div>
            </Show>

            <Show when={settings.saving()}>
                <p class="text-sm text-gray-500">正在保存…</p>
            </Show>
        </div>
    );
}

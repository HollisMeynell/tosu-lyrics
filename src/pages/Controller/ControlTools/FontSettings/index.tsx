// 功能: 面板-字体设置（共用 / 分开字体、主副字体、字体库上传）
import { Show, onMount } from "solid-js";
import FontModeToggle from "./FontModeToggle";
import Font from "./Font";
import FontUpload from "./FontUpload";
import { createSettingsController } from "@/hooks/useSettings";

export default function FontSettings() {
    const settings = createSettingsController();
    onMount(() => {
        void settings.load();
    });

    return (
        <div class="flex flex-col gap-4">
            <h1 class="text-xl font-medium">字体设置</h1>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />

            <Show when={settings.error()}>
                <p class="text-sm text-red-500">
                    设置未保存（已保留原值）：{settings.error()}
                </p>
            </Show>

            <Show
                when={!settings.loading()}
                fallback={<p class="text-gray-500">正在读取设置…</p>}
            >
                <div class="flex flex-col items-start gap-6 md:gap-4">
                    {/* 共用 / 分开字体：放在主副字体设置上方 */}
                    <FontModeToggle />
                    <Font
                        update={settings.update}
                        disabled={settings.saving()}
                    />
                </div>
            </Show>

            <Show when={settings.saving()}>
                <p class="text-sm text-gray-500">正在保存…</p>
            </Show>

            {/* 字体上传：页面底部，上传后可在上方字体下拉里选用 */}
            <FontUpload />
        </div>
    );
}

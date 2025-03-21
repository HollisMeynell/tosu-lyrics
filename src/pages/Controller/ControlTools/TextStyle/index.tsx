// 功能: 面板-文字样式（颜色、字体、显示效果）
import store from "@/stores/indexStore";
import { ToggleSwitch, ToggleNSwitch } from "@/components/ui";
import TextColor from "./TextColor";
import Font from "./Font";
import { alignmentOptions } from "@/types/globalTypes";

export default function TextStyle() {
    return (
        <div class="flex flex-col gap-4">
            <h1 class="text-2xl font-medium">文字样式</h1>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />
            <div class="flex flex-col items-start gap-8 md:gap-4">
                {/* 对齐方式 */}
                <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-6">
                    <h2 class="text-2xl font-normal">对齐方式</h2>
                    <ToggleNSwitch
                        options={alignmentOptions}
                        class="min-w-[15rem]"
                        selectedValue={store.getState.settings.alignment}
                        onUpdateSelectedValue={(value) =>
                            store.setAlignment(value)
                        }
                    />
                </div>
                {/* 字体颜色 */}
                <TextColor />
                {/* 字体 */}
                <Font />
                {/* 显示 */}
                <div class="flex flex-row items-center gap-3">
                    <ToggleSwitch
                        modelValue={
                            store.getState.settings.useTranslationAsMain
                        }
                        onUpdateModelValue={(value) =>
                            store.setUseTranslationAsMain(value)
                        }
                    />
                    <h2 class="text-2xl font-normal">以翻译歌词为主</h2>
                </div>
                <div class="flex flex-row items-center gap-3">
                    <ToggleSwitch
                        modelValue={store.getState.settings.showSecond}
                        onUpdateModelValue={(value) =>
                            store.setShowSecond(value)
                        }
                    />
                    <h2 class="text-2xl font-normal">显示副歌词</h2>
                </div>
            </div>
        </div>
    );
}

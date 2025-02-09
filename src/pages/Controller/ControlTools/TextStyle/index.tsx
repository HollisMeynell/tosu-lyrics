// 功能: 面板-文字样式（颜色、字体、显示效果）
import lyricsStore from "@/stores/lyricsStore.ts";
import ToggleSwitch from "@/components/ui/ToggleSwitch.tsx";
import ToggleNSwitch from "@/components/ui/ToggleNSwitch.tsx";
import TextColor from "./TextColor";
import Font from "./Font";
import { alignmentOptions } from "@/types/globalTypes";

export default function Controller() {
    return (
        <div class="flex flex-col gap-6">
            {/* 对齐方式 */}
            <div class="flex flex-row items-center gap-6">
                <h2 class="text-2xl font-bold">对齐方式</h2>
                <ToggleNSwitch
                    options={alignmentOptions}
                    selectedValue={lyricsStore.getState.alignment}
                    onUpdateSelectedValue={(value) =>
                        lyricsStore.setAlignment(value)
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
                    modelValue={lyricsStore.getState.useTranslationAsMain}
                    onUpdateModelValue={(value) =>
                        lyricsStore.setUseTranslationAsMain(value)
                    }
                />
                <h2 class="text-2xl font-bold">以翻译歌词为主</h2>
            </div>
            <div class="flex flex-row items-center gap-3">
                <ToggleSwitch
                    modelValue={lyricsStore.getState.showSecond}
                    onUpdateModelValue={(value) =>
                        lyricsStore.setShowSecond(value)
                    }
                />
                <h2 class="text-2xl font-bold">显示副歌词</h2>
            </div>
        </div>
    );
}

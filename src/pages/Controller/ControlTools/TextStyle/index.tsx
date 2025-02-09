// 功能: 面板-文字样式（颜色、字体、显示效果）
import lyricsStore from "@/stores/lyricsStore.ts";
import ToggleSwitch from "@/components/ui/ToggleSwitch.tsx";
import ToggleNSwitch from "@/components/ui/ToggleNSwitch.tsx";
import { alignmentOptions } from "@/types/globalTypes";

export default function Controller() {
    return (
        <div class="flex flex-col gap-6">
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
            <div class="flex flex-row gap-6 items-center">
                <h2 class="text-2xl font-bold">文字颜色</h2>
                <div class="flex flex-row items-center">
                    <p>主歌词:</p>
                    <input
                        type="color"
                        class="p-1 h-10 w-14 block bg-white border border-gray-200
                        cursor-pointer rounded-lg disabled:opacity-50 m-2
                        disabled:pointer-events-none
                        dark:bg-neutral-900 dark:border-neutral-700"
                        value={lyricsStore.getState.textColor.first}
                        onInput={(e) =>
                            lyricsStore.setTextColor(
                                "first",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => lyricsStore.sendColorConfig()}
                    />
                    <p>副歌词:</p>
                    <input
                        type="color"
                        class="p-1 h-10 w-14 block bg-white border border-gray-200
                        cursor-pointer rounded-lg disabled:opacity-50 m-2
                        disabled:pointer-events-none
                        dark:bg-neutral-900 dark:border-neutral-700"
                        value={lyricsStore.getState.textColor.second}
                        onInput={(e) =>
                            lyricsStore.setTextColor(
                                "second",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => lyricsStore.sendColorConfig()}
                    />
                </div>
            </div>
            <div class="flex flex-row items-start gap-3 hidden">
                <h2 class="text-2xl font-bold">字体</h2>
                <select class="w-48 h-8 border border-gray-300 rounded-md">
                    <option>默认字体</option>
                </select>
            </div>
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

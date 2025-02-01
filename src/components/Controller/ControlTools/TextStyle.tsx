// 功能: 面板-文字样式（颜色、字体、显示效果）
import lyricsStore from "@/stores/lyricsStore.ts";
import ToggleSwitch from "@/components/ui/ToggleSwitch.tsx";

const changeTextColor = (order: "first" | "second", color: string) => {
    lyricsStore.setTextColor(order, color);
};

export default function Controller() {
    return (
        <div class="flex flex-col gap-6">
            <div class="flex flex-row items-start gap-6">
                <h2 class="text-2xl font-bold">文字颜色</h2>
                <div class="flex flex-row items-center">
                    <p>主歌词:</p>
                    <input
                        type="color"
                        class="mx-4 w-12 h-8 rounded-full cursor-pointer"
                        value={lyricsStore.state.textColor.first}
                        onInput={(e) => changeTextColor("first", e.currentTarget.value)}
                    />
                    <p>副歌词:</p>
                    <input
                        type="color"
                        class="ml-4 w-12 h-8 rounded-full cursor-pointer"
                        value={lyricsStore.state.textColor.second}
                        onInput={(e) => changeTextColor("second", e.currentTarget.value)}
                    />
                </div>
            </div>
            <div class="flex flex-row items-start gap-3">
                <h2 class="text-2xl font-bold">字体</h2>
                <select class="w-48 h-8 border border-gray-300 rounded-md">
                    <option>默认字体</option>
                </select>
            </div>
            <div class="flex flex-row items-center gap-3">
            <ToggleSwitch
                    modelValue={lyricsStore.state.useTranslationAsMain}
                    onUpdateModelValue={lyricsStore.setUseTranslationAsMain}
                />
                <h2 class="text-2xl font-bold">以翻译歌词为主</h2>
            </div>
            <div class="flex flex-row items-center gap-3">
                <ToggleSwitch
                    modelValue={lyricsStore.state.showSecond}
                    onUpdateModelValue={lyricsStore.setShowSecond}
                />
                <h2 class="text-2xl font-bold"
                style={{ color: lyricsStore.state.useTranslationAsMain ? "#a0a0a0" : "#ffffff" }}
                >显示副歌词</h2>
            </div>
        </div>
    );
}

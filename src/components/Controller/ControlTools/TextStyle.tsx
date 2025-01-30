// 功能: 面板-文字样式（颜色、字体、显示效果）
import lyricsStore from "@/stores/lyricsStore.ts";

const changeTextColor = (order: "first" | "second", color: string) => {
    lyricsStore.setTextColor(order, color);
};

export default function Controller() {
    return (
        <div class="flex flex-row items-start gap-6">
            <h2 class="text-2xl font-bold">文字颜色</h2>
            <div class="flex flex-row items-center">
                <p>翻译歌词:</p>
                <input
                    type="color"
                    class="mx-4 w-12 h-8 rounded-full cursor-pointer"
                    value={lyricsStore.getState().textColor.first}
                    onInput={(e) => changeTextColor("first", e.currentTarget.value)}
                />
                <p>原文:</p>
                <input
                    type="color"
                    class="ml-4 w-12 h-8 rounded-full cursor-pointer"
                    value={lyricsStore.getState().textColor.second}
                    onInput={(e) => changeTextColor("second", e.currentTarget.value)}
                />
            </div>
        </div>
    );
}

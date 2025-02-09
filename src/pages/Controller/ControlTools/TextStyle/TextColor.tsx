// 功能: 面板-文字样式（颜色、字体、显示效果）
import lyricsStore from "@/stores/lyricsStore.ts";

export default function TextColor() {
    return (
        <div class="flex flex-row items-center gap-9">
            <h2 class="text-2xl font-bold">文字颜色</h2>
            <div class="flex flex-row items-center">
                <p>主歌词:</p>
                <div class="mx-4 w-6 h-6 relative">
                    <input
                        type="color"
                        class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10"
                        value={lyricsStore.getState.textColor.first}
                        onInput={(e) =>
                            lyricsStore.setTextColor(
                                "first",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => lyricsStore.sendColorConfig()}
                    />
                    <div
                        class="color-mask absolute top-0 left-0 w-full h-full rounded-full z-0 pointer-events-none"
                        style={{
                            "background-color":
                                lyricsStore.getState.textColor.first,
                        }}
                    ></div>
                </div>
                <p>副歌词:</p>
                <div class="mx-4 w-6 h-6 relative">
                    <input
                        type="color"
                        class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10"
                        value={lyricsStore.getState.textColor.second}
                        onInput={(e) =>
                            lyricsStore.setTextColor(
                                "second",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => lyricsStore.sendColorConfig()}
                    />
                    <div
                        class="color-mask absolute top-0 left-0 w-full h-full rounded-full z-0 pointer-events-none"
                        style={{
                            "background-color":
                                lyricsStore.getState.textColor.second,
                        }}
                    ></div>
                </div>
            </div>
        </div>
    );
}

// 功能: 面板-显示当前播放歌曲的歌词
import { createSignal, For } from "solid-js";
import { createEffect } from "solid-js";
import { darkMode } from "@/stores/lyricsStore.ts";
import ToggleList from "@/components/ui/ToggleList.tsx";
import Copy from "@/assets/Icons/Copy.tsx";
import { wsService } from "@/services/webSocketService";
import { LyricRawLine } from "@/types/globalTypes";

export default function Controller() {
    const [lyrics, setLyrics] = createSignal<LyricRawLine[]>([]);
    const searchCacheCurrent = async () => {
        const data = await wsService.defaultClient?.queryNowLyrics();
        if (data) {
            setLyrics(data);
        } else {
            setLyrics([]);
        }
    };

    createEffect(() => {
        // 等待渲染完成后加载
        void searchCacheCurrent();
    });

    const handleRefresh = (e: MouseEvent) => {
        e.stopPropagation(); // 阻止事件冒泡
        void searchCacheCurrent();
    };

    const HeaderContent = (
        <div class="flex flex-row items-center">
            <h2 class="text-2xl font-bold">歌词</h2>
            <button
                class="bg-[#ec4899] text-white border-none px-5 py-1 ml-4 rounded-md cursor-pointer text-base font-bold shadow-sm hover:bg-[#db2777] transition-colors duration-300"
                onClick={handleRefresh}
            >
                刷新
            </button>
        </div>
    );

    const LyricsContent = (
        <div class="h-[calc(100%-45px)] border-2 border-[#f0f0f0] dark:border-[#313131] rounded-lg px-4 max-w-[650px] overflow-auto scrollbar-hide">
            <For each={lyrics()}>
                {(item) => (
                    <div class="flex flex-row items-center my-3 gap-3 max-w-[700px]">
                        <div class="grow flex flex-row justify-between items-center select-none">
                            <p class="text-xl">{item.first}</p>
                            <p class="text-sm text-right text-gray-500">
                                {item.second}
                            </p>
                        </div>
                        <div class="flex justify-end">
                            <button
                                class="h-7 w-18 flex items-center gap-1 bg-white dark:bg-[#21314d] text-gray-500 dark:text-gray-200 border-none px-3 py-1 rounded cursor-pointer text-sm font-bold shadow-md hover:bg-[#f0f0f0] dark:hover:bg-[#3b4a63] transition-colors duration-300"
                                onClick={() => {
                                    void navigator.clipboard.writeText(
                                        `${item.first}\n${item.second}`
                                    );
                                }}
                            >
                                <Copy
                                    stroke={darkMode() ? "#dcdcdc" : "#313131"}
                                    class="w-4 h-4"
                                />
                                复制
                            </button>
                        </div>
                    </div>
                )}
            </For>
        </div>
    );

    return (
        <div>
            <ToggleList header={HeaderContent}>{LyricsContent}</ToggleList>
        </div>
    );
}

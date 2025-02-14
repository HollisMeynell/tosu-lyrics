// 功能: 面板-显示当前播放歌曲的歌词
import { For, Show, Component, Accessor } from "solid-js";
import { createEffect } from "solid-js";
import { darkMode } from "@/stores/settingsStore";
import { ToggleList } from "@/components/ui";
import { Copy } from "@/assets/Icons";
import { wsService } from "@/services/webSocketService";
import { LyricRawLine } from "@/types/lyricTypes.ts";

const CurrentLyrics: Component<{
    lyrics: Accessor<LyricRawLine[]>;
    setLyrics: (lyrics: LyricRawLine[]) => void;
}> = ({ lyrics, setLyrics }) => {
    createEffect(() => {
        void handleRefresh();
    });

    const handleRefresh = async () => {
        const data = await wsService.defaultClient?.queryNowLyrics();
        if (!data) {
            setLyrics([]);
            return;
        }
        setLyrics(data);
    };

    const LyricsHeader = (
        <div class="flex flex-row items-center">
            <h2 class="text-2xl font-normal">当前歌词</h2>
            <button
                class="bg-[#ec4899] text-white border-none px-5 py-1 ml-4 rounded-md
                 cursor-pointer text-base font-bold shadow-sm hover:bg-[#db2777]
                 transition-colors duration-300"
                onClick={handleRefresh}
            >
                刷新
            </button>
            <Show when={lyrics().length === 0}>
                <p class="text-sm text-gray-500 ml-4">暂无歌词</p>
            </Show>
        </div>
    );

    const LyricsItem = (item: LyricRawLine) => (
        <div class="flex flex-row items-center my-3 gap-3 max-w-[700px]">
            <div class="grow flex flex-row justify-between items-center select-none">
                <p class="text-xl">{item.first}</p>
                <p class="text-sm text-right text-gray-500">{item.second}</p>
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
    );

    const LyricsContent = (
        <div class="h-[calc(100%-45px)] border-2 border-[#f0f0f0] dark:border-[#313131] rounded-lg px-4 max-w-[650px] overflow-auto scrollbar-hide">
            <For each={lyrics()}>{LyricsItem}</For>
        </div>
    );

    return <ToggleList header={LyricsHeader}>{LyricsContent}</ToggleList>;
};

export default CurrentLyrics;

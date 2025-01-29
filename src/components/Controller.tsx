// 功能: 功能操作面板
import { LyricLine } from "@/common/music-api.ts";
import Cache from "@/utils/cache.ts";
import { For } from "solid-js";
import { createEffect, createSignal } from "solid-js";
import CopySvg from "@/assets/svg/copy.svg";

export default function Controller() {
    const [currentLyrics, setCurrentLyrics] = createSignal<LyricLine[] | undefined>(
        undefined
    );

    const searchCacheCurrent = async () => {
        const nowPlayingBid = Number(localStorage.getItem("nowPlaying"));
        if (nowPlayingBid) {
            const lyrics = await Cache.getLyricsCache(nowPlayingBid);
            setCurrentLyrics(lyrics); // 更新 currentLyrics
        }
    };

    createEffect(() => {
        searchCacheCurrent();
    });

    return (
        <div class="h-[calc(100%-300px)] bg-white/80 m-6 p-6 overflow-auto rounded-lg shadow-md scrollbar-hide">
            <div class="flex flex-row items-center">
                <h2 class="text-2xl font-bold">歌词</h2>
                <button
                    class="bg-[#ff6f61] text-white border-none px-5 py-2.5 ml-4 rounded cursor-pointer text-base font-bold shadow-sm hover:bg-[#ff6f61] transition-colors duration-300"
                    onClick={searchCacheCurrent}
                >
                    查看当前播放
                </button>
            </div>
            <div class="border border-gray-200 rounded-lg p-4 mt-4 max-w-[650px]">
                <For each={currentLyrics() || []}>
                    {(item) => (
                        <div class="flex flex-row justify-between items-center my-3 gap-2 max-w-[700px]">
                            <div class="grow flex flex-row justify-between items-center select-none">
                                <p class="text-xl">{item.first}</p>
                                <p class="text-sm text-right">{item.second}</p>
                            </div>
                            <div class="flex justify-end w-[100px]">
                                <button
                                    class="h-7 w-18 flex items-center gap-1 bg-gray-100 text-gray-500 border-none px-3 py-1 rounded cursor-pointer text-sm font-bold shadow-sm hover:bg-gray-200 transition-colors duration-300"
                                    onClick={() => {
                                        navigator.clipboard.writeText(item.first);
                                    }}
                                >
                                    <img src={CopySvg} class="w-4 h-4" />
                                    复制
                                </button>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
}

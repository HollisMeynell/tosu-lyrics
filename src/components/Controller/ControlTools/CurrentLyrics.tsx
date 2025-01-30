// 功能: 面板-显示当前播放歌曲的歌词
import Cache from "@/utils/cache.ts";
import { For } from "solid-js";
import { createEffect } from "solid-js";
import lyricsStore from "@/stores/lyricsStore.ts";
import CopySvg from "@/assets/svg/copy.svg";

export default function Controller() {

    const searchCacheCurrent = async () => {
        const nowPlayingBid = Number(localStorage.getItem("nowPlaying"));
        if (nowPlayingBid) {
            const lyrics = await Cache.getLyricsCache(nowPlayingBid);
            lyricsStore.updateCurrentLyrics(lyrics); // 更新 currentLyrics
        }
    };

    createEffect(() => {
        searchCacheCurrent();
    });

    return (
        <div>
            <div class="flex flex-row items-center">
                <h2 class="text-2xl font-bold">歌词</h2>
                <button
                    class="bg-[#ff6f61] text-white border-none px-5 py-1 ml-4 rounded cursor-pointer text-base font-bold shadow-sm hover:bg-[#ff6f61] transition-colors duration-300"
                    onClick={searchCacheCurrent}
                >
                    刷新
                </button>
            </div>
            <div class="h-[calc(100%-45px)] border border-[#313131] rounded-lg px-4 py-2 mt-4 max-w-[650px] overflow-auto scrollbar-hide">
                <For each={lyricsStore.currentLyrics() || []}>
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
                                    class="h-7 w-18 flex items-center gap-1 bg-[#1d1d1d] text-gray-500 border-none px-3 py-1 rounded cursor-pointer text-sm font-bold shadow-sm hover:bg-[#313131] transition-colors duration-300"
                                    onClick={() => {
                                        navigator.clipboard.writeText(
                                            `${item.first}\n${item.second}`
                                        );
                                    }}
                                >
                                    <img src={CopySvg} class="w-3 h-3" />
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

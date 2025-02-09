// 功能: 面板-显示当前播放歌曲的歌词
import { Component, createSignal, For } from "solid-js";
import { createEffect } from "solid-js";
import { darkMode } from "@/stores/lyricsStore.ts";
import ToggleList from "@/components/ui/ToggleList.tsx";
import { Copy } from "@/assets/Icons";
import { wsService } from "@/services/webSocketService";

import {
    LyricRawLine,
    MusicInfo,
    MusicQueryInfoData,
} from "@/types/lyricTypes.ts";

export default function CurrentLyrics() {
    const [lyrics, setLyrics] = createSignal<LyricRawLine[]>([]);
    const [musicInfo, setMusicInfo] = createSignal<MusicQueryInfoData>({});

    const searchCacheCurrent = async () => {
        const data = await wsService.defaultClient?.queryNowLyrics();
        console.log(data);
        if (data) {
            setLyrics(data);
        } else {
            setLyrics([]);
        }
    };

    const searchMusicInfo = async () => {
        const result = await wsService.defaultClient?.queryNowMusicInfo();
        if (!result) {
            return;
        }
        const { title, data } = result;
        // 当前歌曲名
        console.log(title);
        // 所有搜索结果
        console.log(data);
        setMusicInfo(data);
    };

    const getLyricsByKey = async (adapter: string, key: string | number) => {
        const result = await wsService.defaultClient?.queryLyricsByKey(
            adapter,
            key
        );
        console.log(result);
    };

    // 发送换源命令
    const changeOrigin = (adapter: string, key: string | number) => {
        wsService.defaultClient?.changeLyric(adapter, key);
    };

    createEffect(() => {
        // 等待渲染完成后加载
        void searchCacheCurrent();
    });

    const handleRefresh = (e: MouseEvent) => {
        e.stopPropagation(); // 阻止事件冒泡
        void searchCacheCurrent();
    };

    const LyricsHeader = (
        <div class="flex flex-row items-center">
            <h2 class="text-2xl font-bold">当前歌词</h2>
            <button
                class="bg-[#ec4899] text-white border-none px-5 py-1 ml-4 rounded-md
                 cursor-pointer text-base font-bold shadow-sm hover:bg-[#db2777]
                 transition-colors duration-300"
                onClick={handleRefresh}
            >
                刷新
            </button>
        </div>
    );

    const OriginHeader = (
        <div class="flex flex-row items-center">
            <h2 class="text-2xl font-bold">当前歌曲搜索结果</h2>
            <button
                class="bg-[#ec4899] text-white border-none px-5 py-1 ml-4 rounded-md
                 cursor-pointer text-base font-bold shadow-sm hover:bg-[#db2777]
                 transition-colors duration-300"
                onClick={searchMusicInfo}
            >
                搜索
            </button>
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

    const OriginItemList: Component<{ key: string }> = (props) => {
        const items = () => musicInfo().data[props.key] as MusicInfo[];
        return (
            <div>
                <h3>{props.key}</h3>
                <For each={items()}>
                    {(item: MusicInfo) => (
                        <div
                            onClick={() => getLyricsByKey(props.key, item.key)}
                        >
                            <button
                                onClick={() =>
                                    getLyricsByKey(props.key, item.key)
                                }
                            >
                                查看歌词
                            </button>
                            <button
                                onClick={() =>
                                    changeOrigin(props.key, item.key)
                                }
                            >
                                应用歌词
                            </button>
                            <p>{item.title}</p>
                            <p>{item.artist}</p>
                            <p>{item.length}</p>
                        </div>
                    )}
                </For>
            </div>
        );
    };
    const OriginItem: Component = () => {
        const keys = Object.keys(musicInfo());
        return (
            <div>
                <For each={keys}>{(key) => <OriginItemList key={key} />}</For>
            </div>
        );
    };

    const LyricsContent = (
        <div class="h-[calc(100%-45px)] border-2 border-[#f0f0f0] dark:border-[#313131] rounded-lg px-4 max-w-[650px] overflow-auto scrollbar-hide">
            <For each={lyrics()}>{LyricsItem}</For>
        </div>
    );

    return (
        <div>
            <ToggleList header={OriginHeader}>
                <OriginItem />
            </ToggleList>
            <ToggleList header={LyricsHeader}>{LyricsContent}</ToggleList>
        </div>
    );
}

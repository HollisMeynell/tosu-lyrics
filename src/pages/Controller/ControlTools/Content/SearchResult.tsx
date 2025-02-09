// 功能: 面板-显示当前歌曲搜索结果
import { Component, For, Show } from "solid-js";
import ToggleList from "@/components/ui/ToggleList.tsx";
import { wsService } from "@/services/webSocketService";
import { MusicInfo, MusicQueryInfoData } from "@/types/lyricTypes.ts";

const SearchResult: Component<{
    musicInfo: () => MusicQueryInfoData;
    setMusicInfo: (info: MusicQueryInfoData) => void;
    setMusicTitle: (title: string) => void;
}> = ({ musicInfo, setMusicInfo, setMusicTitle }) => {
    const searchMusicInfo = async () => {
        const result = await wsService.defaultClient?.queryNowMusicInfo();
        if (!result) {
            setMusicInfo({});
            return;
        }
        const { title, data } = result; // 当前播放歌曲名, 以及搜索结果
        setMusicInfo(data);
        setMusicTitle(title);
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

    const OriginHeader = (
        <div class="flex flex-row items-center">
            <h2 class="text-2xl font-normal">当前歌曲搜索结果</h2>
            <button
                class="bg-[#ec4899] text-white border-none px-5 py-1 ml-4 rounded-md
                 cursor-pointer text-base font-bold shadow-sm hover:bg-[#db2777]
                 transition-colors duration-300"
                onClick={searchMusicInfo}
            >
                搜索
            </button>
            <Show when={Object.keys(musicInfo()).length === 0}>
                <p class="text-sm text-gray-500 ml-4">暂无搜索结果</p>
            </Show>
        </div>
    );

    const OriginItemList: Component<{ key: string }> = (props) => {
        const items = () => musicInfo()[props.key];
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

    return (
        <div>
            <ToggleList header={OriginHeader}>
                <OriginItem />
            </ToggleList>
        </div>
    );
};

export default SearchResult;

// 功能: 面板-显示当前歌曲搜索结果
import { Component, For, Show, createSignal } from "solid-js";
import { wsService } from "@/services/webSocketService";
import { MusicInfo, MusicQueryInfoData } from "@/types/lyricTypes.ts";
import { ToggleListExtends, Button, DragPanel } from "@/components/ui";
import { parseUnifiecLyric } from "@/adapters/lyricAdapter.ts";
import { Lyric } from "@/services/managers/lyricManager.ts";

const SearchResult: Component<{
    musicInfo: () => MusicQueryInfoData;
    setMusicInfo: (info: MusicQueryInfoData) => void;
    setMusicTitle: (title: string) => void;
}> = ({ musicInfo, setMusicInfo, setMusicTitle }) => {
    // 这里的 title 与 bid 必须要传
    // title 是添加到歌词第一行
    let nowTitle = "";
    // bid 用于判断是否被切歌, 防止切歌后歌词错乱
    let nowBid = 0;

    const [nowLyric, setNowLyric] = createSignal<Lyric>();

    const searchMusicInfo = async () => {
        const result = await wsService.defaultClient?.queryNowMusicInfo();
        if (!result) {
            setMusicInfo({});
            return;
        }
        const { bid, title, data } = result; // 当前播放歌曲名, 以及搜索结果
        nowTitle = title;
        nowBid = bid;
        setMusicInfo(data);
        setMusicTitle(title);
        loadPages();
    };

    const getLyricsByKey = async (adapter: string, key: string | number) => {
        const result = await wsService.defaultClient?.queryLyricsByKey(
            adapter,
            key
        );

        if (!result) return;
        const lyric = parseUnifiecLyric(nowTitle, result);
        // todo: 显示歌词
        console.log(lyric);
        setNowLyric(lyric);
    };

    // 发送换源命令
    const changeSearchResult = () => {
        const lyric = nowLyric();
        if (!lyric) {
            alert("请先查看歌词");
            return;
        }
        wsService.defaultClient?.changeLyric(nowBid, lyric);
    };

    const SearchResultHeader = () => (
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

    const SearchResultItemList: Component<{ key: string }> = (props) => {
        const items = () => musicInfo()[props.key];
        return (
            <div>
                <For each={items()}>
                    {(item: MusicInfo) => (
                        <div
                            class="w-full flex flex-row items-between justify-between p-2
                            border-b border-gray-200 dark:border-gray-700"
                        >
                            <div class="flex flex-row w-[calc(100%-12rem)] items-center gap-2">
                                <DragPanel
                                    items={[item.title, item.artist]}
                                    class="w-[calc(100%-7rem)]"
                                />
                                <p class="w-28 overflow-hidden text-ellipsis text-nowrap">
                                    长度：{item.length}
                                </p>
                            </div>
                            <div class="flex flex-row gap-2 w-44">
                                <Button
                                    onClick={() =>
                                        getLyricsByKey(props.key, item.key)
                                    }
                                >
                                    查看歌词
                                </Button>
                                <Button onClick={changeSearchResult}>
                                    应用歌词
                                </Button>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        );
    };

    const [pages, setPages] = createSignal([
        {
            name: "未知源",
            content: <div>Loading...</div>,
        },
    ]);

    const loadPages = () => {
        const keys = Object.keys(musicInfo());
        const loadedPages = keys.map((key) => ({
            name: key || "未知",
            content: <SearchResultItemList key={key} />,
        }));
        setPages(loadedPages);
    };

    return (
        <>
            <ToggleListExtends header={SearchResultHeader()} pages={pages()} />
        </>
    );
};

export default SearchResult;

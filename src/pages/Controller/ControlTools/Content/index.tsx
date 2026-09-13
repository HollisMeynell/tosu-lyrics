import { onMount, Show } from "solid-js";
import CurrentLyrics from "./CurrentLyrics";
import SearchResult from "./SearchResult";
import { createLyricsContentController } from "@/hooks/useLyricsContent";

/**
 * 歌词内容控制（B-05）。
 *
 * 所有数据来自 `/api/lyrics/*`，不再通过旧 WS 向对端查询。
 */
export default function Content() {
    const controller = createLyricsContentController();

    onMount(() => {
        controller.loadCurrent();
        controller.loadCandidates();
    });

    return (
        <div class="flex flex-col gap-4 h-full">
            <div class="header space-x-4">
                <h2 class="text-2xl font-medium inline">歌词内容控制</h2>
                <Show when={controller.current()}>
                    {(cur) => (
                        <p class="text-sm inline text-gray-500">
                            当前歌曲: {cur().song.title} — {cur().song.artist}
                            （bid {cur().song.bid} / sid {cur().song.sid}）
                        </p>
                    )}
                </Show>
                <Show when={controller.noSong()}>
                    <p class="text-sm inline text-gray-500">
                        当前没有播放中的歌曲
                    </p>
                </Show>
            </div>

            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <Show when={controller.error()}>
                <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                    操作失败：{controller.error()}
                </div>
            </Show>
            <Show when={controller.notice()}>
                <div class="px-4 py-2 rounded-md bg-blue-100 dark:bg-blue-900/40 text-blue-800 dark:text-blue-200 text-sm w-fit">
                    {controller.notice()}
                </div>
            </Show>

            <div class="flex flex-1 flex-col gap-2">
                <CurrentLyrics controller={controller} />
                <SearchResult controller={controller} />
            </div>
        </div>
    );
}

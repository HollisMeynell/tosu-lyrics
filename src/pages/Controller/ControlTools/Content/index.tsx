import { createEffect, on, onMount, Show } from "solid-js";
import CurrentLyrics from "./CurrentLyrics";
import SearchResult from "./SearchResult";
import { createLyricsContentController } from "@/hooks/useLyricsContent";
import { lyrics, lyricLoading } from "@/stores/lyricStore";

export default function Content() {
    const controller = createLyricsContentController();

    onMount(async () => {
        // 先取当前歌曲身份，再加载候选：这样 loadCandidates() 能直接拼出缓存 key，
        // 不必自己再补拉一次 /api/lyrics/current —— 那会把"开始搜索"的时机推后。
        await controller.loadCurrent();
        void controller.loadStoredCandidates();
    });

    // 歌词列表变化（换歌 / 重新加载）后刷新当前歌词数据，预览区随之更新。
    // 复用页面已有的加载函数，不新增接口、也不轮询。
    createEffect(
        on(
            () => lyrics(),
            () => {
                void controller.loadCurrent();
            },
            { defer: true }
        )
    );

    // `current()` 更新后只负责结束"搜索结果预览"，避免把上一首的预览留在新歌上。
    // 候选列表已不再依赖 sid / current()：完全由 `lyricLoading` 驱动 ——
    //   true  → 后端开始处理新歌，立即清空旧候选（不再闪现上一首候选）；
    //   false → 后端这一轮搜索结束，读取 GET /api/lyrics/search-results 并显示。
    // 主歌词是否匹配成功不影响候选显示（匹配失败同样会走到 false）。
    createEffect(
        on(
            () => controller.current()?.song?.sid,
            () => {
                controller.clearPreview();
            },
            { defer: true }
        )
    );

    // 候选列表完全跟随后端的候选状态，与主歌词是否匹配成功无关：
    // - $name 由 true 表示「后端已开始处理新歌」（begin_song 广播）→ 立刻清空上一首候选，消除闪现；
    // - 由 true 变 false 表示「后端这一轮搜索已结束」（load_plan 的所有出口都会广播）
    //   → 此时 music_cache 已是最终结果，直接读取显示。匹配失败也会走这里，因此候选不会丢。
    createEffect(
        on(
            () => lyricLoading(),
            (loading) => {
                if (loading) {
                    controller.startCandidateRead();
                    return;
                }
                // 只做一次兜底读取；**不**停止轮询（轮询改由"读到非空 / 切歌换代 / 安全上限"终止）
                void controller.loadStoredCandidates();
            },
            { defer: true }
        )
    );
    return (
        <div class="flex flex-col gap-4 h-full">
            <div class="header space-x-4">
                <h2 class="text-xl font-medium inline">歌词内容控制</h2>
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

            <div class="flex flex-1 flex-col gap-4 xl:flex-row xl:items-start">
                <div class="flex flex-1 min-w-0 flex-col">
                    <CurrentLyrics controller={controller} />
                </div>
                <div class="flex flex-1 min-w-0 flex-col">
                    <SearchResult controller={controller} />
                </div>
            </div>
        </div>
    );
}

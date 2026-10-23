import { createEffect, on, onMount, Show } from "solid-js";
import CurrentLyrics from "./CurrentLyrics";
import LrcUpload from "./LrcUpload";
import { createLyricsContentController } from "@/hooks/useLyricsContent";
import { lyrics } from "@/stores/lyricStore";

export default function Content() {
    const controller = createLyricsContentController();

    onMount(async () => {
        await controller.loadCurrent();
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
    createEffect(
        on(
            () => controller.current()?.song?.sid,
            () => {
                controller.clearPreview();
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

            <CurrentLyrics controller={controller} />

            {/* 歌词上传：页面底部，上传 .lrc 覆盖当前歌曲的歌词 */}
            <LrcUpload />
        </div>
    );
}

import { Component, createSignal, onMount, Show } from "solid-js";
import BlacklistComponent from "./BlackList";
import { Button } from "@/components/ui";
import { Refresh } from "@/assets/Icons";
import { createBlocksController, defaultBlockInput } from "@/hooks/useBlocks";
import { BlockScope } from "@/services/blocksService";
import { Select } from "@/components/ui";
import { CurrentSong, fetchStatus } from "@/services/statusService";
import { ApiError } from "@/services/settingsService";

const BlackListLyrics: Component = () => {
    const controller = createBlocksController();

    const [song, setSong] = createSignal<CurrentSong | null>(null);
    const [statusError, setStatusError] = createSignal<string | null>(null);
    const [statusLoading, setStatusLoading] = createSignal(false);
    const [notice, setNotice] = createSignal<string | null>(null);
    const [scope, setScope] = createSignal<BlockScope>("sid");

    const refresh = async () => {
        setStatusLoading(true);
        setStatusError(null);
        try {
            const status = await fetchStatus();
            setSong(status.song);
        } catch (err) {
            setSong(null);
            setStatusError(err instanceof ApiError ? err.message : String(err));
        } finally {
            setStatusLoading(false);
        }
    };

    // 现查一次服务端状态再拉黑，不依赖页面缓存（用户可能已切歌）
    const addTitleToBlackList = async () => {
        setNotice(null);
        setStatusError(null);
        setStatusLoading(true);
        let current: CurrentSong | null = null;
        try {
            const status = await fetchStatus();
            current = status.song;
            setSong(current);
        } catch (err) {
            setStatusError(err instanceof ApiError ? err.message : String(err));
            setStatusLoading(false);
            return;
        }
        setStatusLoading(false);

        if (!current) {
            setStatusError("当前没有播放中的歌曲，无法添加到黑名单");
            return;
        }
        const ok = await controller.add({
            ...defaultBlockInput(current),
            scope: scope(),
            value:
                scope() === "title" ? current.title : String(
                    scope() === "sid" ? current.sid : current.bid
                ),
        });
        if (ok) setNotice(`已屏蔽：${current.title}（作用域 ${scope()}）`);
    };

    onMount(() => {
        controller.load();
        refresh();
    });

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl inline">黑名单管理</h2>
                <p class="text-sm inline text-gray-500">
                    被你拉黑的歌曲以后将不会被显示
                </p>
            </div>
            <hr class="w-72 border-gray-400 dark:border-gray-600" />

            <div class="flex flex-row gap-4 items-center flex-wrap">
                <Button
                    class="w-48"
                    onClick={addTitleToBlackList}
                    disabled={controller.saving() || statusLoading()}
                >
                    添加到黑名单
                </Button>
                <button
                    class="text-sm inline leading-10"
                    onClick={refresh}
                    disabled={statusLoading()}
                >
                    <Refresh
                        class="w-4 h-4 inline cursor-pointer mr-2 select-none active:scale-90
                    active:rotate-270 transition-transform duration-400 ease-in-out"
                    />
                    <Show
                        when={song()}
                        fallback={
                            <span class="text-gray-500">
                                {statusLoading() ? "读取中..." : "暂无信息"}
                            </span>
                        }
                    >
                        {(current) => (
                            <span>
                                当前播放: {current().title} — {current().artist}
                                （bid {current().bid} / sid {current().sid}）
                            </span>
                        )}
                    </Show>
                </button>
            </div>

            <div class="flex flex-row items-center gap-3 text-sm">
                <span>作用域：</span>
                <Select
                    class="min-w-0 w-44 py-1"
                    clearable={false}
                    value={scope()}
                    onChange={(v) => setScope(v as BlockScope)}
                    options={[
                        { code: "sid", name: "谱面集 (sid) · 默认" },
                        { code: "bid", name: "单谱面 (bid)" },
                        { code: "title", name: "标题" },
                    ]}
                />
                <span class="text-xs text-gray-500">
                    用于本页的新增与"添加到黑名单"；已有条目的作用域不受影响
                </span>
            </div>

            <Show when={statusError()}>
                <div class="px-4 py-2 rounded-md bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200 text-sm w-fit">
                    {statusError()}
                </div>
            </Show>
            <Show when={notice()}>
                <div class="px-4 py-2 rounded-md bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-200 text-sm w-fit">
                    {notice()}
                </div>
            </Show>

            <BlacklistComponent controller={controller} scope={scope} />
        </div>
    );
};

export default BlackListLyrics;

import { Button } from "@/components/ui";
import { createEffect, createSignal, For, createMemo } from "solid-js";
import { LyricCacheIndex } from "@/utils/cache.ts";
import { wsService } from "@/services/webSocketService.ts";
import { ms2str } from "@/utils/helpers";

function CacheItem({
    item,
    onRemove,
}: {
    item: LyricCacheIndex;
    onRemove: (key: string | number) => void;
}) {
    return (
        <div class="max-w-200 p-2 bg-fuchsia-100 dark:bg-[#364153] dark:hover:bg-gray-600 transition-colors duration-200 ease-in-out rounded-lg flex flex-row items-center justify-between gap-4 overflow-hidden">
            <div class="flex flex-row items-center">
                <span
                    class="inline-flex items-center gap-x-1.5 py-1.5 px-3 mr-4
                rounded-lg text-xs font-medium bg-blue-100 text-blue-800
                dark:bg-blue-500/30 dark:text-white"
                >
                    {ms2str(item.length)}
                </span>
                <span class="max-w-130 truncate">{item.name}</span>
            </div>
            <div class="flex flex-row gap-3 items-center">
                <Button class="w-fit" onClick={() => onRemove(item.sid)}>
                    删除此歌
                </Button>
                <Button class="w-fit" onClick={() => onRemove(item.name)}>
                    删除标题匹配
                </Button>
            </div>
        </div>
    );
}

export default function () {
    const [page, setPage] = createSignal(0);
    const [cache, setCache] = createSignal<LyricCacheIndex[]>([]);
    const [loading, setLoading] = createSignal(false);

    const fetchCache = async (page: number) => {
        setLoading(true);
        try {
            const data = await wsService.defaultClient?.queryCacheList(page);
            if (data) setCache(data);
        } catch (error) {
            console.error("Failed to fetch cache:", error);
        } finally {
            setLoading(false);
        }
    };

    const requestBefore = () => {
        setPage((s) => Math.max(0, s - 1));
    };

    const requestNext = () => {
        setPage((s) => (cache().length < 50 ? s : s + 1));
    };

    const remove = (key: string | number) => {
        wsService.defaultClient
            ?.removeCacheItem(key)
            .then(() => fetchCache(page()));
    };

    const removeAll = () => {
        wsService.defaultClient
            ?.removeAllCache()
            .then(() => fetchCache(page()));
    };

    createEffect(() => {
        fetchCache(page());
    });

    const cacheItems = createMemo(() => cache());

    return (
        <div class="flex flex-col gap-4">
            <Button class="w-56" onClick={removeAll} disabled={loading()}>
                清空所有缓存
            </Button>
            <div class="text-lg font-bold">缓存列表</div>
            <div class="flex flex-row items-center gap-4">
                <Button
                    onClick={requestBefore}
                    disabled={loading() || page() === 0}
                >
                    上一页
                </Button>
                <span class="ml-6 mr-6">{page() + 1}</span>
                <Button
                    onClick={requestNext}
                    disabled={loading() || cache().length < 50}
                >
                    下一页
                </Button>
            </div>
            <div class="w-full pr-4 pb-4 h-fit overflow-auto flex flex-col gap-4">
                <For each={cacheItems()} fallback={<span>当前缓存为空</span>}>
                    {(item) => <CacheItem item={item} onRemove={remove} />}
                </For>
            </div>
        </div>
    );
}

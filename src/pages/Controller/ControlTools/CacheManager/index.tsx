import { Button } from "@/components/ui";
import { createEffect, createSignal, For, createMemo } from "solid-js";
import { LyricCacheIndex } from "@/utils/cache.ts";
import { wsService } from "@/services/webSocketService.ts";

function ms2str(time: number): string {
    const allSeconds = Math.round(time / 1000);
    const sec = String(allSeconds % 60).padStart(2, "0");
    const min = String(Math.floor(allSeconds / 60)).padStart(2, "0");
    return `${min}:${sec}`;
}

function CacheItem({
    item,
    onRemove,
}: {
    item: LyricCacheIndex;
    onRemove: (key: string | number) => void;
}) {
    return (
        <div class="w-full mt-6 p-2 bg-fuchsia-100 dark:bg-[#364153] dark:hover:bg-gray-600 transition-colors duration-200 ease-in-out rounded-lg">
            <Button class="mr-4" onClick={() => onRemove(item.sid)}>
                删除此歌
            </Button>
            <Button class="mr-4" onClick={() => onRemove(item.name)}>
                删除标题匹配
            </Button>
            <span
                class="inline-flex items-center gap-x-1.5 py-1.5 px-3 mr-4
                rounded-lg text-xs font-medium bg-blue-100 text-blue-800
                dark:bg-blue-800/30 dark:text-blue-500"
            >
                {ms2str(item.length)}
            </span>
            <span>{item.name}</span>
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
        <>
            <Button class="mb-6" onClick={removeAll} disabled={loading()}>
                清空所有缓存
            </Button>
            <br />
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
            <br />
            <For each={cacheItems()} fallback={<span>当前缓存为空</span>}>
                {(item) => <CacheItem item={item} onRemove={remove} />}
            </For>
        </>
    );
}

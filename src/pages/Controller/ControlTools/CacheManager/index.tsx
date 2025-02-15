import { Button } from "@/components/ui";
import { createEffect, createSignal, For } from "solid-js";
import { LyricCacheIndex } from "@/utils/cache.ts";
import { wsService } from "@/services/webSocketService.ts";

function ms2str(time: number): string {
    const allSeconds = Math.round(time / 1000);
    const sec = String(allSeconds % 60).padStart(2, "0");
    const min = String(Math.floor(allSeconds / 60)).padStart(2, "0");
    return `${min}:${sec}`;
}

export default function () {
    const [page, setPage] = createSignal(0);
    const [cache, setCache] = createSignal<LyricCacheIndex[]>([]);

    const requestBefore = () => {
        setPage((s) => {
            if (s <= 0) return 0;
            return s - 1;
        });
    };

    const requestNext = () => {
        setPage((s) => {
            if (cache().length < 50) return s;
            return s + 1;
        });
    };

    const remove = (ket: string | number) => {
        wsService.defaultClient?.removeCacheItem(ket).then(update);
    };

    const removeAll = () => {
        wsService.defaultClient?.removeAllCache().then(update);
    };

    const cacheItem = (item: LyricCacheIndex) => {
        return (
            <div class="w-full mt-6 p-2 bg-fuchsia-100 rounded-lg">
                <Button class="mr-4" onClick={() => remove(item.sid)}>
                    删除此歌
                </Button>
                <Button class="mr-4" onClick={() => remove(item.name)}>
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
    };

    const update = () => {
        (async () => {
            const data = await wsService.defaultClient?.queryCacheList(page());
            if (!data) return;
            setCache(data);
        })();
    };

    createEffect(async () => {
        update();
    });

    return (
        <>
            <Button class="mb-6" onClick={removeAll}>
                清空所有缓存
            </Button>
            <br />
            <Button onClick={requestBefore}>上一页</Button>
            <span class="ml-6 mr-6">{page() + 1}</span>
            <Button onClick={requestNext}>下一页</Button>
            <br />
            <For each={cache()} fallback={<span>当前缓存为空</span>}>
                {cacheItem}
            </For>
        </>
    );
}

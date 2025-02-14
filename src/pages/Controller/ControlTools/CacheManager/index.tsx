import { Button } from "@/components/ui";
import {
    Component,
    createEffect,
    createSignal,
    For,
    splitProps,
} from "solid-js";
import { LyricCacheIndex } from "@/utils/cache.ts";
import { wsService } from "@/services/webSocketService.ts";

function ms2str(time: number): string {
    const allSeconds = Math.round(time / 1000);
    const sec = String(allSeconds % 60).padStart(2, "0");
    const min = String(Math.floor(allSeconds / 60)).padStart(2, "0");
    return `${min}:${sec}`;
}

const CacheList: Component<{ page: number }> = (props) => {
    const [local] = splitProps(props, ["page"]);
    const [cache, setCache] = createSignal<LyricCacheIndex[]>([]);

    const remove = (ket: string | number) => {
        wsService.defaultClient?.removeCacheItem(ket);
        update();
    };

    const cacheItem = (item: LyricCacheIndex) => {
        return (
            <>
                <span>{item.name}</span>|<span>{ms2str(item.length)}</span>
                <Button onClick={() => remove(item.sid)}>删除此歌</Button>
                <Button onClick={() => remove(item.name)}>删除标题匹配</Button>
            </>
        );
    };

    const update = () => {
        (async () => {
            const data = await wsService.defaultClient?.queryCacheList(
                local.page
            );
            if (!data) return;
            setCache(data);
        })();
    };

    createEffect(async () => {
        update();
    });
    return <For each={cache()}>{cacheItem}</For>;
};

export default function () {
    const [page, setPage] = createSignal(0);
    return (
        <>
            <Button
                onClick={() => {
                    setPage((s) => s - 1);
                }}
            >
                上一页
            </Button>
            <Button
                onClick={() => {
                    setPage((s) => s + 1);
                }}
            >
                下一页
            </Button>
            <CacheList page={page()} />
        </>
    );
}

import { Button } from "@/components/ui";
import { createSignal, For, onMount, Show } from "solid-js";
import {
    CacheEntry,
    cleanupCache,
    clearCache,
    deleteCacheByTitle,
    deleteCacheItem,
    fetchCachePage,
} from "@/services/cacheService";
import { ApiError } from "@/services/settingsService";
import { ms2str } from "@/utils/helpers";

const PAGE_SIZE = 20;

function CacheItem({
    item,
    onRemove,
    onRemoveTitle,
    busy,
}: {
    item: CacheEntry;
    onRemove: (bid: number) => void;
    onRemoveTitle: (title: string) => void;
    busy: boolean;
}) {
    return (
        <div class="max-w-200 p-2 bg-fuchsia-100 dark:bg-[#364153] dark:hover:bg-gray-600 transition-colors duration-200 ease-in-out rounded-lg flex flex-row items-center justify-between gap-4 overflow-hidden">
            <div class="flex flex-row items-center gap-2 min-w-0">
                <span
                    class="inline-flex items-center gap-x-1.5 py-1.5 px-3 rounded-lg text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-500/30 dark:text-white shrink-0"
                >
                    {ms2str(item.audioLength)}
                </span>
                <Show when={item.expired}>
                    <span class="px-2 py-0.5 text-xs rounded bg-yellow-200 text-yellow-900 dark:bg-yellow-600/40 dark:text-yellow-100 shrink-0">
                        已过期
                    </span>
                </Show>
                <span class="max-w-130 truncate">{item.title}</span>
                <span class="text-xs text-gray-500 shrink-0">
                    bid {item.bid} / sid {item.sid} /{" "}
                    {new Date(item.updatedAt).toLocaleDateString()}
                </span>
            </div>
            <div class="flex flex-row gap-3 items-center shrink-0">
                <Button
                    class="w-fit"
                    onClick={() => onRemove(item.bid)}
                    disabled={busy}
                >
                    删除此歌
                </Button>
                <Button
                    class="w-fit"
                    onClick={() => onRemoveTitle(item.title)}
                    disabled={busy}
                >
                    删除标题匹配
                </Button>
            </div>
        </div>
    );
}

export default function CacheManager() {
    const [page, setPage] = createSignal(1);
    const [items, setItems] = createSignal<CacheEntry[]>([]);
    const [total, setTotal] = createSignal(0);
    const [pages, setPages] = createSignal(0);
    const [loading, setLoading] = createSignal(false);
    const [busy, setBusy] = createSignal(false);
    const [query, setQuery] = createSignal("");
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);

    const load = async (target = page()) => {
        setLoading(true);
        setError(null);
        try {
            const data = await fetchCachePage(target, PAGE_SIZE, query());
            setItems(data.items);
            setTotal(data.total);
            setPages(data.pages);

            // 删除最后一页最后一条后页码越界，自动回退到最后一页
            if (data.items.length === 0 && data.total > 0 && target > data.pages) {
                setPage(data.pages);
                return load(data.pages);
            }
            setPage(target);
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setLoading(false);
        }
    };

    const mutate = async (fn: () => Promise<number>, label: string) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            const removed = await fn();
            setNotice(`${label}：删除 ${removed} 条`);
            await load();
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setBusy(false);
        }
    };

    onMount(() => load(1));

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-2xl font-medium inline">歌词缓存</h2>
                <p class="text-sm inline text-gray-500">
                    缓存只是缓存 —— 删除它不会影响来源绑定、偏移或黑名单
                </p>
            </div>
            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <div class="flex flex-row items-center gap-3 flex-wrap">
                <input
                    type="text"
                    class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md dark:bg-gray-700 dark:text-white"
                    placeholder="按标题筛选"
                    value={query()}
                    onInput={(e) => setQuery(e.currentTarget.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") load(1);
                    }}
                />
                <Button class="w-fit" onClick={() => load(1)} disabled={loading()}>
                    搜索
                </Button>
                <Button
                    class="w-fit"
                    onClick={() => load(page())}
                    disabled={loading()}
                >
                    {loading() ? "刷新中..." : "刷新缓存"}
                </Button>
                <Button
                    class="w-fit"
                    onClick={() => mutate(cleanupCache, "清理过期")}
                    disabled={busy()}
                >
                    清理过期
                </Button>
                <Button
                    class="w-fit"
                    onClick={() => mutate(clearCache, "清空缓存")}
                    disabled={busy()}
                >
                    清空全部
                </Button>
                <span class="text-sm text-gray-500">
                    共 {total()} 条 / 第 {page()} 页 / 共 {pages()} 页
                </span>
            </div>

            <Show when={error()}>
                <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                    操作失败：{error()}
                </div>
            </Show>
            <Show when={notice()}>
                <div class="px-4 py-2 rounded-md bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-200 text-sm w-fit">
                    {notice()}
                </div>
            </Show>

            <div class="flex flex-col gap-2">
                <For each={items()}>
                    {(item) => (
                        <CacheItem
                            item={item}
                            busy={busy()}
                            onRemove={(bid) =>
                                mutate(() => deleteCacheItem(bid), "删除此歌")
                            }
                            onRemoveTitle={(title) =>
                                mutate(
                                    () => deleteCacheByTitle(title),
                                    "删除标题匹配"
                                )
                            }
                        />
                    )}
                </For>
            </div>

            <Show when={loading()}>
                <p class="text-sm text-gray-500">加载中...</p>
            </Show>
            <Show when={!loading() && items().length === 0}>
                <p class="text-sm text-gray-500">暂无缓存</p>
            </Show>

            <div class="flex flex-row gap-3">
                <Button
                    class="w-fit"
                    onClick={() => load(Math.max(1, page() - 1))}
                    disabled={loading() || page() <= 1}
                >
                    上一页
                </Button>
                <Button
                    class="w-fit"
                    onClick={() => load(page() + 1)}
                    disabled={loading() || page() >= pages()}
                >
                    下一页
                </Button>
            </div>
        </div>
    );
}

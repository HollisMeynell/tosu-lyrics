import { Button, Select } from "@/components/ui";
import { createSignal, For, onMount, Show } from "solid-js";
import {
    CacheEntry,
    cleanupCache,
    clearCache,
    deleteCacheByTitle,
    deleteCacheItem,
    fetchCachePage,
    setCacheTtl,
} from "@/services/cacheService";
import { ApiError } from "@/services/settingsService";
import { ms2str } from "@/utils/helpers";

const PAGE_SIZE = 20;

const TTL_OPTIONS = [
    { code: "168", name: "7 天" },
    { code: "720", name: "30 天（默认）" },
    { code: "2160", name: "90 天" },
    { code: "4320", name: "180 天" },
    { code: "0", name: "永不过期" },
];

const hoursFromMs = (ms: number) => (ms <= 0 ? 0 : Math.round(ms / 3_600_000));

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
        <div class="p-2 bg-fuchsia-100 dark:bg-[#364153] dark:hover:bg-gray-600 transition-colors duration-200 ease-in-out rounded-lg flex flex-row flex-wrap items-center justify-between gap-4 overflow-hidden">
            <div class="flex flex-row flex-wrap items-center gap-2 min-w-0">
                <span class="inline-flex items-center gap-x-1.5 py-1.5 px-3 rounded-lg text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-500/30 dark:text-white shrink-0">
                    {ms2str(item.audioLength)}
                </span>
                <Show when={item.expired}>
                    <span class="px-2 py-0.5 text-xs rounded bg-yellow-200 text-yellow-900 dark:bg-yellow-600/40 dark:text-yellow-100 shrink-0">
                        已过期
                    </span>
                </Show>
                <span class="max-w-130 truncate">{item.title}</span>
                <span class="text-xs text-gray-500">
                    更新于 {new Date(item.updatedAt).toLocaleDateString()}
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
                    删除全部同名
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
    /** 当前过期时间（小时）；0 = 永不过期，null = 尚未加载 */
    const [ttlHours, setTtlHours] = createSignal<number | null>(null);

    const load = async (target = page()) => {
        setLoading(true);
        setError(null);
        try {
            const data = await fetchCachePage(target, PAGE_SIZE, query());
            setItems(data.items);
            setTotal(data.total);
            setPages(data.pages);
            setTtlHours(hoursFromMs(data.ttlMs));

            // 删除最后一页最后一条后页码越界，自动回退到最后一页
            if (
                data.items.length === 0 &&
                data.total > 0 &&
                target > data.pages
            ) {
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

    const commitTtl = async (hours: number) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            setTtlHours(hoursFromMs(await setCacheTtl(hours)));
            setNotice("过期时间已更新");
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
        } finally {
            setBusy(false);
        }
    };

    /** 当前值不在预设里（例如 config.json5 自定义过）时补一个自定义选项 */
    const ttlOptions = () => {
        const hours = ttlHours();
        if (
            hours == null ||
            TTL_OPTIONS.some((option) => Number(option.code) === hours)
        ) {
            return TTL_OPTIONS;
        }
        const name =
            hours % 24 === 0
                ? `${hours / 24} 天（自定义）`
                : `${hours} 小时（自定义）`;
        return [{ code: String(hours), name }, ...TTL_OPTIONS];
    };

    onMount(() => load(1));

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl font-medium inline">歌词缓存</h2>
                <p class="text-sm inline text-gray-500">
                    可以放心清理：删除缓存不会影响你绑定的歌词、偏移设置和黑名单
                </p>
            </div>
            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <div class="flex flex-row items-center gap-3 flex-wrap">
                <h3 class="text-xl font-normal">缓存过期时间</h3>
                <Select
                    class="min-w-[10rem]"
                    clearable={false}
                    options={ttlOptions()}
                    value={ttlHours() == null ? "" : String(ttlHours())}
                    disabled={busy() || loading()}
                    onChange={(code) => void commitTtl(Number(code))}
                />
                <span class="text-sm text-gray-500">
                    到期缓存自动失效；「清理过期」删除它们，调大时间后未清理的过期缓存会重新生效
                </span>
            </div>

            <div class="flex flex-row items-center gap-3 flex-wrap">
                <input
                    type="text"
                    class="px-3 py-1 min-h-11 border border-gray-300 dark:border-gray-600 rounded-md dark:bg-gray-700 dark:text-white"
                    placeholder="按标题筛选"
                    value={query()}
                    onInput={(e) => setQuery(e.currentTarget.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") load(1);
                    }}
                />
                <Button
                    class="w-fit"
                    onClick={() => load(1)}
                    disabled={loading()}
                >
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

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 items-start">
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
                                    "删除全部同名"
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

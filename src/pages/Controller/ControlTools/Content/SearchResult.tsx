import { Component, For, Show, createSignal } from "solid-js";
import { Button } from "@/components/ui";
import { Candidate } from "@/services/lyricsContentService";
import { createLyricsContentController } from "@/hooks/useLyricsContent";

const deltaView = (deltaMs: number) => {
    const seconds = Math.round(deltaMs / 1000);
    if (seconds === 0) return { text: "0s", cls: "text-gray-500" };
    if (seconds > 0)
        return { text: `+${seconds}s`, cls: "text-blue-600 dark:text-blue-400" };
    return { text: `${seconds}s`, cls: "text-red-600 dark:text-red-400" };
};

const translationBadge = (has: boolean | undefined) => {
    if (has === undefined)
        return { text: "需加载", cls: "bg-gray-100 dark:bg-gray-800 text-gray-500" };
    if (has)
        return {
            text: "有翻译",
            cls: "bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300",
        };
    return {
        text: "无翻译",
        cls: "bg-gray-100 dark:bg-gray-800 text-gray-500",
    };
};
const SearchResult: Component<{
    controller: ReturnType<typeof createLyricsContentController>;
}> = (props) => {
    const c = props.controller;
    const [keyword, setKeyword] = createSignal("");

    /**
     * 执行搜索。
     *
     * - 输入为空 → 按当前播放歌曲搜索（不传条件）。
     * - 有输入   → 按「歌曲标题/艺术家」解析，`/` 与 `|` 都支持。
     *
     * **为什么要处理单值**：后端 `lyric_content_service::search` 要求 title 与
     * artist **同时非空**，否则会退回"按当前歌曲搜索" —— 表现就是"点了搜索却没生效"。
     * 所以只填一个值时把它同时作为标题与艺术家发送，确保请求真的带上用户输入。
     * 后端与搜索评分逻辑未做任何修改。
     */
    const doSearch = async () => {
        const raw = keyword().trim();
        if (!raw) {
            await c.search();
            return;
        }
        const parts = raw
            .split(/[/|]/)
            .map((s) => s.trim())
            .filter((s) => s.length > 0);
        const title = parts[0] ?? raw;
        const artist = parts[1] ?? title;
        await c.search(title, artist);
    };

    const translationOf = (candidate: Candidate) =>
        c.translations()[c.candidateId(candidate)];

    const currentLength = () => c.current()?.song.length ?? 0;

    return (
        <div class="flex flex-col gap-3">
            <div class="flex flex-row items-center flex-wrap gap-3">
                <h3 class="text-xl font-normal">搜索结果</h3>
                <input
                    type="text"
                    class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md dark:bg-gray-700 dark:text-white min-w-64"
                    placeholder="歌曲标题/艺术家"
                    value={keyword()}
                    onInput={(e) => setKeyword(e.currentTarget.value)}
                />
                <span class="text-sm text-gray-500">留空则为当前歌曲</span>
                <Button
                    class="px-4 py-1"
                    onClick={doSearch}
                    disabled={c.searching()}
                >
                    {c.searching() ? "搜索中..." : "搜索"}
                </Button>
                <Button
                    class="px-4 py-1"
                    disabled={c.listLoading() || c.candidates().length === 0}
                    onClick={() => void c.loadAllCandidateLyrics()}
                >
                    {c.listLoading() ? "加载中..." : "加载列表歌词"}
                </Button>
                <span class="text-sm text-gray-500">
                    共 {c.candidates().length} 项
                </span>
            </div>

            <Show when={c.candidates().length === 0}>
                <p class="text-sm text-gray-500">
                    {c.manualSearch()
                        ? "没有找到匹配的候选，换个关键词试试。"
                        : c.candidateState() === "error" ? "候选读取失败：切歌会自动重新读取后端候选。" : c.candidateState() === "loading" ? "正在加载候选…" : c.candidateState() === "loaded" ? "当前歌曲暂无候选。也可在上方输入「歌曲标题/艺术家」后点击搜索。" : "需加载：尚未从后端读取到当前歌曲的候选。"}
                </p>
            </Show>

            <div class="flex flex-col gap-3 max-h-[420px] overflow-y-auto">
                <For each={c.candidates()}>
                    {(candidate) => (
                        <div class="flex flex-col gap-2 p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                            <div class="flex flex-row items-center gap-3 flex-wrap">
                                <span
                                    class={`px-2 py-0.5 text-xs rounded shrink-0 ${translationBadge(translationOf(candidate)).cls}`}
                                >
                                    {translationBadge(translationOf(candidate)).text}
                                </span>
                                <span class="px-2 py-0.5 text-xs rounded bg-gray-100 dark:bg-gray-800">
                                    {candidate.source}
                                </span>
                                <span class="text-base">
                                    {candidate.title} — {candidate.artist}
                                </span>
                                <span class="text-xs text-gray-500">
                                    {Math.round(candidate.length / 1000)}s
                                </span>
                                <span
                                    class={`text-xs font-medium ${deltaView(candidate.durationDelta).cls}`}
                                    title={
                                        currentLength() > 0
                                            ? `当前 ${Math.round(currentLength() / 1000)}s，候选 ${Math.round(candidate.length / 1000)}s`
                                            : "当前没有在播歌曲"
                                    }
                                >
                                    {deltaView(candidate.durationDelta).text}
                                </span>
                                <span class="text-xs text-gray-400">
                                    key {candidate.key}
                                </span>
                                <Show when={candidate.active}>
                                    <span class="px-2 py-0.5 text-xs rounded bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300">
                                        当前绑定
                                    </span>
                                </Show>
                                <Show
                                    when={
                                        c.previewTarget() ===
                                        c.candidateId(candidate)
                                    }
                                >
                                    <span class="px-2 py-0.5 text-xs rounded bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                                        预览中
                                    </span>
                                </Show>
                                <div class="flex flex-row gap-2 ml-auto">
                                    <Button
                                        class="px-3 py-1"
                                        onClick={() => c.apply(candidate)}
                                        disabled={c.saving()}
                                    >
                                        应用
                                    </Button>
                                </div>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
};

export default SearchResult;

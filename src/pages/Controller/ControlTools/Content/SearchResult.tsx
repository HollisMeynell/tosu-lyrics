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
        return { text: "读取中", cls: "bg-gray-100 dark:bg-gray-800 text-gray-500" };
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

    const doSearch = async () => {
        const raw = keyword().trim();
        if (!raw) {
            await c.search();
            return;
        }
        const [title, artist] = raw.split("|").map((s) => s.trim());
        await c.search(title || raw, artist || "");
    };

    const previewOf = (candidate: Candidate) =>
        c.previews()[c.candidateId(candidate)];
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
                    placeholder="不填 = 按当前歌曲搜索；或填「标题 | 艺术家」"
                    value={keyword()}
                    onInput={(e) => setKeyword(e.currentTarget.value)}
                />
                <Button
                    class="px-4 py-1"
                    onClick={doSearch}
                    disabled={c.searching()}
                >
                    {c.searching() ? "搜索中..." : "搜索"}
                </Button>
                <span class="text-sm text-gray-500">
                    共 {c.candidates().length} 项
                </span>
            </div>

            <Show when={c.candidates().length === 0}>
                <p class="text-sm text-gray-500">
                    暂无候选。点击「搜索」按当前播放歌曲查询。
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
                                <div class="flex flex-row gap-2 ml-auto">
                                    <Button
                                        class="px-3 py-1"
                                        onClick={() => c.preview(candidate)}
                                        disabled={
                                            c.previewing() ===
                                            c.candidateId(candidate)
                                        }
                                    >
                                        {c.previewing() ===
                                        c.candidateId(candidate)
                                            ? "读取中..."
                                            : "预览"}
                                    </Button>
                                    <Button
                                        class="px-3 py-1"
                                        onClick={() => c.apply(candidate)}
                                        disabled={c.saving()}
                                    >
                                        应用
                                    </Button>
                                </div>
                            </div>

                            <Show when={previewOf(candidate)}>
                                {(preview) => (
                                    <div class="pl-3 border-l-2 border-gray-300 dark:border-gray-600 max-h-40 overflow-y-auto">
                                        <p class="text-xs text-gray-500 mb-1">
                                            预览 {preview().source} /{" "}
                                            {preview().key}（{preview().lineCount} 行）
                                        </p>
                                        <For each={preview().lines.slice(0, 20)}>
                                            {(line) => (
                                                <div class="flex flex-col">
                                                    <p class="text-sm">
                                                        {line.origin}
                                                    </p>
                                                    <Show when={line.translation}>
                                                        <p class="text-xs text-gray-500">
                                                            {line.translation}
                                                        </p>
                                                    </Show>
                                                </div>
                                            )}
                                        </For>
                                    </div>
                                )}
                            </Show>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
};

export default SearchResult;

import { For, Show, Component, createSignal, createMemo, onCleanup } from "solid-js";
import { Copy } from "@/assets/Icons";
import { Button } from "@/components/ui";
import { createLyricsContentController } from "@/hooks/useLyricsContent";
import { LyricLineDto } from "@/services/lyricsContentService";

const stateHint = (state: string | undefined, blocked: boolean) => {
    if (blocked || state === "blocked") return "这首歌已被拉黑，不会显示歌词";
    if (state === "none") return "暂时没有歌词（可能正在搜索，或该来源没有这首歌）";
    return "暂无歌词";
};

/** 最后一行返回 null，由调用方显示占位符 */
export const lineDuration = (
    lines: LyricLineDto[],
    index: number
): number | null => {
    const next = lines[index + 1];
    if (!next) return null;
    const delta = next.time - lines[index].time;
    return delta > 0 ? delta : null;
};

const MS_PER_PX = 10;

const CurrentLyrics: Component<{
    controller: ReturnType<typeof createLyricsContentController>;
}> = (props) => {
    const c = props.controller;
    const [copied, setCopied] = createSignal(false);
    const [follow, setFollow] = createSignal(true);
    const [draggingMs, setDraggingMs] = createSignal<number | null>(null);
    let listRef: HTMLDivElement | undefined;

    const lines = c.current()?.lyric ?? [];
    const noSong = () => c.noSong();

    const offsetShown = () =>
        draggingMs() ?? c.current()?.offset ?? 0;

    const durations = createMemo(() =>
        lines.map((_, i) => lineDuration(lines, i))
    );

    const copyAll = async () => {
        const text = lines
            .map((l) => l.origin ?? "")
            .filter((t) => t.length > 0)
            .join("\n");
        if (!text) return;
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
        } catch {
            c.setError("复制失败：浏览器拒绝了剪贴板访问");
        }
    };

    const applyOffset = async (value: string) => {
        const parsed = Number.parseInt(value, 10);
        if (Number.isNaN(parsed)) {
            c.setError("偏移必须是整数毫秒");
            return;
        }
        await c.updateOffset(parsed);
    };

    // 拖动调整 offset：往下拖 = 希望歌词晚一点出现 = offset 变小
    let dragStartY = 0;
    let dragStartOffset = 0;
    let dragging = false;

    const onDragStart = (e: PointerEvent) => {
        if (noSong()) return;
        if (e.button !== 0) return;
        const target = e.currentTarget as HTMLElement;
        target.setPointerCapture(e.pointerId);
        dragging = true;
        dragStartY = e.clientY;
        dragStartOffset = c.current()?.offset ?? 0;
        setDraggingMs(dragStartOffset);
    };

    const onDragMove = (e: PointerEvent) => {
        if (!dragging) return;
        const deltaPx = e.clientY - dragStartY;
        setDraggingMs(dragStartOffset - Math.round(deltaPx * MS_PER_PX));
    };

    const onDragEnd = async (e: PointerEvent) => {
        if (!dragging) return;
        dragging = false;
        const target = e.currentTarget as HTMLElement;
        try {
            target.releasePointerCapture(e.pointerId);
        } catch { /* 指针已释放 */ }
        const pending = draggingMs();
        setDraggingMs(null);
        if (pending === null) return;
        if (pending === (c.current()?.offset ?? 0)) return;
        await c.updateOffset(pending);
    };

    const scrollToCurrent = () => {
        if (!follow() || dragging) return;
        const cur = c.current()?.current ?? -1;
        if (cur < 0 || !listRef) return;
        const el = listRef.querySelector<HTMLElement>(`[data-line="${cur}"]`);
        el?.scrollIntoView({ block: "center", behavior: "smooth" });
    };
    const timer = setInterval(() => scrollToCurrent(), 700);
    onCleanup(() => clearInterval(timer));

    return (
        <div class="flex flex-col gap-3">
            <div class="flex flex-row items-center flex-wrap gap-3">
                <h2 class="text-2xl font-normal">当前歌词</h2>
                <Button
                    class="px-5 py-1"
                    onClick={() => c.loadCurrent()}
                    disabled={c.loading()}
                >
                    {c.loading() ? "加载中..." : "刷新"}
                </Button>

                <Show when={lines.length > 0}>
                    <Button class="px-4 py-1" onClick={copyAll}>
                        <Copy stroke="currentColor" class="mr-2 w-4 h-4 inline" />
                        {copied() ? "已复制" : "复制原文"}
                    </Button>
                </Show>

                <label class="flex flex-row items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={follow()}
                        onChange={(e) => setFollow(e.currentTarget.checked)}
                    />
                    跟随当前行滚动
                </label>
                <span class="text-xs text-gray-500">
                    （仅影响本页阅读区，不影响展示端）
                </span>

                <Button
                    class="px-4 py-1"
                    onClick={() => c.clear()}
                    disabled={c.saving() || noSong()}
                >
                    清屏
                </Button>

                <Show when={c.current()}>
                    {(cur) => (
                        <div class="flex flex-row items-center gap-2 text-sm">
                            <label>偏移(ms)：</label>
                            <input
                                type="number"
                                class="w-24 px-2 py-1 border border-gray-300 dark:border-gray-600 rounded-md dark:bg-gray-700 dark:text-white"
                                value={offsetShown()}
                                onChange={(e) => applyOffset(e.currentTarget.value)}
                                disabled={c.saving()}
                            />
                            <Button
                                class="px-3 py-1"
                                onClick={() => c.updateOffset(0)}
                                disabled={c.saving() || cur().offset === 0}
                            >
                                归零
                            </Button>
                            <span class="text-gray-500">
                                当前行 {cur().current} / 剩余 {cur().nextTime} ms
                            </span>
                        </div>
                    )}
                </Show>
            </div>

            <Show when={c.current()?.source}>
                {(binding) => (
                    <div class="flex flex-row items-center gap-3 text-sm">
                        <span class="px-2 py-1 rounded bg-gray-100 dark:bg-gray-800">
                            已绑定来源：{binding().source} / {binding().key}（sid {binding().sid}）
                        </span>
                        <Button
                            class="px-3 py-1"
                            onClick={() => c.revertToAuto()}
                            disabled={c.saving()}
                        >
                            恢复自动匹配
                        </Button>
                    </div>
                )}
            </Show>

            <Show when={lines.length === 0}>
                <p class="text-sm text-gray-500">
                    {stateHint(c.current()?.lyricState, c.current()?.blocked ?? false)}
                </p>
            </Show>

            <Show when={lines.length > 0}>
                <div class="flex flex-row items-center gap-3 text-xs text-gray-500">
                    <span
                        class="cursor-ns-resize select-none px-2 py-0.5 rounded border border-dashed border-gray-400"
                        onPointerDown={onDragStart}
                        onPointerMove={onDragMove}
                        onPointerUp={onDragEnd}
                        onPointerCancel={onDragEnd}
                        title="按住上下拖动可微调偏移"
                    >
                        ⇕ 拖动这里调整偏移
                    </span>
                    <Show when={draggingMs() !== null}>
                        <span class="text-[#ec4899]">
                            拖动中：{offsetShown()} ms（松手后生效）
                        </span>
                    </Show>
                    <span>共 {lines.length} 行 · 左列为该行持续时间</span>
                </div>
            </Show>

            <div
                ref={listRef}
                class="max-h-[520px] overflow-y-auto rounded-md border border-gray-200 dark:border-gray-700 p-2"
            >
                <For each={lines}>
                    {(item, index) => (
                        <div
                            data-line={index()}
                            class={`flex flex-row items-start my-1 gap-3 max-w-[820px] px-2 py-1 rounded ${
                                index() === c.current()?.current
                                    ? "bg-pink-50 dark:bg-pink-900/20"
                                    : ""
                            }`}
                        >
                            <span class="text-xs text-gray-400 w-20 shrink-0 pt-1 text-right">
                                {/* 这里是**持续时间**，不是开始时间 */}
                                {durations()[index()] !== null
                                    ? `${durations()[index()]} ms`
                                    : "—"}
                            </span>
                            <div class="grow flex flex-col">
                                <p
                                    class={
                                        index() === c.current()?.current
                                            ? "text-xl text-[#ec4899]"
                                            : "text-xl"
                                    }
                                >
                                    {item.origin}
                                </p>
                                <Show when={item.translation}>
                                    <p class="text-sm text-gray-500">
                                        {item.translation}
                                    </p>
                                </Show>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
};

export default CurrentLyrics;

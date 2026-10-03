import { For, Show, Component, createSignal, createMemo, onCleanup } from "solid-js";
import { Copy } from "@/assets/Icons";
import { Button } from "@/components/ui";
import { createLyricsContentController } from "@/hooks/useLyricsContent";
import { cursor } from "@/stores/lyricStore";
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

/**
 * 歌词行开始时间 → 固定 mm:ss（不显示毫秒，分钟允许超过 59）。
 *
 * 单位提醒：`/api/lyrics/current` 与 `/api/lyrics/preview` 返回的 `time` 都是
 * **毫秒**（后端把内部的秒 ×1000，见 server/lyrics.rs 与
 * lyric_content_service.rs 的 lines_to_json），所以必须先 /1000 ——
 * 直接把毫秒当秒用会得到几十倍偏大的分钟数。
 */
const formatTime = (milliseconds: number): string => {
    const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
    const mm = String(Math.floor(totalSeconds / 60)).padStart(2, "0");
    const ss = String(totalSeconds % 60).padStart(2, "0");
    return `${mm}:${ss}`;
};

const MS_PER_PX = 10;

const CurrentLyrics: Component<{
    controller: ReturnType<typeof createLyricsContentController>;
}> = (props) => {
    const c = props.controller;
    const [copied, setCopied] = createSignal(false);
    const [draggingMs, setDraggingMs] = createSignal<number | null>(null);
    let listRef: HTMLDivElement | undefined;

    // 预览优先：搜索结果的"预览"只改本页预览区，不写 lyricStore / 缓存 / 播放状态
    const previewLines = createMemo<LyricLineDto[] | null>(() => {
        const id = c.previewTarget();
        if (!id) return null;
        return c.previews()[id]?.lines ?? null;
    });
    /** 当前正在使用的歌词（来自 /api/lyrics/current） */
    const currentLines = createMemo<LyricLineDto[]>(
        () => c.current()?.lyric ?? []
    );
    /** 预览区实际渲染的行：预览中显示候选歌词，否则显示当前歌词 */
    const lines = createMemo<LyricLineDto[]>(
        () => previewLines() ?? currentLines()
    );
    const isPreviewing = () => previewLines() !== null;
    const noSong = () => c.noSong();

    const offsetShown = () => draggingMs() ?? c.current()?.offset ?? 0;

    const copyAll = async () => {
        const text = lines()
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
        // 拖动过程中立即同步滚动与高亮，不等松手、也不等下一次定时
        scrollToIndex(activeIndex());
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
        if (await c.updateOffset(pending)) {
            // 偏移已在后端生效，但 current 行号还是拖动前那一份；刷新一次，
            // 让高亮/滚动停在"最终偏移对应的位置"，而不是跳回旧行。
            await c.loadCurrent();
        }
    };

    /**
     * 阅读区当前应该高亮/滚动到的行。
     *
     * 默认用后端算好的当前行（`/api/lyrics/current` 的 `current`）；拖动调整
     * 偏移时改用它推算：以当前行的开始时间为基准、加上偏移的变化量做线性平移，
     * 再在现有行时间里找落点 —— 复用现有行时间，不新建时间轴计算。
     * 偏移与行时间同为**毫秒**，可以直接相减。
     */
    const activeIndex = createMemo(() => {
        // 当前行取 WS 实时更新的 `cursor`（与 /lyrics 展示端同一来源）。
        // 原先用 `c.current()?.current` —— 那是 `/api/lyrics/current` 的一次性
        // HTTP 快照，页面停留不动时永远不会刷新，于是歌词不会跟着播放滚动。
        const live = cursor();
        const draggingOffset = draggingMs();
        if (draggingOffset === null) return live;
        const rows = lines();
        if (live < 0 || live >= rows.length) return live;
        const previewTime =
            rows[live].time + (draggingOffset - (c.current()?.offset ?? 0));
        let found = 0;
        for (let i = 0; i < rows.length; i++) {
            if (rows[i].time <= previewTime) found = i;
            else break;
        }
        return found;
    });

    /** 本页阅读区是否自动跟随当前时间戳（只影响本页，不影响展示端） */
    const [autoFollow, setAutoFollow] = createSignal(true);

    /**
     * 只滚动歌词列表**自身**，刻意不使用 `scrollIntoView`。
     *
     * `scrollIntoView` 会连带滚动**所有可滚动祖先** —— 包括 Controller 外层的
     * `overflow-y-auto` 容器，于是每次当前行变化都会把整个页面拽回当前行位置，
     * 表现就是"鼠标滚轮往下滚之后又跳回原位"。这里直接设置列表自身的 scrollTop，
     * 外层页面的滚动位置完全不受影响。
     */
    const scrollToIndex = (index: number) => {
        const box = listRef;
        if (index < 0 || !box) return;
        const el = box.querySelector<HTMLElement>(`[data-line="${index}"]`);
        if (!el) return;
        // 列表容器带 relative，offsetTop 就是相对列表内容区的偏移
        const target = el.offsetTop - (box.clientHeight - el.clientHeight) / 2;
        box.scrollTo({ top: Math.max(0, target), behavior: "smooth" });
    };

    const scrollToCurrent = () => {
        // 拖动调整偏移时持续跟随拖动后的位置
        if (dragging) {
            scrollToIndex(activeIndex());
            return;
        }
        // 预览搜索结果时不跟随；用户点了"停止滚动"也不跟随
        if (isPreviewing() || !autoFollow()) return;
        scrollToIndex(activeIndex());
    };
    const timer = setInterval(() => scrollToCurrent(), 700);
    onCleanup(() => clearInterval(timer));

    return (
        <div class="flex flex-col gap-3">
            <div class="flex flex-row items-center flex-wrap gap-3">
                <Show when={lines().length > 0}>
                    <Button class="px-4 py-1" onClick={copyAll}>
                        <Copy stroke="currentColor" class="mr-2 w-4 h-4 inline" />
                        {copied() ? "已复制" : "复制原文"}
                    </Button>
                </Show>

                <Button
                    class="px-4 py-1"
                    onClick={() => c.clear()}
                    disabled={c.saving() || noSong()}
                >
                    清屏
                </Button>

                <Button
                    class="px-4 py-1"
                    onClick={() => setAutoFollow(!autoFollow())}
                >
                    {autoFollow() ? "停止滚动" : "继续滚动"}
                </Button>

                {/* 预览搜索结果时的标识与出口：只影响本页预览区 */}
                <Show when={isPreviewing()}>
                    <span class="px-2 py-1 text-sm rounded bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                        正在预览搜索结果（不影响展示端与缓存）
                    </span>
                    <Button
                        class="px-3 py-1"
                        onClick={() => c.clearPreview()}
                    >
                        关闭预览
                    </Button>
                </Show>

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

            {/* 预览中但该候选没有歌词 */}
            <Show when={isPreviewing() && lines().length === 0}>
                <p class="text-sm text-gray-500">该搜索结果没有可用歌词。</p>
            </Show>

            {/* 非预览、且当前确实没有歌词时，才提示当前歌的状态 */}
            <Show when={!isPreviewing() && lines().length === 0}>
                <p class="text-sm text-gray-500">
                    {stateHint(c.current()?.lyricState, c.current()?.blocked ?? false)}
                </p>
            </Show>

            <Show when={lines().length > 0}>
                <div class="flex flex-row items-center gap-3 text-xs text-gray-500">
                    <Show when={!isPreviewing()}>
                        <span
                            class="cursor-ns-resize select-none px-6 py-3 text-base rounded-lg border-2 border-dashed border-gray-400 hover:border-[#ec4899] hover:text-[#ec4899] transition-colors"
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
                    </Show>
                    <span>
                        共 {lines().length} 行 · 左列为该行开始时间（mm:ss）
                    </span>
                </div>
            </Show>

            <div
                ref={listRef}
                class="relative max-h-[420px] overflow-y-auto rounded-md border border-gray-200 dark:border-gray-700 p-2"
            >
                <For each={lines()}>
                    {(item, index) => (
                        <div
                            data-line={index()}
                            class={`flex flex-row items-start my-1 gap-3 max-w-[820px] px-2 py-1 rounded ${
                                !isPreviewing() && index() === activeIndex()
                                    ? "bg-pink-50 dark:bg-pink-900/20"
                                    : ""
                            }`}
                        >
                            <span class="text-xs text-gray-400 w-16 shrink-0 pt-1 text-right tabular-nums">
                                {formatTime(item.time)}
                            </span>
                            <div class="grow flex flex-col">
                                <p
                                    class={
                                        !isPreviewing() &&
                                        index() === activeIndex()
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

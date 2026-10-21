import { Show, createSignal, onMount, For } from "solid-js";
import { Button } from "@/components/ui";
import { ApiError } from "@/services/settingsService";
import { fetchStatus } from "@/services/statusService";
import {
    fetchFontList,
    uploadFontFile,
    uploadLrc,
} from "@/services/uploadService";
import type { FontEntry } from "@/services/uploadService";
import { loadFont } from "@/utils/fonts.ts";

export default function Upload() {
    const [songTitle, setSongTitle] = createSignal<string | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);

    const [fonts, setFonts] = createSignal<FontEntry[]>([]);

    const refreshSong = async () => {
        try {
            const status = await fetchStatus();
            setSongTitle(
                status.song
                    ? `${status.song.title} — ${status.song.artist}`
                    : null
            );
        } catch {
            setSongTitle(null);
        }
    };

    const refreshFonts = async () => {
        try {
            const items = await fetchFontList();
            setFonts(items);
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
        }
    };

    const run = async (fn: () => Promise<string>, failLabel: string) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            const message = await fn();
            setNotice(message);
        } catch (err) {
            setError(
                `${failLabel}：${err instanceof ApiError ? err.message : String(err)}`
            );
        } finally {
            setBusy(false);
        }
    };

    const uploadLrcFile = (file: File) =>
        run(async () => {
            const result = await uploadLrc(file);
            await refreshSong();
            return `已上传并应用：${file.name}（${result.lines} 行）`;
        }, "LRC 上传失败");

    const uploadFont = (file: File) =>
        run(async () => {
            const entry = await uploadFontFile(file);
            await refreshFonts();
            // 让新字体立刻生效：重新注册 FontFace
            await loadFont();
            return `字体已上传：${entry.name}（${entry.fileName}，${Math.round(entry.size / 1024)} KB）`;
        }, "字体上传失败");

    const pickHandler = (onFile: (file: File) => void) => (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) onFile(file);
    };

    const dropHandler = (onFile: (f: File) => void) => (e: DragEvent) => {
        e.preventDefault();
        setDragOver(null);
        const file = e.dataTransfer?.files?.[0];
        if (file) onFile(file);
    };

    const [dragOver, setDragOver] = createSignal<string | null>(null);

    onMount(() => {
        void refreshSong();
        void refreshFonts();
    });

    /** 方正的拖拽上传卡片：整块可点击选择文件，也可直接拖入 */
    const DropZone = (props: {
        kind: string;
        hint: string;
        accept: string;
        onFile: (file: File) => void;
    }) => (
        <label
            class={`mx-auto flex w-full max-w-xs min-h-44 cursor-pointer flex-col items-center justify-center gap-3 rounded-md border-2 border-dashed p-6 text-center transition-colors ${
                dragOver() === props.kind
                    ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                    : "border-gray-300 dark:border-gray-600"
            }`}
            onDragOver={(e) => {
                e.preventDefault();
                setDragOver(props.kind);
            }}
            onDragLeave={() => setDragOver(null)}
            onDrop={dropHandler(props.onFile)}
        >
            <svg
                class="size-8 text-gray-400"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                aria-hidden="true"
            >
                <path
                    d="M12 15V4m0 0 4 4m-4-4-4 4"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                <path
                    d="M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"
                    stroke-linecap="round"
                />
            </svg>
            <span class="text-sm text-gray-500">{props.hint}</span>
            <span class="text-sm text-[#ec4899]">或点击选择文件</span>
            <input
                type="file"
                class="sr-only"
                accept={props.accept}
                disabled={busy()}
                onChange={pickHandler(props.onFile)}
            />
        </label>
    );

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl font-medium inline">上传字体或歌词文件</h2>
                <p class="text-sm inline text-gray-500">
                    上传的歌词会用在当前歌曲上；上传的字体可在「文字样式」里直接选用
                </p>
            </div>
            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <Show when={error()}>
                <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                    {error()}
                </div>
            </Show>
            <Show when={notice()}>
                <div class="px-4 py-2 rounded-md bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-200 text-sm w-fit">
                    {notice()}
                </div>
            </Show>

            {/* ---------------- LRC ---------------- */}
            <section class="flex flex-col gap-3 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="flex flex-row items-center gap-3 flex-wrap">
                    <h3 class="text-xl font-normal">歌词文件 (.lrc)</h3>
                    <Button
                        class="px-3 py-1"
                        onClick={() => void refreshSong()}
                        disabled={busy()}
                    >
                        刷新歌曲
                    </Button>
                    <span class="text-sm text-gray-500">
                        当前歌曲：{songTitle() ?? "无（需先播放一首歌）"}
                    </span>
                </div>
                <DropZone
                    kind="lrc"
                    hint="把 .lrc 文件拖到这里"
                    accept=".lrc,.txt,text/plain"
                    onFile={(f) => void uploadLrcFile(f)}
                />
                <p class="text-xs text-gray-500">
                    不支持的文件不会替换当前歌词；上传成功后展示端立即生效。
                </p>
            </section>

            {/* ---------------- 字体 ---------------- */}
            <section class="flex flex-col gap-3 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="flex flex-row items-center gap-3 flex-wrap">
                    <h3 class="text-xl font-normal">字体库</h3>
                    <Button
                        class="px-3 py-1"
                        onClick={() => void refreshFonts()}
                        disabled={busy()}
                    >
                        刷新
                    </Button>
                </div>

                {/* 上传区域 */}
                <DropZone
                    kind="font"
                    hint="把字体文件拖到这里"
                    accept=".ttf,.otf,.woff,.woff2"
                    onFile={(f) => void uploadFont(f)}
                />
                <p class="text-xs text-gray-500">
                    上传后会自动识别字体名称并保存，可在「文字样式」里选用。
                </p>

                {/* 已上传字体列表 */}
                <Show when={fonts().length > 0}>
                    <div class="flex flex-col gap-2">
                        <h4 class="text-sm font-medium text-gray-600 dark:text-gray-400">
                            已上传字体（{fonts().length}）
                        </h4>
                        <For each={fonts()}>
                            {(entry) => (
                                <div class="flex flex-row flex-wrap items-center gap-3 p-2 rounded bg-gray-50 dark:bg-gray-800">
                                    <span class="text-sm font-medium">
                                        {entry.name}
                                    </span>
                                    <span class="text-xs text-gray-500">
                                        {entry.fileName} ·{" "}
                                        {Math.round(entry.size / 1024)} KB
                                    </span>
                                </div>
                            )}
                        </For>
                    </div>
                </Show>
                <Show when={fonts().length === 0}>
                    <p class="text-sm text-gray-500">暂无已上传字体</p>
                </Show>
            </section>
        </div>
    );
}

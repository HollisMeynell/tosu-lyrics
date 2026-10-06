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
            setSongTitle(status.song ? `${status.song.title} — ${status.song.artist}` : null);
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
            setError(`${failLabel}：${err instanceof ApiError ? err.message : String(err)}`);
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
            return `字体已上传：${entry.name}（${entry.fileName}，${entry.size} 字节）`;
        }, "字体上传失败");

    const onLrc = (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) void uploadLrcFile(file);
    };

    const onFont = (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) void uploadFont(file);
    };

    const dropHandler =
        (onFile: (f: File) => void) => (e: DragEvent) => {
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

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl font-medium inline">上传字体或歌词文件</h2>
                <p class="text-sm inline text-gray-500">
                    LRC 绑定到当前歌曲；字体上传后按名称加载，无需手工放文件
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
                    <span class="text-sm text-gray-500">
                        当前歌曲：{songTitle() ?? "无（需先播放一首歌）"}
                    </span>
                </div>
                <div
                    class={`flex flex-row items-center gap-3 p-3 rounded-md border-2 border-dashed transition-colors ${
                        dragOver() === "lrc"
                            ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                            : "border-gray-300 dark:border-gray-600"
                    }`}
                    onDragOver={(e) => {
                        e.preventDefault();
                        setDragOver("lrc");
                    }}
                    onDragLeave={() => setDragOver(null)}
                    onDrop={dropHandler((f) => void uploadLrcFile(f))}
                >
                    <span class="text-sm text-gray-500">把 .lrc 拖到这里，或</span>
                    <input
                        type="file"
                        accept=".lrc,.txt,text/plain"
                        disabled={busy()}
                        onChange={onLrc}
                        class="text-sm"
                    />
                    <Button
                        class="px-3 py-1"
                        onClick={() => void refreshSong()}
                        disabled={busy()}
                    >
                        刷新歌曲
                    </Button>
                </div>
                <p class="text-xs text-gray-500">
                    非法文件不会替换当前歌词；上传成功后展示端按当前播放进度立即刷新。
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
                <div
                    class={`flex flex-row items-center gap-3 p-3 rounded-md border-2 border-dashed transition-colors ${
                        dragOver() === "font"
                            ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                            : "border-gray-300 dark:border-gray-600"
                    }`}
                    onDragOver={(e) => {
                        e.preventDefault();
                        setDragOver("font");
                    }}
                    onDragLeave={() => setDragOver(null)}
                    onDrop={dropHandler((f) => void uploadFont(f))}
                >
                    <span class="text-sm text-gray-500">把字体文件拖到这里，或</span>
                    <input
                        type="file"
                        accept=".ttf,.otf,.woff,.woff2"
                        disabled={busy()}
                        onChange={onFont}
                        class="text-sm"
                    />
                </div>
                <p class="text-xs text-gray-500">
                    上传后自动解析字体名称，保存到字体库。在"文字样式"页面可选择使用。
                </p>

                {/* 已上传字体列表 */}
                <Show when={fonts().length > 0}>
                    <div class="flex flex-col gap-2">
                        <h4 class="text-sm font-medium text-gray-600 dark:text-gray-400">
                            已上传字体（{fonts().length}）
                        </h4>
                        <For each={fonts()}>
                            {(entry) => (
                                <div class="flex flex-row items-center gap-3 p-2 rounded bg-gray-50 dark:bg-gray-800">
                                    <span class="text-sm font-medium">{entry.name}</span>
                                    <span class="text-xs text-gray-500">
                                        {entry.fileName} / {entry.size} 字节
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
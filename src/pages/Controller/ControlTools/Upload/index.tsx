import { Show, createSignal, onMount, For } from "solid-js";
import { Button } from "@/components/ui";
import { ApiError } from "@/services/settingsService";
import { fetchStatus } from "@/services/statusService";
import {
    FontInfo,
    fetchFontInfo,
    uploadFontFile,
    uploadLrc,
} from "@/services/uploadService";
import { loadFont } from "@/utils/fonts.ts";

const KIND_LABEL: Record<"main" | "sub", string> = {
    main: "主字体",
    sub: "副字体",
};

async function probeFont(info: FontInfo): Promise<string> {
    if (!info.exists || !info.url) return "尚未上传";
    try {
        const res = await fetch(info.url);
        if (!res.ok) return `取不到（HTTP ${res.status}）`;
        const buf = await res.arrayBuffer();
        if (buf.byteLength !== info.size) {
            return `字节数不符（期望 ${info.size}，实际 ${buf.byteLength}）`;
        }
        return `可加载（${buf.byteLength} 字节）`;
    } catch (err) {
        return `取不到：${String(err)}`;
    }
}

export default function Upload() {
    const [songTitle, setSongTitle] = createSignal<string | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);

    const [fonts, setFonts] = createSignal<FontInfo[]>([]);
    const [probe, setProbe] = createSignal<Record<string, string>>({});

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
            const items = await fetchFontInfo();
            setFonts(items);
            const results: Record<string, string> = {};
            for (const item of items) {
                results[item.kind] = await probeFont(item);
            }
            setProbe(results);
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

    const uploadFontFor = (kind: "main" | "sub") => (file: File) =>
        run(async () => {
            const info = await uploadFontFile(kind, file);
            await refreshFonts();
            // 让新字体立刻生效：重新注册 FontFace。
            // loadFont 内部按版本号判断是否需要重新拉取，同版本不会重复加载；
            // 之前只刷新了字体列表，上传成功后展示端仍用旧字体（或静态回退）。
            await loadFont();
            return `${KIND_LABEL[kind]}已覆盖：${file.name}（版本 ${info.version}）`;
        }, `${KIND_LABEL[kind]}上传失败`);

    const onLrc = (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) void uploadLrcFile(file);
    };

    const onFont = (kind: "main" | "sub") => (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) void uploadFontFor(kind)(file);
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
                    LRC 绑定到当前歌曲；字体上传后按版本 URL 加载，无需手工放文件
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
                    <h3 class="text-xl font-normal">字体资源</h3>
                    <Button
                        class="px-3 py-1"
                        onClick={() => void refreshFonts()}
                        disabled={busy()}
                    >
                        刷新
                    </Button>
                </div>

                <For each={["main", "sub"] as const}>
                    {(kind) => {
                        const info = () => fonts().find((f) => f.kind === kind);
                        return (
                            <div
                                class={`flex flex-col gap-2 p-3 rounded-md border-2 border-dashed transition-colors ${
                                    dragOver() === kind
                                        ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                                        : "border-transparent bg-gray-50 dark:bg-gray-800"
                                }`}
                                onDragOver={(e) => {
                                    e.preventDefault();
                                    setDragOver(kind);
                                }}
                                onDragLeave={() => setDragOver(null)}
                                onDrop={dropHandler((f) => void uploadFontFor(kind)(f))}
                            >
                                <div class="flex flex-row items-center gap-3 flex-wrap">
                                    <span class="text-base font-medium">
                                        {KIND_LABEL[kind]}
                                    </span>
                                    <Show
                                        when={info()?.exists}
                                        fallback={
                                            <span class="text-sm text-gray-500">
                                                尚未上传（展示端会用随包字体）
                                            </span>
                                        }
                                    >
                                        <span class="text-sm text-gray-500">
                                            版本 {info()!.version} / {info()!.size} 字节
                                            / 家族 {info()!.family}
                                        </span>
                                    </Show>
                                    <span class="text-xs text-gray-400 ml-auto">
                                        拖拽到此处，或
                                    </span>
                                    <input
                                        type="file"
                                        accept=".ttf,.otf,.woff,.woff2"
                                        disabled={busy()}
                                        onChange={onFont(kind)}
                                        class="text-sm"
                                    />
                                </div>
                                <Show when={info()?.exists}>
                                    <div class="flex flex-row items-center gap-2 text-xs text-gray-500">
                                        <code class="break-all">{info()!.url}</code>
                                        <span>· {probe()[kind]}</span>
                                    </div>
                                </Show>
                            </div>
                        );
                    }}
                </For>

                <p class="text-xs text-gray-500">
                    连续覆盖上传后版本号会变化，展示端据此重新拉取，不会继续用旧字体。
                </p>
            </section>
        </div>
    );
}

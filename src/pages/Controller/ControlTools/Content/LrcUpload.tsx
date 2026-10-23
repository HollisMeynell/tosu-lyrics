// 功能: 面板-歌词内容（上传 .lrc 歌词文件）
import { Show, createSignal, onMount } from "solid-js";
import { Button, DropZone } from "@/components/ui";
import { ApiError } from "@/services/settingsService";
import { fetchStatus } from "@/services/statusService";
import { uploadLrc } from "@/services/uploadService";

export default function LrcUpload() {
    const [songTitle, setSongTitle] = createSignal<string | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);

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

    const uploadLrcFile = async (file: File) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            const result = await uploadLrc(file);
            await refreshSong();
            setNotice(`已上传并应用：${file.name}（${result.lines} 行）`);
        } catch (err) {
            setError(
                `LRC 上传失败：${err instanceof ApiError ? err.message : String(err)}`
            );
        } finally {
            setBusy(false);
        }
    };

    onMount(() => {
        void refreshSong();
    });

    return (
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
            <DropZone
                hint="把 .lrc 文件拖到这里"
                accept=".lrc,.txt,text/plain"
                disabled={busy()}
                onFile={(f) => void uploadLrcFile(f)}
            />
            <p class="text-xs text-gray-500">
                不支持的文件不会替换当前歌词；上传成功后展示端立即生效。
            </p>
        </section>
    );
}

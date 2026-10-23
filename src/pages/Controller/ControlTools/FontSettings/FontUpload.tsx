// 功能: 面板-字体设置（字体库上传与列表）
import { Show, createSignal, onMount, For } from "solid-js";
import { Button, DropZone } from "@/components/ui";
import { ApiError } from "@/services/settingsService";
import {
    fetchFontList,
    uploadFontFile,
    type FontEntry,
} from "@/services/uploadService";
import { loadFont, setFontEntries } from "@/utils/fonts.ts";

export default function FontUpload() {
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);
    const [fonts, setFonts] = createSignal<FontEntry[]>([]);

    const refreshFonts = async () => {
        try {
            const items = await fetchFontList();
            setFonts(items);
            // 同步字体选择器的数据源，刷新后下拉里立即能看到新字体
            setFontEntries(items);
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
        }
    };

    const uploadFont = async (file: File) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            const entry = await uploadFontFile(file);
            await refreshFonts();
            // 让新字体立刻生效：重新注册 FontFace
            await loadFont();
            setNotice(
                `字体已上传：${entry.name}（${entry.fileName}，${Math.round(entry.size / 1024)} KB）`
            );
        } catch (err) {
            setError(
                `字体上传失败：${err instanceof ApiError ? err.message : String(err)}`
            );
        } finally {
            setBusy(false);
        }
    };

    onMount(() => {
        void refreshFonts();
    });

    return (
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
                hint="把字体文件拖到这里"
                accept=".ttf,.otf,.woff,.woff2"
                disabled={busy()}
                onFile={(f) => void uploadFont(f)}
            />
            <p class="text-xs text-gray-500">
                上传后会自动识别字体名称并保存，可在上方直接选用。
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
    );
}

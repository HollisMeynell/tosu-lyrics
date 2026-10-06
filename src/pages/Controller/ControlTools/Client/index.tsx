import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { Button, ColorSelector, Select } from "@/components/ui";
import { ApiError } from "@/services/settingsService";
import {
    DisplayClient,
    applyClientSettings,
    blinkAll,
    blinkClient,
    clientLabel,
    fetchClients,
} from "@/services/clientsService";
import { createSettingsController } from "@/hooks/useSettings";
import type { AlignType } from "@/types/globalTypes";

const POLL_MS = 4000;

export default function ClientList() {
    const settings = createSettingsController();

    const [clients, setClients] = createSignal<DisplayClient[]>([]);
    const [selected, setSelected] = createSignal<string>("");
    const [loading, setLoading] = createSignal(false);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [notice, setNotice] = createSignal<string | null>(null);

    const [draft, setDraft] = createSignal({
        first: "#ffffff",
        second: "#e0e0e0",
        sizeFirst: 3,
        sizeSecond: 2,
    });

    const isGlobal = () => selected() === "";

    const load = async () => {
        setLoading(true);
        try {
            const items = await fetchClients();
            setClients(items);
            if (selected() && !items.some((c) => c.id === selected())) {
                setSelected("");
                setNotice("选中的展示端已离线，已回到全局模式");
            }
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setLoading(false);
        }
    };

    const pick = (id: string) => {
        setError(null);
        setNotice(null);
        if (id === selected()) return;
        setSelected(id);
        const s = settings.settings();
        if (s) {
            setDraft({
                first: s.textColor.first,
                second: s.textColor.second,
                sizeFirst: s.fontSize.first,
                sizeSecond: s.fontSize.second,
            });
        }
    };

    const run = async (fn: () => Promise<string>, failLabel: string) => {
        setBusy(true);
        setError(null);
        setNotice(null);
        try {
            setNotice(await fn());
        } catch (err) {
            setError(`${failLabel}：${err instanceof ApiError ? err.message : String(err)}`);
        } finally {
            setBusy(false);
        }
    };

    const test = () =>
        run(async () => {
            if (isGlobal()) {
                const n = await blinkAll();
                return `已让全部 ${n} 个展示端闪烁`;
            }
            const target = selected();
            const okBlink = await blinkClient(target);
            if (!okBlink) {
                await load();
                return "该展示端已离线，已回到全局模式";
            }
            return `已让「${labelOf(target)}」闪烁`;
        }, "测试失败");

    const applyStyle = (patch: {
        textColor?: { first: string; second: string };
        fontSize?: { first: number; second: number };
    }) =>
        run(async () => {
            if (isGlobal()) {
                const ok = await settings.update(patch);
                if (!ok) throw new Error(settings.error() ?? "保存失败");
                return "已应用到全部展示端（全局设置）";
            }
            const target = selected();
            const result = await applyClientSettings(target, patch);
            if (result.keys.length === 0) return "没有变化";
            return `已只应用到「${labelOf(target)}」：${result.keys.join(", ")}`;
        }, "应用样式失败");

    const labelOf = (id: string) => {
        const found = clients().find((c) => c.id === id);
        return found ? clientLabel(found) : "已离线";
    };

    onMount(() => {
        void settings.load();
        void load();
        const timer = setInterval(() => void load(), POLL_MS);
        onCleanup(() => clearInterval(timer));
    });

    const StyleRow = () => (
        <div class="flex flex-col gap-3 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
            <div class="flex flex-row items-center gap-3 flex-wrap">
                <span class="text-base font-medium">样式调整</span>
                <span
                    class={
                        isGlobal()
                            ? "px-2 py-0.5 text-xs rounded bg-blue-100 dark:bg-blue-900/40 text-blue-800 dark:text-blue-200"
                            : "px-2 py-0.5 text-xs rounded bg-pink-100 dark:bg-pink-900/40 text-pink-800 dark:text-pink-200"
                    }
                >
                    {isGlobal() ? "全局（所有展示端）" : `仅「${labelOf(selected())}」`}
                </span>
                <Show when={!isGlobal()}>
                    <span class="text-xs text-gray-500">
                        即时调整，不保存；该端重连后回到全局设置
                    </span>
                </Show>
            </div>

            <div class="flex flex-row items-center gap-4 flex-wrap text-sm">
                {/* 颜色按钮复用「文字样式」页同一个 ColorSelector（圆形色块），
                    不再用原生 input[type=color]，保证两处视觉一致 */}
                <div class="flex flex-row items-center gap-2">
                    主色
                    <ColorSelector
                        class="min-w-6"
                        value={draft().first}
                        disabled={busy()}
                        onCommit={(value) =>
                            applyStyle({
                                textColor: {
                                    first: value,
                                    second: draft().second,
                                },
                            })
                        }
                    />
                </div>
                <div class="flex flex-row items-center gap-2">
                    副色
                    <ColorSelector
                        class="min-w-6"
                        value={draft().second}
                        disabled={busy()}
                        onCommit={(value) =>
                            applyStyle({
                                textColor: {
                                    first: draft().first,
                                    second: value,
                                },
                            })
                        }
                    />
                </div>
                <label class="flex flex-row items-center gap-2">
                    主字号
                    <Select
                        class="min-w-0 w-24 py-1"
                        clearable={false}
                        value={String(draft().sizeFirst)}
                        disabled={busy()}
                        options={["1", "1.5", "2", "2.5", "3", "3.5", "4", "5", "6"].map(
                            (v) => ({ code: v, name: v })
                        )}
                        onChange={(v) =>
                            applyStyle({
                                fontSize: {
                                    first: Number(v),
                                    second: draft().sizeSecond,
                                },
                            })
                        }
                    />
                </label>
                <label class="flex flex-row items-center gap-2">
                    副字号
                    <Select
                        class="min-w-0 w-24 py-1"
                        clearable={false}
                        value={String(draft().sizeSecond)}
                        disabled={busy()}
                        options={["1", "1.5", "2", "2.5", "3", "3.5", "4", "5", "6"].map(
                            (v) => ({ code: v, name: v })
                        )}
                        onChange={(v) =>
                            applyStyle({
                                fontSize: {
                                    first: draft().sizeFirst,
                                    second: Number(v),
                                },
                            })
                        }
                    />
                </label>
            </div>

            <div class="flex flex-row items-center gap-2 flex-wrap text-sm">
                <span>对齐</span>
                <For each={[
                    { code: "left", name: "左" },
                    { code: "center", name: "中" },
                    { code: "right", name: "右" },
                ]}>
                    {(opt) => (
                        <button
                            class="px-3 py-1 rounded border border-gray-300 dark:border-gray-600 disabled:opacity-40"
                            disabled={busy()}
                            onClick={() =>
                                run(async () => {
                                    const patch = {
                                        alignment: opt.code as AlignType,
                                    };
                                    if (isGlobal()) {
                                        const ok = await settings.update(patch);
                                        if (!ok) throw new Error(settings.error() ?? "保存失败");
                                        return "已应用到全部展示端（全局设置）";
                                    }
                                    await applyClientSettings(selected(), patch);
                                    return `已只应用到「${labelOf(selected())}」`;
                                }, "对齐失败")
                            }
                        >
                            {opt.name}
                        </button>
                    )}
                </For>
                <Button
                    class="px-3 py-1 ml-auto"
                    disabled={busy() || isGlobal()}
                    onClick={() => setSelected("")}
                >
                    回到全局
                </Button>
            </div>

            <p class="text-xs text-gray-500">
                需要更多样式（字体 / 阴影 / 副歌词显隐）时，请在
                <span class="font-medium">全局</span>模式下到对应页面调整；
                本页用于快速预览"某个端现在长什么样"。
            </p>
        </div>
    );

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl inline">在线展示端</h2>
                <p class="text-sm inline text-gray-500">
                    默认全局；选中某个端后可只对它调整与测试
                </p>
            </div>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />

            <Show when={error()}>
                <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                    {error()}
                </div>
            </Show>
            <Show when={notice()}>
                <div class="px-4 py-2 rounded-md bg-blue-100 dark:bg-blue-900/40 text-blue-800 dark:text-blue-200 text-sm w-fit">
                    {notice()}
                </div>
            </Show>

            <div class="flex flex-col gap-4 xl:flex-row xl:items-start">
                {/* 左侧：客户端列表 + 操作按钮 */}
                <div class="flex flex-1 min-w-0 flex-col gap-3">
                    <div class="flex flex-row items-center gap-3 flex-wrap">
                        <span class="text-sm text-gray-500">
                            在线 {clients().length} 个（每 {POLL_MS / 1000} 秒自动刷新）
                        </span>
                        {/* 顺序固定为 [刷新] [测试] */}
                        <Button class="px-3 py-1" onClick={() => void load()} disabled={loading()}>
                            {loading() ? "刷新中..." : "刷新"}
                        </Button>
                        <Button
                            class="px-3 py-1"
                            onClick={test}
                            disabled={busy() || (clients().length === 0 && isGlobal())}
                        >
                            {busy()
                                ? "发送中..."
                                : isGlobal()
                                  ? "测试（全部闪烁）"
                                  : "测试（仅选中端闪烁）"}
                        </Button>
                        <span class="text-sm text-gray-500">
                            {isGlobal()
                                ? "当前：全局模式"
                                : `当前：仅「${labelOf(selected())}」`}
                        </span>
                    </div>

                    <Show when={clients().length === 0 && !loading()}>
                        <p class="text-sm text-gray-500">
                            暂无在线展示端。在浏览器 / OBS 中打开歌词页即可看到它。
                        </p>
                    </Show>

                    <div class="flex flex-col gap-2">
                        <For each={clients()}>
                            {(client) => (
                                <button
                                    class={`flex flex-row items-center gap-3 p-3 rounded-md border text-left transition-colors ${
                                        selected() === client.id
                                            ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                                            : "border-gray-200 dark:border-gray-700"
                                    }`}
                                    onClick={() => pick(client.id)}
                                >
                                    <span class="text-base">{clientLabel(client)}</span>
                                    <Show when={client.identity === null}>
                                        <span class="text-xs text-gray-500">
                                            未自报身份（可用 ?id=xxx 指定）
                                        </span>
                                    </Show>
                                    <span class="text-xs text-gray-500 ml-auto">
                                        连接于 {new Date(client.connectedAt).toLocaleTimeString()}
                                    </span>
                                </button>
                            )}
                        </For>
                    </div>
                </div>

                {/* 右侧：样式调整 */}
                <div class="flex flex-1 min-w-0 flex-col gap-4">
                    <StyleRow />
                </div>
            </div>
        </div>
    );
}

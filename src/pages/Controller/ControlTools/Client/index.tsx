import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { Button } from "@/components/ui";
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
            setError(
                `${failLabel}：${err instanceof ApiError ? err.message : String(err)}`
            );
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
                            在线 {clients().length} 个（每 {POLL_MS / 1000}{" "}
                            秒自动刷新）
                        </span>
                        {/* 顺序固定为 [刷新] [测试] */}
                        <Button
                            class="px-3 py-1"
                            onClick={() => void load()}
                            disabled={loading()}
                        >
                            {loading() ? "刷新中..." : "刷新"}
                        </Button>
                        <Button
                            class="px-3 py-1"
                            onClick={test}
                            disabled={
                                busy() || (clients().length === 0 && isGlobal())
                            }
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
                            暂无在线展示端。在浏览器 / OBS
                            中打开歌词页即可看到它。
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
                                    <span class="text-base">
                                        {clientLabel(client)}
                                    </span>
                                    <Show when={client.identity === null}>
                                        <span class="text-xs text-gray-500">
                                            未自报身份（可用 ?id=xxx 指定）
                                        </span>
                                    </Show>
                                    <span class="text-xs text-gray-500 ml-auto">
                                        连接于{" "}
                                        {new Date(
                                            client.connectedAt
                                        ).toLocaleTimeString()}
                                    </span>
                                </button>
                            )}
                        </For>
                    </div>
                </div>
            </div>
        </div>
    );
}

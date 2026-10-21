import { ColorSelector } from "@/components/ui";
import { createSignal, For, Show, onMount } from "solid-js";
import { shadow } from "@/stores/settingsStore";
import { createSettingsController } from "@/hooks/useSettings";
import type { Shadow } from "@/types/globalTypes";

const KIND_LABEL = { first: "主歌词", second: "副歌词" } as const;
type Kind = keyof typeof KIND_LABEL;

const OFFSET_MIN = -20;
const OFFSET_MAX = 20;
const BLUR_MAX = 30;

export default function ShadowPage() {
    const settings = createSettingsController();

    const value = (kind: Kind): Shadow => shadow()[kind];

    const commit = (kind: Kind, next: Partial<Shadow>) => {
        const merged = { ...value(kind), ...next };
        void settings.update({
            shadow:
                kind === "first"
                    ? { first: merged, second: value("second") }
                    : { first: value("first"), second: merged },
        });
    };

    const toggleEnable = (kind: Kind, enabled: boolean) =>
        commit(kind, { enable: enabled });

    onMount(() => void settings.load());

    const KindEditor = (props: { kind: Kind }) => {
        const v = () => value(props.kind);
        return (
            <div class="flex flex-col gap-3 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="flex flex-row items-center gap-3 flex-wrap">
                    <h3 class="text-xl font-normal">
                        {KIND_LABEL[props.kind]}
                    </h3>
                    <button
                        class={`px-3 py-1 min-h-11 select-none rounded border transition-colors ${
                            v().enable
                                ? "border-red-400 text-red-600 dark:text-red-400"
                                : "border-green-500 text-green-700 dark:text-green-400"
                        }`}
                        disabled={settings.saving()}
                        onClick={() => toggleEnable(props.kind, !v().enable)}
                    >
                        {v().enable ? "关闭阴影" : "开启阴影"}
                    </button>
                    <span class="text-xs text-gray-500">
                        {v().enable ? "当前：已启用" : "当前：已关闭"}
                    </span>
                    <Show when={v().enable}>
                        <label class="flex flex-row items-center gap-2 text-sm ml-auto">
                            颜色
                            <ColorSelector
                                value={v().color}
                                disabled={settings.saving()}
                                onCommit={(value) =>
                                    commit(props.kind, { color: value })
                                }
                            />
                        </label>
                    </Show>
                </div>

                <Show when={v().enable}>
                    <div class="flex flex-col gap-3">
                        <For
                            each={[
                                {
                                    key: "offsetX" as const,
                                    label: "X 偏移",
                                    min: OFFSET_MIN,
                                    max: OFFSET_MAX,
                                },
                                {
                                    key: "offsetY" as const,
                                    label: "Y 偏移",
                                    min: OFFSET_MIN,
                                    max: OFFSET_MAX,
                                },
                                {
                                    key: "blur" as const,
                                    label: "模糊",
                                    min: 0,
                                    max: BLUR_MAX,
                                },
                            ]}
                        >
                            {(slider) => (
                                <div class="flex flex-row items-center gap-3 text-sm">
                                    <span class="w-16 shrink-0">
                                        {slider.label}
                                    </span>
                                    <input
                                        type="range"
                                        class="grow max-w-64 h-11"
                                        min={slider.min}
                                        max={slider.max}
                                        step="1"
                                        value={v()[slider.key]}
                                        disabled={settings.saving()}
                                        onChange={(e) =>
                                            commit(props.kind, {
                                                [slider.key]: Number(
                                                    e.currentTarget.value
                                                ),
                                            })
                                        }
                                    />
                                    <span class="w-14 text-right tabular-nums">
                                        {v()[slider.key]} px
                                    </span>
                                </div>
                            )}
                        </For>
                    </div>
                </Show>
            </div>
        );
    };

    const [expanded, setExpanded] = createSignal(false);

    const summary = () => {
        const pair = shadow();
        const enabled = Number(pair.first.enable) + Number(pair.second.enable);
        if (enabled === 0) return "已关闭";
        return enabled === 2 ? "已开启" : "部分开启";
    };

    return (
        <div class="flex flex-col gap-4">
            <button
                type="button"
                class="-mx-2 flex w-full flex-row items-center gap-3 rounded-md px-2 py-1 text-left cursor-pointer select-none transition-colors hover:bg-gray-100 dark:hover:bg-gray-800/60"
                aria-expanded={expanded()}
                aria-controls="shadow-settings-panel"
                onClick={() => setExpanded((v) => !v)}
            >
                <h2 class="text-xl font-normal">阴影</h2>
                <Show when={!settings.loading() && !settings.error()}>
                    <span
                        class={`text-sm ${
                            summary() === "已关闭"
                                ? "text-gray-500"
                                : "text-green-600 dark:text-green-400"
                        }`}
                    >
                        {summary()}
                    </span>
                </Show>
                <span class="ml-auto flex flex-row items-center gap-1 text-sm text-gray-500">
                    {expanded() ? "收起" : "展开"}
                    <svg
                        class="size-4 transition-transform duration-300 motion-reduce:transition-none"
                        classList={{ "rotate-90": expanded() }}
                        viewBox="0 0 16 16"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.5"
                        aria-hidden="true"
                    >
                        <path
                            d="m6 3 5 5-5 5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </span>
            </button>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />

            <div
                id="shadow-settings-panel"
                inert={!expanded()}
                class={`grid transition-[grid-template-rows] duration-300 ease-out motion-reduce:transition-none ${
                    expanded() ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
                }`}
            >
                <div class="min-h-0 overflow-hidden">
                    <div class="flex flex-col gap-4 pb-1">
                        <Show when={settings.error()}>
                            <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                                修改未生效（已保留原值）：{settings.error()}
                            </div>
                        </Show>

                        <Show
                            when={!settings.loading()}
                            fallback={
                                <p class="text-gray-500">正在读取设置…</p>
                            }
                        >
                            <KindEditor kind="first" />
                            <KindEditor kind="second" />
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    );
}

import { For, Show, onMount } from "solid-js";
import { DEFAULT_SHADOW, shadow } from "@/stores/settingsStore";
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
                    <h3 class="text-xl font-normal">{KIND_LABEL[props.kind]}</h3>
                    <button
                        class={`px-3 py-1 rounded border transition-colors ${
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
                            <input
                                type="color"
                                value={v().color}
                                disabled={settings.saving()}
                                onChange={(e) =>
                                    commit(props.kind, {
                                        color: e.currentTarget.value,
                                    })
                                }
                            />
                        </label>
                    </Show>
                </div>

                <Show when={v().enable}>
                    <div class="flex flex-col gap-3">
                        <For
                            each={[
                                { key: "offsetX" as const, label: "X 偏移", min: OFFSET_MIN, max: OFFSET_MAX },
                                { key: "offsetY" as const, label: "Y 偏移", min: OFFSET_MIN, max: OFFSET_MAX },
                                { key: "blur" as const, label: "模糊", min: 0, max: BLUR_MAX },
                            ]}
                        >
                            {(slider) => (
                                <div class="flex flex-row items-center gap-3 text-sm">
                                    <span class="w-16 shrink-0">{slider.label}</span>
                                    <input
                                        type="range"
                                        class="grow max-w-64"
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

                    <div
                        class="px-4 py-2 rounded-md bg-gray-100 dark:bg-gray-800 text-2xl"
                        style={{
                            filter: `drop-shadow(${v().offsetX}px ${v().offsetY}px ${v().blur}px ${v().color})`,
                        }}
                    >
                        预览文字 Preview
                    </div>
                </Show>
            </div>
        );
    };

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-2xl font-medium inline">阴影</h2>
                <p class="text-sm inline text-gray-500">
                    主 / 副独立；**改动立即生效**，无需保存，所有展示端同步
                </p>
            </div>
            <hr class="w-24 border-gray-400 dark:border-gray-600" />

            <Show when={settings.error()}>
                <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm w-fit">
                    修改未生效（已保留原值）：{settings.error()}
                </div>
            </Show>

            <Show
                when={!settings.loading()}
                fallback={<p class="text-gray-500">正在读取设置…</p>}
            >
                <KindEditor kind="first" />
                <KindEditor kind="second" />
                <p class="text-xs text-gray-500">
                    取值范围：偏移 ±{OFFSET_MAX} px，模糊 0~{BLUR_MAX} px。
                    默认值：关闭 / {DEFAULT_SHADOW.offsetX}px {DEFAULT_SHADOW.offsetY}px
                    / 模糊 {DEFAULT_SHADOW.blur}px。
                </p>
            </Show>
        </div>
    );
}

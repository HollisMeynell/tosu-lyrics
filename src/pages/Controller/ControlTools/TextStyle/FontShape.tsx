// 功能: 面板-文字样式（字形：字重 / 字宽 / 倾斜）
import { Component, createSignal, For } from "solid-js";
import { fontShape } from "@/stores/settingsStore";
import type {
    FontShape as FontShapeValue,
    SettingsPatch,
} from "@/types/globalTypes";

interface FontShapeProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

type ShapeKey = keyof FontShapeValue;

type ShapeSliderDef = {
    key: ShapeKey;
    label: string;
    min: number;
    max: number;
    step: number;
    format: (value: number) => string;
    title: string;
};

const SLIDERS: ShapeSliderDef[] = [
    {
        key: "weight",
        label: "字重",
        min: 100,
        max: 900,
        step: 10,
        format: (value) => String(value),
        title: "字重：数值越大越粗",
    },
    {
        key: "width",
        label: "字宽",
        min: 50,
        max: 200,
        step: 5,
        format: (value) => `${value}%`,
        title: "字宽：大于 100% 变宽，小于 100% 变窄",
    },
    {
        key: "slant",
        label: "倾斜",
        min: -20,
        max: 20,
        step: 1,
        format: (value) => `${value}°`,
        title: "倾斜：正数向右斜，负数向左斜",
    },
];

/** 拖动只更新本地预览，松手（change）才提交 */
const ShapeSlider: Component<{
    slider: ShapeSliderDef;
    value: number;
    disabled?: boolean;
    onCommit: (value: number) => void;
}> = (props) => {
    const [preview, setPreview] = createSignal<number | undefined>(undefined);
    const shown = () => preview() ?? props.value;

    return (
        <div class="flex flex-row items-center gap-2">
            <span class="text-sm text-gray-500 w-10 shrink-0">
                {props.slider.label}
            </span>
            <input
                type="range"
                class="w-24 md:w-28 accent-[#ec4899] cursor-pointer disabled:cursor-not-allowed"
                title={props.slider.title}
                min={props.slider.min}
                max={props.slider.max}
                step={props.slider.step}
                value={shown()}
                disabled={props.disabled}
                onInput={(e) => setPreview(Number(e.currentTarget.value))}
                onChange={(e) => {
                    const value = Number(e.currentTarget.value);
                    setPreview(undefined);
                    props.onCommit(value);
                }}
            />
            <span class="text-sm text-gray-500 w-12 tabular-nums">
                {props.slider.format(shown())}
            </span>
        </div>
    );
};

export default function FontShapeEditor(props: FontShapeProps) {
    const [expanded, setExpanded] = createSignal(false);

    const commit = (
        order: "first" | "second",
        key: ShapeKey,
        value: number
    ) => {
        void props.update({
            fontShape: {
                ...fontShape(),
                [order]: { ...fontShape()[order], [key]: value },
            },
        });
    };

    return (
        <div class="flex flex-col gap-4">
            <button
                type="button"
                class="-mx-2 flex w-full flex-row items-center gap-3 rounded-md px-2 py-1 text-left cursor-pointer select-none transition-colors hover:bg-gray-100 dark:hover:bg-gray-800/60"
                aria-expanded={expanded()}
                aria-controls="font-shape-panel"
                onClick={() => setExpanded((v) => !v)}
            >
                <h2 class="text-xl font-normal">字形</h2>
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
                id="font-shape-panel"
                inert={!expanded()}
                class={`grid transition-[grid-template-rows] duration-300 ease-out motion-reduce:transition-none ${
                    expanded() ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
                }`}
            >
                <div class="min-h-0 overflow-hidden">
                    <div class="flex flex-col gap-3 pb-1">
                        <For each={["first", "second"] as const}>
                            {(order) => (
                                <div class="flex flex-row items-center gap-3 flex-wrap">
                                    <p>
                                        {order === "first"
                                            ? "主歌词:"
                                            : "副歌词:"}
                                    </p>
                                    <For each={SLIDERS}>
                                        {(slider) => (
                                            <ShapeSlider
                                                slider={slider}
                                                value={
                                                    fontShape()[order][
                                                        slider.key
                                                    ]
                                                }
                                                disabled={props.disabled}
                                                onCommit={(value) =>
                                                    commit(
                                                        order,
                                                        slider.key,
                                                        value
                                                    )
                                                }
                                            />
                                        )}
                                    </For>
                                </div>
                            )}
                        </For>
                        <p class="text-xs text-gray-500">
                            字重、字宽需要字体本身支持无级调节（常见于可变字体），普通字体可能没有变化；倾斜由浏览器模拟斜切，所有字体都有效。
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}

import { For, Show, createSignal, onCleanup, onMount } from "solid-js";

export interface FontPickerOption {
    code: string;
    name: string;
}

interface FontPickerProps {
    class?: string;
    options: FontPickerOption[];
    value: string;
    disabled?: boolean;
    onChange: (value: string) => void;
}

/**
 * 字体选择器（自绘下拉）。
 *
 * **为什么替换掉原生 `<select>`**：字体选择框的文字会随机跳成微软雅黑，根因不在
 * 我们的 CSS，而在原生控件本身：
 *  1. Tailwind preflight 只给 `button,input,select,optgroup,textarea` 设了
 *     `font: inherit`，**`<option>` 不在其中**，因此选项文字使用的是浏览器/系统的
 *     表单控件默认字体（中文 Windows 上即微软雅黑）；
 *  2. Windows 上 Chromium 的原生下拉弹层由**系统菜单**渲染，任何 CSS（包括内联
 *     `font-family`）都无法改变它。
 * 所以之前"给 `<select>` 加 font-family"治不了本。
 *
 * 自绘之后，收起态与展开态都是普通 DOM 元素，UI 字体稳定生效；歌词的动态字体只
 * 作用在歌词元素上，二者从结构上隔离。
 */
export default function FontPicker(props: FontPickerProps) {
    const [open, setOpen] = createSignal(false);
    let root: HTMLDivElement | undefined;

    const label = () =>
        props.options.find((option) => option.code === props.value)?.name ??
        props.options[0]?.name ??
        "";

    const onDocMouseDown = (event: MouseEvent) => {
        if (root && !root.contains(event.target as Node)) setOpen(false);
    };
    onMount(() => document.addEventListener("mousedown", onDocMouseDown));
    onCleanup(() => document.removeEventListener("mousedown", onDocMouseDown));

    const pick = (code: string) => {
        setOpen(false);
        if (code !== props.value) props.onChange(code);
    };

    return (
        <div ref={root} class={`relative select-none ${props.class ?? ""}`}>
            <button
                type="button"
                disabled={props.disabled}
                onClick={() => setOpen((v) => !v)}
                class="w-full min-w-48 p-2 pl-3 pr-8 border border-[#cbd5e1] rounded-lg shadow-xs bg-white text-left hover:border-[#94a3b8] focus:outline-hidden focus:ring-1 focus:ring-[#eb4898] transition-colors cursor-pointer dark:bg-[#020616] dark:border-[#475569] dark:text-white disabled:opacity-50 disabled:cursor-not-allowed"
            >
                <span class="block truncate">{label()}</span>
                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[#94a3b8] pointer-events-none">
                    ▾
                </span>
            </button>

            <Show when={open()}>
                <div class="absolute z-50 mt-1 w-full max-h-64 overflow-y-auto rounded-lg border border-[#cbd5e1] bg-white shadow-lg dark:bg-[#020616] dark:border-[#475569]">
                    <For each={props.options}>
                        {(option) => (
                            <div
                                class={`px-3 py-2 cursor-pointer truncate ${
                                    option.code === props.value
                                        ? "bg-pink-50 text-[#ec4899] dark:bg-pink-900/20"
                                        : "hover:bg-gray-100 dark:hover:bg-gray-800"
                                }`}
                                onClick={() => pick(option.code)}
                            >
                                {option.name}
                            </div>
                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}

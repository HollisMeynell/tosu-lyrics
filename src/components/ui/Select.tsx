// 功能: 面板-文字样式（颜色、字体、显示效果）
import { For, createMemo } from "solid-js";

export interface Option {
    code: string;
    name: string;
    placeholder?: boolean;
}

interface SelectProps {
    class?: string;
    options: Option[];
    placeholder?: string;
    blankOption?: Option | boolean;
    value: string;
    onChange: (value: string) => void;
}

export default function Select(props: SelectProps) {
    // 向传入的选项组添加空选项用于清除选择
    const wrappedOptions = createMemo(() => {
        let options = [...props.options];

        // 如果 options 为空
        if (options.length === 0) {
            if (props.placeholder) {
                // 添加占位符选项
                options.push({
                    code: "",
                    name: props.placeholder,
                    placeholder: true,
                });
            } else {
                // 添加默认占位符选项
                options.push({ code: "", name: "无选项" });
            }
        } else {
            // 如果 options 不为空
            if (props.blankOption && typeof props.blankOption === "object") {
                // 添加传入的 blankOption
                options.unshift(props.blankOption);
            } else if (props.blankOption) {
                // 添加默认的空选项
                options.unshift({ code: "", name: "无" });
            }
        }

        return options;
    });

    // 清除选择
    const clearSelection = () => {
        props.onChange("");
    };

    return (
        <div class="relative select-none">
            <select
                value={props.value}
                id="dd-language"
                class={`min-w-48 p-2 pl-3 pr-8 border border-[#cbd5e1] rounded-lg shadow-xs appearance-none bg-white focus:outline-hidden focus:ring-1 focus:ring-[#eb4898] focus:border-[#eb4898] hover:border-[#94a3b8] transition-colors cursor-pointer dark:bg-[#020616] dark:border-[#475569] dark:text-white dark:focus:border-[#e169a8] dark:hover:border-[#64748b] ${props.class}`}
                onChange={(e) => {
                    props.onChange(e.target.value);
                }}
            >
                <For each={wrappedOptions()}>
                    {(option) => (
                        <option value={option.code} hidden={option.placeholder}>
                            {option.name}
                        </option>
                    )}
                </For>
            </select>
            <button
                onClick={clearSelection}
                class="absolute right-2 top-1/2 transform -translate-y-1/2 cursor-pointer"
            >
                <span class="text-xl font-[100] text-[#94a3b8] leading-none">
                    ×
                </span>
            </button>
        </div>
    );
}

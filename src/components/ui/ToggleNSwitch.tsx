import { Component, For, createSignal, onMount } from "solid-js";

interface Option {
    key: string;
    value: string;
}

interface ToggleNSwitchProps {
    options: Option[];
    selectedValue?: string;
    disabled?: boolean;
    className?: string;
    onUpdateSelectedValue?: (value: string) => void;
}

const ToggleNSwitch: Component<ToggleNSwitchProps> = (props) => {
    const [selectedValue, setSelectedValue] = createSignal(props.selectedValue || props.options[0].value);
    const [sliderWidth, setSliderWidth] = createSignal(0);
    const [sliderOffset, setSliderOffset] = createSignal(0);

    let containerRef: HTMLDivElement | null = null;
    let itemRefs: { [key: string]: HTMLDivElement } = {};

    // 更新滑块的位置和宽度
    const updateSlider = (value: string) => {
        const selectedItem = itemRefs[value];
        if (selectedItem && containerRef) {
            const containerRect = containerRef.getBoundingClientRect();
            const itemRect = selectedItem.getBoundingClientRect();
            const offset = itemRect.left - containerRect.left;
            const width = itemRect.width;

            setSliderWidth(width);
            setSliderOffset(offset);
        }
    };

    // 初始化或更新选中值时调用
    onMount(() => {
        updateSlider(selectedValue());
    });

    // 点击项时切换选中值
    const handleItemClick = (option: Option) => {
        if (!props.disabled) {
            setSelectedValue(option.value);
            props.onUpdateSelectedValue?.(option.value);
            updateSlider(option.value);
        }
    };

    return (
        <div
            ref={(el) => (containerRef = el)}
            class={[
                "relative h-[2.8rem] rounded-full transition-colors duration-200 cursor-pointer flex items-center gap-2 px-2",
                "bg-[#cbd5e1] hover:bg-[#adbfd7] dark:bg-[#21314d] dark:hover:bg-[#2c3c59]",
                props.disabled ? "opacity-50 cursor-not-allowed" : "",
                props.className,
            ].join(" ")}
        >
            {/* 滑块 */}
            <div
                class="absolute left-0 h-[1.9rem] bg-gray-100 dark:bg-gray-300 rounded-3xl transition-all duration-200 shadow-sm"
                style={{
                    width: `${sliderWidth()}px`,
                    transform: `translateX(${sliderOffset()}px)`,
                }}
            ></div>

            {/* 选项 */}
            <For each={props.options}>
                {(option) => (
                    <div
                        ref={(el) => (itemRefs[option.value] = el)}
                        class={[
                            "w-fit h-full flex justify-center items-center z-10 px-2",
                            option.key === selectedValue() ? "text-gray-900" : "text-gray-500",
                        ].join(" ")}
                        onClick={() => handleItemClick(option)}
                    >
                        {option.key}
                    </div>
                )}
            </For>
        </div>
    );
};

export default ToggleNSwitch;
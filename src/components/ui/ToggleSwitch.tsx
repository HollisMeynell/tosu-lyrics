import { Component } from "solid-js";

interface ToggleSwitchProps {
    modelValue?: boolean;
    disabled?: boolean;
    class?: string;
    onUpdateModelValue?: (value: boolean) => void;
}

const ToggleSwitch: Component<ToggleSwitchProps> = (props) => {
    const toggle = () => {
        if (!props.disabled) {
            const newValue = !props.modelValue;
            props.onUpdateModelValue?.(newValue);
        }
    };

    return (
        <div
            class={[
                "relative w-[2.4rem] h-[1.45rem] rounded-full transition-colors duration-200 cursor-pointer flex items-center",
                props.modelValue
                    ? "bg-[#ec4899] hover:bg-[#db2777]"
                    : "bg-[#cbd5e1] hover:bg-[#94a3b8] dark:bg-[#21314d] dark:hover:bg-[#3b4a63]",
                props.disabled ? "opacity-50 cursor-not-allowed" : "",
                props.class,
            ].join(" ")}
            onClick={toggle}
        >
            <div
                class={[
                    "absolute left-[0.2rem] w-[0.95rem] h-[0.95rem] bg-white rounded-full transition-transform duration-200",
                    props.modelValue ? "translate-x-[0.95rem]" : "",
                ].join(" ")}
            ></div>
        </div>
    );
};

export default ToggleSwitch;

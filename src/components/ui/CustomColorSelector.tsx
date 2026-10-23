import { Component, createSignal, onCleanup, onMount } from "solid-js";
import Coloris from "@melloware/coloris";
import "@melloware/coloris/dist/coloris.css";
import "./CustomColorSelector.css";

interface CustomColorSelectorProps {
    class?: string;
    value: string;
    disabled?: boolean;
    onCommit: (value: string) => void;
}

let initialized = false;

const CustomColorSelector: Component<CustomColorSelectorProps> = (props) => {
    let input!: HTMLInputElement;
    let opened = false;
    let disposed = false;
    const [preview, setPreview] = createSignal<string>();
    const shown = () => preview() ?? props.value;

    onMount(() => {
        if (!initialized) {
            Coloris.init();
            Coloris({
                el: ".lyrics-color-input",
                wrap: false,
                format: "hex",
                formatToggle: false,
                alpha: true,
                themeMode: "light",
                focusInput: false,
                closeButton: true,
                closeLabel: "完成",
            });
            initialized = true;
        }

        const handleOpen = () => {
            opened = true;
        };
        const handleClose = () => {
            opened = false;
            const value = input.value;
            setPreview(undefined);
            input.value = props.value;
            if (!disposed && !props.disabled && value !== props.value) {
                props.onCommit(value);
            }
        };
        input.addEventListener("open", handleOpen);
        input.addEventListener("close", handleClose);
        onCleanup(() => {
            disposed = true;
            if (opened) Coloris.close(true);
            input.removeEventListener("open", handleOpen);
            input.removeEventListener("close", handleClose);
        });
    });

    return (
        <div class={`w-6 h-6 relative ${props.class ?? ""}`}>
            <input
                ref={input}
                type="text"
                readonly
                aria-label="选择颜色和不透明度"
                class="lyrics-color-input absolute -inset-2 opacity-0 cursor-pointer z-10 disabled:cursor-not-allowed"
                value={shown()}
                disabled={props.disabled}
                onInput={(e) => setPreview(e.currentTarget.value)}
                onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        input.click();
                    }
                }}
            />
            <div class="absolute inset-0 rounded-full border-[1.5px] border-[#e5e7eb] dark:border-[#475569] pointer-events-none overflow-hidden lyrics-color-swatch">
                <div
                    class="w-full h-full"
                    style={{ "background-color": shown() }}
                />
            </div>
        </div>
    );
};

export default CustomColorSelector;

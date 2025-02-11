// 功能: 面板-文字样式（颜色、字体、显示效果）
import store from "@/stores/indexStore";
import { Component } from "solid-js";

interface customColorSelectorProps {
    className?: string;
    colorType: "first" | "second";
}

const customColorSelector: Component<customColorSelectorProps> = (props) => {
    return (
        <div class={`w-6 h-6 relative ${props.className}`}>
            <input
                type="color"
                class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10"
                value={store.getState.textColor[props.colorType]}
                onInput={(e) =>
                    store.setTextColor(props.colorType, e.currentTarget.value)
                }
                onChange={() => store.sendColorConfig()}
            />
            <div
                class="color-mask absolute top-0 left-0 w-full h-full rounded-full border-[1.5px] border-[#e5e7eb] dark:border-[#475569] z-0 pointer-events-none"
                style={{
                    "background-color": store.getState.textColor[props.colorType],
                }}
            ></div>
        </div>
    );
}

export default customColorSelector;

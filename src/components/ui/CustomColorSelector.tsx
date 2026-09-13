// 功能: 面板-文字样式（颜色、字体、显示效果）
//
// 受控组件：值由父组件传入，提交时机由父组件决定。
// 拖动取色器时只更新本地预览，松手（change）才提交，避免连续发 HTTP 请求；
// 若提交失败，父组件不会改动数据源，本地预览会被清掉从而回到服务端值。
import { Component, createSignal } from "solid-js";

interface customColorSelectorProps {
    class?: string;
    value: string;
    disabled?: boolean;
    /** 松手时提交最终颜色 */
    onCommit: (value: string) => void;
}

const customColorSelector: Component<customColorSelectorProps> = (props) => {
    const [preview, setPreview] = createSignal<string | undefined>(undefined);
    const shown = () => preview() ?? props.value;

    return (
        <div class={`w-6 h-6 relative ${props.class ?? ""}`}>
            <input
                type="color"
                class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10 disabled:cursor-not-allowed"
                value={shown()}
                disabled={props.disabled}
                onInput={(e) => setPreview(e.currentTarget.value)}
                onChange={(e) => {
                    const value = e.currentTarget.value;
                    setPreview(undefined);
                    props.onCommit(value);
                }}
            />
            <div
                class="color-mask absolute top-0 left-0 w-full h-full rounded-full border-[1.5px] border-[#e5e7eb] dark:border-[#475569] z-0 pointer-events-none"
                style={{ "background-color": shown() }}
            ></div>
        </div>
    );
};

export default customColorSelector;

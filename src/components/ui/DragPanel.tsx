import { createSignal, Component } from "solid-js";

interface DragPanelProps {
    class?: string;
    items: string[];
}

const DragPanel: Component<DragPanelProps> = (props) => {
    const [leftWidth, setLeftWidth] = createSignal(50); // 初始左侧宽度
    const [rightWidth, setRightWidth] = createSignal(50); // 初始右侧宽度

    const startDrag = (e: MouseEvent) => {
        e.preventDefault();
        const initialX = e.clientX;
        const initialLeftWidth = leftWidth();
        const initialRightWidth = rightWidth();

        const onMouseMove = (e: MouseEvent) => {
            const deltaX = e.clientX - initialX;
            const newLeftWidth = initialLeftWidth + deltaX;
            const newRightWidth = initialRightWidth - deltaX;

            // 限制左侧最小宽度和右侧最小宽度
            if (newLeftWidth >= 20 && newRightWidth >= 20) {
                setLeftWidth(newLeftWidth);
                setRightWidth(newRightWidth);
            }
        };

        const onMouseUp = () => {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
        };

        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", onMouseUp);
    };

    console.log("props.items", props.items);

    return (
        <div class={["px-3 h-full", props.class].join(" ")}>
            <div class="splitter flex h-full w-full overflow-hidden">
                <div
                    class="left-panel flex-none flex items-center overflow-hidden text-ellipsis text-nowrap"
                    style={{ flex: `0 0 ${leftWidth()}px` }}
                >
                    {props.items[0]}
                </div>
                <div
                    class="divider w-2 flex-none bg-gray-200 dark:bg-gray-700 cursor-col-resize"
                    onMouseDown={startDrag}
                ></div>
                <div class="right-panel flex-1 flex items-center overflow-hidden min-w-[20px] text-ellipsis text-nowrap">
                    {props.items[1]}
                </div>
            </div>
        </div>
    );
};

export default DragPanel;

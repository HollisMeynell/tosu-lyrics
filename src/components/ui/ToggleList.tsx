import { createSignal, JSX, createEffect } from "solid-js";

interface ToggleListProps {
    header: JSX.Element;
    children: JSX.Element;
}

const ToggleList = (props: ToggleListProps) => {
    const [active, setActive] = createSignal<boolean>(false);
    const [maxHeight, setMaxHeight] = createSignal<string>("0px");
    let panelContentRef: HTMLDivElement | undefined;

    // 切换展开/收起面板
    const togglePanel = (e: MouseEvent) => {
        // 检查事件源是否为header本身
        if (e.currentTarget === e.target) {
            setActive(!active());
        }
    };

    createEffect(() => {
        if (active() && panelContentRef) {
            // 获取内容的高度
            const height = panelContentRef.scrollHeight;
            setMaxHeight(`${height}px`);
        } else {
            setMaxHeight("0px");
        }
    });

    return (
        <div class="rounded-md overflow-hidden flex flex-col justify-center items-center w-[650px] mb-2">
            <div
                onClick={togglePanel}
                class="panel-header w-[650px] text-left p-4 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors rounded-md cursor-pointer flex items-center justify-between z-10"
            >
                {props.header}
            </div>

            <div
                ref={panelContentRef}
                class="panel-content overflow-hidden transition-all duration-500 ease-in-out pb-2 -translate-y-2"
                style={{ "max-height": maxHeight() }}
            >
                <div class="p-4 mx-auto w-[94%] bg-white dark:bg-gray-800 rounded-b-lg shadow-md">
                    {props.children}
                </div>
            </div>
        </div>
    );
};

export default ToggleList;

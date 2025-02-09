import { createSignal, JSX, createEffect, For, Show } from "solid-js";

interface Page {
    name: string;
    content: JSX.Element;
}

interface ToggleListWithPages {
    pages: Page[];
    header: JSX.Element;
    children?: never; // 确保 children 不能传入
}

interface ToggleListWithChildren {
    pages?: never; // 确保 pages 不能传入
    header: JSX.Element;
    children: JSX.Element;
}

type ToggleListProps = ToggleListWithPages | ToggleListWithChildren; // 交叉类型

const ToggleListExtends = (props: ToggleListProps) => {
    const [selectedPage, setSelectedPage] = createSignal<string>(props.pages?.[0].name || "");
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

    const selectThisPage = (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        setSelectedPage(target.innerText);
        console.log("selectedPage", selectedPage());
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
        <div class="rounded-md overflow-hidden flex flex-col justify-center items-center max-w-[630px]">
            <div
                onClick={togglePanel}
                class="panel-header w-full text-left p-4 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors rounded-md cursor-pointer flex items-center justify-between z-10 shadow-sm"
            >
                <div class="flex flex-row items-center h-[4rem] my-[-1rem]">
                    <For each={props.pages}>
                        {(page) => (
                            <button
                                class={[
                                    `${
                                        selectedPage() === page.name
                                            ? "text-[#ec4899] font-bold inset-shadow-sm"
                                            : ""
                                    }`,
                                    "hover:text-[#ec489f] hover:inset-shadow-xs transition-colors ease-in-out duration-300",
                                    "bg-[#ebedf1] dark:bg-gray-800",
                                    "h-full my-[-1rem] inset-shadow-2xs px-2 min-w-18 dark:inset-shadow-[#33455f]",
                                ].join(" ")}
                                onClick={selectThisPage}
                            >
                                {page.name}
                            </button>
                        )}
                    </For>
                </div>
                <div>{props.header}</div>
            </div>

            <div
                ref={panelContentRef}
                class="panel-content w-full overflow-hidden transition-all duration-500 ease-in-out pb-2 -translate-y-2"
                style={{ "max-height": maxHeight() }}
            >
                <div class="p-4 mx-auto w-[94%] bg-white dark:bg-gray-800 rounded-b-lg shadow-md">
                    {props.children}
                    <For each={props.pages}>
                        {(page) => (
                            <Show when={selectedPage() === page.name}>
                                {page.content}
                            </Show>
                        )}
                    </For>
                </div>
            </div>
        </div>
    );
};

export default ToggleListExtends;

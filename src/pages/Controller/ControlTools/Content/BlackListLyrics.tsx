import { getTitleBlackList, lyricsStore } from "@/stores/lyricsStore.ts";
import { Component, createSignal, For } from "solid-js";
import { wsService } from "@/services/webSocketService";
import Refresh from "@/assets/Icons/Refresh.tsx";
import ShutDown from "@/assets/Icons/ShutDown.tsx";

const BlackListItem: Component<{ title: string }> = (props) => {
    const { title } = props;
    const deleteThis = () => {
        lyricsStore.deleteTitleBlackList(title);
    };
    return (
        <div class={"mt-2 mb-2"}>
            <ShutDown
                class="mr-2 w-6 h-6 inline cursor-pointer "
                onClick={deleteThis}
            />
            {title}
        </div>
    );
};
export default function BlackListLyrics() {
    const [nowTitle, setNowTitle] = createSignal("");

    const addBlackList = async () => {
        if (!nowTitle()) return;
        lyricsStore.addTitleBlackList(nowTitle());
    };

    const refresh = async (e: { target: { tagName: string } }) => {
        // 仅按钮生效
        if (e.target.tagName !== "svg") return;
        const name = await wsService.defaultClient?.getNowTitle();
        setNowTitle(name || "");
    };

    const saveBlackList = () => {
        lyricsStore.asyncTitleBlackList();
    };

    return (
        <>
            <h3 class="text-2xl leading-10 mb-6" onClick={refresh}>
                <Refresh
                    class="w-10 h-10 inline cursor-pointer mr-6 select-none active:scale-90
                    active:rotate-270 transition-transform duration-200 ease-in-out"
                />
                {nowTitle() || "暂无信息"}
            </h3>
            <button
                class="bg-gray-50 border border-gray-300 text-gray-900 mr-2
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400
                dark:text-whitedark:focus:ring-blue-500 dark:focus:border-blue-500 mt-2 mb-2"
                onClick={addBlackList}
            >
                拉黑当前
            </button>
            <button
                class="bg-gray-50 border border-gray-300 text-gray-900 mr-2
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400
                dark:text-whitedark:focus:ring-blue-500 dark:focus:border-blue-500 mt-2 mb-2"
                onClick={saveBlackList}
            >
                永久保存
            </button>
            <br />
            <For each={getTitleBlackList()}>
                {(title) => <BlackListItem title={title} />}
            </For>
        </>
    );
}

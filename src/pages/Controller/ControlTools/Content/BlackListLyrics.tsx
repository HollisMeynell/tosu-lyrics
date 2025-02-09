import { getTitleBlackList, lyricsStore } from "@/stores/lyricsStore.ts";
import { Component, For } from "solid-js";
import { wsService } from "@/services/webSocketService";

const BlackListItem: Component<{ title: string }> = (props) => {
    const { title } = props;
    const deleteThis = () => {
        lyricsStore.deleteTitleBlackList(title);
    };
    return (
        <div class={"mt-4 mb-4"}>
            <p>{title}</p>
            <button
                class="bg-gray-50 border border-gray-300 text-gray-900
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400
                dark:text-whitedark:focus:ring-blue-500 dark:focus:border-blue-500 mt-2 mb-2"
                onClick={deleteThis}
            >
                删除
            </button>
            <br />
        </div>
    );
};
export default function BlackListLyrics() {
    const addBlackList = async () => {
        const name = await wsService.defaultClient?.getNowTitle();
        if (name) {
            lyricsStore.addTitleBlackList(name);
        }
    };

    const saveBlackList = () => {
        lyricsStore.asyncTitleBlackList();
    };

    return (
        <>
            <button
                class="bg-gray-50 border border-gray-300 text-gray-900
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400
                dark:text-whitedark:focus:ring-blue-500 dark:focus:border-blue-500 mt-2 mb-2"
                onClick={addBlackList}
            >
                拉黑当前
            </button>
            <button
                class="bg-gray-50 border border-gray-300 text-gray-900
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400
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

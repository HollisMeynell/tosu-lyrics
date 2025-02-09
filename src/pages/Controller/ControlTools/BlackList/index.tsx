import { getTitleBlackList, lyricsStore } from "@/stores/lyricsStore.ts";
import { Component, createSignal, For } from "solid-js";
import { wsService } from "@/services/webSocketService";
import { Refresh, Delete } from "@/assets/Icons";
import { Button } from "@/components/ui";

const BlackListItem: Component<{ title: string }> = (props) => {
    const { title } = props;
    const deleteThis = () => {
        lyricsStore.deleteTitleBlackList(title);
    };
    return (
        <div class="mt-2 mb-2">
            <Delete
                class="mr-2 w-6 h-6 inline cursor-pointer select-none active:scale-90"
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
        <div class="flex flex-col gap-4">
            {/* Header */}
            <div class="header space-x-4">
                <h2 class="text-2xl inline">黑名单</h2>
                <p class="text-sm inline text-gray-500">
                    被你拉黑的歌曲以后将不会被显示
                </p>
            </div>

            <hr class="w-64 border-gray-400 dark:border-gray-600" />

            {/* Actions */}
            <div class="flex flex-row items-center gap-2">
                <Button onClick={addBlackList}>拉黑当前歌曲</Button>
                <Button onClick={saveBlackList}>永久保存</Button>
            </div>

            {/* BlackList */}
            <h3 class="text-2xl leading-10" onClick={refresh}>
                <Refresh
                    class="w-8 h-8 inline cursor-pointer mr-6 select-none active:scale-90
                    active:rotate-270 transition-transform duration-400 ease-in-out"
                />
                {nowTitle() || "暂无信息"}
            </h3>

            <For each={getTitleBlackList()}>
                {(title) => <BlackListItem title={title} />}
            </For>
        </div>
    );
}

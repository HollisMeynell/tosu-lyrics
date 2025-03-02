import { Component, createSignal } from "solid-js";
import BlacklistComponent from "./BlackList";
import store from "@/stores/indexStore";
import { wsService } from "@/services/webSocketService";
import { Refresh } from "@/assets/Icons";
import { Button } from "@/components/ui";

const BlackListLyrics: Component = () => {
    const [nowTitle, setNowTitle] = createSignal("");

    const addTitleToBlackList = async () => {
        if (!nowTitle()) return;
        store.addTitleToBlackList(nowTitle());
    };

    const refresh = async () => {
        // 仅按钮生效
        const name = await wsService.defaultClient?.getNowTitle();
        console.log(name);
        setNowTitle(name || "");
    };

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-2xl inline">黑名单管理</h2>
                <p class="text-sm inline text-gray-500">
                    被你拉黑的歌曲以后将不会被显示
                </p>
            </div>
            <hr class="w-72 border-gray-400 dark:border-gray-600" />

            <div class="flex flex-row gap-4">
                <Button class="w-48" onClick={addTitleToBlackList}>
                    添加到黑名单
                </Button>
                <button class="text-sm inline leading-10" onClick={refresh}>
                    <Refresh
                        class="w-4 h-4 inline cursor-pointer mr-2 select-none active:scale-90
                    active:rotate-270 transition-transform duration-400 ease-in-out"
                    />
                    当前播放: {nowTitle() || "暂无信息"}
                </button>
            </div>

            {/* 使用黑名单组件 */}
            <BlacklistComponent />
        </div>
    );
};

export default BlackListLyrics;

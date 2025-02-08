import { getTitleBlackList, lyricsStore } from "@/stores/lyricsStore.ts";
import { Component, For } from "solid-js";
import { wsService } from "@/services/WebSocketService.ts";


const BlackListItem: Component<{ title: string }> = (props) => {
    const {title} = props;
    const deleteThis = () => {
        lyricsStore.deleteTitleBlackList(title);
    }
    return <div>
        <br/>
        <br/>
        <p>{title}</p>
        <button onClick={deleteThis}>删除</button>
        <br/>
    </div>
};
export default function BlackListLyrics() {
    const addBlackList = async () => {
        const name = await wsService.defaultClient?.getNowTitle();
        if (name) {
            lyricsStore.addTitleBlackList(name);
        }
    }

    const saveBlackList = () => {
        lyricsStore.asyncTitleBlackList();
    }

    return <>
        <button onClick={addBlackList}>拉黑当前</button>
        <br/>
        <button onClick={saveBlackList}>永久保存</button>
        <br/>
        <For each={getTitleBlackList()}>
            {(title) => <BlackListItem title={title} />}
        </For>

    </>;
}

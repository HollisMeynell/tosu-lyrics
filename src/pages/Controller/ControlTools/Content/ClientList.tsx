import { wsService } from "@/services/WebSocketService.ts";
import { Component, createSignal, For } from "solid-js";

export default function ClientList() {
    const [clients, setClients] = createSignal<string[]>(wsService.getOnlineClients());
    const [selectedClient, setSelectedClient] = createSignal<string>(clients()[0] || "");
    const [buttonCollDown, setButtonCollDown] = createSignal(false);

    const updateClient = () => {
        const data = wsService.getOnlineClients();
        setClients(data);
        setSelectedClient(data[0] || "");
    };

    const blinkOtherClient = () => {
        setButtonCollDown(true);
        wsService.blinkOtherClient(selectedClient());
        setTimeout(() => setButtonCollDown(false), 3000);
    };

    const Client: Component<{ id: string, index: number }> = (props) => {
        return <option value={props.id}>
            客户端-{props.index}
        </option>;
    };

    return <>
        如果客户端未显示, 请打开obs后点击下方按钮刷新
        <br />
        <button
            class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4
            focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5
            me-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none
             dark:focus:ring-blue-800"
            onClick={updateClient}
        >
            更新
        </button>
        <br />
        <select
            value={selectedClient()}
            onInput={(e) => setSelectedClient(e.currentTarget.value)}
            class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
            block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white
             dark:focus:ring-blue-500 dark:focus:border-blue-500"
        >
            {clients().length === 0 && <option value={""} disabled>等待其他客户端</option>}
            <For each={clients()}>
                {(id, index) =>
                    <Client id={id} index={index()} />}
            </For>
        </select>
        <h3>点击下方按钮, 对应客户端的歌词会出现闪烁</h3>
        <button
            class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4
            focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5
            me-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none
             dark:focus:ring-blue-800"
            onClick={blinkOtherClient}
            disabled={buttonCollDown() || !selectedClient()}
        >
            测试
        </button>
    </>;
}

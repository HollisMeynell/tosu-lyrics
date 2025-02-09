import { wsService } from "@/services/webSocketService";
import { Component, createSignal, For } from "solid-js";

export default function ClientList() {
    const [clients, setClients] = createSignal<string[]>(
        wsService.getOnlineClients()
    );
    const [selectedClient, setSelectedClient] = createSignal<string>(
        clients()[0] || ""
    );
    const [buttonCollDown, setButtonCollDown] = createSignal(false);

    const updateClient = () => {
        const data = wsService.getOnlineClients();
        setClients(data);
        setSelectedClient(data[0] || "");
    };

    const blinkOtherClient = () => {
        setButtonCollDown(true);
        wsService.blinkOtherClient(selectedClient());
        wsService.setDefaultClient(selectedClient());
        setTimeout(() => setButtonCollDown(false), 3000);
    };

    const Client: Component<{ id: string; index: number }> = (props) => {
        return <option value={props.id}>客户端-{props.index}</option>;
    };

    const Controller = () => (
        <div class="flex flex-row items-center mb-6">
            <button
                class="px-5 py-2 inline-flex items-center gap-x-2 text-sm hover:cursor-pointer
                font-medium rounded-lg border border-transparent bg-blue-600
                text-white hover:bg-blue-700 focus:outline-none focus:bg-blue-700
                disabled:opacity-50 disabled:pointer-events-none"
                onClick={updateClient}
            >
                更新
            </button>

            <select
                value={selectedClient()}
                onInput={(e) => setSelectedClient(e.currentTarget.value)}
                class="py-2 ml-4 w-40 bg-gray-50 border border-gray-300 text-gray-900
                text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500
                dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white
                dark:focus:ring-blue-500 dark:focus:border-blue-500 text-center"
            >
                {clients().length === 0 && (
                    <option value={""} disabled>
                        等待其他客户端
                    </option>
                )}
                <For each={clients()}>
                    {(id, index) => <Client id={id} index={index()} />}
                </For>
            </select>
        </div>
    );

    return (
        <div class="flex flex-col items-start">
            <h2 class="text-2xl mb-4">歌词端选择:</h2>
            <h3>如果客户端未显示, 请打开obs后点击下方按钮刷新</h3>

            <Controller />

            <h3>点击下方按钮, 对应客户端的歌词会出现闪烁</h3>

            <button
                class="px-5 py-2 inline-flex items-center gap-x-2 text-sm hover:cursor-pointer
            font-medium rounded-lg border border-transparent bg-blue-600
            text-white hover:bg-blue-700 focus:outline-none focus:bg-blue-700
            disabled:opacity-50 disabled:pointer-events-none"
                onClick={blinkOtherClient}
                disabled={buttonCollDown() || !selectedClient()}
            >
                测试
            </button>
        </div>
    );
}

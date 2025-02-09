import { wsService } from "@/services/webSocketService";
import { Component, createSignal, For } from "solid-js";
import Button from "@/components/ui/Button";
import { ChevronRight } from "@/assets/Icons";

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
        <div class="flex flex-row items-center gap-3">
            <div class="relative w-full md:w-48">
                <select
                    value={selectedClient()}
                    onInput={(e) => setSelectedClient(e.currentTarget.value)}
                    class="w-full py-1.5 pl-3 pr-8 border border-[#cbd5e1] rounded-lg shadow-xs appearance-none bg-white focus:outline-hidden focus:ring-1 focus:ring-[#eb4898] focus:border-[#eb4898] hover:border-[#94a3b8] transition-colors cursor-pointer dark:bg-[#020616] dark:border-[#475569] dark:text-white dark:focus:border-[#e169a8] dark:hover:border-[#64748b]"
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
                <button class="absolute right-2 top-1/2 transform -translate-y-1/2 cursor-pointer">
                    <span class="text-xl font-[100] text-[#94a3b8] leading-none">
                        ×
                    </span>
                </button>
            </div>
            <Button className="py-[.5rem]" onClick={updateClient}>
                刷新
            </Button>
        </div>
    );

    return (
        <div class="flex flex-col gap-4">
            <h2 class="text-2xl">歌词端选择</h2>
            <h3>
                客户端列表 ( 包含你用于控制的端，以及在 OBS 中打开以用于展示的端
                ){" "}
            </h3>

            <div class="flex flex-row gap-4 items-center">
                <Controller />

                <ChevronRight />

                <Button
                    className="py-[.5rem]"
                    onClick={blinkOtherClient}
                    disabled={buttonCollDown() || !selectedClient()}
                >
                    测试(对应端的歌词将会闪烁)
                </Button>
            </div>
        </div>
    );
}

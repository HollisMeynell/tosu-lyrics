import { wsService } from "@/services/webSocketService";
import { createSignal, For } from "solid-js";
import Button from "@/components/ui/Button";
import { ChevronRight } from "@/assets/Icons";

export default function ClientList() {
    const [clients, setClients] = createSignal<string[]>(
        wsService.getOnlineClients()
    );
    const [selectedClient, setSelectedClient] = createSignal<string>(
        clients()[0] || ""
    );
    const [buttonDisabled, setButtonDisabled] = createSignal(!selectedClient());

    const updateClient = () => {
        const data = wsService.getOnlineClients();
        setClients(data);
        setSelectedClient(data[0] || "");
    };

    const blinkOtherClient = () => {
        setButtonDisabled(true);
        wsService.blinkOtherClient(selectedClient());
        setTimeout(() => setButtonDisabled(false), 3000);
    };

    const Selector = () => (
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
                        {(id, index) => (
                            <option value={id}>客户端-{index()}</option>
                        )}
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
            <hr class="w-24 border-gray-400 dark:border-gray-600" />
            <div class="text-gray-900 dark:text-gray-200">
                <p>
                    客户端列表包含你用于控制的端，以及在 OBS
                    中打开以用于展示的端。
                </p>
                <p>请选一个客户端作为你将展示在 OBS 中的端。</p>
                <p>
                    你可以通过测试按钮来测试端是否正确。当测试时，对应端将会闪烁。
                </p>
            </div>

            <h3 class="text-xl">当前选择的端:</h3>

            <div class="flex flex-row items-center gap-3">
                <p class="text-xl">
                    {selectedClient()
                        ? `客户端-${clients().indexOf(selectedClient())}`
                        : "无"}
                </p>
            </div>

            <div class="flex flex-row gap-2 items-center">
                <Selector />
                <ChevronRight />
                <Button
                    className="py-[.5rem]"
                    onClick={blinkOtherClient}
                    disabled={buttonDisabled()}
                >
                    测试
                </Button>
                <ChevronRight />
                <Button
                    className="py-[.5rem]"
                    onClick={() => wsService.setDefaultClient(selectedClient())}
                >
                    确定
                </Button>
            </div>
        </div>
    );
}

import { wsService } from "@/services/webSocketService";
import { createSignal } from "solid-js";
import { Button, Select } from "@/components/ui";
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
            <Select
                className="w-48 py-1.5"
                options={clients().map((id) => ({
                    code: id,
                    name: `客户端-${clients().indexOf(id)}`,
                }))}
                value={selectedClient()}
                onChange={setSelectedClient}
                placeholder="等待其他客户端"
            />
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

// 功能: 面板-连接二维码（手机扫码打开控制台）
import { Show, createEffect, createSignal, onMount } from "solid-js";
import QRCode from "qrcode";
import { Copy } from "@/assets/Icons";
import { Button, Select } from "@/components/ui";
import { createServerAddressesController } from "@/hooks/useServerAddresses";

const CONTROLLER_PATH = "/lyrics/controller/content";
const LOOPBACK_HOSTS = ["127.0.0.1", "localhost", "::1", "0:0:0:0:0:0:0:1"];

export default function QrCode() {
    const c = createServerAddressesController();
    const [qrDataUrl, setQrDataUrl] = createSignal("");
    const [copied, setCopied] = createSignal(false);

    const isLoopbackHost = () => LOOPBACK_HOSTS.includes(c.host());

    const url = () => {
        const ip = c.selected();
        if (!ip) return "";
        const host = ip.includes(":") ? `[${ip}]` : ip;
        const port = c.port() ? `:${c.port()}` : "";
        return `http://${host}${port}${CONTROLLER_PATH}`;
    };

    createEffect(() => {
        const target = url();
        if (!target) {
            setQrDataUrl("");
            return;
        }
        QRCode.toDataURL(target, {
            width: 512,
            margin: 2,
            errorCorrectionLevel: "M",
        })
            .then((dataUrl) => setQrDataUrl(dataUrl))
            .catch(() => setQrDataUrl(""));
    });

    const copy = async () => {
        const target = url();
        if (!target) return;
        try {
            await navigator.clipboard.writeText(target);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
        } catch {
            setCopied(false);
        }
    };

    onMount(() => void c.load());

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-xl inline">连接二维码</h2>
                <p class="text-sm inline text-gray-500">
                    手机与本机连同一网络，扫码后在手机上打开控制台
                </p>
            </div>

            <Show when={c.error()}>
                <p class="text-sm text-red-500">{c.error()}</p>
            </Show>

            <Show when={c.host() && isLoopbackHost()}>
                <p class="text-sm text-amber-600 dark:text-amber-400">
                    服务当前仅监听 {c.host()}，其他设备无法访问。请把
                    config.json5 的 server 改为 0.0.0.0 后重启
                </p>
            </Show>

            <div class="flex flex-col gap-2">
                <p class="text-sm text-gray-500">访问地址</p>
                <Select
                    class="w-full max-w-96"
                    value={c.selected()}
                    clearable={false}
                    placeholder={
                        c.loading() ? "正在获取本机地址…" : "未检测到可用地址"
                    }
                    options={c.addresses().map((item) => ({
                        code: item.address,
                        name: `${item.address} · ${item.name}`,
                    }))}
                    onChange={(value) => c.select(value)}
                />
            </div>

            <Show when={qrDataUrl()}>
                <div class="flex flex-col gap-3 items-start">
                    <img
                        src={qrDataUrl()}
                        alt="连接二维码"
                        class="w-72 h-72 rounded-lg bg-white p-2"
                    />
                    <div class="flex flex-row items-center flex-wrap gap-3">
                        <p class="text-sm break-all text-gray-500">{url()}</p>
                        <Button class="px-4 py-1" onClick={copy}>
                            <Copy
                                stroke="currentColor"
                                class="mr-2 w-4 h-4 inline"
                            />
                            {copied() ? "已复制" : "复制链接"}
                        </Button>
                    </div>
                </div>
            </Show>
        </div>
    );
}

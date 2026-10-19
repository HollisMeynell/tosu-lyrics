import { createSignal } from "solid-js";
import { ApiError } from "@/services/settingsService";
import { NetAddress, fetchServerAddresses } from "@/services/networkService";

/** 扫码连接：本机地址列表与选中项。默认选服务端推荐的局域网地址。 */
export function createServerAddressesController() {
    const [addresses, setAddresses] = createSignal<NetAddress[]>([]);
    const [host, setHost] = createSignal("");
    const [port, setPort] = createSignal(0);
    const [selected, setSelected] = createSignal("");
    const [loading, setLoading] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);

    const load = async () => {
        setLoading(true);
        setError(null);
        try {
            const info = await fetchServerAddresses();
            setAddresses(info.addresses);
            setHost(info.host);
            setPort(info.port);
            setSelected((prev) => {
                if (info.addresses.some((item) => item.address === prev)) {
                    return prev;
                }
                return (
                    info.addresses.find((item) => item.preferred)?.address ??
                    info.addresses[0]?.address ??
                    ""
                );
            });
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setLoading(false);
        }
    };

    return {
        addresses,
        host,
        port,
        selected,
        loading,
        error,
        load,
        select: (address: string) => setSelected(address),
    };
}

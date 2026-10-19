import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export interface NetAddress {
    address: string;
    name: string;
    /** 服务端推荐项（局域网地址优先排序后的首位） */
    preferred: boolean;
}

export interface ServerAddressesDto {
    host: string;
    port: number;
    addresses: NetAddress[];
}

const ADDRESSES_URL = `${BACKEND_API_BASE}/server/addresses`;

async function toApiError(res: Response): Promise<ApiError> {
    let code = "unknown";
    let message = `服务器返回错误（编号 ${res.status}）`;
    try {
        const body = await res.json();
        if (body?.error?.code) {
            code = body.error.code;
            message = body.error.message ?? message;
        }
    } catch {
        // 非 JSON 错误体时保留默认信息
    }
    return new ApiError(code, message, res.status);
}

export async function fetchServerAddresses(): Promise<ServerAddressesDto> {
    const res = await fetch(ADDRESSES_URL);
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as ServerAddressesDto;
}

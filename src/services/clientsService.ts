import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";
import type { SettingsPatch } from "@/types/globalTypes";

/**
 * 在线展示端 HTTP 客户端（B-08 / F-07）。
 *
 * 列表与定向 blink 全部走 HTTP —— WS 只负责展示事件传输，
 * 不再有 setter / 管理通道。
 */

export interface DisplayClient {
    /** 会话标识：每次连接都不同 */
    id: string;
    /** 客户端自报的稳定身份（`ws://.../ws?id=obs-main`）；匿名为 null */
    identity: string | null;
    connectedAt: number;
    userAgent: string | null;
}

/** 给展示端起一个可读名字：优先用自报身份，其次用会话 id 前缀 */
export function clientLabel(client: DisplayClient): string {
    if (client.identity) return client.identity;
    return `匿名端 ${client.id.slice(0, 6)}`;
}

async function toApiError(res: Response): Promise<ApiError> {
    let code = "unknown";
    let message = `HTTP ${res.status}`;
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

export async function fetchClients(): Promise<DisplayClient[]> {
    const res = await fetch(`${BACKEND_API_BASE}/clients`);
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { items: DisplayClient[] };
    return body.items ?? [];
}

/**
 * 让指定展示端闪烁。
 *
 * `target` 可以是会话 id，也可以是自报身份。
 * 目标离线时后端返回 404 `not_found` —— 那是**正常结果**，转成 `false`
 * 让调用方刷新列表而不是报错。
 */
export async function blinkClient(target: string): Promise<boolean> {
    const res = await fetch(
        `${BACKEND_API_BASE}/clients/${encodeURIComponent(target)}/blink`,
        { method: "POST" }
    );
    if (res.status === 404) return false;
    if (!res.ok) throw await toApiError(res);
    return true;
}

/**
 * 只把样式改动推给**指定展示端**（不落库）。
 *
 * 这是"单独客户端调整"：选中某个端之后改样式，只有它变，
 * 其余端与全局设置都不受影响。后端不保存 per-client 配置，
 * 因此该端**重连后会回到全局设置** —— 这是有意为之，不是缺陷。
 */
export async function applyClientSettings(
    target: string,
    patch: SettingsPatch
): Promise<{ clients: number; keys: string[] }> {
    const res = await fetch(
        `${BACKEND_API_BASE}/clients/${encodeURIComponent(target)}/settings`,
        {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(patch),
        }
    );
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as { clients: number; keys: string[] };
}

/** 让**全部**在线展示端闪烁 */
export async function blinkAll(): Promise<number> {
    const res = await fetch(`${BACKEND_API_BASE}/clients/blink`, {
        method: "POST",
    });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { blinked: number };
    return body.blinked;
}

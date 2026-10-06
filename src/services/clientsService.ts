import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";
import type { SettingsPatch } from "@/types/globalTypes";

export interface DisplayClient {
    id: string;
    identity: string | null;
    connectedAt: number;
    userAgent: string | null;
}

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
    } catch {}
    return new ApiError(code, message, res.status);
}

export async function fetchClients(): Promise<DisplayClient[]> {
    const res = await fetch(`${BACKEND_API_BASE}/clients`);
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { items: DisplayClient[] };
    return body.items ?? [];
}

/** 目标离线时返回 false（404），不抛错 */
export async function blinkClient(target: string): Promise<boolean> {
    const res = await fetch(
        `${BACKEND_API_BASE}/clients/${encodeURIComponent(target)}/blink`,
        { method: "POST" }
    );
    if (res.status === 404) return false;
    if (!res.ok) throw await toApiError(res);
    return true;
}

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

export async function blinkAll(): Promise<number> {
    const res = await fetch(`${BACKEND_API_BASE}/clients/blink`, {
        method: "POST",
    });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { blinked: number };
    return body.blinked;
}

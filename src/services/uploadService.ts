import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export interface UploadResult {
    ok: boolean;
    lines: number;
    song: { bid: number; sid: number; title: string };
}

export interface FontEntry {
    name: string;
    fileName: string;
    size: number;
    version: string;
    url: string;
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

export async function uploadLrc(file: File): Promise<UploadResult> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${BACKEND_API_BASE}/lyrics/upload`, {
        method: "POST",
        body: form,
    });
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as UploadResult;
}

export async function fetchFontList(): Promise<FontEntry[]> {
    const res = await fetch(`${BACKEND_API_BASE}/font/list`);
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { items: FontEntry[] };
    return body.items ?? [];
}

export async function uploadFontFile(file: File): Promise<FontEntry> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${BACKEND_API_BASE}/font/upload`, {
        method: "POST",
        body: form,
    });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { font: FontEntry };
    return body.font;
}
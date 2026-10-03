import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export interface UploadResult {
    ok: boolean;
    lines: number;
    song: { bid: number; sid: number; title: string };
}

export interface FontInfo {
    kind: "main" | "sub";
    /** FontFace 名（`LRC` / `LRC-Sub`）：注册与实际渲染用的就是它 */
    family: string;
    /** 字体文件内部的真实名称，**仅供 UI 显示**（后端解析 name 表得到） */
    displayName: string;
    exists: boolean;
    version: string;
    size: number;
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

export async function fetchFontInfo(): Promise<FontInfo[]> {
    const res = await fetch(`${BACKEND_API_BASE}/font/info`);
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { items: FontInfo[] };
    return body.items ?? [];
}

export async function uploadFontFile(
    kind: "main" | "sub",
    file: File
): Promise<FontInfo> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${BACKEND_API_BASE}/font/${kind}`, {
        method: "POST",
        body: form,
    });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { font: FontInfo };
    return body.font;
}

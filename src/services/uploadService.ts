import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

/**
 * 上传与字体资源 HTTP 客户端（B-07 / F-06）。
 *
 * 后端是资源真源：字体由后端按**版本化 URL** 提供，
 * 覆盖上传后版本号变化，展示端据此重新拉取，不依赖浏览器缓存失效。
 */

export interface UploadResult {
    ok: boolean;
    /** 上传的歌词行数 */
    lines: number;
    song: { bid: number; sid: number; title: string };
}

export interface FontInfo {
    kind: "main" | "sub";
    family: string;
    exists: boolean;
    /** 版本号 = 文件 mtime-size；重启后仍是同一个值 */
    version: string;
    size: number;
    /** 后端给的版本化 URL，展示端直接用它加载 FontFace */
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

/**
 * 上传 LRC。绑定到**发起时正在播放的那首歌的 sid**。
 *
 * 非法文件后端不会替换旧歌词，并返回结构化错误，调用方据此展示可重试状态。
 */
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

/** 上传并覆盖指定字体，返回**新的**字体信息（含新版本号） */
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

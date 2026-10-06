import { BACKEND_API_BASE } from "@/config/constants";
import { SettingsDto, SettingsPatch } from "@/types/globalTypes";

export class ApiError extends Error {
    constructor(
        public readonly code: string,
        message: string,
        public readonly status: number
    ) {
        super(message);
        this.name = "ApiError";
    }
}

const SETTINGS_URL = `${BACKEND_API_BASE}/settings`;

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
        // 后端不是 JSON 错误体（例如 502）时保留默认信息
    }
    return new ApiError(code, message, res.status);
}

export async function fetchSettings(): Promise<SettingsDto> {
    const res = await fetch(SETTINGS_URL);
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as SettingsDto;
}

/**
 * 局部更新设置。
 *
 * 成功时返回**服务端最终状态**（不是我们提交的内容），调用方应当用它覆盖本地状态；
 * 失败时抛出 `ApiError`，本地状态必须保持不变。
 */
export async function patchSettings(
    patch: SettingsPatch
): Promise<SettingsDto> {
    const res = await fetch(SETTINGS_URL, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(patch),
    });
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as SettingsDto;
}

import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

/**
 * 后端运行状态 HTTP 客户端。
 *
 * 对应 `GET /api/status`：当前歌曲 + 歌词展示状态 + 生效偏移。
 * Controller 的"当前播放"一律走这里，不再通过旧 WS 向对端查询。
 */

export interface CurrentSong {
    bid: number;
    sid: number;
    title: string;
    artist: string;
    /** 毫秒；-1 表示未能读取到音频时长 */
    length: number;
}

export interface LyricState {
    loaded: boolean;
    cleared: boolean;
    lineCount: number;
    /** 当前行下标；无歌词时为 -1 */
    current: number;
    /** 距离下一行开始的剩余毫秒；末行为 -1 */
    nextTime: number;
}

export interface StatusDto {
    /** 菜单 / 无歌时为 null */
    song: CurrentSong | null;
    lyric: LyricState;
    offset: number;
}

const STATUS_URL = `${BACKEND_API_BASE}/status`;

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
 * POST /api/display/clear —— 清空所有展示端（持续清屏）。
 *
 * 语义是"持续清屏，直到换歌 / 换源 / 上传歌词才恢复"，
 * 因此它是一个独立的展示控制，不属于样式设置。
 */
export async function clearDisplay(): Promise<void> {
    const res = await fetch(`${BACKEND_API_BASE}/display/clear`, {
        method: "POST",
    });
    if (!res.ok) throw await toApiError(res);
}

export async function fetchStatus(): Promise<StatusDto> {
    const res = await fetch(STATUS_URL);
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as StatusDto;
}

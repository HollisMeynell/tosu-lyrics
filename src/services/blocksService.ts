import { BACKEND_API_BASE } from "@/config/constants";
import { ApiError } from "@/services/settingsService";

export type BlockScope = "bid" | "sid" | "title";

export interface BlockRule {
    id: number;
    scope: BlockScope;
    value: string;
    title: string;
    sid: number;
    reason: string;
    createdAt: number;
}

export interface BlockRuleInput {
    scope: BlockScope;
    value: string;
    title?: string;
    sid?: number;
    reason?: string;
}

const BLOCKS_URL = `${BACKEND_API_BASE}/blocks`;

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

export async function fetchBlocks(): Promise<BlockRule[]> {
    const res = await fetch(BLOCKS_URL);
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { items: BlockRule[] };
    return body.items ?? [];
}

/** 新增规则。幂等：同一 scope+value 重复提交只会更新标题。 */
export async function addBlock(input: BlockRuleInput): Promise<BlockRule> {
    const res = await fetch(BLOCKS_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(input),
    });
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as BlockRule;
}

/**
 * 修改展示用元数据（标题 / 备注）。
 *
 * 作用域与取值**不可改**，因此不会改变规则身份，也不会影响命中判定。
 */
export async function updateBlock(
    id: number,
    meta: { title?: string; reason?: string }
): Promise<BlockRule> {
    const res = await fetch(`${BLOCKS_URL}/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(meta),
    });
    if (!res.ok) throw await toApiError(res);
    return (await res.json()) as BlockRule;
}

/**
 * 删除单条规则。
 *
 * 规则本来就不存在时后端返回 404 `not_found`。这在"重复删除"场景下是正常结果，
 * 因此这里把它转成 `false` 而不是抛错，让调用方可以静默刷新列表。
 */
export async function deleteBlock(id: number): Promise<boolean> {
    const res = await fetch(`${BLOCKS_URL}/${id}`, { method: "DELETE" });
    if (res.status === 404) return false;
    if (!res.ok) throw await toApiError(res);
    return true;
}

/** 清空全部规则（幂等），返回实际删除条数。 */
export async function clearBlocks(): Promise<number> {
    const res = await fetch(BLOCKS_URL, { method: "DELETE" });
    if (!res.ok) throw await toApiError(res);
    const body = (await res.json()) as { removed: number };
    return body.removed ?? 0;
}

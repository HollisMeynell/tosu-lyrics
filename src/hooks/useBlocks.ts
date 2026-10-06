import { createSignal } from "solid-js";
import {
    BlockRule,
    BlockRuleInput,
    addBlock,
    clearBlocks,
    deleteBlock,
    fetchBlocks,
    updateBlock,
} from "@/services/blocksService";
import { ApiError } from "@/services/settingsService";

/** 所有写操作成功后重新拉取列表，保证与服务端一致。 */
export function createBlocksController() {
    const [items, setItems] = createSignal<BlockRule[]>([]);
    const [loading, setLoading] = createSignal(false);
    const [saving, setSaving] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);

    const load = async () => {
        setLoading(true);
        setError(null);
        try {
            setItems(await fetchBlocks());
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setLoading(false);
        }
    };

    const mutate = async (fn: () => Promise<unknown>): Promise<boolean> => {
        setSaving(true);
        setError(null);
        try {
            await fn();
            setItems(await fetchBlocks());
            return true;
        } catch (err) {
            setError(err instanceof ApiError ? err.message : String(err));
            return false;
        } finally {
            setSaving(false);
        }
    };

    return {
        items,
        loading,
        saving,
        error,
        load,
        add: (input: BlockRuleInput) => mutate(() => addBlock(input)),
        update: (id: number, meta: { title?: string; reason?: string }) =>
            mutate(() => updateBlock(id, meta)),
        remove: (id: number) => mutate(() => deleteBlock(id)),
        clear: () => mutate(() => clearBlocks()),
        clearError: () => setError(null),
    };
}

export function defaultBlockInput(song: {
    bid: number;
    sid: number;
    title: string;
}): BlockRuleInput {
    if (song.bid > 0) {
        return {
            scope: "bid",
            value: String(song.bid),
            title: song.title,
            sid: song.sid,
        };
    }
    return { scope: "title", value: song.title, title: song.title };
}

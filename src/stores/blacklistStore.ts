import { createStore } from "solid-js/store";
import { createSignal } from "solid-js";
import { BlacklistItem } from "@/types/globalTypes";

// 使用Set数据结构管理黑名单
export class BlacklistSet {
    private items: Set<string>;
    private itemsMap: Map<string, BlacklistItem>;

    constructor() {
        this.items = new Set<string>();
        this.itemsMap = new Map<string, BlacklistItem>();
    }

    // 添加黑名单项
    add(item: BlacklistItem): boolean {
        if (this.items.has(item.id)) {
            return false;
        }

        this.items.add(item.id);
        this.itemsMap.set(item.id, item);
        return true;
    }

    // 从黑名单中移除
    remove(id: string): boolean {
        if (!this.items.has(id)) {
            return false;
        }

        this.items.delete(id);
        this.itemsMap.delete(id);
        return true;
    }

    // 更新黑名单项
    update(id: string, newData: Partial<Omit<BlacklistItem, "id">>): boolean {
        if (!this.items.has(id)) {
            return false;
        }

        const oldItem = this.itemsMap.get(id);
        if (!oldItem) return false;

        const updatedItem = { ...oldItem, ...newData };
        this.itemsMap.set(id, updatedItem);
        return true;
    }

    // 查询黑名单项
    get(id: string): BlacklistItem | undefined {
        return this.itemsMap.get(id);
    }

    // 检查是否在黑名单中
    has(id: string): boolean {
        return this.items.has(id);
    }

    // 获取所有黑名单项
    getAll(): BlacklistItem[] {
        return Array.from(this.itemsMap.values());
    }

    // 获取黑名单大小
    get size(): number {
        return this.items.size;
    }

    // 清空黑名单
    clear(): void {
        this.items.clear();
        this.itemsMap.clear();
    }

    // 从存储恢复
    restore(items: BlacklistItem[]): void {
        this.clear();

        // 检查items是否为数组且不为空
        if (!items || !Array.isArray(items)) {
            console.warn(
                "Attempted to restore blacklist with invalid data:",
                items
            );
            return;
        }

        try {
            // 遍历每一项并添加到Set中
            for (const item of items) {
                if (
                    item &&
                    typeof item === "object" &&
                    "id" in item &&
                    typeof item.id === "string"
                ) {
                    this.items.add(item.id);
                    this.itemsMap.set(item.id, {
                        id: item.id,
                        name: String(item.name || ""),
                        reason: item.reason ? String(item.reason) : undefined,
                        timestamp:
                            typeof item.timestamp === "number"
                                ? item.timestamp
                                : Date.now(),
                    });
                } else {
                    console.warn("Invalid item in blacklist data:", item);
                }
            }
        } catch (error) {
            console.error("Error restoring blacklist data:", error);
            // 发生错误时清空，确保数据一致性
            this.clear();
        }
    }
}

// 创建SolidJS状态
const createBlacklistStore = () => {
    const blacklistSet = new BlacklistSet();

    // 创建存储状态
    const [blacklist, setBlacklist] = createStore<{
        items: BlacklistItem[];
        size: number;
    }>({
        items: [],
        size: 0,
    });

    // 用于触发重新渲染的信号
    const [version, setVersion] = createSignal(0);

    // 更新状态并触发重新渲染
    const updateState = () => {
        setBlacklist({
            items: blacklistSet.getAll(),
            size: blacklistSet.size,
        });
        setVersion((v) => v + 1);
    };

    // 公开的API
    return {
        // 获取黑名单状态
        get items() {
            version(); // 依赖于version信号以便在变更时触发更新
            return blacklist.items;
        },
        get size() {
            version();
            return blacklist.size;
        },

        // 添加黑名单项
        add(item: Omit<BlacklistItem, "timestamp">): boolean {
            const result = blacklistSet.add({
                ...item,
                timestamp: Date.now(),
            });
            if (result) updateState();
            return result;
        },

        // 删除黑名单项
        remove(id: string): boolean {
            const result = blacklistSet.remove(id);
            if (result) updateState();
            return result;
        },

        // 更新黑名单项
        update(
            id: string,
            newData: Partial<Omit<BlacklistItem, "id">>
        ): boolean {
            const result = blacklistSet.update(id, newData);
            if (result) updateState();
            return result;
        },

        // 查询黑名单项
        get(id: string): BlacklistItem | undefined {
            version();
            return blacklistSet.get(id);
        },

        // 检查是否在黑名单中
        has(id: string): boolean {
            version();
            return blacklistSet.has(id);
        },

        // 清空黑名单
        clear(): void {
            blacklistSet.clear();
            updateState();
        },

        // 从存储恢复
        restore(items: BlacklistItem[]): void {
            blacklistSet.restore(items);
            updateState();
        },
    };
};

// 导出全局单例
export const blacklistStore = createBlacklistStore();

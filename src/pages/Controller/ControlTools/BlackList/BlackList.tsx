import { Component, createSignal, For, Show } from "solid-js";
import { blacklistStore } from "@/stores/blacklistStore";
import { BlacklistItem } from "@/types/globalTypes";
import { Button } from "@/components/ui";
import { Delete, Save, Cancel, Edit } from "@/assets/Icons";
import store from "@/stores/indexStore";

const CustomTd = (props: { children: string }) => {
    return (
        <td class="py-3 px-4 text-sm text-gray-700 dark:text-gray-300">
            {props.children}
        </td>
    );
};

const CustomTh = (props: { children: string }) => {
    return (
        <th class="py-3 px-4 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
            {props.children}
        </th>
    );
};

const BlacklistComponent: Component = () => {
    const [newItem, setNewItem] = createSignal({
        id: "",
        name: "",
        reason: "",
    });

    const [editingItem, setEditingItem] = createSignal<BlacklistItem | null>(
        null
    );

    // 添加新的黑名单项
    const handleAdd = () => {
        const item = newItem();
        if (!item.id || !item.name) return;

        blacklistStore.add({
            id: item.id,
            name: item.name,
            reason: item.reason,
        });

        // 重置表单
        setNewItem({
            id: "",
            name: "",
            reason: "",
        });
    };

    // 启动编辑模式
    const handleEdit = (item: BlacklistItem) => {
        setEditingItem(item);
    };

    // 保存编辑
    const handleSaveEdit = () => {
        const item = editingItem();
        if (!item) return;

        blacklistStore.update(item.id, {
            name: item.name,
            reason: item.reason,
        });

        setEditingItem(null);
    };

    // 取消编辑
    const handleCancelEdit = () => {
        setEditingItem(null);
    };

    // 移除黑名单项
    const handleRemove = (id: string) => {
        store.deleteTitleBlackList(id);
    };

    // 保存黑名单
    const saveBlackList = () => {
        store.syncTitleBlackList();
    };

    {
        /* 添加新黑名单项表单 */
    }
    const addItemForm = () => {
        return (
            <div class="flex flex-col gap-4 w-full lg:w-1/3 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg shadow-sm">
                <h3 class="text-xl font-semibold text-gray-800 dark:text-white border-b pb-2 mb-2">
                    添加新的黑名单项
                </h3>
                <div class="flex flex-col gap-4">
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            ID：
                        </label>
                        <input
                            type="text"
                            value={newItem().id}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    id: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder="输入ID"
                        />
                    </div>
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            名称：
                        </label>
                        <input
                            type="text"
                            value={newItem().name}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    name: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder="输入名称"
                        />
                    </div>
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            原因：
                        </label>
                        <input
                            type="text"
                            value={newItem().reason || ""}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    reason: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder="输入添加原因（可选）"
                        />
                    </div>
                </div>
                <Button
                    class="w-full mt-2 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
                    onClick={handleAdd}
                >
                    添加
                </Button>

                <div class="flex flex-col sm:flex-row gap-3 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                    <Button
                        class="flex-1 bg-green-600 hover:bg-green-700 text-white py-2 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-opacity-50"
                        onClick={saveBlackList}
                    >
                        保存黑名单
                    </Button>

                    <Button
                        class="flex-1"
                        onClick={() => blacklistStore.clear()}
                    >
                        <Delete class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                        清空黑名单
                    </Button>
                </div>
            </div>
        );
    };

    {
        /* 黑名单项表格行 */
    }
    const trItem = (item: BlacklistItem) => {
        return (
            <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                <CustomTd>{`${item.id}`}</CustomTd>
                <td class="py-3 px-4">
                    <input
                        type="text"
                        value={editingItem()?.name || ""}
                        onInput={(e) =>
                            setEditingItem({
                                ...editingItem()!,
                                name: e.currentTarget.value,
                            })
                        }
                        class="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    />
                </td>
                <td class="py-3 px-4">
                    <input
                        type="text"
                        value={editingItem()?.reason || ""}
                        onInput={(e) =>
                            setEditingItem({
                                ...editingItem()!,
                                reason: e.currentTarget.value,
                            })
                        }
                        class="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    />
                </td>
                <CustomTd>
                    {`${new Date(item.timestamp).toLocaleString()}`}
                </CustomTd>
                <td class="py-3 px-4">
                    <div class="flex space-x-2">
                        <button
                            onClick={handleSaveEdit}
                            class="px-3 py-1 border border-green-500 text-white rounded-md hover:bg-green-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-green-500/50"
                        >
                            <Save class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                            保存
                        </button>
                        <button
                            onClick={handleCancelEdit}
                            class="px-3 py-1 border border-gray-500 text-white rounded-md hover:bg-gray-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500/50"
                        >
                            <Cancel class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                            取消
                        </button>
                    </div>
                </td>
            </tr>
        );
    };

    {
        /* 黑名单项表格行回退 */
    }
    const trFallback = (item: BlacklistItem) => {
        return (
            <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                <CustomTd>{`${item.id}`}</CustomTd>
                <CustomTd>{`${item.name}`}</CustomTd>
                <CustomTd>{`${item.reason}`}</CustomTd>
                <CustomTd>
                    {`${new Date(item.timestamp).toLocaleString()}`}
                </CustomTd>
                <td class="py-3 pl-4">
                    <div class="flex space-x-2">
                        <button
                            onClick={() => handleEdit(item)}
                            class="px-3 py-1 border border-blue-500 text-white rounded-md hover:bg-blue-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        >
                            <Edit class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                            编辑
                        </button>
                        <button
                            onClick={() => handleRemove(item.id)}
                            class="px-2 pb-0.5 border border-red-500 text-white rounded-md hover:bg-red-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-red-500/50"
                        >
                            <Delete class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90 mb-0.5" />
                            删除
                        </button>
                    </div>
                </td>
            </tr>
        );
    };

    return (
        <div class="flex flex-col lg:flex-row gap-6 p-6 bg-white dark:bg-gray-900 rounded-lg shadow-md">
            {/* 添加新黑名单项表单 */}
            {addItemForm()}

            {/* 显示黑名单列表 */}
            <div class="blacklist-items flex-1 flex flex-col gap-4">
                {/* Header */}
                <div class="flex justify-between items-center mb-2">
                    <h3 class="text-xl font-semibold text-gray-800 dark:text-white">
                        黑名单列表
                    </h3>
                    <span class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-full text-sm font-medium">
                        共 {blacklistStore.size} 项
                    </span>
                </div>

                {/* Table本体 */}
                <div class="overflow-x-auto rounded-lg border border-gray-200 dark:border-gray-700">
                    <table class="w-full border-collapse">
                        <thead class="bg-gray-100 dark:bg-gray-800">
                            <tr>
                                <CustomTh>ID</CustomTh>
                                <CustomTh>名称</CustomTh>
                                <CustomTh>原因</CustomTh>
                                <CustomTh>时间</CustomTh>
                                <CustomTh>操作</CustomTh>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200 dark:bg-gray-900 dark:divide-gray-700">
                            <For each={blacklistStore.items}>
                                {(item) => (
                                    <Show
                                        when={editingItem()?.id !== item.id}
                                        fallback={trItem(item)}
                                    >
                                        {trFallback(item)}
                                    </Show>
                                )}
                            </For>
                        </tbody>
                    </table>
                </div>

                {/* 无黑名单项提示 */}
                <Show when={blacklistStore.size === 0}>
                    <div class="flex justify-center items-center p-8 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded-lg border border-dashed border-gray-300 dark:border-gray-700">
                        暂无黑名单项，请添加新项
                    </div>
                </Show>
            </div>
        </div>
    );
};

export default BlacklistComponent;

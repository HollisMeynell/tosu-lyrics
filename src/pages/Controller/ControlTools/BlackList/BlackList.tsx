import { Component, For, Show, createSignal } from "solid-js";
import { BlockRule, BlockScope } from "@/services/blocksService";
import { createBlocksController } from "@/hooks/useBlocks";
import { Button } from "@/components/ui";
import { Delete, Save, Cancel, Edit } from "@/assets/Icons";

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

const SCOPE_LABEL: Record<BlockScope, string> = {
    bid: "单谱面 (bid)",
    sid: "谱面集 (sid)",
    title: "标题",
};

const scopeLabel = (scope: BlockScope) => SCOPE_LABEL[scope] ?? scope;

const BlacklistComponent: Component<{
    controller: ReturnType<typeof createBlocksController>;
    scope: () => BlockScope;
}> = (props) => {
    const c = props.controller;

    const [newItem, setNewItem] = createSignal<{
        value: string;
        title: string;
        reason: string;
    }>({ value: "", title: "", reason: "" });

    const [editingId, setEditingId] = createSignal<number | null>(null);
    const [editingTitle, setEditingTitle] = createSignal("");
    const [editingReason, setEditingReason] = createSignal("");

    const handleAdd = async () => {
        const item = newItem();
        if (!item.value.trim()) return;
        const ok = await c.add({
            scope: props.scope(),
            value: item.value.trim(),
            title: item.title.trim(),
            reason: item.reason.trim(),
        });
        // 只有服务端确实写入成功才清空表单，失败时保留用户输入以便重试
        if (ok) setNewItem({ value: "", title: "", reason: "" });
    };

    const startEdit = (rule: BlockRule) => {
        setEditingId(rule.id);
        setEditingTitle(rule.title);
        setEditingReason(rule.reason);
    };

    const handleSaveEdit = async () => {
        const id = editingId();
        if (id === null) return;
        const ok = await c.update(id, {
            title: editingTitle(),
            reason: editingReason(),
        });
        if (ok) setEditingId(null);
    };

    const handleRemove = async (rule: BlockRule) => {
        await c.remove(rule.id);
    };

    const addItemForm = () => {
        return (
            <div class="flex flex-col gap-4 w-full lg:w-1/3 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg shadow-sm">
                <h3 class="text-xl font-semibold text-gray-800 dark:text-white border-b pb-2 mb-2">
                    添加新的黑名单项
                </h3>
                <div class="flex flex-col gap-4">
                    <p class="text-xs text-gray-500">
                        新增项使用页面当前作用域：
                        <span class="font-medium">{scopeLabel(props.scope())}</span>
                    </p>
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            {props.scope() === "title" ? "标题：" : "ID："}
                        </label>
                        <input
                            type="text"
                            value={newItem().value}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    value: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder={
                                props.scope() === "title" ? "输入标题" : "输入ID"
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            备注：
                        </label>
                        <input
                            type="text"
                            value={newItem().reason}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    reason: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder="输入备注（可选）"
                        />
                    </div>
                    <div class="form-group">
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            名称：
                        </label>
                        <input
                            type="text"
                            value={newItem().title}
                            onInput={(e) =>
                                setNewItem({
                                    ...newItem(),
                                    title: e.currentTarget.value,
                                })
                            }
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                            placeholder="输入名称（可选）"
                        />
                    </div>
                </div>
                <Button
                    class="w-full mt-2 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
                    onClick={handleAdd}
                    disabled={c.saving()}
                >
                    添加
                </Button>

                <div class="flex flex-col sm:flex-row gap-3 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                    <Button
                        class="flex-1"
                        onClick={() => c.load()}
                        disabled={c.loading()}
                    >
                        刷新列表
                    </Button>

                    <Button
                        class="flex-1"
                        onClick={() => c.clear()}
                        disabled={c.saving()}
                    >
                        <Delete class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                        清空黑名单
                    </Button>
                </div>
            </div>
        );
    };

    const trItem = (rule: BlockRule) => {
        return (
            <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                <CustomTd>{rule.value}</CustomTd>
                <td class="py-3 px-4">
                    <input
                        type="text"
                        value={editingTitle()}
                        onInput={(e) => setEditingTitle(e.currentTarget.value)}
                        class="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    />
                </td>
                <td class="py-3 px-4">
                    <input
                        type="text"
                        value={editingReason()}
                        onInput={(e) => setEditingReason(e.currentTarget.value)}
                        class="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                    />
                </td>
                <CustomTd>{scopeLabel(rule.scope)}</CustomTd>
                <CustomTd>{`${new Date(rule.createdAt).toLocaleString()}`}</CustomTd>
                <td class="py-3 px-4">
                    <div class="flex space-x-2">
                        <button
                            onClick={handleSaveEdit}
                            disabled={c.saving()}
                            class="px-3 py-1 border border-green-500 text-white rounded-md hover:bg-green-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-green-500/50"
                        >
                            <Save class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                            保存
                        </button>
                        <button
                            onClick={() => setEditingId(null)}
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

    const trFallback = (rule: BlockRule) => {
        return (
            <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                <CustomTd>{rule.value}</CustomTd>
                <CustomTd>{rule.title}</CustomTd>
                <CustomTd>{rule.reason}</CustomTd>
                <CustomTd>{scopeLabel(rule.scope)}</CustomTd>
                <CustomTd>{`${new Date(rule.createdAt).toLocaleString()}`}</CustomTd>
                <td class="py-3 pl-4">
                    <div class="flex space-x-2">
                        <button
                            onClick={() => startEdit(rule)}
                            class="px-3 py-1 border border-blue-500 text-white rounded-md hover:bg-blue-600/50 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        >
                            <Edit class="mr-2 w-4 h-4 inline cursor-pointer select-none active:scale-90" />
                            编辑
                        </button>
                        <button
                            onClick={() => handleRemove(rule)}
                            disabled={c.saving()}
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
            {addItemForm()}

            <div class="blacklist-items flex-1 flex flex-col gap-4">
                <div class="flex justify-between items-center mb-2">
                    <h3 class="text-xl font-semibold text-gray-800 dark:text-white">
                        黑名单列表
                    </h3>
                    <span class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-full text-sm font-medium">
                        共 {c.items().length} 项
                    </span>
                </div>

                <Show when={c.error()}>
                    <div class="px-4 py-2 rounded-md bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300 text-sm">
                        操作失败：{c.error()}
                    </div>
                </Show>

                <div class="overflow-x-auto rounded-lg border border-gray-200 dark:border-gray-700">
                    <table class="w-full border-collapse">
                        <thead class="bg-gray-100 dark:bg-gray-800">
                            <tr>
                                <CustomTh>ID</CustomTh>
                                <CustomTh>名称</CustomTh>
                                <CustomTh>备注</CustomTh>
                                <CustomTh>作用域</CustomTh>
                                <CustomTh>时间</CustomTh>
                                <CustomTh>操作</CustomTh>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200 dark:bg-gray-900 dark:divide-gray-700">
                            <For each={c.items()}>
                                {(rule) => (
                                    <Show
                                        when={editingId() === rule.id}
                                        fallback={trFallback(rule)}
                                    >
                                        {trItem(rule)}
                                    </Show>
                                )}
                            </For>
                        </tbody>
                    </table>
                </div>

                <Show when={c.loading()}>
                    <div class="flex justify-center items-center p-4 text-gray-500 dark:text-gray-400">
                        加载中...
                    </div>
                </Show>

                <Show when={!c.loading() && c.items().length === 0}>
                    <div class="flex justify-center items-center p-8 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded-lg border border-dashed border-gray-300 dark:border-gray-700">
                        暂无黑名单项，请添加新项
                    </div>
                </Show>
            </div>
        </div>
    );
};

export default BlacklistComponent;

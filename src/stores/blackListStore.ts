import { createSignal } from "solid-js";

export const [blackListSignal, setBlackListSignal] = createSignal<number>(0);
export const blackListStore = {
    list: [] as string[],
    set: new Set<string>(),
};

// fixme: 这玩意在 tosuManager 中无法实时生效, set 数据不能及时同步 ???
export const inTitleBlackList = (title: string) => {
    return blackListStore.set.has(title);
};

export const getTitleBlackList = () => blackListStore.list;

export const resetTitleBlackList = (data?: string[]) => {
    if (data) {
        blackListStore.list = data;
        blackListStore.set = new Set(data);
        setBlackListSignal(data.length);
    } else {
        blackListStore.list = [];
        blackListStore.set = new Set();
        setBlackListSignal(0);
    }
};

export const addTitleBlackListItem = (title: string) => {
    if (blackListStore.set.has(title)) return;
    blackListStore.list.unshift(title);
    blackListStore.set.add(title);
    setBlackListSignal(blackListStore.list.length);
};

export const deleteTitleBlackListItem = (title: string) => {
    if (!blackListStore.set.has(title)) return;
    const index = blackListStore.list.indexOf(title);
    blackListStore.list.splice(index, 1);
    blackListStore.set.delete(title);
    setBlackListSignal(blackListStore.list.length);
};

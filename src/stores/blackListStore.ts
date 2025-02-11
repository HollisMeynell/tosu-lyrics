import { createStore, produce } from "solid-js/store";

export const [titleBlackList, setTitleBlackList] = createStore({
    list: [] as string[],
    set: new Set<string>(),
});

export const inTitleBlackList = (title: string) =>
    titleBlackList.set.has(title);

export const getTitleBlackList = () => titleBlackList.list;

export const resetTitleBlackList = (data?: string[]) => {
    if (data) {
        setTitleBlackList({
            list: data,
            set: new Set(data),
        });
    } else {
        setTitleBlackList({
            list: [],
            set: new Set(),
        });
    }
};

export const addTitleBlackListItem = (title: string) => {
    // if (titleBlackList.set.has(title)) return;
    if (titleBlackList.list.indexOf(title) >= 0) return;
    setTitleBlackList(
        "set",
        produce((set) => {
            set.add(title);
        })
    );
    setTitleBlackList(
        "list",
        produce((list) => {
            list.unshift(title);
        })
    );
};

export const deleteTitleBlackListItem = (title: string) => {
    // if (!titleBlackList.set.has(title)) return;
    if (titleBlackList.list.indexOf(title) < 0) return;
    setTitleBlackList(
        "set",
        produce((set) => {
            set.delete(title);
        })
    );
    setTitleBlackList(
        "list",
        produce((list) => {
            const index = list.indexOf(title);
            list.splice(index, 1);
        })
    );
};

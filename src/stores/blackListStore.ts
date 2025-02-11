import { createStore, produce } from "solid-js/store";
import { useNavigate } from "@solidjs/router";

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

function toggleController() {
    // 切换设置页
    let isController = window.location.href.indexOf("controller") >= 0;
    const navigate = useNavigate(); // fixme: 不知道为什么会报错?
    if (isController) {
        navigate("/lyrics", { replace: true });
    } else {
        navigate("/lyrics/controller", { replace: true });
    }
}

// 触发 Controller
window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.altKey && e.key === "t") {
        toggleController();
    }
});

// 触发 Controller (Mobile)
window.addEventListener("touchstart", (e) => {
    if (e.touches.length === 3) {
        toggleController();
        e.preventDefault();
    }
});
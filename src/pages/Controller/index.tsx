// 功能: 功能操作面板
import { DarkModeToggle } from "@/components/ui";
import { Component, JSX } from "solid-js";
import { A } from "@solidjs/router";
import { SettingIcon } from "@/assets/Icons";
import { useLocation } from "@solidjs/router";

interface ControllerProps {
    children: JSX.Element;
}

interface CustomAProps {
    href: string;
    icon: string;
    classList?: Record<string, boolean>;
}

const CustomA: Component<CustomAProps> = (props) => {
    const location = useLocation();
    const isActive = () =>
        location.pathname === props.href ||
        (props.icon == "default" && location.pathname == "/lyrics/controller");
    return (
        <A
            href={props.href}
            classList={{
                "before:transform before:scale-0": !isActive(),
                ...props.classList,
            }}
            class="h-10 rounded-lg p-2 relative before:w-full before:h-full
            before:absolute before:top-0 before:left-0 before:z-[-1]
            before:bg-[#ffc5e2bc] dark:before:bg-[#ec4899] before:rounded
            before:transition-transform before:duration-300"
        >
            <SettingIcon type={props.icon} class="w-6 h-6" />
        </A>
    );
};

const Controller: Component<ControllerProps> = (props) => {
    // 这里原先有一层"必须先选择一个客户端才能管理"的遮罩，依赖旧前端的
    // `wsService.clientSignal()`（靠旧协议的 online/互查消息维护）。
    // 新版后端是**全局单一状态**（一首当前歌曲 + 一份全局设置），
    // 根本不存在"其他浏览器客户端"这个概念，那个信号因此恒为 false，
    // 遮罩会把所有管理页都盖住、真实用户点不动任何控件。
    // 迁移到 HTTP 的全局管理页不需要这个前置条件，故移除（F-01 的最小前置修复）。
    // `/lyrics/controller/client` 页面本身保持原样，留给 B-08 / F-07。

    // 导航栏配置
    const navItems = [
        { href: "/lyrics/controller/client", icon: "default" },
        { href: "/lyrics/controller/content", icon: "content" },
        { href: "/lyrics/controller/textstyle", icon: "palette" },
        { href: "/lyrics/controller/blackList", icon: "blackList" },
        { href: "/lyrics/controller/cacheManager", icon: "cache" },
        { href: "/lyrics/controller/upload", icon: "content" },
        { href: "/lyrics/controller/shadow", icon: "palette" },
    ];

    //通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560
    return (
        <div
            class="h-[calc(100%-300px)] bg-[#ffffff] dark:bg-[#141414]
            m-6 pl-6 pr-8 py-8 rounded-lg shadow-md overflow-hidden scrollbar-hide
            dark:text-[#dcdcdc] text-ellipsis text-nowrap selection:bg-[#ffd4ea] selection:text-[#ec4899] dark:selection:bg-fuchsia-900 dark:selection:text-fuchsia-100 relative transform-3d"
        >
            <div class="fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
                <nav class="w-10 mx-auto flex flex-col justify-center items-center gap-4">
                    {navItems.map((item) => (
                        <CustomA href={item.href} icon={item.icon} />
                    ))}
                </nav>
            </div>
            <div class="ml-18 mr-8 h-full relative overflow-y-auto overflow-x-hidden scrollbar-hide">
                {props.children}
            </div>
            <DarkModeToggle />
        </div>
    );
};

export default Controller;

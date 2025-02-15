// 功能: 功能操作面板
import { Button, DarkModeToggle } from "@/components/ui";
import { Component, JSX, Show } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { SettingIcon } from "@/assets/Icons";
import { useLocation } from "@solidjs/router";
import { Mask } from "@/components/ui";
import { wsService } from "@/services/webSocketService.ts";

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
    const location = useLocation();
    const navigate = useNavigate();
    const shouldShowMask = () => {
        const isClientPage =
            location.pathname === "/lyrics/controller/client" ||
            location.pathname === "/lyrics/controller/" ||
            location.pathname === "/lyrics/controller";
        return !isClientPage && !wsService.clientSignal();
    };

    const jumpToClient = () => {
        navigate("/lyrics/controller/client");
    };

    const Tips = () => (
        <Button onClick={jumpToClient}>当前客户端不可用, 请选择客户端</Button>
    );

    // 导航栏配置
    const navItems = [
        { href: "/lyrics/controller/client", icon: "default" },
        { href: "/lyrics/controller/content", icon: "content" },
        { href: "/lyrics/controller/textstyle", icon: "palette" },
        { href: "/lyrics/controller/blackList", icon: "blackList" },
        { href: "/lyrics/controller/cacheManager", icon: "cache" },
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
            <Show when={shouldShowMask()}>
                <Mask class="ml-16 h-full">
                    <Tips />
                </Mask>
            </Show>
            <DarkModeToggle />
        </div>
    );
};

export default Controller;

// 功能: 功能操作面板
import { DarkModeToggle } from "@/components/ui";
import { Component, JSX } from "solid-js";
import { A } from "@solidjs/router";
import { SettingIcon } from "@/assets/Icons";
import { useLocation } from "@solidjs/router";

interface ControllerProps {
    children: JSX.Element;
}

const CustomA: Component<{
    href: string;
    classList?: Record<string, boolean>;
    icon: string;
}> = (props) => {
    const location = useLocation();
    return (
        <A
            href={props.href}
            classList={{
                "bg-[#ffc5e2bc]": location.pathname === props.href,
            }}
            class="h-10 rounded-lg p-2"
        >
            <SettingIcon type={props.icon} class="w-6 h-6" />
        </A>
    );
};

const Controller: Component<ControllerProps> = (props) => {
    //通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560
    return (
        <div
            class="h-[calc(100%-300px)] bg-[#ffffff] dark:bg-[#141414]
            m-6 pl-6 pr-8 py-8 rounded-lg shadow-md overflow-hidden scrollbar-hide
            dark:text-[#dcdcdc] text-ellipsis text-nowrap relative transform-3d"
        >
            <div class="fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
                <nav class="w-10 mx-auto flex flex-col justify-center items-center gap-4">
                    <CustomA href="/lyrics/controller/client" icon="client" />
                    <CustomA href="/lyrics/controller/content" icon="content" />
                    <CustomA
                        href="/lyrics/controller/textstyle"
                        icon="palette"
                    />
                    <CustomA
                        href="/lyrics/controller/blackList"
                        icon="blackList"
                    />
                    <CustomA href="/lyrics/controller/blackList" icon="cache" />
                </nav>
            </div>
            <div class="ml-18 mr-8 h-full overflow-y-auto overflow-x-hidden scrollbar-hide">
                {props.children}
            </div>
            <DarkModeToggle />
        </div>
    );
};

export default Controller;

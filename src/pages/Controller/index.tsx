// 功能: 功能操作面板
import DarkModeToggle from "@/components/ui/DarkModeToggle.tsx";
import { Component, JSX } from "solid-js";
import { A } from "@solidjs/router";
import Palette from "@/assets/Icons/Palette.tsx";
import Content from "@/assets/Icons/Content.tsx";

interface ControllerProps {
    children: JSX.Element;
}

const Controller: Component<ControllerProps> = (props) => {
    //通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560
    return (
        <div
            class="h-[calc(100%-300px)] bg-[#ffffff] dark:bg-[#141414]
            m-6 pl-6 pr-8 py-8 rounded-lg shadow-md overflow-auto scrollbar-hide
            dark:text-[#dcdcdc] relative transform-3d"
        >
            <div class="fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
                <nav class="w-6 mx-auto flex flex-col gap-6">
                    <A href="/lyrics/controller/client">
                        O
                    </A>
                    <A href="/lyrics/controller/blackList">
                        B
                    </A>
                    <A href="/lyrics/controller/textstyle">
                        <Palette class="w-6 h-6" />
                    </A>
                    <A href="/lyrics/controller/content">
                        <Content class="w-6 h-6" />
                    </A>
                </nav>
            </div>
            <div class="ml-10 px-8">{props.children}</div>
            <DarkModeToggle />
        </div>
    );
};

export default Controller;

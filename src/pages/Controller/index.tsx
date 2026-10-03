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
    // 导航栏配置（阴影 / 上传与字体已并入文字样式页，不再单独占一项）
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
            m-4 pl-6 pr-6 py-4 rounded-lg shadow-md overflow-hidden scrollbar-hide
            dark:text-[#dcdcdc] text-ellipsis text-nowrap selection:bg-[#ffd4ea] selection:text-[#ec4899] dark:selection:bg-fuchsia-900 dark:selection:text-fuchsia-100 relative transform-3d"
        >
            <div class="fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
                <nav class="w-10 mx-auto flex flex-col justify-center items-center gap-4">
                    {navItems.map((item) => (
                        <CustomA href={item.href} icon={item.icon} />
                    ))}
                </nav>
            </div>
            {/* 公共层：把控制台 UI 的字体显式钉在界面字体上。
                动态的歌词 font-family 只允许作用在歌词元素（LyricsBox / 歌词预览），
                绝不能顺着继承链污染表单控件 —— 原生 <select> 一旦继承到缺字的
                自定义字体就会整体回退成微软雅黑。这里作用范围仅限 Controller 内容区，
                ControllerLayout 里的 <LyricsBox /> 在它之外，不受影响。 */}
            <div
                class="ml-18 mr-8 h-full relative overflow-y-auto overflow-x-hidden scrollbar-hide"
                style={{
                    "font-family":
                        "var(--font-sans, ui-sans-serif, system-ui, sans-serif)",
                }}
            >
                {props.children}
            </div>
            <DarkModeToggle />
        </div>
    );
};

export default Controller;

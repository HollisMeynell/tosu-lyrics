// 功能: 功能操作面板
import { DarkModeToggle, Mask } from "@/components/ui";
import { Component, JSX, Show, createEffect, createSignal, on } from "solid-js";
import { A, useLocation } from "@solidjs/router";
import { SettingIcon } from "@/assets/Icons";

interface ControllerProps {
    children: JSX.Element;
}

interface NavItem {
    href: string;
    icon: string;
    label: string;
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
        (props.icon == "content" && location.pathname == "/lyrics/controller");
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
            before:transition-transform before:duration-300
            after:absolute after:-inset-1 after:content-['']"
        >
            <SettingIcon type={props.icon} class="w-6 h-6" />
        </A>
    );
};

const DrawerItem: Component<NavItem & { onClick: () => void }> = (props) => {
    const location = useLocation();
    const isActive = () =>
        location.pathname === props.href ||
        (props.icon == "content" && location.pathname == "/lyrics/controller");
    return (
        <A
            href={props.href}
            onClick={props.onClick}
            classList={{
                "bg-[#ffc5e2bc] dark:bg-[#ec4899]": isActive(),
            }}
            class="flex items-center gap-3 rounded-lg p-3 select-none
            active:scale-95 transition-transform"
        >
            <SettingIcon type={props.icon} class="w-6 h-6" />
            <span>{props.label}</span>
        </A>
    );
};

const HamburgerIcon: Component<{ class: string }> = (props) => {
    return (
        <svg
            {...props}
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
        >
            <path d="M4 6h16" />
            <path d="M4 12h16" />
            <path d="M4 18h16" />
        </svg>
    );
};

const Controller: Component<ControllerProps> = (props) => {
    // 导航栏配置（阴影 / 上传与字体已并入文字样式页，不再单独占一项）
    const navItems: NavItem[] = [
        {
            href: "/lyrics/controller/content",
            icon: "content",
            label: "歌词内容",
        },
        {
            href: "/lyrics/controller/lyricSearch",
            icon: "search",
            label: "歌词搜索",
        },
        {
            href: "/lyrics/controller/textstyle",
            icon: "palette",
            label: "文字样式",
        },
        {
            href: "/lyrics/controller/blackList",
            icon: "blackList",
            label: "黑名单",
        },
        {
            href: "/lyrics/controller/cacheManager",
            icon: "cache",
            label: "缓存管理",
        },
        {
            href: "/lyrics/controller/qrcode",
            icon: "qr",
            label: "连接二维码",
        },
    ];

    const location = useLocation();
    const [drawerOpen, setDrawerOpen] = createSignal(false);
    const currentLabel = () =>
        navItems.find(
            (item) =>
                location.pathname === item.href ||
                (item.icon === "content" &&
                    location.pathname === "/lyrics/controller")
        )?.label ?? "控制台";

    createEffect(
        on(
            () => location.pathname,
            () => setDrawerOpen(false)
        )
    );

    //通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560
    return (
        <div
            class="flex-1 min-h-0 flex flex-col bg-[#ffffff] dark:bg-[#141414]
            m-2 px-3 py-4 rounded-lg shadow-md overflow-hidden scrollbar-hide md:m-4 md:px-6
            dark:text-[#dcdcdc] md:text-ellipsis md:text-nowrap selection:bg-[#ffd4ea] selection:text-[#ec4899] dark:selection:bg-fuchsia-900 dark:selection:text-fuchsia-100 relative transform-3d"
        >
            <div class="hidden md:block fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
                <nav class="w-10 mx-auto flex flex-col justify-center items-center gap-4">
                    {navItems.map((item) => (
                        <CustomA href={item.href} icon={item.icon} />
                    ))}
                </nav>
            </div>

            {/* 移动端顶栏与抽屉：图标栏收进抽屉并补文字，桌面仍用左侧栏 */}
            <div class="md:hidden flex items-center gap-2 pr-12 pb-2">
                <button
                    aria-label="打开菜单"
                    class="p-2 -ml-2 rounded-lg cursor-pointer select-none
                    active:scale-90 transition-transform
                    after:absolute after:-inset-1 after:content-['']"
                    onClick={() => setDrawerOpen(true)}
                >
                    <HamburgerIcon class="w-6 h-6" />
                </button>
                <span class="font-medium truncate">{currentLabel()}</span>
            </div>

            <Show when={drawerOpen()}>
                <Mask class="md:hidden" onClick={() => setDrawerOpen(false)} />
                <nav
                    class="fixed top-0 left-0 z-[110] h-full w-64 max-w-[80%]
                    flex flex-col gap-1 py-6 px-4
                    bg-[#ffffff] dark:bg-[#141414] border-r-2 border-[#f0f0f0] dark:border-[#313131]
                    shadow-xl animate-drawer-in md:hidden"
                >
                    {navItems.map((item) => (
                        <DrawerItem
                            href={item.href}
                            icon={item.icon}
                            label={item.label}
                            onClick={() => setDrawerOpen(false)}
                        />
                    ))}
                </nav>
            </Show>
            {/* 公共层：把控制台 UI 的字体显式钉在界面字体上。
                动态的歌词 font-family 只允许作用在歌词元素（LyricsBox / 歌词预览），
                绝不能顺着继承链污染表单控件 —— 原生 <select> 一旦继承到缺字的
                自定义字体就会整体回退成微软雅黑。这里作用范围仅限 Controller 内容区，
                ControllerLayout 里的 <LyricsBox /> 在它之外，不受影响。 */}
            <div
                class="mr-10 flex-1 min-h-0 relative overflow-y-auto overflow-x-hidden scrollbar-hide md:ml-18 md:mr-8"
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

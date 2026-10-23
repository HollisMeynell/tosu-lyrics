// routes/index.tsx
import { onMount, onCleanup } from "solid-js";
import {
    useNavigate,
    useLocation,
    Route,
    RouteSectionProps,
} from "@solidjs/router";
import { Component } from "solid-js";
import { lazy } from "solid-js";
import { CONTROLLER_VIEWPORT_LINES } from "@/utils/lyricLines";

const LyricsBox = lazy(() => import("@/pages/LyricsBox"));
const Controller = lazy(() => import("@/pages/Controller"));
const ClientList = lazy(() => import("@/pages/Controller/ControlTools/Client"));
const BlackListLyrics = lazy(
    () => import("@/pages/Controller/ControlTools/BlackList")
);
const Content = lazy(() => import("@/pages/Controller/ControlTools/Content"));
const LyricSearch = lazy(
    () => import("@/pages/Controller/ControlTools/LyricSearch")
);
const TextStyle = lazy(
    () => import("@/pages/Controller/ControlTools/TextStyle")
);
const FontSettings = lazy(
    () => import("@/pages/Controller/ControlTools/FontSettings")
);
const CacheManager = lazy(
    () => import("@/pages/Controller/ControlTools/CacheManager")
);
const QrCode = lazy(() => import("@/pages/Controller/ControlTools/QrCode"));

const RoutesRoot: Component<RouteSectionProps<unknown>> = (props) => {
    const navigate = useNavigate();
    const location = useLocation();

    function toggleController() {
        // 切换设置页
        const isController = location.pathname.startsWith("/lyrics/controller");
        if (isController) {
            navigate("/lyrics");
        } else {
            navigate("/lyrics/controller");
        }
    }

    onMount(() => {
        const handleKeydown = (e: KeyboardEvent) => {
            if (e.ctrlKey && e.altKey && e.key === "t") {
                toggleController();
            }
        };

        const handleTouchstart = (e: TouchEvent) => {
            if (e.touches.length === 3) {
                toggleController();
                e.preventDefault();
            }
        };

        window.addEventListener("keydown", handleKeydown);
        window.addEventListener("touchstart", handleTouchstart);

        onCleanup(() => {
            window.removeEventListener("keydown", handleKeydown);
            window.removeEventListener("touchstart", handleTouchstart);
        });
    });

    return <>{props.children}</>;
};

/**
 * Controller 布局：顶部是歌词预览，下面是各控制面板。
 *
 * 顶部的 `<LyricsBox />` 与 `/lyrics` 是同一个组件、同一份 `lyricStore` 与
 * `lyricLoading`（控制台现在也接入展示端 WS），所以歌词内容、当前行与加载动画
 * 天然与展示端一致，不存在第二套实现。
 *
 * 但控制台传 `viewportLines`：预览高度**固定为默认 3 行**，不随用户设置的
 * lyricLines(5/7/…/15) 变高，控制面板的位置始终与 3 行时一致；多出来的歌词在
 * 这个固定 viewport 内被裁剪/隐藏。`/lyrics` 不传该 prop，仍按设置完整显示。
 *
 * 移动端窄屏时预览区整体用 `zoom` 缩小（布局高度按比例变矮，跑马灯测宽
 * 数学不受影响）；`zoom` 只作用在预览上，`/lyrics` 展示页不受影响。
 */
const ControllerLayout: Component<RouteSectionProps<unknown>> = (props) => (
    <div class="h-full flex flex-col justify-center bg-white">
        <div class="max-md:[zoom:0.7]">
            <LyricsBox viewportLines={CONTROLLER_VIEWPORT_LINES} />
        </div>
        <Controller children={props.children} />
    </div>
);

/**
 * `/lyrics` 展示页。
 *
 * OBS 浏览器源是细指针（`pointer: fine`），保持原有顶对齐 + 透明背景；
 * 触屏设备（手机 / 平板，`pointer: coarse`）竖直居中、避开刘海安全区，
 * 并铺深色底 —— 默认歌词是浅色，透明背景在手机浏览器里会白底白字。
 */
const LyricsDisplay = () => (
    <div
        class="h-full flex flex-col justify-start
        [@media(pointer:coarse)]:justify-center
        [@media(pointer:coarse)]:bg-[#141414]
        [@media(pointer:coarse)]:pt-[env(safe-area-inset-top)]
        [@media(pointer:coarse)]:pb-[env(safe-area-inset-bottom)]"
    >
        <LyricsBox />
    </div>
);

export default function AppRoutes() {
    return (
        <Route path="/lyrics" component={(props) => <RoutesRoot {...props} />}>
            <Route path="/" component={LyricsDisplay} />
            <Route path="/lyric" component={LyricsDisplay} />
            <Route path="/controller" component={ControllerLayout}>
                <Route path="/" component={Content} />
                <Route path="/blackList" component={BlackListLyrics} />
                <Route path="/client" component={ClientList} />
                <Route path="/content" component={Content} />
                <Route path="/lyricSearch" component={LyricSearch} />
                <Route path="/textstyle" component={TextStyle} />
                <Route path="/fontSettings" component={FontSettings} />
                <Route path="/cacheManager" component={CacheManager} />
                <Route path="/qrcode" component={QrCode} />
            </Route>
        </Route>
    );
}

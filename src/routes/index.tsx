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
const TextStyle = lazy(
    () => import("@/pages/Controller/ControlTools/TextStyle")
);
const CacheManager = lazy(
    () => import("@/pages/Controller/ControlTools/CacheManager")
);

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
 */
const ControllerLayout: Component<RouteSectionProps<unknown>> = (props) => (
    <>
        <LyricsBox viewportLines={CONTROLLER_VIEWPORT_LINES} />
        <Controller children={props.children} />
    </>
);

export default function AppRoutes() {
    return (
        <Route path="/lyrics" component={(props) => <RoutesRoot {...props} />}>
            <Route path="/" component={() => <LyricsBox />} />
            <Route path="/lyric" component={() => <LyricsBox />} />
            <Route path="/controller" component={ControllerLayout}>
                <Route path="/" component={ClientList} />
                <Route path="/blackList" component={BlackListLyrics} />
                <Route path="/client" component={ClientList} />
                <Route path="/content" component={Content} />
                <Route path="/textstyle" component={TextStyle} />
                <Route path="/cacheManager" component={CacheManager} />
            </Route>
        </Route>
    );
}

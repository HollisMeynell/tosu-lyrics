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

export default function AppRoutes() {
    return (
        <Route path="/lyrics" component={(props) => <RoutesRoot {...props} />}>
            <Route path="/" component={() => <LyricsBox debug={false} />} />
            <Route
                path="/lyric"
                component={() => <LyricsBox debug={false} />}
            />
            <Route
                path="/controller"
                component={(props) => (
                    <>
                        <LyricsBox debug={true} />
                        <Controller children={props.children} />
                    </>
                )}
            >
                <Route path="/" component={ClientList} />
                <Route path="/blackList" component={BlackListLyrics} />
                <Route path="/client" component={ClientList} />
                <Route path="/content" component={Content} />
                <Route path="/textstyle" component={TextStyle} />
            </Route>
        </Route>
    );
}

// routes/index.tsx
import { Route } from "@solidjs/router";
import { lazy } from "solid-js";

const LyricsBox = lazy(() => import("@/pages/LyricsBox"));
const Controller = lazy(() => import("@/pages/Controller"));
const ClientList = lazy(
    () => import("@/pages/Controller/ControlTools/Content/ClientList")
);
const CurrentLyrics = lazy(
    () => import("@/pages/Controller/ControlTools/Content/CurrentLyrics")
);
const TextStyle = lazy(
    () => import("@/pages/Controller/ControlTools/TextStyle")
);

export default function AppRoutes() {
    return (
        <Route path="/lyrics" component={(props) => <>{props.children}</>}>
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
                <Route path="/client" component={ClientList} />
                <Route path="/content" component={CurrentLyrics} />
                <Route path="/textstyle" component={TextStyle} />
            </Route>
        </Route>
    );
}

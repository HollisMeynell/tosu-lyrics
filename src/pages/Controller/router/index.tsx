import { lazy } from "solid-js";
import { Router, Route, A, RouteSectionProps } from "@solidjs/router";
import Palette from "@/assets/Icons/Palette.tsx";
import Content from "@/assets/Icons/Content.tsx";

const CurrentLyrics = lazy(
    () => import("@/pages/Controller/ControlTools/CurrentLyrics.tsx")
);
const TextStyle = lazy(
    () => import("@/pages/Controller/ControlTools/TextStyle.tsx")
);

const App = (props: RouteSectionProps<unknown>) => (
    <>
        <div class="fixed top-0 left-0 w-16 h-full border-r-2 border-[#f0f0f0] dark:border-[#313131] py-6">
            <nav class="w-6 mx-auto flex flex-col gap-6">
                <A href="/textstyle">
                    <Palette class="w-6 h-6" />
                </A>
                <A href="/lyrics">
                    <Content class="w-6 h-6" />
                </A>
            </nav>
        </div>
        <div class="ml-10 px-8">{props.children}</div>
    </>
);

const AppRoutes = () => (
    <Router root={App}>
        <Route path="/" component={CurrentLyrics} />
        <Route path="/lyrics" component={CurrentLyrics} />
        <Route path="/textstyle" component={TextStyle} />
    </Router>
);

export default AppRoutes;

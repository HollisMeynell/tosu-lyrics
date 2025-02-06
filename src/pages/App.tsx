import { Component, lazy, onMount } from "solid-js";
import { paramParse, parseUrlParams } from "@/utils/param-parse.ts";
import { configService } from "@/services/ConfigService.ts";
import { lyricsStore } from "@/stores/lyricsStore.ts";
import { Route, Router, RouteSectionProps } from "@solidjs/router";

const LyricsBox = lazy(() => import("@/pages/LyricsBox"));
const Controller = lazy(() => import("@/pages/Controller"));

const AppRoot: Component<RouteSectionProps> = (props) => {
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    onMount(() => {
        const params = parseUrlParams(window.location.href);
        paramParse(params);
    });

    onMount(async () => {
        try {
            const config = await configService.fetchConfig();
            lyricsStore.parseSettings(config);
        } catch (error) {
            console.error("Failed to initialize:", error);
        }
    });

    return <div class="h-full flex flex-col justify-center bg-transparent">
        {props.children}
    </div>;
};

export default function App() {
    return <Router root={AppRoot}>
        <Route path={"/"} component={() => <LyricsBox debug={false} />} />
        <Route path={"/controller"} component={() => <>
            <LyricsBox debug={true} />
            <Controller />
        </>}
        />
    </Router>;
}

import { onMount, Show } from "solid-js";
import { paramParse, parseUrlParams } from "@/utils/param-parse.ts";
import { configService } from "@/services/ConfigService.ts";
import LyricsBox from "@/pages/LyricsBox";
import Controller from "@/pages/Controller";
import { lyricsStore, showController } from "@/stores/lyricsStore.ts";

export default function App() {
    // @ts-ignore
    if (import.meta.env.MODE === "development") {
        document.body.style.backgroundColor = "#3d2932";
    }

    onMount(() => {
        const params = parseUrlParams(window.location.href);
        paramParse(params);
    })

    onMount(async () => {
        try {
            const config = await configService.fetchConfig();
            lyricsStore.parseSettings(config);
        } catch (error) {
            console.error("Failed to initialize:", error);
        }
    });

    return (
        <div class="h-full flex flex-col justify-center bg-transparent">
            <LyricsBox debug={showController()} />
            <Show when={showController()}>
                <Controller />
            </Show>
        </div>
    );
}

// 功能: 功能操作面板
import { lyricsStore } from "@/stores/lyricsStore";
import { Match, Switch } from "solid-js";

export default function Controller() {
    return (
        <button
            class="fixed top-4 right-4 w-8 h-8 border-2 border-[#f0f0f0] dark:border-[#313131] rounded-md cursor-pointer flex items-center justify-center"
            onClick={lyricsStore.toggleDarkMode}
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="w-6 h-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="#6a7282"
                stroke-width={2}
            >
                <Switch>
                    <Match when={lyricsStore.getState.darkMode}>
                        <circle cx="12" cy="12" r="4" />
                        <path d="M12 2v2" />
                        <path d="M12 20v2" />
                        <path d="m4.93 4.93 1.41 1.41" />
                        <path d="m17.66 17.66 1.41 1.41" />
                        <path d="M2 12h2" />
                        <path d="M20 12h2" />
                        <path d="m6.34 17.66-1.41 1.41" />
                        <path d="m19.07 4.93-1.41 1.41" />
                    </Match>
                    <Match when={!lyricsStore.getState.darkMode}>
                        <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
                    </Match>
                </Switch>
            </svg>
        </button>
    );
}

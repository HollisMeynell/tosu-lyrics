/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import LyricsBox from "@/components/LyricsBox";
import Controller from "@/components/Controller";
import { onMount, Show } from "solid-js";
import { lyricsStore } from "@/stores/lyricsStore";
import { configService } from "@/services/ConfigService";
import {
    consoleEnabled, lyricDebugEnabled,
    paramParse,
    parseUrlParams, setConsoleEnabled, setLyricDebugEnabled,
} from "@/utils/param-parse";

declare global {
    interface Window {
        ignoreCORS?: boolean;
    }
}

const root = document.getElementById("root");

if (import.meta.env.DEV && (root === null || !(root instanceof HTMLElement))) {
    throw new Error(
        "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
    );
}

// 错误回退组件
const Fallback = (err: Error) => {
    console.error(err);
    return (
        <div
            class="bg-gradient-to-br from-[#ff9a9e] to-[#fad0c4] text-white p-10 rounded-lg shadow-md text-center max-w-[500px] mx-auto my-12 font-sans">
            <h2 class="text-2xl mb-5">⚠️ Oops! Something went wrong</h2>
            <div class="bg-white/20 p-4 rounded mb-5">
                <p class="m-0 break-words">{err.toString()}</p>
            </div>
            <button
                class="bg-white text-[#ff6f61] border-none px-5 py-2.5 rounded cursor-pointer text-base font-bold shadow-sm hover:bg-gray-100 transition-colors duration-300"
                onClick={() => window.location.reload()}
            >
                Reload Page
            </button>
            <div class="mt-5">
                <p class="text-sm text-white/80">
                    If the problem persists, please contact support.
                </p>
            </div>
        </div>
    );
};

// 根组件
const Root = () => {
    onMount(() => {
        const params = parseUrlParams(window.location.href);
        paramParse(params);

        // dev 模式时增加背景色
        if (import.meta.env.MODE === "development") {
            document.body.style.backgroundColor = "#3d2932";
            setConsoleEnabled(true);
            setLyricDebugEnabled(false);
        }
    });

    onMount(async () => {
        try {
            // 初始化, 优先加载本地配置
            lyricsStore.executeDarkModeToggle();
            const config = await configService.fetchConfig();
            lyricsStore.parseSettings(config);
        } catch (error) {
            console.error("Failed to initialize:", error);
        }
    });

    return (
        <div class="h-full flex flex-col justify-center bg-transparent">
            <LyricsBox debug={lyricDebugEnabled} />
            <Show when={consoleEnabled()}>
                <Controller />
            </Show>
        </div>
    );
};

// 渲染应用
render(
    () => (
        <ErrorBoundary fallback={Fallback}>
            <Root />
        </ErrorBoundary>
    ),
    root!,
);

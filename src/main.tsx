/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import Styles from "./main.module.scss";
import LyricsBox from "./components/lyrics-box";
import CenterBox from "./components/center-box";

const root = document.getElementById("root");

if (import.meta.env.DEV && (root === null || !(root instanceof HTMLElement))) {
    throw new Error(
        "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?"
    );
}

// 错误回退组件
const Fallback = (err: Error) => {
    return (
        <div class={Styles.fallbackContainer}>
            <h2 class={Styles.fallbackTitle}>
                ⚠️ Oops! Something went wrong
            </h2>
            <div class={Styles.fallbackError}>
                <p>{err.toString()}</p>
            </div>
            <button
                class={Styles.fallbackButton}
                onClick={() => window.location.reload()}
            >
                Reload Page
            </button>
            <div class={Styles.fallbackFooter}>
                <p>If the problem persists, please contact support.</p>
            </div>
        </div>
    );
};

// 根组件
const Root = () => (
    <CenterBox>
        <LyricsBox />
    </CenterBox>
);

// 渲染应用
render(
    () => (
        <ErrorBoundary fallback={Fallback}>
            <Root />
        </ErrorBoundary>
    ),
    root!
);

/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
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
        <div class="fallback-container">
            <h2 class="fallback-title">
                ⚠️ Oops! Something went wrong
            </h2>
            <div class="fallback-error">
                <p>{err.toString()}</p>
            </div>
            <button
                class="fallback-button"
                onClick={() => window.location.reload()}
            >
                Reload Page
            </button>
            <div class="fallback-footer">
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
    document.getElementById("root")!
);

/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import App from "@/pages/App.tsx";
import ErrorFallback from "@/components/error/ErrorFallback.tsx";

declare global {
    interface Window {
        ignoreCORS?: boolean;
    }
}

const root = document.getElementById("root");

if (root === null || !(root instanceof HTMLElement)) {
    throw new Error(
        "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
    );
}

render(
    () => (
        <ErrorBoundary fallback={ErrorFallback}>
            <App />
        </ErrorBoundary>
    ),
    root!
);

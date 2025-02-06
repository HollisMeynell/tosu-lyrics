/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import ErrorPage from "@/pages/ErrorPage.tsx";
import App from "@/pages/App.tsx";

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
        <ErrorBoundary fallback={ErrorPage}>
            <App />
        </ErrorBoundary>
    ),
    root!
);

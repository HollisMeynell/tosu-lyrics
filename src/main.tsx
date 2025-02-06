/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import ErrorFallback from "@/components/error/ErrorFallback";
import App from "@/App";

const root = document.getElementById("root");

render(
    () => (
        <ErrorBoundary fallback={ErrorFallback}>
            <App />
        </ErrorBoundary>
    ),
    root!
);

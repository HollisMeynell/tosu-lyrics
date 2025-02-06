/* @refresh reload */

import { render, ErrorBoundary } from "solid-js/web";
import "./index.css";
import ErrorPage from "@/components/error/ErrorFallback";
import App from "@/pages/App.tsx";

const root = document.getElementById("root");

render(
    () => (
        <ErrorBoundary fallback={ErrorPage}>
            <App />
        </ErrorBoundary>
    ),
    root!
);

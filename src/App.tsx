import { Component, onMount } from "solid-js";
import { Router, RouteSectionProps } from "@solidjs/router";
import AppRoutes from "@/routes";
import { initializeApp } from "./hooks/initializeApp";

const AppRoot: Component<RouteSectionProps> = (props) => {
    onMount(() => {
        initializeApp();
    });

    return (
        <div class="h-full flex flex-col justify-center bg-transparent">
            {props.children}
        </div>
    );
};

export default function App() {
    return (
        <Router root={AppRoot}>
            <AppRoutes />
        </Router>
    );
}

import { ParentComponent, onMount, Show } from "solid-js";
import { consoleEnabled, paramParse, parseUrlParams } from "@/utils/param-parse";

declare global {
    interface Window {
        ignoreCORS?: boolean;
    }
}

const CenterBox: ParentComponent = (props) => {
    onMount(() => {
        const params = parseUrlParams(window.location.href);
        paramParse(params);
    });

    return (
        <div class="flex items-center justify-center">
            {props.children}
            <Show when={consoleEnabled()}>
                <div class="bg-gray-800 text-white p-2 rounded text-sm">Debug console is enabled.</div>
            </Show>
        </div>
    );
};

export default CenterBox;

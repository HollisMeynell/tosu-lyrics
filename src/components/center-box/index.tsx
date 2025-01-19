import {children as createChildren, ParentComponent, onMount, Show } from "solid-js";
import {consoleShow, paramParse} from "../../common/param-parse.ts";
import styles from './styles.module.scss';

declare global {
    interface Window {
        ignoreCORS?: boolean;
    }
}

const CenterBox: ParentComponent = (props) => {
    const children = createChildren(() => props.children);

    onMount(() => {
        const url = new URL(window.location.href);
        const params = new URLSearchParams(url.search);
        paramParse(params)
    });

    return (
        <div class={styles.box}>
            {children()}
            <Show when={consoleShow()}>
                <div class={styles.console}>
                    Debug console is enabled.
                </div>
            </Show>
        </div>
    );
};

export default CenterBox;

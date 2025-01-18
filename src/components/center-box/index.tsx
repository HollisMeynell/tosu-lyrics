import { ParentComponent, onMount, createMemo, Show } from "solid-js";
import styles from './styles.module.scss';

declare global {
    interface Window {
        ignoreCORS?: boolean;
    }
}

const CenterBox: ParentComponent = (props) => {
    const consoleShow = createMemo(() => {
        const params = new URLSearchParams(window.location.search);
        return !!params.get('console');
    });

    onMount(() => {
        const params = new URLSearchParams(window.location.search);
        if (params.get('cors')) {
            window.ignoreCORS = true;
        }
    });

    return (
        <div class={styles.box}>
            {props.children}
            <Show when={consoleShow()}>
                <div class={styles.console}>
                    Debug console is enabled.
                </div>
            </Show>
        </div>
    );
};

export default CenterBox;

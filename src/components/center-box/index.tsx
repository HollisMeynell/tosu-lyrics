import styles from './styles.module.scss';
import {children as createChildren, JSX, Show} from "solid-js";

type CenterBoxProps = {
    children: JSX.Element
}

declare global {
    interface Window {
        ignoreCORS?: boolean
    }
}

export default function CenterBox(props: CenterBoxProps) :JSX.Element{
    const children = createChildren(() => props.children);

    const url = new URL(window.location.href);
    const params = new URLSearchParams(url.search);
    if (params.get('cors')) {
        window.ignoreCORS = true;
    }
    const consoleShow = !!params.get('console')

    return <div class={styles.box}>
        {children()}
        <Show when={consoleShow}>
            <div>...</div>
        </Show>
    </div>
}

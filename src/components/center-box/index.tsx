import styles from './styles.module.scss';
import {children as createChildren, JSX, Show} from "solid-js";
import {consoleShow, paramParse} from "../../common/param-parse.ts";

type CenterBoxProps = {
    children: JSX.Element
}

export default function CenterBox(props: CenterBoxProps) :JSX.Element{
    const children = createChildren(() => props.children);

    const url = new URL(window.location.href);
    const params = new URLSearchParams(url.search);
    paramParse(params)

    return <div class={styles.box}>
        {children()}
        <Show when={consoleShow()}>
            <div>...</div>
        </Show>
    </div>
}

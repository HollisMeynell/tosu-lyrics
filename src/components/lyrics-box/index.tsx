import styles from "./styles.module.scss";
import TosuAdapter, { LyricLine } from "../../common/tosu-adapter.ts";
import {
    createEffect,
    createSignal,
    Index,
    on,
    onCleanup,
    onMount,
    Show,
} from "solid-js";

export default function LyricsBox() {
    const [scroll, setScroll] = createSignal(false);
    const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
    const [cursor, setCursor] = createSignal(0);

    let lyricUL: HTMLUListElement | undefined;
    let tosu: TosuAdapter | undefined;

    onMount(() => {
        tosu = new TosuAdapter(setLyrics, setCursor);
    });

    onCleanup(() => {
        tosu?.stop();
    });

    // 更新滚动
    const updateScroll = (p: HTMLLIElement) => {
        let maxWidth: number;
        if (p.children.length === 2) {
            maxWidth =
                Math.max(p.children[0].scrollWidth, p.children[1].scrollWidth) * 1.2;
        } else {
            maxWidth = p.children[0].scrollWidth * 1.2;
        }

        if (maxWidth > 1200) {
            const offset = Math.round((maxWidth - 1200) / 2) + 10;
            p.style.setProperty("--offset", `${offset}px`);
            p.style.setProperty("--offset-f", `-${offset}px`);
            p.style.setProperty("--time", `${tosu?.getNextTime()}s`);
            setScroll(true);
        } else if (scroll()) {
            setScroll(false);
        }
    };

    createEffect(
        on(
            [lyrics, cursor],
            () => {
                if (!lyricUL) return;
                const p = lyricUL.children[cursor()] as HTMLLIElement;
                if (!p) return;
                updateScroll(p);
            },
            { defer: true }
        )
    );

    return (
        <div class={styles.box}>
            <ul
                ref={lyricUL}
                class={styles.lyricBox}
                style={{ transform: `translateY(${-(cursor() - 1) * 100}px)` }}
            >
                <Index each={lyrics()}>
                    {(lyric, index) => (
                        <li
                            classList={{
                                [styles.lyric]: true,
                                [styles.lyricNow]: cursor() === index,
                                [styles.lyricScrolling]: cursor() === index && scroll(),
                            }}
                        >
                            <p>{lyric().main}</p>
                            <Show when={lyric().origin}>
                                <p>{lyric().origin}</p>
                            </Show>
                        </li>
                    )}
                </Index>
            </ul>
        </div>
    );
}

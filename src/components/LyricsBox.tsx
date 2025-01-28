// 功能: 歌词展示组件

import TosuAdapter, { LyricLine } from "@/common/tosu-adapter.ts";
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
        <div class="w-full h-[300px] bg-transparent overflow-hidden">
            <ul
                ref={lyricUL}
                class="list-none transition-transform duration-300"
                style={{ transform: `translateY(${-(cursor() - 1) * 100}px)` }}
            >
                <Index each={lyrics()}>
                    {(lyric, index) => (
                        <li
                            classList={{
                                "h-[100px] mx-auto flex flex-col justify-center items-center scale-[0.6] transition-all duration-200":
                                    true,
                                "scale-[1.2]": cursor() === index,
                                "text-white": cursor() === index,
                                "animate-scroll": cursor() === index && scroll(),
                            }}
                        >
                            <p class="font-tLRC whitespace-nowrap text-4xl font-bold text-white drop-shadow-[5px_5px_3px_rgba(0,0,0,1)] shadow-[#fff]">
                                {lyric().main}
                            </p>
                            <Show when={lyric().origin}>
                                <p
                                    classList={{
                                        "font-oLRC whitespace-nowrap text-2xl font-bold text-[#a0a0a0] drop-shadow-[5px_5px_2.5px_rgba(0,0,0,1)] mt-4":
                                            true,
                                        block: cursor() === index,
                                        hidden: cursor() !== index,
                                    }}
                                >
                                    {lyric().origin}
                                </p>
                            </Show>
                        </li>
                    )}
                </Index>
            </ul>
        </div>
    );
}

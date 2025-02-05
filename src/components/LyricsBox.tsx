// 功能: 歌词展示组件

import TosuAdapter, { LyricLine } from "@/common/tosu-adapter.ts";
import lyricsStore from "@/stores/lyricsStore.ts";
import {
    Accessor,
    createEffect,
    createSignal,
    Index,
    on,
    onCleanup,
    onMount,
    Show,
} from "solid-js";

export default function LyricsBox({ debug }: { debug: boolean }) {
    const [scroll, setScroll] = createSignal(false);
    const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
    const [cursor, setCursor] = createSignal(0);

    let lyricUL: HTMLUListElement | undefined;
    let tosu: TosuAdapter | undefined;

    onMount(() => {
        if (debug) {
            console.log("Debug mode enabled");
            setLyrics([
                { main: "测试歌词1", origin: "Test lyrics 1" },
                { main: "测试歌词2", origin: "Test lyrics 2" },
                { main: "测试歌词3", origin: "Test lyrics 3" },
            ]);
            setCursor(1);
        } else {
            tosu = new TosuAdapter(setLyrics, setCursor);
        }
    });

    onCleanup(() => {
        tosu?.stop();
    });

    // 更新滚动
    const updateScroll = (p: HTMLLIElement) => {
        let maxWidth: number;
        if (p.children.length === 2) {
            maxWidth =
                Math.max(p.children[0].scrollWidth, p.children[1].scrollWidth) *
                1.2;
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

    // 正常模式下的
    if (!debug) {
        createEffect(
            on(
                [lyrics, cursor],
                () => {
                    if (!lyricUL) return;
                    const p = lyricUL.children[cursor()] as HTMLLIElement;
                    if (!p) return;
                    updateScroll(p);
                },
                { defer: true },
            ),
        );
    }

    const MainLyric = (
        { text }: { text: Accessor<string | undefined> },
    ) => <p
        class="font-tLRC whitespace-nowrap text-4xl font-bold drop-shadow-[5px_5px_3px_rgba(0,0,0,1)] shadow-[#fff]"
        style={{
            color: lyricsStore.getState.textColor.first,
        }}
    >
        {text()}
    </p>;

    const SecondLyric = (
        { block, text }: { block: Accessor<boolean>, text: Accessor<string | undefined> },
    ) => <p
        classList={{
            "font-oLRC whitespace-nowrap text-2xl font-bold text-[#a0a0a0] drop-shadow-[5px_5px_2.5px_rgba(0,0,0,1)] mt-4":
                true,
            block: block(),
            hidden: !block(),
        }}
        style={{
            color: lyricsStore.getState.textColor
                .second,
        }}
    >
        {text()}
    </p>;

    const lines = (lyric: Accessor<LyricLine>, index: number) => {
        console.log(lyric());
        const getMainLyric = () =>
            lyricsStore.getState.useTranslationAsMain ? lyric().main || lyric().origin : lyric().origin;

        const getSecondLyric = () => lyricsStore.getState.useTranslationAsMain
            ? lyric().origin
            : lyric().main;
        return (
            <li
                classList={{
                    "h-[100px] mx-auto flex flex-col justify-center items-center select-none scale-[0.6] transition-all duration-200":
                        true,
                    "scale-[1.2]": cursor() === index,
                    "text-white": cursor() === index,
                    "animate-scroll":
                        cursor() === index && scroll(),
                }}
            >
                <MainLyric
                    text={getMainLyric}
                />
                <Show
                    when={
                        lyric().origin &&
                        lyricsStore.getState.showSecond
                    }
                >
                    <SecondLyric
                        block={() => cursor() === index}
                        text={getSecondLyric} />
                </Show>
            </li>
        );
    };

    return (
        <div class="w-full h-[300px] overflow-hidden">
            <ul
                ref={lyricUL}
                class="list-none transition-transform duration-300"
                style={{ transform: `translateY(${-(cursor() - 1) * 100}px)` }}
            >
                <Index each={lyrics()}>
                    {lines}
                </Index>
            </ul>
        </div>
    );
}

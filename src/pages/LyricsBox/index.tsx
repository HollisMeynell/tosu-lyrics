import {
    alignment,
    font,
    fontSize,
    secondFont,
    shadow,
    textColor,
    useTranslationAsMain,
    showSecond,
} from "@/stores/settingsStore";
import {
    cursor,
    lyrics,
    nextTime,
    setCursor,
    setLyrics,
} from "@/stores/lyricStore";
import {
    Component,
    Accessor,
    createEffect,
    createSignal,
    Index,
    on,
    onMount,
    Show,
} from "solid-js";
import { LyricLine } from "@/types/lyricTypes.ts";
import { loadFont } from "@/utils/fonts.ts";
import { measureLineWidth } from "@/utils/lyricScroll.ts";

let blink = () => void 0;

export const lyricBlink = () => {
    blink();
};

interface LyricsBoxProps {
    debug: boolean;
}

interface MainLyricProps {
    text: string | undefined;
    align?: "left" | "center" | "right";
    active: boolean;
}

interface SecondLyricProps {
    block: boolean;
    text: string | undefined;
    align?: "left" | "center" | "right";
    active: boolean;
}

const LyricsBox: Component<LyricsBoxProps> = (props) => {
    const isDebug = props.debug && !(import.meta.env.MODE === "development");
    const [scroll, setScroll] = createSignal(false);
    const [lyricLIRef, setLyricLIRef] = createSignal<HTMLLIElement | undefined>(
        undefined
    );
    let lyricUL: HTMLUListElement | undefined;

    const lyricShow = (show?: boolean) => {
        if (show == undefined) show = true;
        let lyricLI = lyricLIRef();
        if (lyricLI) {
            lyricLI.style.visibility = show ? "visible" : "hidden";
        }
    };

    let blinkKey = 0;
    blink = () => {
        if (blinkKey > 0) {
            blinkKey += 6;
            return;
        } else {
            blinkKey = 10;
        }
        let needBack: boolean;
        const lyricsBack = lyrics();
        const cursorBack = cursor();

        if (lyricsBack.length < 3) {
            needBack = true;
            setLyrics([
                { main: "调试中", origin: "Testing" },
                { main: "正在调试中", origin: "Testing..." },
                { main: "调试中", origin: "Testing" },
            ]);
            setCursor(1);
        } else {
            needBack = false;
        }

        let isVisible = true;

        const interval = setInterval(() => {
            if (blinkKey <= 0) {
                clearInterval(interval);
                lyricShow(true);
                if (needBack) {
                    setLyrics(lyricsBack);
                    setCursor(cursorBack);
                }
                return;
            }
            isVisible = !isVisible;
            lyricShow(isVisible);
            blinkKey--;
        }, 250);
    };

    const updateScroll = (p: HTMLLIElement) => {
        setLyricLIRef(p);
        // 按目标(active)字号补偿后再比较, 详见 utils/lyricScroll.ts。
        // 目标字号必须与下面 MainLyric / SecondLyric 实际用的 font-size 一致。
        const sizes = fontSize();
        const maxWidth = measureLineWidth(p, sizes.first, sizes.second);

        if (!lyricUL) return;

        const clientWidth = lyricUL.clientWidth;

        const alignmentStyle = p.style.alignItems || "center";

        if (maxWidth > clientWidth) {
            const ulPadding = 40;
            if (alignmentStyle === "flex-start") {
                const offset =
                    Math.round(maxWidth - clientWidth) + 2 * ulPadding;
                p.style.setProperty("--offset", `${0}px`);
                p.style.setProperty("--offset-f", `-${offset}px`);
            } else if (alignmentStyle === "center") {
                const offset =
                    Math.round((maxWidth - clientWidth) / 2) + ulPadding;
                p.style.setProperty("--offset", `${offset}px`);
                p.style.setProperty("--offset-f", `-${offset}px`);
            } else if (alignmentStyle === "flex-end") {
                const offset =
                    Math.round(maxWidth - clientWidth) + 2 * ulPadding;
                p.style.setProperty("--offset", `${offset}px`);
                p.style.setProperty("--offset-f", `${0}px`);
            }
            p.style.setProperty(
                "--time",
                `${nextTime() > 0 ? nextTime() / 1000 : 0}s`
            );
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
                const currentLine = lyricUL.children[cursor()] as HTMLLIElement;
                if (currentLine) {
                    updateScroll(currentLine);
                }
            },
            { defer: true }
        )
    );

    createEffect(
        on([font, secondFont], () => {
            if (!lyricUL) return;
            if (font().length === 0 || secondFont().length === 0) {
                void loadFont();
            }
        })
    );

    onMount(() => {
        if (isDebug) {
            setLyrics([
                { main: "测试歌词1", origin: "Test Lyrics 1" },
                { main: "测试歌词2", origin: "Test Lyrics 2" },
                { main: "测试歌词3", origin: "Test Lyrics 3" },
            ]);
            setCursor(1);
        }
    });


    // 未启用阴影时返回 undefined，不写空 filter
    const shadowFilter = (which: "first" | "second") => {
        const s = which === "first" ? shadow().first : shadow().second;
        if (!s.enable) return undefined;
        // CSS drop-shadow: <x> <y> <blur> <color>
        return `drop-shadow(${s.offsetX}px ${s.offsetY}px ${s.blur}px ${s.color})`;
    };

    const MainLyric: Component<MainLyricProps> = (props) => (
        <p
            class="font-tLRC whitespace-nowrap text-4xl font-bold transition-[font-size] duration-300"
            style={{
                filter: shadowFilter("first"),
                color: textColor().first,
                "font-family": font() || undefined,
                "text-align": props.align || "center",
                "font-size": `${props.active ? fontSize().first : fontSize().first / 2}em`,
            }}
        >
            {props.text}
        </p>
    );

    const SecondLyric: Component<SecondLyricProps> = (props) => (
        <p
            classList={{
                "font-oLRC whitespace-nowrap text-2xl font-bold mt-4 transition-[font-size] duration-300":
                    true,
                block: props.block,
                hidden: !props.block,
            }}
            style={{
                filter: shadowFilter("second"),
                color: textColor().second,
                "font-family": secondFont() || font() || undefined,
                "text-align": props.align || "center",
                "font-size": `${props.active ? fontSize().second : fontSize().second / 2}em`,
            }}
        >
            {props.text}
        </p>
    );

    const lyricAlignmentStyle = () => {
        let alignmentStyle = "";
        let transformOrigin = "";
        switch (alignment()) {
            case "left":
                alignmentStyle = "flex-start";
                transformOrigin = "left center";
                break;
            case "right":
                alignmentStyle = "flex-end";
                transformOrigin = "right center";
                break;
            default:
            case "center":
                alignmentStyle = "center";
                transformOrigin = "center";
                break;
        }

        return {
            "align-items": alignmentStyle,
            "transform-origin": transformOrigin,
        };
    };

    const lines = (lyric: Accessor<LyricLine>, index: number) => {
        const getMainLyric = () =>
            useTranslationAsMain()
                ? lyric().main
                    ? lyric().main
                    : lyric().origin
                : lyric().origin
                  ? lyric().origin
                  : lyric().main;

        const getSecondLyric = () =>
            useTranslationAsMain()
                ? lyric().origin
                : lyric().main;

        return (
            <li
                classList={{
                    "w-fit h-[100px] flex flex-col justify-center items-center select-none":
                        true,
                    "animate-scroll": cursor() === index && scroll(),
                }}
                style={lyricAlignmentStyle()}
            >
                <MainLyric text={getMainLyric()} active={cursor() === index} />
                <Show
                    when={lyric().origin && showSecond()}
                >
                    <SecondLyric
                        block={cursor() === index}
                        text={getSecondLyric()}
                        active={cursor() === index}
                    />
                </Show>
            </li>
        );
    };

    return (
        <div class="w-full h-[300px] overflow-hidden">
            <ul
                ref={lyricUL}
                class="w-full px-10 flex flex-col list-none transition-transform duration-300"
                style={{
                    transform: `translateY(${-(cursor() - 1) * 100}px)`,
                    ...lyricAlignmentStyle(),
                }}
            >
                <Index each={lyrics()}>{lines}</Index>
            </ul>
        </div>
    );
};

export default LyricsBox;

import TosuManager from "@/services/managers/tosuManager";
import store from "@/stores/indexStore";
import { alignment, font } from "@/stores/settingsStore";
import {
    Component,
    Accessor,
    createEffect,
    createSignal,
    Index,
    on,
    onCleanup,
    onMount,
    Show,
} from "solid-js";
import { LyricLine } from "@/types/lyricTypes.ts";
import { loadFont } from "@/utils/fonts.ts";

let blink = () => void 0;
let tosu: TosuManager | undefined;

// 触发歌词闪烁三次
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
    const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
    const [cursor, setCursor] = createSignal(0);
    const [lyricLIRef, setLyricLIRef] = createSignal<HTMLLIElement | undefined>(
        undefined
    );
    let lyricUL: HTMLUListElement | undefined;

    const linkTosu = () => {
        if (!isDebug) tosu = new TosuManager(setLyrics, setCursor);
    };

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
        tosu?.pause();
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
                tosu?.continue();
                return;
            }
            isVisible = !isVisible;
            lyricShow(isVisible);
            blinkKey--;
        }, 250);
    };

    onCleanup(() => {
        tosu?.stop();
    });

    // 更新滚动
    const updateScroll = (p: HTMLLIElement) => {
        setLyricLIRef(p);
        const getMaxWidth = () => {
            if (p.children.length === 2) {
                return Math.max(
                    p.children[0].scrollWidth * 2,
                    p.children[1].scrollWidth
                );
            }
            return p.children[0].scrollWidth;
        };

        const maxWidth = getMaxWidth();

        if (!lyricUL) return;

        const clientWidth = lyricUL.clientWidth;

        const alignmentStyle = p.style.alignItems || "center";

        console.log(`${maxWidth} || ${clientWidth}`);
        if (maxWidth > clientWidth) {
            const ulPadding = 40;
            // 计算偏移量(非常精细)

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
            p.style.setProperty("--time", `${tosu?.getNextTime()}s`);
            setScroll(true);
        } else if (scroll()) {
            setScroll(false);
        }
    };

    // 更新歌词滚动
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

    // 更新字体
    createEffect(
        on([font], () => {
            if (!lyricUL) return;
            const fontName = font();
            if (fontName.length === 0) {
                loadFont().then((font) => {
                    lyricUL.style.fontStyle = font;
                });
            }
            lyricUL.style.fontFamily = fontName;
        })
    );

    // 初始化歌词
    onMount(() => {
        if (isDebug) {
            tosu?.stop();
            setLyrics([
                { main: "测试歌词1", origin: "Test Lyrics 1" },
                { main: "测试歌词2", origin: "Test Lyrics 2" },
                { main: "测试歌词3", origin: "Test Lyrics 3" },
            ]);
            setCursor(1);
        } else {
            linkTosu();
        }
    });

    // 子组件
    const MainLyric: Component<MainLyricProps> = (props) => (
        <p
            class="font-tLRC whitespace-nowrap text-4xl font-bold drop-shadow-[5px_5px_3px_rgba(0,0,0,1)] shadow-[#fff] transition-[font-size] duration-300"
            style={{
                color: store.getState.settings.textColor.first,
                "text-align": props.align || "center",
                "font-size": props.active ? "3em" : "1.5em",
            }}
        >
            {props.text}
        </p>
    );

    const SecondLyric: Component<SecondLyricProps> = (props) => (
        <p
            classList={{
                "font-oLRC whitespace-nowrap text-2xl font-bold text-[#a0a0a0] drop-shadow-[5px_5px_2.5px_rgba(0,0,0,1)] mt-4 transition-[font-size] duration-300":
                    true,
                block: props.block,
                hidden: !props.block,
            }}
            style={{
                color: store.getState.settings.textColor.second,
                "text-align": props.align || "center",
                "font-size": props.active ? "2em" : "1em",
            }}
        >
            {props.text}
        </p>
    );

    // 歌词对齐样式
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

    // 渲染歌词行
    const lines = (lyric: Accessor<LyricLine>, index: number) => {
        const getMainLyric = () =>
            store.getState.settings.useTranslationAsMain
                ? lyric().main
                    ? lyric().main
                    : lyric().origin
                : lyric().origin
                  ? lyric().origin
                  : lyric().main;

        const getSecondLyric = () =>
            store.getState.settings.useTranslationAsMain
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
                    when={lyric().origin && store.getState.settings.showSecond}
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

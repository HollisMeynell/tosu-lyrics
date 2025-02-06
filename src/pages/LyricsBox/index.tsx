
import TosuAdapter, { LyricLine } from "@/common/tosu-adapter.ts";
import lyricsStore from "@/stores/lyricsStore.ts";
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
import { wsService } from "@/services/WebSocketService.ts";

interface LyricsBoxProps {
    debug: boolean;
}

const LyricsBox: Component<LyricsBoxProps> = (props) => {
    const isDebug = props.debug && !(import.meta.env.MODE === "development");
    const [scroll, setScroll] = createSignal(false);
    const [lyrics, setLyrics] = createSignal<LyricLine[]>([]);
    const [cursor, setCursor] = createSignal(0);

    let lyricUL: HTMLUListElement | undefined;
    let tosu: TosuAdapter | undefined;

    onMount(() => {
        // 注册歌词闪烁处理器
        wsService.registerHandler("blink-lyric", lyricBlink);
    });

    onCleanup(() => {
        tosu?.stop();
    });

    // 触发歌词闪烁三次
    const lyricBlink = () => {
        if (!lyricUL) return;

        let count = 0;
        let isVisible = true;
        const interval = setInterval(() => {
            if (count >= 10) {
                clearInterval(interval);
                return;
            }
            isVisible = !isVisible;
            lyricUL.style.visibility = isVisible ? "visible" : "hidden";
            count++;
        }, 500);
    };

    // 更新滚动
    const updateScroll = (p: HTMLLIElement) => {
        const getMaxWidth = () => {
            if (p.children.length === 2) {
                return (
                    Math.max(
                        p.children[0].scrollWidth,
                        p.children[1].scrollWidth
                    ) * 1.2
                );
            }
            return p.children[0].scrollWidth * 1.2;
        };

        const maxWidth = getMaxWidth();

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
                if (!lyricUL || isDebug) return;
                const currentLine = lyricUL.children[cursor()] as HTMLLIElement;
                if (currentLine) updateScroll(currentLine);
            },
            { defer: true }
        )
    );

    if (isDebug) {
        tosu?.stop();
        setLyrics([
            { main: "测试歌词1", origin: "Test Lyrics 1" },
            { main: "测试歌词2", origin: "Test Lyrics 2" },
            { main: "测试歌词3", origin: "Test Lyrics 3" },
        ]);
        setCursor(1);
    } else {
        tosu = new TosuAdapter(setLyrics, setCursor);
    }

    // 子组件
    const MainLyric: Component<{ text: string | undefined }> = (props) => (
        <p
            class="font-tLRC whitespace-nowrap text-4xl font-bold drop-shadow-[5px_5px_3px_rgba(0,0,0,1)] shadow-[#fff]"
            style={{
                color: lyricsStore.getState.textColor.first,
            }}
        >
            {props.text}
        </p>
    );

    const SecondLyric: Component<{
        block: boolean;
        text: string | undefined;
    }> = (props) => (
        <p
            classList={{
                "font-oLRC whitespace-nowrap text-2xl font-bold text-[#a0a0a0] drop-shadow-[5px_5px_2.5px_rgba(0,0,0,1)] mt-4":
                    true,
                block: props.block,
                hidden: !props.block,
            }}
            style={{
                color: lyricsStore.getState.textColor.second,
            }}
        >
            {props.text}
        </p>
    );

    // 渲染歌词行
    const lines = (lyric: Accessor<LyricLine>, index: number) => {
        const getMainLyric = () =>
            lyricsStore.getState.useTranslationAsMain
                ? lyric().main || lyric().origin
                : lyric().origin;

        const getSecondLyric = () =>
            lyricsStore.getState.useTranslationAsMain
                ? lyric().origin
                : lyric().main;

        return (
            <li
                classList={{
                    "h-[100px] mx-auto flex flex-col justify-center items-center select-none scale-[0.6] transition-all duration-200":
                        true,
                    "scale-[1.2]": cursor() === index,
                    "text-white": cursor() === index,
                    "animate-scroll": cursor() === index && scroll(),
                }}
            >
                <MainLyric text={getMainLyric()} />
                <Show when={lyric().origin && lyricsStore.getState.showSecond}>
                    <SecondLyric
                        block={cursor() === index}
                        text={getSecondLyric()}
                    />
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
                <Index each={lyrics()}>{lines}</Index>
            </ul>
        </div>
    );
};

export default LyricsBox;

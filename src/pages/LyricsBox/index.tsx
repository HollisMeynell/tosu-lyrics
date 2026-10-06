import {
    alignment,
    font,
    fontSize,
    loadedSubFamily,
    lyricLines,
    secondFont,
    shadow,
    textColor,
    useTranslationAsMain,
    showSecond,
} from "@/stores/settingsStore";
import {
    cursor,
    lyricLoading,
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
    onCleanup,
    Show,
} from "solid-js";
import { LyricLine } from "@/types/lyricTypes.ts";
import { loadFont, resolveFamily } from "@/utils/fonts.ts";
import { measureLineWidth } from "@/utils/lyricScroll.ts";
import {
    LYRIC_LINE_HEIGHT,
    LyricLineLevel,
    boxLineCount,
    lineDistance,
    lineLevel,
    lineLevelScale,
    lyricBoxHeight,
    visibleSideCount,
    windowLineCount,
    windowTranslateY,
} from "@/utils/lyricLines.ts";

/** 加载提示最多显示的点数; 到顶后回到 1 个点继续循环 */
const LOADING_DOT_MAX = 5;

let blink = () => void 0;

export const lyricBlink = () => {
    blink();
};

interface MainLyricProps {
    text: string | undefined;
    align?: "left" | "center" | "right";
    /** 样式层级：0 当前歌词 / 1 相邻 / ≥2 最外层（见 utils/lyricLines.ts） */
    level: LyricLineLevel;
}

interface SecondLyricProps {
    block: boolean;
    text: string | undefined;
    align?: "left" | "center" | "right";
    level: LyricLineLevel;
}

export interface LyricsBoxProps {
    /**
     * 固定 viewport 行数（只有 Controller 顶部预览传 3）。
     *
     * 传了之后窗口高度恒定 = viewportLines 行，不再跟随 lyricLines：
     * 下面的控制面板位置始终保持与 3 行时一致，多出来的歌词只在固定 viewport
     * 内显示、其余整行隐藏。
     *
     * `/lyrics` 不传，行为与之前完全一致（窗口高度 = lyricLines，按设置完整显示）。
     */
    viewportLines?: number;
}

const LyricsBox: Component<LyricsBoxProps> = (props) => {
    const [scroll, setScroll] = createSignal(false);
    const [lyricLIRef, setLyricLIRef] = createSignal<HTMLLIElement | undefined>(
        undefined
    );
    let lyricUL: HTMLUListElement | undefined;

    // 加载提示: 从 1 个点开始, 每 0.5s 增加一个, 到上限后回到 1 个点循环。
    const [loadingDots, setLoadingDots] = createSignal(1);
    // 只在"正在加载且还没有歌词可显示"时出现, 不覆盖正常歌词
    const showLoading = () => lyricLoading() && lyrics().length === 0;

    /**
     * 窗口高度 / 居中位移用的行数。
     *
     * Controller（传了 viewportLines）固定 3 行高度，控制面板位置永不改变；
     * /lyrics 没传，跟随 lyricLines（1/3/…/15）—— 展示端行为不变。
     */
    const boxLines = () => boxLineCount(lyricLines(), props.viewportLines);
    /** 允许参与显示的最大行数：Controller 不会超过固定 viewport */
    const visibleLines = () => windowLineCount(lyricLines(), props.viewportLines);
    /** 当前歌词上下各允许显示多少行（Controller 里 lyricLines=1 时为 0） */
    const visibleSide = () => visibleSideCount(visibleLines());

    createEffect(() => {
        if (!lyricLoading()) {
            setLoadingDots(1);
            return;
        }
        const timer = setInterval(() => {
            setLoadingDots((n) => (n >= LOADING_DOT_MAX ? 1 : n + 1));
        }, 500);
        onCleanup(() => clearInterval(timer));
    });

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

    // 字体选择变化时重新解析并注册所需字体。不再要求"选择为空"才加载 ——
    // 现在选择码可能指向"内置"或"上传"两套 family，且内置字体是按需注册的。
    // registered 的版本检查保证同一字体不会重复下载。
    createEffect(
        on([font, secondFont], () => {
            void loadFont();
        })
    );

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
                "font-family": resolveFamily(font()),
                "text-align": props.align || "center",
                // 层级 0 为 1 倍、层级 1 与 2 为 1/2 倍 —— 与改动前的 active/inactive 完全一致
                "font-size": `${fontSize().first * lineLevelScale(props.level)}em`,
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
                // 副歌词：有独立选择就按副字体解析；否则沿用原有语义
                // （先看已注册的副字体，再回落到主字体的选择）
                "font-family": secondFont()
                    ? resolveFamily(secondFont())
                    : loadedSubFamily() || resolveFamily(font()),
                "text-align": props.align || "center",
                "font-size": `${fontSize().second * lineLevelScale(props.level)}em`,
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

        // 行距离 → 样式层级：距离 0 当前行 / 1 相邻行 / ≥2 最外层行，
        // 距离再大也不会继续缩小字号（15 行时距离 2…7 都是最外层样式）
        const distance = () => lineDistance(index, cursor());
        const level = () => lineLevel(distance());
        /**
         * Controller 固定 viewport 下，超出「允许显示行数」的行整行隐藏。
         *
         * 用 visibility 而不是 display：每行仍占满 100px，居中位移与相邻行位置
         * 完全不变（15 行时 Controller 显示的仍是与 3 行时相同的三行）。
         * /lyrics 不加这个限制 —— 它的窗口高度本身就会裁掉多余行。
         */
        const clipped = () =>
            props.viewportLines !== undefined && distance() > visibleSide();

        return (
            <li
                classList={{
                    "w-fit flex flex-col justify-center items-center select-none":
                        true,
                    "animate-scroll": cursor() === index && scroll(),
                    invisible: clipped(),
                }}
                style={{
                    ...lyricAlignmentStyle(),
                    height: `${LYRIC_LINE_HEIGHT}px`,
                }}
            >
                <MainLyric text={getMainLyric()} level={level()} />
                <Show
                    when={lyric().origin && showSecond()}
                >
                    <SecondLyric
                        block={level() === 0}
                        text={getSecondLyric()}
                        level={level()}
                    />
                </Show>
            </li>
        );
    };

    return (
        // 窗口高度 = boxLines × 单行高度：/lyrics 为 lyricLines（3 行即原来的 300px），
        // Controller 恒为固定 viewport 的 3 行高度（300px），所以面板位置不会被挤动。
        // 歌词列表仍是整份渲染，由 overflow-hidden 裁出可见窗口，不会生成 undefined / 重复歌词
        <div
            class="w-full overflow-hidden relative"
            style={{ height: `${lyricBoxHeight(boxLines())}px` }}
        >
            <Show when={showLoading()}>
                <div
                    class="absolute inset-0 flex flex-col justify-center pointer-events-none px-10"
                    style={lyricAlignmentStyle()}
                >
                    <p
                        class="font-tLRC whitespace-nowrap font-bold"
                        style={{
                            filter: shadowFilter("first"),
                            color: textColor().first,
                            "font-family": resolveFamily(font()),
                            // 复用主歌词字号体系(用户调整主字号时提示同步变化),
                            // 取 1.15 倍: 比正常歌词再大一点, 让加载状态足够醒目
                            "font-size": `${fontSize().first * 1.15}em`,
                            // 点与点之间留出明显间隔, 让每个点独立可辨;
                            // 负 margin 抵消最后一个点之后的空白,
                            // 保证居中 / 左右对齐时整串不会偏移
                            "letter-spacing": "0.4em",
                            "margin-right": "-0.4em",
                        }}
                    >
                        {".".repeat(loadingDots())}
                    </p>
                </div>
            </Show>
            <ul
                ref={lyricUL}
                class="w-full px-10 flex flex-col list-none transition-transform duration-300"
                style={{
                    // 当前歌词始终落在窗口正中间（3 行时即原来的 -(cursor - 1) * 100；
                    // Controller 恒按 3 行窗口居中，/lyrics 按设置的 windowLineCount 居中）
                    transform: `translateY(${windowTranslateY(
                        cursor(),
                        boxLines()
                    )}px)`,
                    ...lyricAlignmentStyle(),
                }}
            >
                <Index each={lyrics()}>{lines}</Index>
            </ul>
        </div>
    );
};

export default LyricsBox;

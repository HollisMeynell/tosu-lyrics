import { batch, createSignal } from "solid-js";
import { AlignType, Pair, SettingsDto } from "@/types/globalTypes";
import { alignmentOptions, Shadow } from "@/types/globalTypes";
import {
    DEFAULT_LYRIC_LINES,
    normalizeLyricLines,
} from "@/utils/lyricLines";

export const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};

export const DEFAULT_FONT_SIZE: Pair<number> = { first: 3, second: 2 };

/**
 * 阴影默认值：主 / 副都**开启**、纯黑，偏移与模糊都是 3px。
 *
 * 「恢复默认样式」直接用这份常量，后端 `ShadowSettings::default()` 也是同一组值 ——
 * 两处必须一致，否则新装与"恢复默认"会给出不同的结果。
 * 注意：已经保存过的用户自定义阴影不受影响（它们存在设置里，不会回落到默认值）。
 */
export const DEFAULT_SHADOW: Shadow = {
    enable: true,
    color: "#000000",
    blur: 3,
    offsetX: 3,
    offsetY: 3,
};

export const [font, setFont] = createSignal("");
/** 副歌词字体（后端 font.second；为空时回落到主字体） */
export const [secondFont, setSecondFont] = createSignal("");
/**
 * 已注册到 `document.fonts` 的**上传字体 family**（由 `loadFont()` 填写）。
 *
 * 上面的 `font` / `secondFont` 是**设置里填的字体名**，默认为空串；上传的字体是
 * 由 `FontFace` 以固定 family（`LRC` / `LRC-Sub`）注册的，跟字体文件内部的名称
 * （例如 Unifont-JP）无关 —— CSS 必须用 `FontFace` 的那个 family 才能命中。
 * 设置名为空时回落到这里；否则 `font-family` 为空，浏览器会静默使用默认字体，
 * 表现就是"上传成功、FontFace 也 loaded，但歌词字形没变"。
 */
export const [loadedMainFamily, setLoadedMainFamily] = createSignal("");
export const [loadedSubFamily, setLoadedSubFamily] = createSignal("");
export const [fontSize, setFontSize] =
    createSignal<Pair<number>>(DEFAULT_FONT_SIZE);
export const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
export const [shadow, setShadow] = createSignal<Pair<Shadow>>({
    first: DEFAULT_SHADOW,
    second: DEFAULT_SHADOW,
});
export const [useTranslationAsMain, setUseTranslationAsMain] =
    createSignal(true);
export const [showSecond, setShowSecond] = createSignal(true);
/**
 * 歌词行数（可见窗口）：渲染层的唯一取值来源。
 *
 * 写入这里的每个路径（HTTP 快照 / WS 广播）都已经过 `normalizeLyricLines`，
 * 所以渲染层只会拿到 1 / 3 / 5 / … / 15。
 */
export const [lyricLines, setLyricLines] = createSignal<number>(
    DEFAULT_LYRIC_LINES
);
export const [alignment, setAlignment] = createSignal<AlignType>(
    alignmentOptions[1].value
);

export const [darkMode, setDarkMode] = createSignal(
    localStorage.getItem("darkMode") === "true"
);

export const applySettings = (settings: SettingsDto) => {
    batch(() => {
        setTextColor({
            first: settings.textColor.first,
            second: settings.textColor.second,
        });
        setFontSize({
            first: settings.fontSize.first,
            second: settings.fontSize.second,
        });
        setFont(settings.font.first);
        setSecondFont(settings.font.second);
        setAlignment(settings.alignment);
        setUseTranslationAsMain(settings.translationMain);
        setShowSecond(settings.secondShow);
        // 缺失 / 越界 / 偶数都在这里收敛成合法奇数，渲染层不再自己判断
        setLyricLines(normalizeLyricLines(settings.lyricLines));
        setShadow({
            first: settings.shadow?.first ?? DEFAULT_SHADOW,
            second: settings.shadow?.second ?? DEFAULT_SHADOW,
        });
    });
};

export const initializeDarkMode = () => {
    applyDarkMode(darkMode());
};

export const toggleDarkMode = () => {
    const next = !darkMode();
    setDarkMode(next);
    localStorage.setItem("darkMode", String(next));
    applyDarkMode(next);
};

const applyDarkMode = (enabled: boolean) => {
    document.documentElement.classList.toggle("dark", enabled);
};

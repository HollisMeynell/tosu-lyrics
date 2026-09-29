import { batch, createSignal } from "solid-js";
import { AlignType, Pair, SettingsDto } from "@/types/globalTypes";
import { alignmentOptions, Shadow } from "@/types/globalTypes";

export const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};

export const DEFAULT_FONT_SIZE: Pair<number> = { first: 3, second: 2 };

/**
 * 阴影默认值：主 / 副都**开启**、纯黑。
 *
 * 「恢复默认样式」直接用这份常量，后端 `ShadowSettings::default()` 也是同一组值 ——
 * 两处必须一致，否则新装与"恢复默认"会给出不同的结果。
 */
export const DEFAULT_SHADOW: Shadow = {
    enable: true,
    color: "#000000",
    blur: 3,
    offsetX: 2,
    offsetY: 2,
};

export const [font, setFont] = createSignal("");
/** 副歌词字体（后端 font.second；为空时回落到主字体） */
export const [secondFont, setSecondFont] = createSignal("");
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

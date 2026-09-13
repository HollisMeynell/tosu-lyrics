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

// 同步信息
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

// 非同步信息
export const [darkMode, setDarkMode] = createSignal(
    localStorage.getItem("darkMode") === "true"
);

/**
 * 用**服务端返回的完整设置**覆盖本地展示状态。
 *
 * 无论是 `GET/PATCH /api/settings` 的响应，还是 WS 下发的完整快照，
 * 都通过这里写入，避免出现"本地乐观值"和"服务端真实值"两套状态。
 */
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

/** 把持久化的夜间模式应用到 <html> class（两端初始化时调用） */
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

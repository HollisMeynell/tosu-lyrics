import { createSignal } from "solid-js";
import { Shadow } from "@/types/globalTypes";
import { AlignType, alignmentOptions } from "@/types/globalTypes";

export const DEFAULT_TEXT_COLOR = {
    first: "#ffffff",
    second: "#e0e0e0",
};

const DEFAULT_SHADOW: Shadow = {
    enable: false,
    inset: false,
    color: "#000000",
    type: undefined,
};

// 同步信息
export const [font, setFont] = createSignal("");
export const [textColor, setTextColor] = createSignal(DEFAULT_TEXT_COLOR);
export const [shadow, setShadow] = createSignal<Shadow>(DEFAULT_SHADOW);
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

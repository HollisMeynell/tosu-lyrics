export type AlignType = "left" | "center" | "right";

/** 主 / 副一对值，与后端 `Pair<T>` 对应 */
export type Pair<T> = { first: T; second: T };

/**
 * 后端 `/api/settings` 的完整设置（B-03）。
 * 字段名与后端 `LyricSettings` 的 camelCase 序列化一一对应。
 */
export type SettingsDto = {
    /** 主 / 副歌词颜色，`#rrggbb` */
    textColor: Pair<string>;
    /** 主 / 副歌词字号（em，active 时） */
    fontSize: Pair<number>;
    /** 主 / 副歌词字体名，空串表示默认字体 */
    font: Pair<string>;
    alignment: AlignType;
    /** 是否以翻译为主 */
    translationMain: boolean;
    /** 是否显示副歌词 */
    secondShow: boolean;
    /**
     * 歌词行数（可见窗口）：只允许 1 / 3 / 5 / 7 / 9 / 11 / 13 / 15，
     * 默认 3，当前歌词始终位于窗口中心。
     */
    lyricLines: number;
    /** 主 / 副歌词阴影 */
    shadow: Pair<Shadow>;
};

/** 需要更新的设置字段（PATCH 用，未出现的字段保持服务端原值） */
export type SettingsPatch = Partial<SettingsDto>;

/** 后端统一错误结构 */
export type ApiErrorBody = {
    error: { code: string; message: string };
};

export type AlignOptions = { key: string; value: AlignType };

export const alignmentOptions: AlignOptions[] = [
    { key: "左对齐", value: "left" },
    { key: "居中对齐", value: "center" },
    { key: "右对齐", value: "right" },
];

export type TextColorValue = { first: string; second: string };

export type Shadow = {
    /** 是否启用阴影（UI 上是"关闭阴影"的反面） */
    enable: boolean;
    color: string;
    /** 模糊半径（px） */
    blur: number;
    /** X 偏移（px） */
    offsetX: number;
    /** Y 偏移（px） */
    offsetY: number;
};

export type Settings = {
    font: string;
    textColor: TextColorValue;
    shadow: Shadow;
    useTranslationAsMain: boolean;
    showSecond: boolean;
    alignment: AlignType;
};

// 定义黑名单项的类型
export type BlacklistItem = {
    id: string;
    name: string;
    reason?: string;
    timestamp: number;
};

export type Config = {
    settings: Settings;
    titleBlackList?: BlacklistItem[];
};

export type FontData = {
    postscriptName: string;
    fullName: string;
    family: string;
    style: string;
};

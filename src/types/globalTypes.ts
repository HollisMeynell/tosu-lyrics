export type AlignType = "left" | "center" | "right";

export type AlignOptions = { key: string; value: AlignType };

export const alignmentOptions: AlignOptions[] = [
    { key: "左对齐", value: "left" },
    { key: "居中对齐", value: "center" },
    { key: "右对齐", value: "right" },
];

export type TextColorValue = { first: string; second: string };

export type Shadow = {
    enable: boolean;
    inset: boolean;
    color: string;
    type: string | undefined; // e.g. "1px 1px": x 轴偏移 1px y 轴偏移 1px
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

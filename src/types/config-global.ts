export type LyricLine = {
    time: number;
    first: string;
    second?: string;
};

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
    textColor: TextColorValue;
    shadow: Shadow;
    useTranslationAsMain: boolean;
    showSecond: boolean;
    currentLyrics: LyricLine[] | undefined;
    alignment: AlignType;
    // 歌词黑名单
    blackListBid?: Set<number>;
    // 标题黑名单
    blackListTitle?: Set<string>;
};

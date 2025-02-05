export type LyricLine = {
    time: number;
    first: string;
    second?: string;
};

export type AlignType = "left" | "center" | "right";

export type TextColorValue = { first: string; second: string };

export type Shadow = {
    enable: boolean;
    inset: boolean;
    color: string;
    // 类似 "1px 1px" 就是 x 轴偏移 1px，y 轴偏移 1px
    type: string | undefined;
}

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

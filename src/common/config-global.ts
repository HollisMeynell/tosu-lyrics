import { LyricLine } from "@/common/music-api.ts";

export type AlignType = "left" | "center" | "right";

export type TextColorValue = { first: string; second: string };

export type Settings = {
    textColor: TextColorValue;
    useTranslationAsMain: boolean;
    showSecond: boolean;
    currentLyrics: LyricLine[] | undefined;
    alignment: AlignType;
    // 歌词黑名单
    blackListBid: Set<number>,
    // 标题黑名单
    blackListTitle: Set<string>,
}

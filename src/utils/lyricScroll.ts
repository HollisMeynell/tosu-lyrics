/**
 * 歌词行跑马灯宽度测量。
 *
 * 主 / 副歌词在 active 与 inactive 之间用 CSS transition 过渡
 * (主歌词 1.5em <-> 3em, 副歌词 1em <-> 2em)。光标刚切到某一行时,
 * 元素的 computed font-size 还停在过渡起点, 此时 `scrollWidth` 量到的是
 * **过渡起点**的尺寸, 而不是过渡结束后的最终尺寸。
 *
 * 这里按「目标字号 / 当前实际字号」做比例补偿, 使
 * 「歌词行创建时就已是 active」与「该行由 inactive 过渡到 active」
 * 两条路径得到同一个宽度。
 * 若只对前者套用固定的 x2, 会把已经是最终尺寸的宽度再放大一倍,
 * 导致宽度并不超出的行也误触发跑马灯。
 */

/** 主歌词 active 时的默认字号(em), 实际值由设置的 fontSize.first 传入 */
export const DEFAULT_ACTIVE_MAIN_EM = 3;
/** 副歌词 active 时的默认字号(em), 实际值由设置的 fontSize.second 传入 */
export const DEFAULT_ACTIVE_SECOND_EM = 2;

/** 元素按「目标 em」折算后的宽度(px) */
const widthAtActiveSize = (
    el: Element,
    baseFontSizePx: number,
    targetEm: number
): number => {
    const element = el as HTMLElement;
    const currentPx = parseFloat(getComputedStyle(element).fontSize);
    const targetPx = targetEm * baseFontSizePx;
    // 读不到字号时不补偿, 直接用测量值
    const scale = currentPx > 0 && targetPx > 0 ? targetPx / currentPx : 1;
    return element.scrollWidth * scale;
};

/**
 * 计算某一行用于「是否需要跑马灯」判定的宽度(px)。
 *
 * `li` 为歌词行元素: `children[0]` 是主歌词, `children[1]`(存在时)是副歌词。
 * 取两者中较宽者, 与原来的 `max` 语义一致。
 *
 * `mainEm` / `secondEm` 是主副歌词 **active 时** 的字号(em)，
 * 由设置的 `fontSize` 传入，必须与 `MainLyric` / `SecondLyric` 的 font-size 一致。
 */
export const measureLineWidth = (
    li: HTMLElement,
    mainEm: number = DEFAULT_ACTIVE_MAIN_EM,
    secondEm: number = DEFAULT_ACTIVE_SECOND_EM
): number => {
    const children = li.children;
    if (children.length === 0) return 0;

    // em 相对该行自身字号, 与 CSS 中的 `font-size: Nem` 保持一致
    const baseFontSizePx = parseFloat(getComputedStyle(li).fontSize) || 16;

    const mainWidth = widthAtActiveSize(children[0], baseFontSizePx, mainEm);
    if (children.length < 2) return mainWidth;

    const secondWidth = widthAtActiveSize(
        children[1],
        baseFontSizePx,
        secondEm
    );
    return Math.max(mainWidth, secondWidth);
};

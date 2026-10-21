/**
 * 「歌词行数」相关计算的**唯一出处**。
 *
 * `/lyrics` 展示端与控制页顶部预览用的是同一个 `pages/LyricsBox` 组件，
 * 所以可见行数 / 行距离 / 样式层级只有这一份实现，不存在两套算法。
 * 8 套行数（1/3/5/…/15）也不复制 8 套渲染代码，全部由这里的共享计算驱动。
 */

/** 允许的歌词行数：只允许奇数 */
export const LYRIC_LINE_OPTIONS = [1, 3, 5, 7, 9, 11, 13, 15] as const;

export const DEFAULT_LYRIC_LINES = 3;
export const MIN_LYRIC_LINES = 1;
export const MAX_LYRIC_LINES = 15;

/** 单行高度（px）：必须与 LyricsBox 里中间 3 行 <li> 的实际高度一致 */
export const LYRIC_LINE_HEIGHT = 100;

/**
 * 额外歌词行（距当前行 ≥ 2）的行高（px）。
 *
 * 中间 3 行（距离 ≤ 1）保持 LYRIC_LINE_HEIGHT 的完整间距，之外的行按
 * 0.5 倍字号同比压缩，避免 3 行以上时外围行之间空隙过大。
 */
export const LYRIC_LINE_HEIGHT_OUTER = LYRIC_LINE_HEIGHT / 2;

/** 行高：距离 ≤ 1（中间 3 行）完整高度，其余行压缩高度 */
export const lineRowHeight = (distance: number): number =>
    distance <= 1 ? LYRIC_LINE_HEIGHT : LYRIC_LINE_HEIGHT_OUTER;

/**
 * 归一化：任何来源（缺失 / 非数字 / 越界 / 偶数）的值都收敛到合法奇数。
 *
 * - 缺失（undefined / null / 空串）→ 3（默认值）
 * - < 1 → 1
 * - > 15 → 15
 * - 偶数 → 相邻合法奇数（向上优先，15 封顶时向下）
 *
 * 归一化后渲染层只会拿到 1 / 3 / 5 / 7 / 9 / 11 / 13 / 15。
 */
export const normalizeLyricLines = (value: unknown): number => {
    if (value === null || value === undefined || value === "") {
        return DEFAULT_LYRIC_LINES;
    }
    const raw = typeof value === "number" ? value : Number(value);
    if (!Number.isFinite(raw)) return DEFAULT_LYRIC_LINES;
    const n = Math.trunc(raw);
    if (n < MIN_LYRIC_LINES) return MIN_LYRIC_LINES;
    if (n > MAX_LYRIC_LINES) return MAX_LYRIC_LINES;
    if (n % 2 === 1) return n;
    return n + 1 <= MAX_LYRIC_LINES ? n + 1 : n - 1;
};

/**
 * 可见窗口：当前歌词**上下各**显示多少行。
 *
 * 1 → 0，3 → 1，5 → 2，……，15 → 7
 */
export const visibleSideCount = (lines: number): number =>
    (normalizeLyricLines(lines) - 1) / 2;

/**
 * 当前歌词中心在列表中的纵向位置（px）。
 *
 * 行高按到当前行的距离分段（见 lineRowHeight），所以要分段累加：
 * 距离 1 的那行完整高度，其余在当前行之前的行都是压缩高度。
 */
export const cursorCenterY = (cursor: number): number =>
    Math.max(0, cursor - 1) * LYRIC_LINE_HEIGHT_OUTER +
    (cursor >= 1 ? LYRIC_LINE_HEIGHT : 0) +
    LYRIC_LINE_HEIGHT / 2;

/**
 * 歌词窗口总高度（px）：中间 3 行完整高度 + 其余行压缩高度。
 *
 * 3 行即为原来的 300px，逐像素一致；1 行为单行高度。
 */
export const lyricBoxHeight = (lines: number): number => {
    const n = normalizeLyricLines(lines);
    return n <= 1
        ? LYRIC_LINE_HEIGHT
        : 3 * LYRIC_LINE_HEIGHT + (n - 3) * LYRIC_LINE_HEIGHT_OUTER;
};

/**
 * 列表需要的纵向位移（px）：让当前歌词落在窗口正中间。
 *
 * 窗口高度按压缩后的行高计算，位移 = 窗口中心 - 当前歌词中心；
 * 3 行时可见行的位置与原来的 `-(cursor - 1) * 100` 逐像素一致。
 */
export const windowTranslateY = (cursor: number, lines: number): number =>
    lyricBoxHeight(lines) / 2 - cursorCenterY(cursor);

/** 行距离：0 表示当前歌词，1 表示上下相邻，以此类推 */
export const lineDistance = (index: number, cursor: number): number =>
    Math.abs(index - cursor);

/**
 * Controller 顶部预览的固定 viewport 行数。
 *
 * 固定为「默认 3 行」的高度，**不随用户设置的 lyricLines 变化**：
 * 控制面板的位置必须始终与 3 行时一致，歌词只在固定 viewport 内裁剪/隐藏。
 * `/lyrics` 不使用这个上限（不传 viewportLines → 窗口高度 = lyricLines）。
 */
export const CONTROLLER_VIEWPORT_LINES = 3;

/**
 * 窗口高度（以及居中位移）用的行数。
 *
 * 传了固定 viewport 就始终用它（Controller），否则跟随用户设置（/lyrics）。
 */
export const boxLineCount = (lines: number, viewportLines?: number): number =>
    viewportLines === undefined
        ? normalizeLyricLines(lines)
        : normalizeLyricLines(viewportLines);

/**
 * 允许参与显示的最大行数：用户设置与固定 viewport 取较小者。
 *
 * 两个值都是合法奇数，所以结果也是合法奇数（1/3/5/…/15）。
 * 超出这个范围的行在 Controller 里被隐藏；/lyrics 由窗口高度自然裁剪。
 */
export const windowLineCount = (
    lines: number,
    viewportLines?: number
): number =>
    viewportLines === undefined
        ? normalizeLyricLines(lines)
        : Math.min(
              normalizeLyricLines(lines),
              normalizeLyricLines(viewportLines)
          );

export type LyricLineLevel = 0 | 1 | 2;

/**
 * 样式层级（只有 3 级，不随距离无限细分）：
 *
 * - 距离 0 → 0：当前歌词样式
 * - 距离 1 → 1：3 行模式下相邻歌词的样式
 * - 距离 ≥ 2 → 2：3 行模式下最外层歌词的样式
 */
export const lineLevel = (distance: number): LyricLineLevel => {
    if (distance <= 0) return 0;
    if (distance === 1) return 1;
    return 2;
};

/**
 * 各层级相对「当前歌词字号」的缩放。
 *
 * 层级 1 与层级 2 取值相同：3 行模式下最外层就是距离 1 的那一行，
 * 所以 15 行时距离 2…7 与相邻行完全一致，不会越缩越小；
 * 3 行（只有距离 0 / 1）的结果与改动前逐像素一致（1 与 1/2）。
 */
export const LINE_LEVEL_FONT_SCALE: Readonly<Record<LyricLineLevel, number>> = {
    0: 1,
    1: 0.5,
    2: 0.5,
};

/** 层级对应的字号缩放 */
export const lineLevelScale = (level: LyricLineLevel): number =>
    LINE_LEVEL_FONT_SCALE[level];

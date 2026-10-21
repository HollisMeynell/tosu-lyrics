const expand = (hex: string) =>
    hex
        .split("")
        .map((c) => c + c)
        .join("");

/**
 * 解析 CSS hex 颜色（#rgb / #rgba / #rrggbb / #rrggbbaa，`#` 可省略）。
 * 返回 6 位 `rgb` 与 `alpha`（0~1）；无法解析时按不透明黑色处理。
 */
export const parseColor = (value: string): { rgb: string; alpha: number } => {
    const hex = value.trim().replace(/^#/, "");
    const full = hex.length === 3 || hex.length === 4 ? expand(hex) : hex;
    const rgb = full.slice(0, 6).toLowerCase();
    const alphaHex = full.slice(6, 8);
    const ok =
        /^[0-9a-f]{6}$/.test(rgb) &&
        (alphaHex === "" || /^[0-9a-f]{2}$/.test(alphaHex));
    if (!ok) return { rgb: "#000000", alpha: 1 };
    return {
        rgb: `#${rgb}`,
        alpha: alphaHex ? Number.parseInt(alphaHex, 16) / 255 : 1,
    };
};

/**
 * 组合颜色与不透明度：alpha=1 时保持 `#rrggbb`（与历史存储一致），
 * 否则输出 `#rrggbbaa`（CSS Color 4 顺序，可直接用于 `color`）。
 */
export const withAlpha = (value: string, alpha: number): string => {
    const clamped = Math.min(1, Math.max(0, alpha));
    const { rgb } = parseColor(value);
    if (clamped >= 1) return rgb;
    const a = Math.round(clamped * 255)
        .toString(16)
        .padStart(2, "0");
    return `${rgb}${a}`;
};

import { createSignal } from "solid-js";

/**
 * 主副歌词字体「共用 / 分开」模式。
 *
 * **为什么放 localStorage 而不是后端设置字段**：实际渲染只由 `font.first` /
 * `font.second` 决定。共用模式生效时这两个值会被写成同一个字体，所以换浏览器、
 * 换展示端看到的字形本来就一致；「模式」只影响控制台的编辑形态。因此不值得为它
 * 新增后端设置字段（也避免动到已验证正常的设置链路）。
 *
 * 切入共用前会把当前的分开设置备份下来，切回时恢复，避免来回切换丢失用户选择。
 */
export type FontMode = "split" | "shared";

const MODE_KEY = "osu-lyrics.font-mode";
const BACKUP_KEY = "osu-lyrics.font-split-backup";

export interface SplitBackup {
    first: string;
    second: string;
}

const readMode = (): FontMode => {
    try {
        return localStorage.getItem(MODE_KEY) === "shared" ? "shared" : "split";
    } catch {
        return "split";
    }
};

/** 默认「分开」——与项目原有行为一致 */
export const [fontMode, setFontModeSignal] = createSignal<FontMode>(readMode());

export const readSplitBackup = (): SplitBackup | null => {
    try {
        const raw = localStorage.getItem(BACKUP_KEY);
        if (!raw) return null;
        const parsed = JSON.parse(raw) as Partial<SplitBackup>;
        if (
            typeof parsed?.first === "string" &&
            typeof parsed?.second === "string"
        ) {
            return { first: parsed.first, second: parsed.second };
        }
    } catch {
        /* 备份损坏时按"没有备份"处理 */
    }
    return null;
};

export const writeSplitBackup = (backup: SplitBackup) => {
    try {
        localStorage.setItem(BACKUP_KEY, JSON.stringify(backup));
    } catch {
        /* 隐私模式等写不了时忽略：后果只是无法恢复分开设置 */
    }
};

export const clearSplitBackup = () => {
    try {
        localStorage.removeItem(BACKUP_KEY);
    } catch {
        /* ignore */
    }
};

export const setFontMode = (mode: FontMode) => {
    try {
        localStorage.setItem(MODE_KEY, mode);
    } catch {
        /* 同上 */
    }
    setFontModeSignal(mode);
};

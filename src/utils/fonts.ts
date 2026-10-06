import { createSignal } from "solid-js";
import { BACKEND_API_BASE } from "@/config/constants";
import {
    font,
    secondFont,
    setLoadedMainFamily,
    setLoadedSubFamily,
} from "@/stores/settingsStore";

/**
 * 字体系统（字体库模型）。
 *
 * 后端维护一个字体库（`font/` 目录），每个字体有内部名称。
 * 前端通过 `/api/font/list` 获取可用字体列表，
 * 按名称从 `/api/font/:name` 加载字体文件并注册为 FontFace。
 *
 * 设置中的 `font.first` / `font.second` 存储的是字体名称：
 * - 空串 `""` → 使用系统默认字体
 * - 非空串 → 从字体库加载对应字体
 */

/** 后端字体条目结构（与 FontEntry camelCase 一致） */
export interface FontEntry {
    name: string;
    fileName: string;
    size: number;
    version: string;
    url: string;
}

/**
 * 「默认字体」解析到的**系统回退字体栈**。
 *
 * font 为空串时使用此值，等同于 Tailwind preflight 的 `var(--default-font-family)`。
 */
export const DEFAULT_SYSTEM_FAMILY =
    "var(--font-sans, ui-sans-serif, system-ui, sans-serif)";

/** 最近一次 `/api/font/list` 的结果，供字体选择 UI 直接复用 */
export const [fontEntries, setFontEntries] = createSignal<FontEntry[]>([]);

/** 查询后端当前的字体列表 */
export async function fetchFontList(): Promise<FontEntry[]> {
    const res = await fetch(`${BACKEND_API_BASE}/font/list`);
    if (!res.ok) return [];
    const body = (await res.json()) as { items: FontEntry[] };
    const items = body.items ?? [];
    setFontEntries(items);
    return items;
}

/** 上传字体到字体库。返回新的字体条目。 */
export async function uploadFont(file: File): Promise<FontEntry> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${BACKEND_API_BASE}/font/upload`, {
        method: "POST",
        body: form,
    });
    const body = await res.json().catch(() => null);
    if (!res.ok) {
        throw new Error(body?.error?.message ?? `HTTP ${res.status}`);
    }
    return body.font as FontEntry;
}

/**
 * 已注册的 FontFace：name -> 该 face 当前生效的版本。
 *
 * 保存 face 引用是为了覆盖上传：同一个 name 再 add 一次时，浏览器可能继续
 * 使用先前那个 face。换版本前先 delete 旧的。
 */
const registered = new Map<string, { face: FontFace; version: string }>();

async function loadFamily(
    name: string,
    url: string,
    version: string
): Promise<boolean> {
    const current = registered.get(name);
    if (current?.version === version) return true;
    try {
        const face = new FontFace(name, `url(${url})`);
        const loaded = await face.load();
        if (current) document.fonts.delete(current.face);
        document.fonts.add(loaded);
        registered.set(name, { face: loaded, version });
        return true;
    } catch {
        return false;
    }
}

/**
 * 加载主 / 副字体。
 *
 * 根据设置中的字体名，从字体库加载对应的 FontFace。
 * 返回实际可用的字体名（用于 CSS font-family）。
 */
export async function loadFont(): Promise<{
    main: string;
    sub: string;
}> {
    const mainName = font();
    const subName = secondFont();

    // 加载字体列表
    const entries = await fetchFontList().catch(() => [] as FontEntry[]);

    // 按名称查找并加载字体
    let mainFamily = DEFAULT_SYSTEM_FAMILY;
    let subFamily = DEFAULT_SYSTEM_FAMILY;

    if (mainName) {
        const entry = entries.find((e) => e.name === mainName);
        if (entry) {
            const ok = await loadFamily(entry.name, entry.url, entry.version);
            if (ok) mainFamily = entry.name;
        }
    }

    if (subName) {
        const entry = entries.find((e) => e.name === subName);
        if (entry) {
            const ok = await loadFamily(entry.name, entry.url, entry.version);
            if (ok) subFamily = entry.name;
        }
    }

    setLoadedMainFamily(mainFamily);
    setLoadedSubFamily(subFamily);

    return { main: mainFamily, sub: subFamily };
}

/**
 * 把字体名解析成**实际 CSS font-family**。
 *
 * - 空串 → 系统回退字体栈
 * - 非空 → 如果已注册为 FontFace 则使用该名称，否则原样返回（可能是系统字体名）
 */
export function resolveFamily(name: string): string {
    if (!name) return DEFAULT_SYSTEM_FAMILY;
    // 如果已经注册为 FontFace，直接用名称
    if (registered.has(name)) return name;
    // 否则当作系统字体名原样返回
    return name;
}

/** 兼容旧调用点 */
export const loadDefaultFont = async () => (await loadFont()).main;
export const loadOriginFont = loadDefaultFont;
export const loadTranslateFont = async () => (await loadFont()).sub;
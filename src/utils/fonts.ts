import { BACKEND_API_BASE } from "@/config/constants";

/**
 * 字体加载（B-07）。
 *
 * 字体由后端按**版本化 URL**提供：`/api/font/main?v=<mtime>-<size>`。
 * 版本一变 URL 就变，`FontFace` 会重新拉取，浏览器不会继续用旧字体 ——
 * 这正是"连续上传不同字体，旧资源不会被缓存住"的机制。
 *
 * 没有上传过字体时回退到随包分发的静态文件，保证开箱可用。
 */

const STATIC_MAIN = new URL("/LRC.otf", import.meta.url).href;
const STATIC_SUB = new URL("/tLRC.otf", import.meta.url).href;

export interface FontInfo {
    kind: "main" | "sub";
    family: string;
    exists: boolean;
    version: string;
    size: number;
    url: string;
}

/** 查询后端当前的主 / 副字体信息 */
export async function fetchFontInfo(): Promise<FontInfo[]> {
    const res = await fetch(`${BACKEND_API_BASE}/font/info`);
    if (!res.ok) return [];
    const body = (await res.json()) as { items: FontInfo[] };
    return body.items ?? [];
}

/** 上传字体（覆盖）。返回新的字体信息。 */
export async function uploadFont(
    kind: "main" | "sub",
    file: File
): Promise<FontInfo> {
    const form = new FormData();
    form.append("file", file);
    const res = await fetch(`${BACKEND_API_BASE}/font/${kind}`, {
        method: "POST",
        body: form,
    });
    const body = await res.json().catch(() => null);
    if (!res.ok) {
        throw new Error(body?.error?.message ?? `HTTP ${res.status}`);
    }
    return body.font as FontInfo;
}

/** 已注册的 FontFace 家族，避免重复添加同名家族 */
const registered = new Map<string, string>();

async function loadFamily(
    family: string,
    url: string,
    version: string
): Promise<string | null> {
    // 版本没变就不用重新加载
    if (registered.get(family) === version) return family;
    try {
        const face = new FontFace(family, `url(${url})`);
        const loaded = await face.load();
        document.fonts.add(loaded);
        registered.set(family, version);
        return family;
    } catch {
        return null;
    }
}

/**
 * 加载主 / 副字体。
 *
 * 返回实际可用的家族名（上传过就是后端给的，否则是静态回退）。
 */
export async function loadFont(): Promise<{
    main: string;
    sub: string;
}> {
    const infos = await fetchFontInfo().catch(() => [] as FontInfo[]);
    const main = infos.find((f) => f.kind === "main");
    const sub = infos.find((f) => f.kind === "sub");

    const mainFamily = main?.exists
        ? ((await loadFamily(main.family, main.url, main.version)) ?? "LRC")
        : ((await loadFamily("LRC", STATIC_MAIN, "static")) ?? "LRC");

    const subFamily = sub?.exists
        ? ((await loadFamily(sub.family, sub.url, sub.version)) ?? "LRC-Sub")
        : ((await loadFamily("LRC-Sub", STATIC_SUB, "static")) ?? "LRC-Sub");

    return { main: mainFamily, sub: subFamily };
}

/** 兼容旧调用点 */
export const loadDefaultFont = async () => (await loadFont()).main;
export const loadOriginFont = loadDefaultFont;
export const loadTranslateFont = async () => (await loadFont()).sub;

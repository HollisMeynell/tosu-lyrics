import { createSignal } from "solid-js";
import { BACKEND_API_BASE } from "@/config/constants";
import {
    font,
    loadedMainFamily,
    loadedSubFamily,
    secondFont,
    setLoadedMainFamily,
    setLoadedSubFamily,
} from "@/stores/settingsStore";

/**
 * 字体加载（B-07）。
 *
 * 字体由后端按**版本化 URL**提供：`/api/font/main?v=<mtime>-<size>`。
 * 版本一变 URL 就变，`FontFace` 会重新拉取，浏览器不会继续用旧字体 ——
 * 这正是"连续上传不同字体，旧资源不会被缓存住"的机制。
 *
 * 没有上传过字体时回退到随包分发的静态文件，保证开箱可用。
 */

// 静态回退：静态资源服务挂在 `/lyrics/{**path}` 下（见 server/file.rs 的
// CONFIG_FRONTEND），查找路径含程序目录 "./"，所以必须带 /lyrics 前缀才能命中
// 随包分发的字体。原先用 `new URL("/LRC.otf", import.meta.url)` 得到的是根路径
// `/LRC.otf`，必然 404；这里改用 Vite 的 BASE_URL 跟随 vite.config.ts 的 base。
// 注意：BASE_URL 等于 base 原值，本项目是 "/lyrics"（**没有**尾斜杠），
// 直接拼接会得到 "/lyricsLRC.otf"，所以必须先补齐分隔符。
const BASE = import.meta.env.BASE_URL.endsWith("/")
    ? import.meta.env.BASE_URL
    : `${import.meta.env.BASE_URL}/`;
const STATIC_MAIN = `${BASE}LRC.otf`;
const STATIC_SUB = `${BASE}tLRC.otf`;

export interface FontInfo {
    kind: "main" | "sub";
    /** FontFace 名（`LRC` / `LRC-Sub`）：注册与实际渲染用的就是它 */
    family: string;
    /** 字体文件内部的真实名称，**仅供 UI 显示**（后端解析 name 表得到） */
    displayName: string;
    exists: boolean;
    version: string;
    size: number;
    url: string;
}

/**
 * 四套互相独立的 FontFace family。
 *
 * **为什么内置字体要单独一套**：上传字体与内置字体原先都用 `LRC` / `LRC-Sub`，
 * 于是"上传了字体"就等于把默认字体顶掉了，UI 上无法做到
 * 「主 = LRC.otf，副 = Unifont-JP」这种组合。分开之后两者可以同时注册、并存。
 *
 * 上传侧的两个 family **保持不变**（`LRC` / `LRC-Sub`），已验证正常的注册链路不受影响。
 */
export const UPLOADED_MAIN_FAMILY = "LRC";
export const UPLOADED_SUB_FAMILY = "LRC-Sub";
export const DEFAULT_MAIN_FAMILY = "LRC-Default";
export const DEFAULT_SUB_FAMILY = "LRC-Sub-Default";

/** 字体选择码：强制使用内置 LRC.otf / tLRC.otf */
export const FONT_CODE_DEFAULT = "default";
/** 字体选择码：强制使用上传字体（不存在时回落内置） */
export const FONT_CODE_UPLOADED = "uploaded";

/**
 * 历史存储值归一化 —— 这些取值在旧版本里都表示「默认字体」：
 *
 * - `""`：设置字段的默认值。旧版本把它当「自动」（有上传字体就用上传字体），
 *   于是界面上显示「默认字体」、实际渲染的却是上传字体 —— 必须收敛到默认字体标识。
 * - `"LRC.otf"` / `"tLRC.otf"`：更早的版本直接把文件名存进设置。
 *
 * 归一化后只会指向**程序自带的默认字体资源**，绝不会解析到上传字体。
 * UI（FontPicker 的选中项/显示名）与渲染（resolveFamily）共用这一个函数，
 * 不会再出现"界面说默认、渲染成上传字体"的分歧。
 */
const LEGACY_DEFAULT_CODES = new Set(["", "LRC.otf", "tLRC.otf"]);

export const normalizeFontCode = (code: string): string =>
    LEGACY_DEFAULT_CODES.has(code) ? FONT_CODE_DEFAULT : code;

/** 最近一次 `/api/font/info` 的结果，供字体选择 UI 直接复用，避免重复请求 */
export const [fontInfos, setFontInfos] = createSignal<FontInfo[]>([]);

/** 某个 family 是否已经注册进 document.fonts（用于判断上传字体是否可用） */
export const isFamilyRegistered = (family: string) => registered.has(family);

/** 查询后端当前的主 / 副字体信息 */
export async function fetchFontInfo(): Promise<FontInfo[]> {
    const res = await fetch(`${BACKEND_API_BASE}/font/info`);
    if (!res.ok) return [];
    const body = (await res.json()) as { items: FontInfo[] };
    const items = body.items ?? [];
    // 供字体选择 UI 复用：上传页与文字样式页看到的是同一份数据
    setFontInfos(items);
    return items;
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

/**
 * 已注册的 FontFace：family -> 该 family 当前生效的 face 与版本。
 *
 * 保存 face 引用是为了**覆盖上传**：同一个 family 再 add 一次时，浏览器可能继续
 * 使用先前那个 face，表现为"上传了新字体但字形没变"。所以换版本前先 delete 旧的。
 */
const registered = new Map<string, { face: FontFace; version: string }>();

async function loadFamily(
    family: string,
    url: string,
    version: string
): Promise<string | null> {
    const current = registered.get(family);
    // 版本没变就不用重新加载
    if (current?.version === version) return family;
    try {
        const face = new FontFace(family, `url(${url})`);
        const loaded = await face.load();
        if (current) document.fonts.delete(current.face);
        document.fonts.add(loaded);
        registered.set(family, { face: loaded, version });
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
    const uploadedMain = main?.exists ? main : undefined;
    const uploadedSub = sub?.exists ? sub : undefined;

    // 上传字体：沿用原来的 family（LRC / LRC-Sub），已验收的注册链路不变
    if (uploadedMain) {
        await loadFamily(
            UPLOADED_MAIN_FAMILY,
            uploadedMain.url,
            uploadedMain.version
        );
    }
    if (uploadedSub) {
        await loadFamily(
            UPLOADED_SUB_FAMILY,
            uploadedSub.url,
            uploadedSub.version
        );
    }

    // 内置字体：**按需注册**。LRC.otf 约 8MB，不该在每次打开页面时都下载，
    // 所以只在当前选择真的要用到它时才加载（registered 的版本检查保证只下一次）。
    if (needsBuiltinFont(font(), !!uploadedMain)) {
        await loadFamily(DEFAULT_MAIN_FAMILY, STATIC_MAIN, "static-main");
    }
    if (needsBuiltinFont(secondFont(), !!uploadedSub)) {
        await loadFamily(DEFAULT_SUB_FAMILY, STATIC_SUB, "static-sub");
    }

    // "自动"档实际用哪个 family：有上传用上传，否则内置（沿用原有语义）。
    // 把"实际注册成功的 family"暴露给渲染层，CSS 才能真正用上它。
    const mainFamily = uploadedMain
        ? UPLOADED_MAIN_FAMILY
        : DEFAULT_MAIN_FAMILY;
    const subFamily = uploadedSub ? UPLOADED_SUB_FAMILY : DEFAULT_SUB_FAMILY;

    setLoadedMainFamily(mainFamily);
    setLoadedSubFamily(subFamily);

    return { main: mainFamily, sub: subFamily };
}

/**
 * 旧配置里可能存过的**字体家族名**：它们以前被直接当成 CSS family 使用
 * （上传字体与内置字体共用同一个 family），所以统一按"自动"处理。
 *
 * 注意 `""` **不在**这里：空值必须解析成程序自带的默认字体，
 * 否则"默认字体"会被上传字体顶掉（见 `normalizeFontCode`）。
 */
const LEGACY_AUTO_CODES = new Set([
    UPLOADED_MAIN_FAMILY,
    UPLOADED_SUB_FAMILY,
]);

const isAutoCode = (code: string) => LEGACY_AUTO_CODES.has(code);

/** 这个选择码是否需要用内置字体（据此决定要不要下载它） */
function needsBuiltinFont(rawCode: string, hasUploaded: boolean): boolean {
    const code = normalizeFontCode(rawCode);
    // 自动档：没有上传字体时只能用内置
    if (isAutoCode(code)) return !hasUploaded;
    if (code === FONT_CODE_DEFAULT) return true;
    // 强制上传但当前没有上传字体 → 会回落到内置，所以要内置可用
    if (code === FONT_CODE_UPLOADED) return !hasUploaded;
    // 系统字体名：与改动前一致，不需要内置字体
    return false;
}

/**
 * 把字体选择码解析成**实际 CSS font-family**。
 *
 * 永远返回非空字符串：选择码指向的自定义字体不存在时回落内置 LRC.otf，
 * 不会产生 `font-family: undefined`，也不会产生空的选择列表。
 *
 * 映射关系（先经 `normalizeFontCode` 归一化历史值）：
 * - `""` / `"LRC.otf"` / `"tLRC.otf"`（旧配置）→ **默认字体**（随包内置资源）
 * - `"LRC"` / `"LRC-Sub"`（旧 family 名）→ 自动（有上传用上传，否则内置）
 * - `FONT_CODE_DEFAULT`  → `LRC-Default` / `LRC-Sub-Default`（内置随包字体）
 * - `FONT_CODE_UPLOADED` → `LRC` / `LRC-Sub`（上传字体，缺失则回落内置）
 * - 其它                 → 系统字体名，原样使用
 */
export function resolveFamily(code: string, kind: "main" | "sub"): string {
    const normalized = normalizeFontCode(code);
    const isMain = kind === "main";
    const auto = isMain ? loadedMainFamily() : loadedSubFamily();
    const builtin = isMain ? DEFAULT_MAIN_FAMILY : DEFAULT_SUB_FAMILY;
    const uploaded = isMain ? UPLOADED_MAIN_FAMILY : UPLOADED_SUB_FAMILY;

    if (isAutoCode(normalized)) return auto || builtin;
    if (normalized === FONT_CODE_DEFAULT) return builtin;
    if (normalized === FONT_CODE_UPLOADED) {
        return isFamilyRegistered(uploaded) ? uploaded : builtin;
    }
    return normalized;
}

/** 兼容旧调用点 */
export const loadDefaultFont = async () => (await loadFont()).main;
export const loadOriginFont = loadDefaultFont;
export const loadTranslateFont = async () => (await loadFont()).sub;

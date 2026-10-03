// 功能: 面板-文字样式（主 / 副歌词字体选择）
import { Show } from "solid-js";
import { FontPicker } from "@/components/ui";
import { fontMode } from "@/stores/fontModeStore";
import { font, secondFont } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";
import {
    FONT_CODE_DEFAULT,
    FONT_CODE_UPLOADED,
    fontInfos,
    normalizeFontCode,
} from "@/utils/fonts";

interface FontProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

/**
 * 系统字体：与项目原有列表保持一致，继续作为额外选项保留。
 * cSpell: ignore msyh simsun fangsong kaiti
 */
const SYSTEM_FONTS = [
    { code: "msyh", name: "微软雅黑" },
    { code: "simsun", name: "宋体" },
    { code: "fangsong", name: "仿宋" },
    { code: "kaiti", name: "楷体" },
    { code: "arial", name: "Arial" },
];

/**
 * 归一化**历史存储值**：早期版本的字体选择码曾是文件名（`LRC.otf` / `tLRC.otf`）或空串，
 * 归一化到 `FONT_CODE_DEFAULT` 后，选择框才会显示「默认字体」而不是文件名。
 *
 * 实现已收敛到 `utils/fonts.ts` 的 `normalizeFontCode()` —— **渲染层用的是同一个函数**
 * （`resolveFamily` 内部也会先归一化），所以不会再出现"界面显示默认字体、
 * 实际渲染成上传字体"的分歧。
 */
export default function Controller(props: FontProps) {
    const shared = () => fontMode() === "shared";

    /**
     * 选项列表 = 默认 `LRC.otf` + 当前上传的主/副字体 + 系统字体。
     *
     * 默认字体**永远存在**，上传了自定义字体也不会把它顶掉；上传字体的显示名用
     * 后端解析出的 `displayName`（字体文件内部名称），与 FontFace family 分离。
     * 这份数据来自 `fontInfos()`，与字体资源区共用同一次 `/api/font/info` 请求。
     */
    const optionsFor = (kind: "main" | "sub") => {
        const options = [{ code: FONT_CODE_DEFAULT, name: "默认字体" }];
        const uploaded = fontInfos().find((item) => item.kind === kind);
        // 上传字体槽位**始终存在**：未上传时显示「上传字体」，上传成功后显示后端解析出的真实字体名。
        // 选中它但实际没有上传字体时，`resolveFamily()` 会安全回落内置字体。
        options.push({
            code: FONT_CODE_UPLOADED,
            name: uploaded?.exists
                ? uploaded.displayName || "上传字体"
                : "上传字体",
        });
        return [...options, ...SYSTEM_FONTS];
    };

    // SettingsPatch 里 font 必须是完整的一对，所以每次都带上另一侧的原值。
    const applyMain = (value: string) => {
        void props.update({
            // 共用模式下改主字体要同时写两侧，否则会出现"UI 共用、渲染却不共用"
            font: { first: value, second: shared() ? value : secondFont() },
        });
    };

    const applySub = (value: string) => {
        void props.update({ font: { first: font(), second: value } });
    };

    return (
        <div class="flex flex-col items-start gap-4">
            <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                <h2 class="text-xl font-normal">主歌词字体</h2>
                <FontPicker
                    class="w-56"
                    options={optionsFor("main")}
                    value={normalizeFontCode(font())}
                    disabled={props.disabled}
                    onChange={applyMain}
                />
            </div>

            <Show
                when={!shared()}
                fallback={
                    <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                        <h2 class="text-xl font-normal">副歌词字体</h2>
                        <span class="text-sm text-gray-500">
                            共用字体模式：副歌词跟随主字体
                        </span>
                    </div>
                }
            >
                <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
                    <h2 class="text-xl font-normal">副歌词字体</h2>
                    <FontPicker
                        class="w-56"
                        options={optionsFor("sub")}
                        value={normalizeFontCode(secondFont())}
                        disabled={props.disabled}
                        onChange={applySub}
                    />
                </div>
            </Show>
        </div>
    );
}

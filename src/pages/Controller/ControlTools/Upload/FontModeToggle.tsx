import { ToggleNSwitch } from "@/components/ui";
import { createSettingsController } from "@/hooks/useSettings";
import {
    fontMode,
    readSplitBackup,
    setFontMode,
    writeSplitBackup,
    type FontMode,
} from "@/stores/fontModeStore";
import { font, secondFont } from "@/stores/settingsStore";

/** 与「对齐方式」同款的并排切换控件 */
const MODE_OPTIONS = [
    { key: "共用", value: "shared" },
    { key: "分开", value: "split" },
];

/**
 * 主副歌词「共用字体 / 分开字体」开关。
 *
 * UI 与「对齐方式」保持一致（ToggleNSwitch）；逻辑与之前完全相同，未做任何改动：
 * - **分开**（默认，项目原有行为）：`font.first` / `font.second` 各管各的。
 * - **共用**：两者写成同一个字体。
 * 切入共用前把当前分开设置备份到 localStorage，切回时恢复，
 * 因此来回切换不会丢掉用户原来选的两个字体，也不需要新增后端设置字段。
 */
export default function FontModeToggle() {
    const settings = createSettingsController();
    const shared = () => fontMode() === "shared";

    const toShared = async () => {
        // 备份当前分开设置，再合并成同一个字体（优先沿用主字体）
        writeSplitBackup({ first: font(), second: secondFont() });
        const common = font() || secondFont();
        if (await settings.update({ font: { first: common, second: common } })) {
            setFontMode("shared");
        }
    };

    const toSplit = async () => {
        const backup = readSplitBackup();
        // 有备份就恢复；没有备份也不阻塞切换（只是沿用当前值）
        if (!backup || (await settings.update({ font: backup }))) {
            setFontMode("split");
        }
    };

    const applyMode = (value: string) => {
        const next = value as FontMode;
        if (next === fontMode()) return;
        void (next === "shared" ? toShared() : toSplit());
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-6">
            <h2 class="text-xl font-normal">共用字体</h2>
            {/* 只有两个选项：用 w-fit 让容器贴合两格宽度，
                不要用 min-w（那会在右侧留出一个空格的观感） */}
            <ToggleNSwitch
                options={MODE_OPTIONS}
                class="w-fit"
                disabled={settings.saving()}
                selectedValue={shared() ? "shared" : "split"}
                onUpdateSelectedValue={applyMode}
            />
        </div>
    );
}

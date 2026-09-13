// 功能: 面板-文字样式（颜色、字体、显示效果）
import { Select } from "@/components/ui";
import { font } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";

interface FontProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

export default function Controller(props: FontProps) {
    const fonts = [
        // cSpell: ignore msyh simsun fangsong kaiti
        { code: "", name: "LRC.otf" },
        { code: "msyh", name: "微软雅黑" },
        { code: "simsun", name: "宋体" },
        { code: "fangsong", name: "仿宋" },
        { code: "kaiti", name: "楷体" },
        { code: "arial", name: "Arial" },
    ];

    // 后端契约支持主 / 副独立字体，但当前 UI 只有一个选择器：
    // 主副一起设置，等后续做独立选择控件时再拆开。
    const apply = (value: string) => {
        void props.update({ font: { first: value, second: value } });
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
            <h2 class="text-2xl font-normal">字体</h2>
            <Select
                class="w-56"
                options={fonts}
                value={font()}
                disabled={props.disabled}
                onChange={apply}
            />
        </div>
    );
}

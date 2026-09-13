// 功能: 面板-文字样式（颜色、字体、显示效果）
import { ColorSelector } from "@/components/ui";
import { textColor } from "@/stores/settingsStore";
import type { SettingsPatch } from "@/types/globalTypes";

interface TextColorProps {
    update: (patch: SettingsPatch) => Promise<boolean>;
    disabled?: boolean;
}

export default function TextColor(props: TextColorProps) {
    const commit = (order: "first" | "second", value: string) => {
        void props.update({
            textColor: { ...textColor(), [order]: value },
        });
    };

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-9">
            <h2 class="text-2xl font-normal">文字颜色</h2>
            <div class="flex flex-row items-center gap-4">
                <p>主歌词:</p>
                <ColorSelector
                    class="min-w-6"
                    value={textColor().first}
                    disabled={props.disabled}
                    onCommit={(value) => commit("first", value)}
                />
                <p>副歌词:</p>
                <ColorSelector
                    class="min-w-6"
                    value={textColor().second}
                    disabled={props.disabled}
                    onCommit={(value) => commit("second", value)}
                />
            </div>
        </div>
    );
}

// 功能: 面板-文字样式（颜色、字体、显示效果）
import { ColorSelector } from "@/components/ui";

export default function TextColor() {
    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-9">
            <h2 class="text-2xl font-normal">文字颜色</h2>
            <div class="flex flex-row items-center gap-4">
                <p>主歌词:</p>
                <ColorSelector colorType="first" className="min-w-6" />
                <p>副歌词:</p>
                <ColorSelector colorType="second" className="min-w-6" />
            </div>
        </div>
    );
}

// 功能: 面板-文字样式（颜色、字体、显示效果）
import ColorSelector from "@/components/ui/customColorSelector";

export default function TextColor() {
    return (
        <div class="flex flex-row items-center gap-9">
            <h2 class="text-2xl font-normal">文字颜色</h2>
            <div class="flex flex-row items-center gap-4">
                <p>主歌词:</p>
                <ColorSelector colorType="first" />
                <p>副歌词:</p>
                <ColorSelector colorType="second" />
            </div>
        </div>
    );
}

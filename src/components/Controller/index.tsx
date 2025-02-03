// 功能: 功能操作面板
import CurrentLyrics from "./ControlTools/CurrentLyrics";
import TextStyle from "./ControlTools/TextStyle";
import DarkModeToggle from "./ControlTools/DarkModeToggle";

export default function Controller() {
    return (
        <div class="h-[calc(100%-300px)] bg-[#ffffff] dark:bg-[#141414] m-6  rounded-lg shadow-md overflow-auto scrollbar-hide dark:text-[#dcdcdc] relative transform-3d">
            {/* 通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560 */}
            <DarkModeToggle />
            <div class="h-full max-w-[calc(100%-50px)] p-6 overflow-auto scrollbar-hide md:flex-row flex flex-col gap-6">
                <CurrentLyrics />
                <div>
                    <TextStyle />
                    ...
                </div>
            </div>
        </div>
    );
}

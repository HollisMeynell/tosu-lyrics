// 功能: 功能操作面板
import DarkModeToggle from "./ControlTools/DarkModeToggle.tsx";
import AppRoutes from "@/pages/Controller/router";

export default function Controller() {
    return (
        <div class="h-[calc(100%-300px)] bg-[#ffffff] dark:bg-[#141414] m-6 px-6 py-8 rounded-lg shadow-md overflow-auto scrollbar-hide dark:text-[#dcdcdc] relative transform-3d">
            {/* 通过 relative 和 transform-3d 实现 DarkModeToggle 组件的 fixed 定位相对父元素而非视窗 666 借鉴 https://www.cnblogs.com/ai888/p/18598560 */}
            <AppRoutes />
            <DarkModeToggle />
        </div>
    );
}

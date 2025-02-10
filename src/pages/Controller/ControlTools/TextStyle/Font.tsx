// 功能: 面板-文字样式（颜色、字体、显示效果）
import { For } from "solid-js";
import lyricsStore from "@/stores/lyricsStore.ts";

export default function Controller() {
    const fonts = [
        // cSpell: ignore msyh simsun fangsong kaiti
        { code: "", name: "LRC.otf" },
        { code: "msyh", name: "微软雅黑" },
        { code: "simsun", name: "宋体" },
        { code: "fangsong", name: "仿宋" },
        { code: "kaiti", name: "楷体" },
        { code: "arial", name: "Arial" },
    ];

    // async function loadFontData() {
    //     try {
    //         if ("queryLocalFonts" in window) {
    //             const availableFonts = await window.queryLocalFonts();
    //             console.log(availableFonts);
    //             for (const fontData of availableFonts) {
    //                 fonts.push({
    //                     code: fontData.postscriptName,
    //                     name: fontData.fullName,
    //                 });
    //             }
    //         }
    //     } catch (err) {
    //         if (err instanceof Error) console.error(err.name, err.message);
    //     }
    // }

    // createEffect(() => {
    //     loadFontData();
    // });

    const clearSelection = () => {
        lyricsStore.setFont("");
    };

    return (
        <div class="flex flex-row items-center gap-19">
            <h2 class="text-2xl font-normal">字体</h2>
            <div class="relative w-full md:w-56">
                <select
                    value={lyricsStore.getState.font}
                    id="dd-language"
                    class="w-full p-2 pl-3 pr-8 border border-[#cbd5e1] rounded-lg shadow-xs appearance-none bg-white focus:outline-hidden focus:ring-1 focus:ring-[#eb4898] focus:border-[#eb4898] hover:border-[#94a3b8] transition-colors cursor-pointer dark:bg-[#020616] dark:border-[#475569] dark:text-white dark:focus:border-[#e169a8] dark:hover:border-[#64748b]"
                    onChange={(e) => {
                        const selectedFont = e.target.value;
                        lyricsStore.setFont(selectedFont); // 更新字体状态
                    }}
                >
                    <For each={fonts}>
                        {(font) => (
                            <option value={font.code}>{font.name}</option>
                        )}
                    </For>
                </select>
                <button
                    onClick={clearSelection}
                    class="absolute right-2 top-1/2 transform -translate-y-1/2 cursor-pointer"
                >
                    <span class="text-xl font-[100] text-[#94a3b8] leading-none">
                        ×
                    </span>
                </button>
            </div>
        </div>
    );
}

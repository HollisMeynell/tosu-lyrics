// 功能: 面板-文字样式（颜色、字体、显示效果）
import store from "@/stores/indexStore";
import { Select } from "@/components/ui";

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

    return (
        <div class="flex flex-col items-start md:flex-row md:items-center gap-4 md:gap-19">
            <h2 class="text-2xl font-normal">字体</h2>
            <Select
                class="w-56"
                options={fonts}
                value={store.getState.settings.font}
                onChange={(value) => store.setFont(value)}
            />
        </div>
    );
}

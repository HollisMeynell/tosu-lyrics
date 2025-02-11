// 功能: 面板-文字样式（颜色、字体、显示效果）
import store from "@/stores/indexStore";

export default function TextColor() {
    return (
        <div class="flex flex-row items-center gap-9">
            <h2 class="text-2xl font-normal">文字颜色</h2>
            <div class="flex flex-row items-center">
                <p>主歌词:</p>
                <div class="mx-4 w-6 h-6 relative">
                    <input
                        type="color"
                        class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10"
                        value={store.getState.textColor.first}
                        onInput={(e) =>
                            store.setTextColor(
                                "first",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => store.sendColorConfig()}
                    />
                    <div
                        class="color-mask absolute top-0 left-0 w-full h-full rounded-full z-0 pointer-events-none"
                        style={{
                            "background-color":
                            store.getState.textColor.first,
                        }}
                    ></div>
                </div>
                <p>副歌词:</p>
                <div class="mx-4 w-6 h-6 relative">
                    <input
                        type="color"
                        class="absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10"
                        value={store.getState.textColor.second}
                        onInput={(e) =>
                            store.setTextColor(
                                "second",
                                e.currentTarget.value
                            )
                        }
                        onChange={() => store.sendColorConfig()}
                    />
                    <div
                        class="color-mask absolute top-0 left-0 w-full h-full rounded-full z-0 pointer-events-none"
                        style={{
                            "background-color":
                            store.getState.textColor.second,
                        }}
                    ></div>
                </div>
            </div>
        </div>
    );
}

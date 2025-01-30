// 功能: 功能操作面板
import CurrentLyrics from "./ControlTools/CurrentLyrics";
import TextStyle from "./ControlTools/TextStyle";

export default function Controller() {
    return (
        <div class="h-[calc(100%-300px)] bg-[#141414] m-6 p-6 overflow-auto rounded-lg shadow-md scrollbar-hide flex md:flex-row flex-col gap-6 text-[#dcdcdc]">
            <CurrentLyrics />
            <div>
                <TextStyle />
                ...
            </div>
        </div>
    );
}

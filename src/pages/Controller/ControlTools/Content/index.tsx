import CurrentLyrics from "./CurrentLyrics";
import SearchResult from "./SearchResult";
import { createSignal, Show } from "solid-js";
import { LyricRawLine } from "@/types/lyricTypes.ts";
import { MusicQueryInfoData } from "@/types/lyricTypes.ts";

export default function Content() {
    const [lyrics, setLyrics] = createSignal<LyricRawLine[]>([]);
    const [musicInfo, setMusicInfo] = createSignal<MusicQueryInfoData>({});
    const [musicTitle, setMusicTitle] = createSignal<string>("");

    return (
        <div class="flex flex-col gap-4">
            <div class="header space-x-4">
                <h2 class="text-2xl font-medium inline">歌词内容控制</h2>
                <Show when={musicTitle().length > 0}>
                    <p class="text-sm inline text-gray-500">
                        当前歌曲: {musicTitle()}
                    </p>
                </Show>
            </div>

            <hr class="w-30 border-gray-400 dark:border-gray-600" />

            <div class="flex flex-col gap-2">
                <CurrentLyrics lyrics={lyrics} setLyrics={setLyrics} />
                <SearchResult
                    musicInfo={musicInfo}
                    setMusicInfo={setMusicInfo}
                    setMusicTitle={setMusicTitle}
                />
            </div>
        </div>
    );
}

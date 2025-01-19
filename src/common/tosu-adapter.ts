import ReconnectingWebSocket from "reconnecting-websocket";
import {WS_DELAY_TIME, WS_URL} from "./constant.ts";
import {getLyrics, Lyric} from "./music-api.ts";
import Cache from "./cache.ts";
import {TosuAPi} from "./tosu-types.ts";

export type LyricLine = {
    main: string,
    origin?: string,
}

type Temp = {
    title: string,
    songTime?: number,
    currentTimeId?: number,
    lyric?: Lyric,
}

export default class TosuAdapter {
    private readonly setLyrics: (value: LyricLine[]) => void;
    private readonly setCursor: (value: number) => void;
    private temp: Temp = {
        title: "",
    };
    private readonly ws: ReconnectingWebSocket;

    constructor(
        setLyrics: (value: LyricLine[]) => void,
        setCursor: (value: number) => void,
    ) {
        this.setLyrics = setLyrics;
        this.setCursor = setCursor;
        this.ws = new ReconnectingWebSocket(WS_URL);
        this.ws.onmessage = (event) => this.handleMessage(event);
    }

    private print(text: string = "") {
        this.temp.lyric = void 0;
        this.setLyrics([{main: text}]);
        this.setCursor(0);
    }

    private show(now: number) {
        if (!this.temp.lyric) {
            return;
        }
        const time = (now || 0) / 1000
        this.temp.lyric?.jump(time)
        this.setCursor(this.temp.lyric?.cursor || 0);
    }

    private showLyric(lyric: Lyric) {
        this.temp.lyric = lyric;
        const list = lyric.lyrics.map((x) => {
            return x.second ? {main: x.first, origin: x.second} : {main: x.first}
        });
        this.setLyrics(list);
        if (this.temp.songTime) {
            this.show(this.temp.songTime);
        } else {
            this.setCursor(lyric.cursor);
        }
    }

    private async updateLyric(title: string) {
        const lyric = await getLyrics(title);
        if (lyric.lyrics.length == 0) {
            this.print();
            return;
        }
        Cache.setLyricsCache(title, lyric);
        this.showLyric(lyric);
    }

    private async handleMessage(event: MessageEvent) {
        const data: TosuAPi = JSON.parse(event.data);
        const title = data.beatmap.titleUnicode;

        if (title == this.temp.title) {
            this.show(data.beatmap.time.live);
            return;
        } else {
            this.temp.title = title;
            this.print()
        }

        const lyric = await Cache.getLyricsCache(data.beatmap.titleUnicode);
        if (lyric && lyric.lyrics.length > 0) {
            this.temp.songTime = data.beatmap.time.live;
            this.showLyric(lyric);
            return;
        }

        if (this.temp.currentTimeId) {
            clearTimeout(this.temp.currentTimeId);
        }

        this.temp.currentTimeId = setTimeout(() => {
            this.temp.songTime = data.beatmap.time.live;
            this.updateLyric(title);
        }, WS_DELAY_TIME);
    }

    getNextTime () {
        return this.temp.lyric?.nextTime() || 0;
    }

    stop() {
        this.ws.close();
    }
}

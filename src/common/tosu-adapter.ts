import ReconnectingWebSocket from "reconnecting-websocket";
import {AUDIO_URL, WS_DELAY_TIME, WS_URL} from "./constant.ts";
import {Lyric} from "./music-api.ts";
import Cache from "./cache.ts";
import {TosuAPi} from "./tosu-types.ts";
import NeteastLyricAdaptor from "./neteast";
import QQLyricAdaptor from "./qq";

export type LyricLine = {
    main: string,
    origin?: string,
}

type Temp = {
    latestId: number,
    songTime?: number,
    currentTimeId?: number,
    lyric?: Lyric,
}

export async function getLyrics(title: string): Promise<Lyric> {
    const length = await getAudioLength();

    const [neteast, qq] = await Promise.all([NeteastLyricAdaptor.hasLyrics(title, length), QQLyricAdaptor.hasLyrics(title, length)]);

    if (neteast) {
        return await NeteastLyricAdaptor.getLyrics();
    } else if (qq) {
        return await QQLyricAdaptor.getLyrics();
    } else {
        throw new Error("No lyrics found");
    }
}

export async function getAudioLength(): Promise<number> {
    return new Promise((resolve) => {
        const audio = new Audio(AUDIO_URL);
        audio.preload = "metadata";
        audio.onloadedmetadata = () => {
            resolve(audio.duration * 1000);
        };
        audio.load();
    });
}

export default class TosuAdapter {
    private readonly setLyrics: (value: LyricLine[]) => void;
    private readonly setCursor: (value: number) => void;
    private temp: Temp = {
        latestId: 0,
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

    private async updateLyric(title: string, bid: number) {
        try {
            const lyric = await getLyrics(title);
            if (lyric.lyrics.length == 0) {
                this.print();
                return;
            }
            Cache.setLyricsCache(title, lyric);
            this.assertBid(bid);
            this.showLyric(lyric);
        } catch (e) {
            console.error("Failed to get lyrics:", e);
        }
    }

    private async handleMessage(event: MessageEvent) {
        const data: TosuAPi = JSON.parse(event.data);
        const bid = data.beatmap.id;

        if (this.temp.latestId == bid) {
            this.show(data.beatmap.time.live);
            return;
        }

        this.temp.latestId = bid;
        this.print()


        const title = data.beatmap.titleUnicode;

        const lyric = await Cache.getLyricsCache(title);
        if (lyric && lyric.lyrics.length > 0) {
            this.temp.songTime = data.beatmap.time.live;
            this.assertBid(bid);
            this.showLyric(lyric);
            return;
        }

        if (this.temp.currentTimeId) {
            clearTimeout(this.temp.currentTimeId);
        }

        this.temp.currentTimeId = setTimeout(() => {
            this.temp.songTime = data.beatmap.time.live;
            this.updateLyric(title, bid);
        }, WS_DELAY_TIME);
    }

    getNextTime() {
        return this.temp.lyric?.nextTime() || 0;
    }

    stop() {
        this.ws.close();
    }

    /**
     * 防止前面请求延迟, 覆盖了后面请求
     */
    private assertBid(bid: number) {
        if (this.temp.latestId != bid) {
            throw new Error("Invalid bid");
        }
    }
}

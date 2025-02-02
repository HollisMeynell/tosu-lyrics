import ReconnectingWebSocket from "reconnecting-websocket";
import { AUDIO_URL, WS_URL } from "@/config/constants";
import { Lyric } from "@/common/music-api.ts";
import Cache from "@/utils/cache.ts";
import { TosuAPi } from "@/types/tosu-types.ts";
import { QQLyricAdaptor, NeteaseLyricAdaptor } from "@/adapters";
import lyricsStore from "@/stores/lyricsStore.ts";

export type LyricLine = {
    main: string;
    origin?: string;
};

type Temp = {
    latestId: number;
    songTime?: number;
    currentTimeId?: ReturnType<typeof setTimeout>;
    lyric?: Lyric;
};


const WS_DELAY_TIME = 100;
const WAIT_AUDIO_METADATA = 1000;

/**
 * 获取音频长度, 毫秒, 3秒超时
 */
export async function getAudioLength(): Promise<number> {
    return await new Promise<number>((resolve) => {
        const audio = new Audio(AUDIO_URL + "?t=" + Date.now());
        audio.preload = "metadata";
        let timeoutFlag = false;
        // 超时
        const backup = setTimeout(() => {
            timeoutFlag = true;
            resolve(-1);
        }, WAIT_AUDIO_METADATA);
        const onLoad = () => {
            // 超时了就不再管了
            if (timeoutFlag) return;
            if (isNaN(audio.duration) || audio.duration == Infinity) {
                return;
            }
            clearTimeout(backup);
            resolve(audio.duration * 1000);
        };
        audio.onloadedmetadata = onLoad;
        audio.ondurationchange = onLoad;

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
        setCursor: (value: number) => void
    ) {
        this.setLyrics = setLyrics;
        this.setCursor = setCursor;
        this.ws = new ReconnectingWebSocket(WS_URL);
        this.ws.onmessage = (event) => this.handleMessage(event);
    }

    private print(text: string = "") {
        this.temp.lyric = void 0;
        this.setLyrics([{ main: text }]);
        this.setCursor(0);
    }

    private show(now: number) {
        if (!this.temp.lyric) return;

        const time = (now || 0) / 1000;
        this.temp.lyric?.jump(time);
        this.setCursor(this.temp.lyric?.cursor || 0);
    }

    private showLyric(lyric: Lyric) {
        this.temp.lyric = lyric;
        const list = lyric.lyrics.map((x) => {
            return x.second ? { main: x.first, origin: x.second } : { main: x.first };
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
            const lyric = await this.getLyrics(title, bid);
            if (lyric.lyrics.length == 0) {
                this.print();
                return;
            }
            lyricsStore.updateCurrentLyrics(lyric.lyrics);
            Cache.setLyricsCache(bid, title, lyric);
            this.assertBid(bid);
            this.showLyric(lyric);
        } catch (e) {
            console.error("Failed to get lyrics:", e);
        }
    }

    private async handleMessage(event: MessageEvent) {
        const data: TosuAPi = JSON.parse(event.data);
        const bid = data.beatmap.id;

        localStorage.setItem("nowPlaying", JSON.stringify(bid));

        if (this.temp.latestId == bid) {
            this.show(data.beatmap.time.live);
            return;
        }

        this.temp.latestId = bid;
        this.print();

        const title = data.beatmap.titleUnicode;

        const result = await Cache.getLyricsCache(bid);
        const lyric = new Lyric();
        if (result) {
            lyric.lyrics = result;
        }
        if (lyric && lyric.lyrics.length > 0) {
            this.temp.songTime = data.beatmap.time.live;
            this.assertBid(bid);
            this.showLyric(lyric);
            lyricsStore.updateCurrentLyrics(lyric.lyrics);
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
            throw new Error("请求过期, 无需处理");
        }
    }

    private async getLyrics(title: string, bid: number): Promise<Lyric> {
        const length = await getAudioLength();
        this.assertBid(bid);

        console.log(`title: ${title}, length: ${length}`);

        // 并行检查是否有歌词
        const [neteaseHasLyrics, qqHasLyrics] = await Promise.all([
            NeteaseLyricAdaptor.hasLyrics(title, length),
            QQLyricAdaptor.hasLyrics(title, length),
        ]);

        const newLyric = new Lyric();

        // 如果网易云适配器有歌词
        if (neteaseHasLyrics) {
            try {
                const lyric = await NeteaseLyricAdaptor.getLyricsFromResult();
                newLyric.insertAll(lyric.lyric, lyric.trans);
                return newLyric;
            } catch (err) {
                console.info(NeteaseLyricAdaptor.name, err);
            }
        }

        // 如果 QQ 音乐适配器有歌词
        if (qqHasLyrics) {
            try {
                const lyric = await QQLyricAdaptor.getLyricsFromResult();
                newLyric.insertAll(lyric.lyric, lyric.trans);
                return newLyric;
            } catch (err) {
                console.info(QQLyricAdaptor.name, err);
            }
        }

        // 如果都没有歌词，抛出错误
        throw new Error("No lyrics found");
    }
}

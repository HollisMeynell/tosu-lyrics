import ReconnectingWebSocket from "reconnecting-websocket";
import { AUDIO_URL, WS_URL } from "@/config/constants";
import { Lyric } from "@/common/music-api.ts";
import Cache from "@/utils/cache.ts";
import { TosuAPi } from "@/types/tosu-types.ts";
import { QQLyricAdapter, NeteaseLyricAdapter, LyricAdapter } from "@/adapters";
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
 * 获取音频长度(毫秒), 超时机制为 3 秒
 */
export async function getAudioLength(): Promise<number> {
    return await new Promise<number>((resolve) => {
        const audio = new Audio(AUDIO_URL + "?t=" + Date.now());
        audio.preload = "metadata";
        let timeoutFlag = false;

        // 超时
        const backupTimeout = setTimeout(() => {
            timeoutFlag = true;
            resolve(-1);
        }, WAIT_AUDIO_METADATA);

        const onLoadHandler = () => {
            // 超时了就不再管了
            if (timeoutFlag) return;
            if (isNaN(audio.duration) || audio.duration == Infinity) return;

            clearTimeout(backupTimeout);
            resolve(audio.duration * 1000);
        };

        audio.onloadedmetadata = onLoadHandler;
        audio.ondurationchange = onLoadHandler;
        audio.load();
    });
}

export default class TosuAdapter {
    private currentState: Temp = {
        latestId: 0,
        songTime: 0,
        currentTimeId: undefined as ReturnType<typeof setTimeout> | undefined,
        lyric: undefined as Lyric | undefined,
    };
    private readonly ws: ReconnectingWebSocket;

    constructor(
        private setLyrics: (value: LyricLine[]) => void,
        private setCursor: (value: number) => void
    ) {
        this.ws = new ReconnectingWebSocket(WS_URL);
        this.ws.onmessage = this.handleWebSocketMessage.bind(this);
    }

    /**
     * 清空 currentState 中的歌词, 设置指针为 0
     */
    private resetLyrics(text: string = "") {
        this.currentState.lyric = undefined;
        this.setLyrics([{ main: text }]);
        this.setCursor(0);
    }

    /**
     * 跳转到指定时间
     * @param now 当前时间
     */
    private updateCursor(now: number) {
        if (!this.currentState.lyric) return;

        const time = (now || 0) / 1000;
        this.currentState.lyric.jump(time);
        this.setCursor(this.currentState.lyric.cursor || 0);
    }

    /**
     * 将传入的歌词挂载到 currentState.lyric 上，并跳转到当前播放位置
     * @param lyric 歌词
     */
    private displayLyric(bid: number, lyric: Lyric) {
        if (this.currentState.latestId !== bid) return; // 防止请求过期
        this.currentState.lyric = lyric;
        const list = lyric.lyrics.map((x) => {
            return x.second
                ? { main: x.first, origin: x.second }
                : { main: x.first };
        });
        this.setLyrics(list);
        if (this.currentState.songTime) {
            this.updateCursor(this.currentState.songTime);
        } else {
            this.setCursor(lyric.cursor);
        }
    }

    /**
     * 直接从网络获取歌词, 并储存到本地缓存, 并显示
     * @param title 歌曲标题
     * @param bid 歌曲 id
     */
    private async processBeatmap(title: string, bid: number) {
        if (this.currentState.latestId !== bid) return; // 防止请求过期

        try {
            const length = await getAudioLength();
            const lyric = await this.fetchLyrics(title, length);

            if (this.currentState.latestId !== bid) return; // 再次防止请求过期

            if (lyric.lyrics.length === 0) {
                this.resetLyrics();
                return;
            }

            await Cache.setLyricsCache(bid, title, length, lyric);
            this.displayLyric(bid, lyric);
        } catch (e) {
            console.error("Failed to get lyrics:", e);
        }
    }

    /**
     * 处理 WebSocket 消息, 分为几种情况: 与上一首为相同歌曲, 不相同(分为有缓存和无缓存)
     * 与上一首相同: 更新播放位置即可
     * 有缓存: 直接读取缓存显示歌词
     * 无缓存: 通过定时器延迟获取歌词
     * @param event 消息事件
     */
    private async handleWebSocketMessage(event: MessageEvent) {
        const data: TosuAPi = JSON.parse(event.data);
        const { id: bid, titleUnicode: title } = data.beatmap;
        const { live: liveTime } = data.beatmap.time;

        localStorage.setItem("nowPlaying", JSON.stringify(bid));

        // 如果是同一首歌, 则更新播放位置
        if (this.currentState.latestId == bid) {
            this.updateCursor(liveTime);
            return;
        }

        // 如果是新歌, 则更新当前播放 bid 并重新获取歌词
        this.currentState.latestId = bid;
        this.resetLyrics();

        // 尝试从缓存中获取歌词
        const cachedLyrics = await Cache.getLyricsCache(bid);
        const lyric = new Lyric();
        if (cachedLyrics) {
            lyric.lyrics = cachedLyrics;
        }

        if (lyric && lyric.lyrics.length > 0) {
            this.currentState.songTime = liveTime;
            this.displayLyric(bid, lyric);
            lyricsStore.updateCurrentLyrics(lyric.lyrics);
            return;
        }

        if (this.currentState.currentTimeId) {
            clearTimeout(this.currentState.currentTimeId);
        }

        this.currentState.currentTimeId = setTimeout(() => {
            this.currentState.songTime = liveTime;
            this.processBeatmap(title, bid);
        }, WS_DELAY_TIME);
    }

    /**
     * 从网络获取歌词
     * @param title 歌曲标题
     * @param length 歌曲长度
     */
    private async fetchLyrics(title: string, length: number): Promise<Lyric> {
        const adapters: LyricAdapter[] = [NeteaseLyricAdapter, QQLyricAdapter];
        const newLyric = new Lyric();

        for (const adapter of adapters) {
            const hasLyrics = await adapter.hasLyrics(title, length);
            if (hasLyrics) {
                try {
                    const lyric = await adapter.getLyricsFromResult();
                    newLyric.insertAll(lyric.lyric, lyric.trans);
                    return newLyric;
                } catch (err) {
                    console.info(adapter.constructor.name, err);
                }
            }
        }

        if (newLyric.lyrics.length === 0) {
            throw new Error("No lyrics found");
        }

        return newLyric;
    }

    /**
     * 获取下一个时间
     */
    getNextTime() {
        return this.currentState.lyric?.nextTime() || 0;
    }

    /**
     * 停止 WebSocket
     */
    stop() {
        this.ws.close();
    }
}

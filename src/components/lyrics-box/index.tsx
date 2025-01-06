import React, {useEffect, useRef, useState} from "react";
import ReconnectingWebSocket from "reconnecting-websocket"
import {WS_URL} from "../../common/constant.ts";
import {TosuAPi} from "../../common/tosu-types.ts";
import {getLyrics, Lyric} from "../../common/music-api.ts";
import styles from './styles.module.scss';
import clsx from "clsx";

type Temp = {
    title: string,
    currentTimeId?: number,
    lyric?: Lyric,
}

class Cache {
    set(title: string, lyric: Lyric) {
        if (lyric.lyrics.length == 0) return;
        const dataString = JSON.stringify(lyric.lyrics)
        localStorage.setItem(title, dataString);
    }

    get(title: string): Lyric | undefined {
        const dataString = localStorage.getItem(title);
        if (!dataString) return undefined;
        const data = JSON.parse(dataString);
        const lyric = new Lyric();
        lyric.lyrics = data;
        return lyric;
    }
}

const cache = new Cache();
const temp: Temp = {
    title: "",
};

type LyricLine = {
    main: string,
    translate?: string,
}

function handleMessage(
    setLyrics: React.Dispatch<React.SetStateAction<LyricLine[]>>,
    setCursor: React.Dispatch<React.SetStateAction<number>>,
) {
    const print = (text: string = "") => {
        temp.lyric = void 0;
        setLyrics([{main: text}]);
        setCursor(0);
    }
    const show = (now: number) => {
        if (!temp.lyric) {
            return;
        }
        const time = (now || 0) / 1000
        temp.lyric?.jump(time)
        console.log(time, temp.lyric?.cursor, temp.lyric)
        setCursor(temp.lyric?.cursor || 0);
    }
    return (event: MessageEvent) => {
        const data: TosuAPi = JSON.parse(event.data);
        const title = data.beatmap.title;
        if (title == temp.title) {
            show(data.beatmap.time.live);
            return;
        } else {
            temp.title = title;
            print()
        }
        const showLyric = (lyric: Lyric) => {
            temp.lyric = lyric;
            const list = lyric.lyrics.map((x) => {
                return x.second ? {main: x.first, translate: x.second} : {main: x.first}
            });
            setLyrics(list);
            setCursor(lyric.cursor);
        }

        const lyric = cache.get(data.beatmap.titleUnicode);
        if (lyric && lyric.lyrics.length > 0) {
            showLyric(lyric);
            return;
        }

        const action = async () => {
            const lyric = await getLyrics(data.beatmap.titleUnicode);
            if (lyric.lyrics.length == 0) {
                print();
                return;
            }
            cache.set(data.beatmap.titleUnicode, lyric);
            temp.lyric = lyric;
            showLyric(lyric);
        }

        if (temp.currentTimeId) {
            clearTimeout(temp.currentTimeId);
        }
        temp.currentTimeId = setTimeout(() => {
            action().then(() => {
                show(data.beatmap.time.live)
            });
        }, 50);
    }
}

export default function LyricsBox(): React.JSX.Element {
    const ws = useRef<ReconnectingWebSocket | null>(null);
    const lyricUL = useRef<HTMLUListElement>(null);
    const [scroll, setScroll] = useState(false);
    const [lyrics, setLyrics] = React.useState<LyricLine[]>([]);
    const [cursor, setCursor] = useState<number>(0);

    useEffect(() => {
        ws.current = new ReconnectingWebSocket(WS_URL);
        ws.current.onmessage = handleMessage(setLyrics, setCursor);
        return () => {
            ws.current?.close();
        }
    }, []);

    useEffect(() => {
        if (!lyricUL.current) return;
        const p: HTMLLIElement = lyricUL.current.children[cursor] as HTMLLIElement;
        if (!p) return;
        let maxWidth: number;
        if (p.children.length == 2) {
            maxWidth = Math.max(p.children[0].scrollWidth, p.children[1].scrollWidth) * 1.2;
        } else {
            maxWidth = p.children[0].scrollWidth * 1.2;
        }
        if (maxWidth > 1200) {
            const offset = Math.round((maxWidth - 1200) / 2) + 10;
            p.style.setProperty('--offset', `${offset}px`);
            p.style.setProperty('--offset-f', `-${offset}px`);
            p.style.setProperty('--time', `${temp.lyric?.nextTime() || 0}s`);
            setScroll(true);
        } else {
            setScroll(false);
        }
    }, [lyrics, cursor]);

    return <>
        <div className={styles.box}>
            <ul className={styles.lyricBox}
                style={{transform: `translateY(${-(cursor - 1) * 100}px)`}}
                ref={lyricUL}>
                {lyrics.map((lyric, index) =>
                    <li key={index}
                        className={clsx(
                            styles.lyric,
                            cursor == index && styles.lyricNow,
                            cursor == index && scroll && styles.lyricScrolling,
                        )}>
                        <p>
                            {lyric.main}
                        </p>
                        {lyric.translate && <p>{lyric.translate}</p>}
                    </li>
                )}
            </ul>
        </div>
    </>
}

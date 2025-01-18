import React, {useEffect, useRef, useState} from "react";
import styles from './styles.module.scss';
import clsx from "clsx";
import TosuAdapter from "../../common/tosu-adapter.ts";


type LyricLine = {
    main: string,
    origin?: string,
}


export default function LyricsBox(): React.JSX.Element {
    const tosu = useRef<TosuAdapter>(null);
    const lyricUL = useRef<HTMLUListElement>(null);
    const [scroll, setScroll] = useState(false);
    const [lyrics, setLyrics] = React.useState<LyricLine[]>([]);
    const [cursor, setCursor] = useState<number>(0);

    useEffect(() => {
        tosu.current = new TosuAdapter(setLyrics, setCursor);
        return () => {
            tosu.current.stop()
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
            p.style.setProperty('--time', `${tosu.current.getNextTime()}s`);
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
                        {lyric.origin && <p>{lyric.origin}</p>}
                    </li>
                )}
            </ul>
        </div>
    </>
}

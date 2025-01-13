import React, {useEffect, useState} from "react";
import styles from './styles.module.scss';

type CenterBoxProps = {
    children?: React.ReactNode
}

declare global {
    interface Window {
        ignoreCORS?: boolean
    }
}

export default function CenterBox({children}: CenterBoxProps): React.JSX.Element {
    const [consoleShow, setConsoleShow] = useState(false);
    useEffect(() => {
        const url = new URL(window.location.href);
        const params = new URLSearchParams(url.search);
        if (params.get('cors')) {
            window.ignoreCORS = true;
        }
        if (params.get('console')) {
            setConsoleShow(true)
        }
    }, []);
    return <>
        <div className={styles.box}>
            {children}
            {consoleShow && <div>...</div>}
        </div>
    </>
}

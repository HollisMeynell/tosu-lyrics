import React from "react";
import styles from './styles.module.scss';

type CenterBoxProps = {
    children?: React.ReactNode
}

export default function CenterBox({children}: CenterBoxProps): React.JSX.Element {
    return <>
        <div className={styles.box}>
            {children}
        </div>
    </>
}

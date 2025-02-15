import { Component, JSX } from "solid-js";

interface MaskProps {
    class?: string;
    children?: JSX.Element;
}

const Mask: Component<MaskProps> = (props) => {
    return (
        <div
            class={`absolute inset-0 z-100 flex items-center justify-center bg-black/50 dark:bg-black/90 ${props.class}`}
        >
            {props.children}
        </div>
    );
};

export default Mask;

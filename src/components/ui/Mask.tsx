import { Component, JSX } from "solid-js";

const Mask: Component<{ children?: JSX.Element }> = (props) => {
    return (
        <div
            class="absolute inset-0 z-100 flex items-center justify-center
            before:absolute before:z-[-1] before:inset-0 before:rounded-lg
            before:opacity-40 before:bg-black dark:before:bg-white"
            {...props}
        />
    );
};

export default Mask;

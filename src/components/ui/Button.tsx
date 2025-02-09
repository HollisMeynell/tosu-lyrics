export default function Button(props: {
    onClick: () => void;
    className?: string;
    children: string;
    disabled?: boolean;
}) {
    return (
        <button
            class={[props.className, `bg-gray-100 dark:bg-[#21314d]
                    hover:bg-[#f0f0f0] dark:hover:bg-[#3b4a63]
                    border border-gray-300 dark:border-gray-600
                    text-sm text-gray-900 dark:text-gray-200
                    min-w-18 mr-2 p-2.5 rounded-md cursor-pointer font-bold shadow-sm  transition-colors duration-300
                    disabled:opacity-50 disabled:cursor-not-allowed`].join(" ")}
            onClick={props.onClick}
            disabled={props.disabled}
        >
            {props.children}
        </button>
    );
}

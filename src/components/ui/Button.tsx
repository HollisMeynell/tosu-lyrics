export default function Button(props: {
    onClick: () => void;
    className?: string;
    children: string;
    disabled?: boolean;
}) {
    const { onClick, className, children, disabled } = props;
    return (
        <button
            class={[className, `bg-gray-100 dark:bg-[#21314d]
                    hover:bg-[#f0f0f0] dark:hover:bg-[#3b4a63]
                    border border-gray-300 dark:border-gray-600
                    text-sm text-gray-900 dark:text-gray-200
                    min-w-18 mr-2 p-2.5 rounded-md cursor-pointer font-bold shadow-sm  transition-colors duration-300`].join(" ")}
            onClick={onClick}
            disabled={disabled}
        >
            {children}
        </button>
    );
}

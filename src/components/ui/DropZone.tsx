// 功能: 通用拖拽上传卡片（整块可点击选择文件，也可直接拖入）
import { createSignal } from "solid-js";

interface DropZoneProps {
    hint: string;
    accept: string;
    disabled?: boolean;
    onFile: (file: File) => void;
}

export default function DropZone(props: DropZoneProps) {
    const [dragOver, setDragOver] = createSignal(false);

    const pickHandler = (e: Event) => {
        const input = e.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = "";
        if (file) props.onFile(file);
    };

    const dropHandler = (e: DragEvent) => {
        e.preventDefault();
        setDragOver(false);
        const file = e.dataTransfer?.files?.[0];
        if (file) props.onFile(file);
    };

    return (
        <label
            class={`mx-auto flex w-full max-w-xs min-h-44 cursor-pointer flex-col items-center justify-center gap-3 rounded-md border-2 border-dashed p-6 text-center transition-colors ${
                dragOver()
                    ? "border-[#ec4899] bg-pink-50 dark:bg-pink-900/20"
                    : "border-gray-300 dark:border-gray-600"
            }`}
            onDragOver={(e) => {
                e.preventDefault();
                setDragOver(true);
            }}
            onDragLeave={() => setDragOver(false)}
            onDrop={dropHandler}
        >
            <svg
                class="size-8 text-gray-400"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                aria-hidden="true"
            >
                <path
                    d="M12 15V4m0 0 4 4m-4-4-4 4"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                <path
                    d="M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"
                    stroke-linecap="round"
                />
            </svg>
            <span class="text-sm text-gray-500">{props.hint}</span>
            <span class="text-sm text-[#ec4899]">或点击选择文件</span>
            <input
                type="file"
                class="sr-only"
                accept={props.accept}
                disabled={props.disabled}
                onChange={pickHandler}
            />
        </label>
    );
}

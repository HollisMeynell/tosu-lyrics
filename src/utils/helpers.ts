export function debounce<T extends (...args: unknown[]) => void>(
    func: T,
    wait: number
): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout> | null = null;

    return function (this: unknown, ...args: Parameters<T>): void {
        if (timeout !== null) {
            clearTimeout(timeout);
        }
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
}

/**
 * 将毫秒数转换为 mm:ss 格式
 * @param time 毫秒数
 */
export function ms2str(time: number): string {
    const allSeconds = Math.round(time / 1000);
    const sec = String(allSeconds % 60).padStart(2, "0");
    const min = String(Math.floor(allSeconds / 60)).padStart(2, "0");
    return `${min}:${sec}`;
}

// prettier-ignore
const RANDOM_STRING_MAP = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const RANDOM_STRING_MAP_LENGTH = RANDOM_STRING_MAP.length;

/**
 * 生成指定长度的随机字符串
 * @param length 字符串长度
 */
export function generateRandomString(length: number): string {
    let result = new Array(length);
    for (let i = 0; i < length; i++) {
        result[i] =
            RANDOM_STRING_MAP[
                Math.floor(Math.random() * RANDOM_STRING_MAP_LENGTH)
            ];
    }
    return result.join("");
}

/**
 * 防抖函数
 * @param func 需要防抖的函数
 * @param wait 防抖时间
 */
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

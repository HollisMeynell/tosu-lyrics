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

// 功能: 解析 URL 中清除缓存参数

// 拿到 URL 中的参数
export function parseUrlParams(url: string): URLSearchParams {
    const urlObj = new URL(url);
    return new URLSearchParams(urlObj.search);
}

// 解析这些参数
export function paramParse() {
    const searchParams = parseUrlParams(window.location.href);
    return Object.fromEntries(searchParams.entries());
}

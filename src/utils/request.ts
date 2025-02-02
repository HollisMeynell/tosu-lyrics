// 功能: 封装请求方法
import { PROXY_URL } from "@/config/constants";

export type RequestResult = {
    status: number,
    headers: Map<string, string>,
    body: string,
}

export type RequestProp = {
    url: string;
    method?: string;
    header?: object;
    body?: object;
}



export async function doRequest(prop: RequestProp): Promise<RequestResult> {
    // 使用 obs 的 '--disable-web-security' 参数禁用浏览器的 CORS 限制
    // if obs is used, disable the CORS restriction of the browser with '--disable-web-security'
    if (window.ignoreCORS) {
        const result = await fetch(prop.url, {
            method: prop.method || "GET",
            headers: {
                ...prop.header
            },
            body: JSON.stringify(prop.body)
        })
        const body = await result.text();

        const headers = new Map<string, string>();
        result.headers.forEach((value, key) => {
            headers.set(key, value);
        });
        return {
            status: result.status,
            headers,
            body,
        };
    }
    const result = await fetch(PROXY_URL, {
        method: "POST",
        headers: {
            "Accept": "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify(prop)
    })
    return result.json()
}

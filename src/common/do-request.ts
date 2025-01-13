import {PROXY_URL} from "./constant.ts";

type RequestResult = {
    status: number,
    headers: Map<string, string>,
    body: string,
}

type RequestProp = {
    url: string;
    method?: string;
    header?: object;
    body?: object;
}

export async function doRequest(prop: RequestProp): Promise<RequestResult> {
    // 使用 obs 的 '--disable-web-security' 参数禁用浏览器的 CORS 限制
    if (window.ignoreCORS) {
        const result = await fetch(prop.url, {
            method: prop.method || "GET",
            headers: {
                ...prop.header
            },
            body: JSON.stringify(prop.body)
        })
        return await result.json()
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

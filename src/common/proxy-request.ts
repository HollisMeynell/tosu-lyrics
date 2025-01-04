import {PROXY_URL} from "./constant.ts";

type ProxyResult = {
    status: number,
    headers: Map<string, string>,
    body: string,
}

type ProxyProp = {
    url: string;
    method?: string;
    header?: object;
    body?: object;
}

export async function proxyRequest(prop: ProxyProp): Promise<ProxyResult> {
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

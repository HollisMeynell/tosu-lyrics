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

// 功能: 封装请求方法
import { PROXY_URL } from "@/config/constants";

export type RequestResult = {
    status: number;
    headers: Map<string, string>;
    body: string;
};

export type RequestProp = {
    url: string;
    method?: string;
    header?: object;
    body?: object;
};

export async function customFetch(prop: RequestProp): Promise<RequestResult> {
    const result = await fetch(PROXY_URL, {
        method: "POST",
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
        },
        body: JSON.stringify(prop),
    });
    return result.json();
}

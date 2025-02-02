/**
 function testWorker(): boolean {
 return !!window.Worker;
 }

 const code = `
 const worker = self as SharedWorkerGlobalScope;
 import ReconnectingWebSocket from "reconnecting-websocket";

 `;
 const codeBlob = new Blob([code], { type: "application/javascript" });

 const sharedWorker = new SharedWorker(URL.createObjectURL(codeBlob));

 sharedWorker.port.start();
 sharedWorker.port.onmessage = (e) => {
 const text: string = e.data;
 };
 */


import { Settings } from "@/common/config-global.ts";
import lyricsStore from "@/stores/lyricsStore.ts";
import ReconnectingWebSocket from "reconnecting-websocket";

// 最好跟 request.ts 统一配置, 而且最好所有的配置类常量都在一个文件中, 散落在各处找起来麻烦而且容易漏
const BACKEND_CONFIG_URL = "http://127.0.0.1:41280/api/config";
const BACKEND_WEBSOCKET_URL = "http://127.0.0.1:41280/api/ws";

const ALL_ONLINE_CLIENTS: string[] = [];
let selfId: string = "";

const websocket = (
    () => {
        const ws = new ReconnectingWebSocket(BACKEND_WEBSOCKET_URL);
        ws.onmessage = (event) => {
            try {
                if (typeof event.data !== "string") {
                    handleWebsocketMessage(event.data);
                } else {
                    const data = JSON.parse(event.data) as WebsocketMessage;
                    handleWebsocketMessage(data);

                }
            } catch (e) {
                console.trace(e);
            }
        };
        return ws;
    }
)();

// 同步配置, 最好是页面加载后执行一次用来统一配置
export async function asyncConfig() {
    const response = await fetch(BACKEND_CONFIG_URL, { method: "GET" });
    const config = await response.json() as Settings;
    lyricsStore.parseSettings(config);
}

export async function saveConfig(config: Settings) {
    try {
        const response = await fetch(BACKEND_CONFIG_URL, {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(config), // 将配置对象转换为 JSON 字符串
        });

        if (!response.ok) {
            console.error(`HTTP error! status: ${response.status}`);
            return void 0;
        }

        const result = await response.json();
        console.log("Config saved successfully:", result);
    } catch (error) {
        console.error("Failed to save config:", error);
    }
}

interface CommandMessage {
    type: string,
}

// 设置方面的 ws 消息
interface SettingMessage extends CommandMessage {
    type: "setting",
    key: string,
    value: unknown,
}

interface OnlineMessage extends CommandMessage{
    type: "online",
    id: string,
    status: "online" | "offline" | "conflict"
    others: string[] | null,
}

export interface WebsocketMessage {
    command: CommandMessage,
    echo: string | null;
}

export interface SettingHandle {
    key: string,
    handle: (v: unknown) => void,
}

const SETTER_HANDLES: Map<string, SettingHandle> = new Map();


export function registerSetterHandle({ key, handle }: SettingHandle) {
    SETTER_HANDLES.set(key, { key, handle });
}

function handleWebsocketMessage(e: WebsocketMessage) {
    let data = e.command;
    switch (data["type"]) {
        // 样式设定
        case "setting": {
            const setData = data as SettingMessage;
            const setter = SETTER_HANDLES.get(setData.key);
            if (setter) {
                setter.handle(setData.value);
            }
            break;
        }

        // 其他端上线
        case "online" : {
            const onlineData = data as OnlineMessage;
            if (onlineData.others != null) {
                selfId = onlineData.id;
                ALL_ONLINE_CLIENTS.push(...onlineData.others);
                console.log(`Self id: ${selfId}`);
                break;
            }
            switch (onlineData.status) {
                case "online": {
                    ALL_ONLINE_CLIENTS.push(onlineData.id);
                    break;
                }
                case "offline": {
                    const index = ALL_ONLINE_CLIENTS.indexOf(onlineData.id);
                    if (index !== -1) {
                        ALL_ONLINE_CLIENTS.splice(index, 1);
                    }
                    break;
                }
            }
            break;
        }
        // 后续打算加上 查询搜索歌曲的结果, 收到后 echo 以 id 为前缀, 则返回查询数据
        // ...
        default: {
            console.error("Unknown type:", data["type"]);
        }
    }
}

/**
 * 推送配置, key 与监听的 key 保持一致
 */
export function pushSettingMessage(key: string, value: unknown) {
    const message: SettingMessage = {
        type: "setting",
        key,
        value,
    };
    websocket.send(JSON.stringify(message));
}

export function getAllOnlineClients(): string[] {
    return ALL_ONLINE_CLIENTS;
}

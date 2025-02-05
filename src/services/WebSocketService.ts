import ReconnectingWebSocket from "reconnecting-websocket";
import {
    WebSocketMessage,
    SettingMessage,
    OnlineMessage,
    QueryResponseMessage,
    QueryRequestMessage,
} from "@/types/ws-types";
import { BACKEND_WEBSOCKET_URL, WS_QUERY_TIMEOUT } from "@/config/constants";
import { generateRandomString } from "@/utils/utils.ts";

export type MessageHandler = (value: unknown) => void;

export class WebSocketService {
    private ws: ReconnectingWebSocket;
    private settingHandlers = new Map<string, MessageHandler>();
    // 接受查询 并响应查询结果
    private queryHandlers = new Map<string, (params?: unknown) => unknown>();
    private onlineClients: string[] = [];
    // 发起查询方 请求回调队列 (应该是队列, 但是用 key 做索引所以用 map 了)
    private queryQueue: Map<string, MessageHandler> = new Map();
    private selfId: string = "";

    constructor() {
        this.ws = new ReconnectingWebSocket(BACKEND_WEBSOCKET_URL);
        this.setupWebSocket();
    }

    private setupWebSocket() {
        this.ws.onmessage = (event) => {
            try {
                const data =
                    typeof event.data === "string"
                        ? JSON.parse(event.data)
                        : event.data;
                this.handleMessage(data);
            } catch (error) {
                console.error("WebSocket message error:", error);
            }
        };
    }

    private handleMessage(message: WebSocketMessage) {
        const { echo, command } = message;

        switch (command.type) {
            case "setting": {
                // 设定
                const { key, value, target } = command as SettingMessage;
                // 忽略有指向 target 且并非自身的消息
                if (target && !this.isSelf(target)) {
                    break;
                }
                const handler = this.settingHandlers.get(key);
                if (handler) {
                    handler(value);
                }
                break;
            }
            case "online": {
                // 其他端上线
                this.handleOnlineStatus(command as OnlineMessage);
                break;
            }
            case "query-request": {
                // 收到查询请求
                if (!this.isSelf(echo)) return;
                const { key, query, params } = command as QueryRequestMessage;
                const handler = this.queryHandlers.get(query);
                if (!handler) return;
                const data = handler(params);
                this.queryResponse(key, data);
                break;
            }
            case "query-response": {
                // 受到查询响应
                const { key, value } = command as QueryResponseMessage;
                const handler = this.queryQueue.get(key);
                if (handler) {
                    handler(value);
                }
                break;
            }
            // ...
            default:
                console.warn("Unknown message type:", command.type);
        }
    }

    private handleOnlineStatus(data: OnlineMessage) {
        if (data.others != null) {
            this.selfId = data.id;
            this.onlineClients = data.others;
            console.log(`Connected with ID: ${this.selfId}`);
            return;
        }

        if (data.status === "online") {
            this.onlineClients.push(data.id);
        } else if (data.status === "offline") {
            this.onlineClients = this.onlineClients.filter(
                (id) => id !== data.id,
            );
        }
    }

    /**
     * 注册处理设置类的回调
     */
    registerHandler(key: string, handler: MessageHandler) {
        this.settingHandlers.set(key, handler);
    }

    /**
     * 注册处理查询类的回调
     */
    registerQueryHandler(key: string, handler: (params?: unknown) => unknown) {
        this.queryHandlers.set(key, handler);
    }

    pushSetting(key: string, value?: unknown, target?: string) {
        const message: WebSocketMessage = {
            command: {
                type: "setting",
                key,
                target,
                value,
            } as SettingMessage,
            echo: null,
        };
        this.ws.send(JSON.stringify(message));
    }

    private isSelf(key: string | undefined | null) {
        return key?.startsWith(this.selfId);
    }

    blinkOtherClient(id: string) {
        this.pushSetting("blink-lyric", void 0, id);
    }

    /**
     * 获取在线客户端列表
     */
    getOnlineClients(): string[] {
        return this.onlineClients.filter((id) => this.selfId != id);
    }

    /**
     * 通过 ws 发起查询
     * @param clientId 对方客户端的 id
     * @param query 需要的查询, 与注册回调 key 保持一致
     * @param params 参数, 可为空
     */
    async postQuery<T>(
        clientId: string,
        query: string,
        params?: unknown,
    ): Promise<T> {
        const key = generateRandomString(8);
        const message: WebSocketMessage = {
            command: {
                type: "query-request",
                key,
                params,
                query: query,
            } as QueryRequestMessage,
            echo: `${clientId}-${Date.now()}`,
        };
        this.ws.send(JSON.stringify(message));
        return new Promise((resolve, reject) => {
            const outTime = setTimeout(() => {
                this.queryQueue.delete(key);
                reject({ error: "time out" });
            }, WS_QUERY_TIMEOUT);
            const handler = (data: unknown) => {
                clearTimeout(outTime);
                resolve(data as T);
            };
            this.queryQueue.set(key, handler);
        });
    }

    /**
     * 发送响应
     */
    private queryResponse(key: string, value: unknown) {
        const message: WebSocketMessage = {
            command: {
                type: "query-response",
                key,
                value,
            } as QueryResponseMessage,
            echo: null,
        };
        this.ws.send(JSON.stringify(message));
    }
}

export const wsService = new WebSocketService();

/**
 * 封装对其他客户端的操作
 */
export class OtherClient {
    id: string;

    constructor(id: string) {
        this.id = id;
    }

    async queryCacheList() {
        return wsService.postQuery<{ bid: number, title: string }[]>(this.id, "query-cache-list");
    }

    async removeCacheItem(bid: number) {
        return wsService.pushSetting("remove-cache-item", { bid }, this.id);
    }

    async removeAllCache() {
        return wsService.pushSetting("remove-all-cache", void 0, this.id);
    }
}

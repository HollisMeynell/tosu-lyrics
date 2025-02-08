import ReconnectingWebSocket from "reconnecting-websocket";
import {
    WebSocketMessage,
    SettingMessage,
    OnlineMessage,
    QueryResponseMessage,
    QueryRequestMessage,
} from "@/types/ws-types";
import { BACKEND_WEBSOCKET_URL, WS_QUERY_TIMEOUT } from "@/config/constants";
import { generateRandomString } from "@/utils/helpers";

export type MessageHandler = (value: unknown) => void;
export type QueryHandler = (params?: unknown) => Promise<unknown>;
export type QueryResultHandler = (value: unknown, error?: string) => void;

export class WebSocketService {
    private ws: ReconnectingWebSocket;
    private settingHandlers = new Map<string, MessageHandler>();
    // 接受查询 并响应查询结果
    private queryHandlers = new Map<string, QueryHandler>();
    // 发起查询方 请求回调队列 (应该是队列, 但是用 key 做索引所以用 map 了)
    private queryQueue = new Map<string, QueryResultHandler>();
    private onlineClients: string[] = [];
    private selfId: string = "";
    defaultClient: OtherClient | undefined;

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
            case "setting":
                this.handleSettingMessage({
                    ...command,
                    echo,
                } as SettingMessage & { echo: string | null });
                break;
            case "online":
                this.handleOnlineStatus(command as OnlineMessage);
                break;
            case "query-request":
                this.handleQueryRequest({
                    ...command,
                    echo,
                } as QueryRequestMessage & { echo: string | null }).then();
                break;
            case "query-response":
                this.handleQueryResponse(command as QueryResponseMessage);
                break;
            default:
                console.warn("Unknown message type:", command.type);
        }
    }

    private handleSettingMessage = (
        message: SettingMessage & { echo: string | null },
    ) => {
        const { key, value, target } = message;
        if (target && !this.isSelf(target)) return;

        const handler = this.settingHandlers.get(key);
        handler?.(value);
    };

    private handleOnlineStatus(message: OnlineMessage) {
        if (message.others != null) {
            this.selfId = message.id;
            this.onlineClients = message.others;
            console.log(`Connected with ID: ${this.selfId}`);
            return;
        }

        if (message.status === "online") {
            this.onlineClients.push(message.id);
        } else if (message.status === "offline") {
            this.onlineClients = this.onlineClients.filter(
                (id) => id !== message.id,
            );
        }
    }

    private handleQueryRequest = async (
        message: QueryRequestMessage & { echo: string | null },
    ) => {
        console.log("Query request:", message);
        if (!this.isSelf(message.echo)) return;

        const handler = this.queryHandlers.get(message.query);
        if (!handler) {
            this.sendQueryResponse(message.key, void 0, "No handler found");
            return;
        }

        try {
            const response = await handler(message.params);
            this.sendQueryResponse(message.key, response);
        } catch (e) {
            this.sendQueryResponse(message.key, void 0, (e as Error).message);
        }
    };

    /**
     * 收到响应, 并将结果返回到调用处
     * @param message
     */
    private handleQueryResponse = (message: QueryResponseMessage) => {
        if (!this.isSelf(message.key)) return;
        const handler = this.queryQueue.get(message.key);
        if (typeof handler != "function") {
            return;
        }
        this.queryQueue.delete(message.key);
        if (message.error != null) {
            handler(void 0, message.error);
        } else {
            handler(message.value);
        }
    };

    /**
     * 发送响应
     */
    private sendQueryResponse(
        key: string,
        value: unknown,
        error?: string,
    ): void {
        this.sendMessage({
            command: {
                type: "query-response",
                key,
                value,
                error,
            } as QueryResponseMessage,
            echo: null,
        });
    }

    private sendMessage(message: WebSocketMessage) {
        if (this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn("WebSocket is not open. Message not sent:", message);
        }
    }

    private isSelf(key: string | undefined | null): boolean {
        return Boolean(key?.startsWith(this.selfId));
    }

    // Public API

    /**
     * 注册处理设置类的回调
     * 即响应单向消息
     */
    public registerHandler(key: string, handler: MessageHandler) {
        this.settingHandlers.set(key, handler);
    }

    /**
     * 注册处理查询类的回调
     * 即响应双向消息
     */
    public registerQueryHandler(key: string, handler: QueryHandler) {
        this.queryHandlers.set(key, handler);
    }

    /**
     * 推送设置 (单向消息)
     * @param key 设置 key,
     * @param value 值, 可为空
     * @param target 目标客户端 id, 为空时应用于自己以外的所有客户端
     */
    public pushSetting(key: string, value?: unknown, target?: string) {
        this.sendMessage({
            command: {
                type: "setting",
                key,
                value,
                target,
            } as SettingMessage,
            echo: null,
        });
    }

    /**
     * 闪烁其他客户端
     * @param id 客户端 id
     */
    public blinkOtherClient(id: string) {
        this.pushSetting("blink-lyric", void 0, id);
    }

    /**
     * 获取在线客户端列表
     */
    public getOnlineClients(): string[] {
        return this.onlineClients.filter((id) => id !== this.selfId);
    }

    /**
     * 通过 ws 发起查询 (双向消息)
     * @param clientId 对方客户端的 id
     * @param query 需要的查询, 与注册回调 key 保持一致
     * @param params 参数, 可为空
     */
    public async postQuery<T>(
        clientId: string,
        query: string,
        params?: unknown,
    ): Promise<T> {
        const key = generateRandomString(8);

        this.sendMessage({
            command: {
                type: "query-request",
                key,
                query,
                params,
            } as QueryRequestMessage,
            echo: `${clientId}-${Date.now()}`,
        });

        return new Promise((resolve, reject) => {
            const outTime = setTimeout(() => {
                this.queryQueue.delete(key);
                reject(new Error("Query timeout"));
            }, WS_QUERY_TIMEOUT);

            this.queryQueue.set(key, (data, err?: string) => {
                clearTimeout(outTime);
                if (err) {
                    reject(new Error(err));
                }
                resolve(data as T);
            });
        });
    }

    setDefaultClient(key: string) {
        if (!key) return;
        this.defaultClient = new OtherClient(key, this);
    }
}

// Singleton instance 暴露
export const wsService = new WebSocketService();

/**
 * 封装对其他客户端的操作
 */
export class OtherClient {
    private serveice: WebSocketService;

    constructor(
        private id: string,
        wsService: WebSocketService,
    ) {
        this.serveice = wsService;
    }

    public async queryCacheList(): Promise<
        Array<{ bid: number; title: string }>
    > {
        return this.serveice.postQuery(this.id, "query-cache-list");
    }

    public async removeCacheItem(bid: number): Promise<void> {
        this.serveice.pushSetting("remove-cache-item", { bid }, this.id);
    }

    public async removeAllCache(): Promise<void> {
        this.serveice.pushSetting("remove-all-cache", undefined, this.id);
    }
}

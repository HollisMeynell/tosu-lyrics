import ReconnectingWebSocket from "reconnecting-websocket";
import {
    WebSocketMessage,
    SettingMessage,
    OnlineMessage,
} from "@/types/ws-types";
import { BACKEND_WEBSOCKET_URL } from "@/config/constants";

export type MessageHandler = (value: unknown) => void;

export class WebSocketService {
    private ws: ReconnectingWebSocket;
    private handlers = new Map<string, MessageHandler>();
    private onlineClients: string[] = [];
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
        const { command } = message;

        switch (command.type) {
            case "setting": { // 样式设定
                const { key, value } = command as SettingMessage;
                const handler = this.handlers.get(key);
                if (handler) {
                    handler(value);
                }
                break;
            }
            case "online": { // 其他端上线
                this.handleOnlineStatus(command as OnlineMessage);
                break;
            }
            // 后续打算加上 查询搜索歌曲的结果, 收到后 echo 以 id 为前缀, 则返回查询数据
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
                (id) => id !== data.id
            );
        }
    }

    registerHandler(key: string, handler: MessageHandler) {
        this.handlers.set(key, handler);
    }

    pushSetting(key: string, value: unknown) {
        const message: WebSocketMessage = {
            command: {
                type: "setting",
                key,
                value,
            },
            echo: null,
        };
        this.ws.send(JSON.stringify(message));
    }

    getOnlineClients() {
        return [...this.onlineClients];
    }
}

export const wsService = new WebSocketService();

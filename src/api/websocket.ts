import ReconnectingWebSocket from "reconnecting-websocket";
import {
    isLyric,
    isSetting,
    isWebsocketMessage,
    WebsocketLyric,
    WebsocketSetting,
    WebsocketSettingTypeMap,
} from "@/api/model.ts";
import { BACKEND_WEBSOCKET_URL } from "@/config/constants.ts";
import { generateRandomString } from "@/utils/helpers";

export type SettingHandler = (data: WebsocketSetting) => void;
export type LyricHandler = (data: WebsocketLyric) => void;

export class Websocket {
    private ws: ReconnectingWebSocket;
    private actions = new Map<string, SettingHandler>();

    /**
     * 初始化 WebSocket 对象
     * @param isClient 设置端要传 false, 此时不再收到歌词下发
     */
    constructor(isClient: boolean = true) {
        const url = isClient
            ? BACKEND_WEBSOCKET_URL
            : BACKEND_WEBSOCKET_URL + "?setter=true";
        this.ws = new ReconnectingWebSocket(url);
        this.setupWebsocket();
    }

    private setupWebsocket() {
        this.ws.onmessage = (event) => {
            if (typeof event.data !== "string") {
                console.error(
                    "Websocket message data is not string",
                    event.data
                );
                return;
            }
            let json: unknown;
            try {
                json = JSON.parse(event.data);
            } catch (e) {
                console.error("Websocket message error:", e);
                return;
            }
            if (!isWebsocketMessage(json)) return;
            if (isLyric(json)) {
                this.onLyric(json);
            } else if (isSetting(json)) {
                this.onSetting(json);
            }
        };
    }

    private sendData(data: unknown) {
        if (this.ws.readyState !== WebSocket.OPEN) {
            console.error("Websocket is not open");
            return;
        }
        this.ws.send(JSON.stringify(data));
    }

    private onLyric(data: WebsocketLyric) {
        // todo: 这里应该存一个 callback ?
        console.log(data);
    }

    private onSetting(data: WebsocketSetting) {
        if (data.echo != null) {
            this.actions.get(data.echo)?.(data);
            return;
        }
    }

    private genKey(): string {
        let key: string;
        do {
            key = generateRandomString(16);
        } while (this.actions.has(key));
        return key;
    }

    private async sendSetting<
        K extends keyof WebsocketSettingTypeMap = keyof WebsocketSettingTypeMap,
    >(data: WebsocketSettingTypeMap[K]): Promise<WebsocketSetting<K>> {
        return new Promise<WebsocketSetting<K>>((resolve, reject) => {
            const key = this.genKey();
            this.actions.set(key, resolve as SettingHandler);
            this.sendData(data);
            setTimeout(
                () => reject(new Error("Websocket setting timeout")),
                1000
            );
        });
    }

    /**
     * 清空歌词
     */
    public async setClear() {
        const data: WebsocketSettingTypeMap["setClear"] = null;
        await this.sendSetting(data);
    }

    // todo: 将其他事件补全
}

import ReconnectingWebSocket from "reconnecting-websocket";
import {
    isLyric,
    isSetting,
    isWebsocketMessage,
    WebsocketLyric,
    WebsocketSetting,
} from "@/api/model.ts";
import { BACKEND_WEBSOCKET_URL } from "@/config/constants.ts";

export type SettingHandler = (data: WebsocketSetting) => void;
export type LyricHandler = (data: WebsocketLyric) => void;

/** 只接收展示端事件，不发送任何管理请求。 */
export class Websocket {
    private ws: ReconnectingWebSocket;
    private lyricHandler: LyricHandler | undefined;
    private settingHandler: SettingHandler | undefined;

    constructor() {
        this.ws = new ReconnectingWebSocket(BACKEND_WEBSOCKET_URL);
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
                this.lyricHandler?.(json);
            } else if (isSetting(json)) {
                this.settingHandler?.(json);
            }
        };
    }

    public setLyricHandler(handler: LyricHandler | undefined) {
        this.lyricHandler = handler;
    }

    public setSettingHandler(handler: SettingHandler | undefined) {
        this.settingHandler = handler;
    }

    public close() {
        this.ws.close();
    }
}

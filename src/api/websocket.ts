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

/**
 * 新版后端 WebSocket 客户端 —— **只接收，不发送**（F-10）。
 *
 * 改造前这里承担了两件事：接收展示推送，以及通过 `sendSetting` + echo
 * 请求队列做**管理 RPC**。管理已全部迁到 HTTP（B-11 删除了后端入口），
 * 因此整个发送侧被删除：
 *
 * - 没有 `sendData` / `sendSetting` / echo 队列
 * - 没有 `?setter=true` 连接模式
 * - 没有 `getFont` / `setBlock` / `setCacheClean` 之类的管理方法
 *
 * 保留的只有展示事件接收：歌词推送、`setColor` / `setFont` / `setFontSize` /
 * `setAlignment` / `setTranslationMain` / `setSecondShow` / `setShadow` /
 * `setClear` / `setBlink` 等**后端主动推给展示端**的事件。
 *
 * 这样就不存在"通过 WS 再建一个隐藏管理通道"的可能。
 */
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

    /** 歌词推送处理器 */
    public setLyricHandler(handler: LyricHandler | undefined) {
        this.lyricHandler = handler;
    }

    /** 展示设置广播处理器 */
    public setSettingHandler(handler: SettingHandler | undefined) {
        this.settingHandler = handler;
    }

    public close() {
        this.ws.close();
    }
}

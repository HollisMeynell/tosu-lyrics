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
    >(type: K, data: WebsocketSettingTypeMap[K]): Promise<WebsocketSetting<K>> {
        return new Promise<WebsocketSetting<K>>((resolve, reject) => {
            const key = this.genKey();
            this.actions.set(key, resolve as SettingHandler);
            const message: WebsocketSetting<K> = {
                type: "setting",
                key: type,
                value: data,
                echo: key,
            };
            this.sendData(message);
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
        await this.sendSetting("setClear", data);
    }

    public async setFont(
        data: WebsocketSettingTypeMap["setFont"]
    ): Promise<WebsocketSettingTypeMap["setFont"]> {
        const result = await this.sendSetting("setFont", data);
        return result.value!;
    }

    public async getFont(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async setFontSize(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async getFontSize(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async setAlignment(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async getAlignment(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async setColor(
        data: WebsocketSettingTypeMap["getFont"]
    ): Promise<WebsocketSettingTypeMap["getFont"]> {
        const result = await this.sendSetting("getFont", data);
        return result.value!;
    }

    public async getColor(
        data: WebsocketSettingTypeMap["getColor"]
    ): Promise<WebsocketSettingTypeMap["getColor"]> {
        const result = await this.sendSetting("getColor", data);
        return result.value!;
    }

    public async setTranslationMain(
        data: WebsocketSettingTypeMap["setTranslationMain"]
    ): Promise<WebsocketSettingTypeMap["setTranslationMain"]> {
        const result = await this.sendSetting("setTranslationMain", data);
        return result.value!;
    }

    public async getTranslationMain(
        data: WebsocketSettingTypeMap["getTranslationMain"]
    ): Promise<WebsocketSettingTypeMap["getTranslationMain"]> {
        const result = await this.sendSetting("getTranslationMain", data);
        return result.value!;
    }

    public async setSecondShow(
        data: WebsocketSettingTypeMap["setSecondShow"]
    ): Promise<WebsocketSettingTypeMap["setSecondShow"]> {
        const result = await this.sendSetting("setSecondShow", data);
        return result.value!;
    }

    public async getSecondShow(
        data: WebsocketSettingTypeMap["getSecondShow"]
    ): Promise<WebsocketSettingTypeMap["getSecondShow"]> {
        const result = await this.sendSetting("getSecondShow", data);
        return result.value!;
    }

    public async setLyricSource(
        data: WebsocketSettingTypeMap["setLyricSource"]
    ): Promise<WebsocketSettingTypeMap["setLyricSource"]> {
        const result = await this.sendSetting("setLyricSource", data);
        return result.value!;
    }

    public async getLyricList(
        data: WebsocketSettingTypeMap["getLyricList"]
    ): Promise<WebsocketSettingTypeMap["getLyricList"]> {
        const result = await this.sendSetting("getLyricList", data);
        return result.value!;
    }

    public async getAllLyric(
        data: WebsocketSettingTypeMap["getAllLyric"]
    ): Promise<WebsocketSettingTypeMap["getAllLyric"]> {
        const result = await this.sendSetting("getAllLyric", data);
        return result.value!;
    }

    public async setBlock(
        data: WebsocketSettingTypeMap["setBlock"]
    ): Promise<WebsocketSettingTypeMap["setBlock"]> {
        const result = await this.sendSetting("setBlock", data);
        return result.value!;
    }

    public async setUnblock(
        data: WebsocketSettingTypeMap["setUnblock"]
    ): Promise<WebsocketSettingTypeMap["setUnblock"]> {
        const result = await this.sendSetting("setUnblock", data);
        return result.value!;
    }

    public async getBlockList(
        data: WebsocketSettingTypeMap["getBlockList"]
    ): Promise<WebsocketSettingTypeMap["getBlockList"]> {
        const result = await this.sendSetting("getBlockList", data);
        return result.value!;
    }

    public async getCacheCount(
        data: WebsocketSettingTypeMap["getCacheCount"]
    ): Promise<WebsocketSettingTypeMap["getCacheCount"]> {
        const result = await this.sendSetting("getCacheCount", data);
        return result.value!;
    }

    public async setCacheClean(
        data: WebsocketSettingTypeMap["setCacheClean"]
    ): Promise<WebsocketSettingTypeMap["setCacheClean"]> {
        const result = await this.sendSetting("setCacheClean", data);
        return result.value!;
    }

    public async getLyricOffset(
        data: WebsocketSettingTypeMap["getLyricOffset"]
    ): Promise<WebsocketSettingTypeMap["getLyricOffset"]> {
        const result = await this.sendSetting("getLyricOffset", data);
        return result.value!;
    }

    public async setLyricOffset(
        data: WebsocketSettingTypeMap["setLyricOffset"]
    ): Promise<WebsocketSettingTypeMap["setLyricOffset"]> {
        const result = await this.sendSetting("setLyricOffset", data);
        return result.value!;
    }
}

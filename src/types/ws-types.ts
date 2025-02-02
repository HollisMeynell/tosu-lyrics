export interface CommandMessage {
    type: string;
}

export interface WebSocketMessage {
    command: CommandMessage | SettingMessage | OnlineMessage;
    echo: string | null;
}

export interface SettingHandle {
    key: string;
    handle: (v: unknown) => void;
}

// 设置方面的 ws 消息
export interface SettingMessage extends CommandMessage {
    type: "setting";
    key: string;
    value: unknown;
}

export interface OnlineMessage extends CommandMessage {
    type: "online";
    id: string;
    status: "online" | "offline" | "conflict";
    others: string[] | null;
}
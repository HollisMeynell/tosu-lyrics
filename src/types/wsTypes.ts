export type CommandType =
    | "setting"
    | "online"
    | "query-request"
    | "query-response";

export interface CommandMessage {
    type: CommandType;
}

export interface WebSocketMessage {
    command: CommandMessage;
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
    target?: string;
    value?: unknown;
}

export interface OnlineMessage extends CommandMessage {
    type: "online";
    id: string;
    status: "online" | "offline" | "conflict";
    others: string[] | null;
}

export interface QueryRequestMessage extends CommandMessage {
    type: "query-request";
    key: string;
    params?: unknown;
    query: string;
}

export interface QueryResponseMessage extends CommandMessage {
    type: "query-response";
    key: string;
    value: unknown;
    error?: string;
}

export type MessageType = "lyric" | "setting";
export type SequenceType = "up" | "down";

export interface Message {
    type: MessageType;
}

export interface LyricMessage extends Message {
    first: string;
    second?: string;
    content?: string[];
    sequence: SequenceType;
}

export interface SettingMessage extends Message {
    key: string;
    value?: unknown;
    error?: string;
    echo?: string;
}

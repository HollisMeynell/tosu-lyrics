use serde::{Deserialize, Serialize};

/// 表示歌词的序列方向。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SequenceType {
    Up,
    Down,
}

/// 表示包含歌词信息的消息负载。
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricPayload {
    pub first: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<String>>,
    pub sequence: SequenceType,
}

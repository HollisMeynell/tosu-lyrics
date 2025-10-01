use std::sync::Arc;
use crate::lyric::LyricLine;
use crate::setting::global_setting;
use serde::{Deserialize, Serialize};

/// 表示歌词的序列方向。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SequenceType {
    Up,
    Down,
}

/// 歌词行
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricLinePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

/// 表示包含歌词信息的消息负载。
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyric: Option<Arc<[LyricLinePayload]>>,
    pub current: i32,
    pub next_time: i32,
    pub sequence: SequenceType,
}

impl Default for LyricPayload {
    fn default() -> Self {
        Self {
            lyric: None,
            current: 0,
            next_time: -1,
            sequence: SequenceType::Up,
        }
    }
}
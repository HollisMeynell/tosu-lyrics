use crate::lyric::LyricLine;
use crate::setting::global_setting;
use salvo::websocket::Message;
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
    pub first: Option<String>,
    pub second: Option<String>,
}

/// 表示包含歌词信息的消息负载。
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LyricPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<LyricLinePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<LyricLinePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<LyricLinePayload>,
    pub next_time: i32,
    pub sequence: SequenceType,
}

impl Default for LyricPayload {
    fn default() -> Self {
        Self {
            previous: None,
            current: None,
            next: None,
            next_time: 0,
            sequence: SequenceType::Up,
        }
    }
}

impl Into<Message> for LyricPayload {
    fn into(self) -> Message {
        Message::text(serde_json::to_string(&self).unwrap())
    }
}

impl LyricPayload {
    pub async fn set_previous_lyric(&mut self, line: &LyricLine) {
        self.previous =
            Self::gen_lyric_line(line.origin.as_deref(), line.translation.as_deref()).await
    }
    pub async fn set_current_lyric(&mut self, line: &LyricLine) {
        self.current =
            Self::gen_lyric_line(line.origin.as_deref(), line.translation.as_deref()).await
    }
    pub async fn set_next_lyric(&mut self, line: &LyricLine) {
        self.next = Self::gen_lyric_line(line.origin.as_deref(), line.translation.as_deref()).await
    }

    async fn gen_lyric_line(
        origin: Option<&str>,
        translation: Option<&str>,
    ) -> Option<LyricLinePayload> {
        match (origin, translation) {
            (Some(origin), Some(translation)) => {
                let trans_main = { global_setting().await.read().await.trans_main };
                if trans_main {
                    // 中文翻译为主
                    Some(LyricLinePayload {
                        first: Some(translation.to_string()),
                        second: Some(origin.to_string()),
                    })
                } else {
                    // 原文为主
                    Some(LyricLinePayload {
                        first: Some(origin.to_string()),
                        second: Some(translation.to_string()),
                    })
                }
            }
            (Some(origin), None) => Some(LyricLinePayload {
                first: Some(origin.to_string()),
                second: None,
            }),
            (None, Some(translation)) => Some(LyricLinePayload {
                first: None,
                second: Some(translation.to_string()),
            }),
            (None, None) => None,
        }
    }
}

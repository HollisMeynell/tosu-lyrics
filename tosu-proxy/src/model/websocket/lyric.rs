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

impl LyricPayload {
    pub fn set_previous_lyric(&mut self, origin: Option<&str>, translation: Option<&str>) {
        self.previous = Self::gen_lyric_line(origin, translation)
    }
    pub fn set_current_lyric(&mut self, origin: Option<&str>, translation: Option<&str>) {
        self.current = Self::gen_lyric_line(origin, translation)
    }
    pub fn set_next_lyric(&mut self, origin: Option<&str>, translation: Option<&str>) {
        self.next = Self::gen_lyric_line(origin, translation)
    }

    fn gen_lyric_line(origin: Option<&str>, translation: Option<&str>) -> Option<LyricLinePayload> {
        match (origin, translation) {
            (Some(origin), Some(translation)) => {
                // true 翻译为主, false 原文为主 
                if true {
                    Some(LyricLinePayload {
                        first: Some(translation.to_string()),
                        second: Some(origin.to_string()),
                    })
                } else {
                    Some(LyricLinePayload {
                        first: Some(origin.to_string()),
                        second: Some(translation.to_string()),
                    })
                }
            },
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

pub mod setting;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LyricLineData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

impl From<&crate::lyric::LyricLine> for LyricLineData {
    fn from(line: &crate::lyric::LyricLine) -> Self {
        Self {
            origin: line.origin.clone(),
            translation: line.translation.clone(),
        }
    }
}

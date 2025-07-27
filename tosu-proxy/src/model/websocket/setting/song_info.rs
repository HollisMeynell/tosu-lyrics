// 直接一个 type alias

use serde::{Deserialize, Serialize};

type SongInfo = crate::lyric::SongInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfoList {
    #[serde(rename = "QQ")]
    pub qq: Vec<SongInfo>,
    #[serde(rename = "QQ")]
    pub netease: Vec<SongInfo>,
}

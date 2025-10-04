use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberName {
    #[serde(default)]
    pub number: i64,
    pub name: String,
}

/// 谱面的时间信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapTime {
    /// 实时位置
    pub live: i32,

    /// 第一个物件的时间
    #[serde(rename = "firstObject")]
    pub first_object: i64,

    /// 最后一个物件的时间
    #[serde(rename = "lastObject")]
    pub last_object: i64,

    /// MP3 文件长度
    #[serde(rename = "mp3Length", default)]
    pub mp3_length: i64,
}

/// 谱面信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Beatmap {
    /// 是否为转换谱面
    #[serde(rename = "isConvert")]
    pub is_convert: bool,

    /// 时间
    pub time: BeatmapTime,

    /// Rank 状态
    pub status: NumberName,

    /// 谱面校验和
    #[serde(default)]
    pub checksum: String,

    /// 谱面ID
    #[serde(default)]
    pub id: i64,

    /// 谱面集ID
    #[serde(default)]
    pub set: i64,

    /// 游戏模式
    pub mode: NumberName,

    /// 艺术家名
    pub artist: Option<String>,

    /// Unicode格式的艺术家名
    #[serde(rename = "artistUnicode")]
    pub artist_unicode: Option<String>,

    /// 歌曲标题
    pub title: Option<String>,

    /// Unicode格式的歌曲标题
    #[serde(rename = "titleUnicode")]
    pub title_unicode: Option<String>,

    /// 谱面作者
    #[serde(default)]
    pub mapper: String,

    /// 难度名称
    #[serde(default)]
    pub version: String,
}

/// 谱面相关文件的路径
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    /// 谱面文件相对路径
    #[serde(default)]
    pub beatmap: String,

    /// bg相对路径
    #[serde(default)]
    pub background: String,

    /// 音频相对路径
    #[serde(default)]
    pub audio: String,
}

/// 相关文件夹的路径
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Folders {
    /// 主目录
    #[serde(default)]
    pub game: String,

    /// 皮肤目录
    #[serde(default)]
    pub skin: String,

    /// 歌曲目录
    #[serde(default)]
    pub songs: String,

    /// 当前谱面目录
    #[serde(default)]
    pub beatmap: String,
}

/// Tosu API 响应的主结构
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TosuApi {
    /// 谱面信息
    pub beatmap: Beatmap,

    /// 文件路径
    pub files: Files,

    /// 文件夹路径
    pub folders: Folders,
}

impl TosuApi {
    /// 获取音频文件的完整路径
    pub fn audio_path(&self) -> PathBuf {
        Path::new(&self.folders.songs)
            .join(&self.folders.beatmap)
            .join(&self.files.audio)
    }
}

impl TryFrom<&str> for TosuApi {
    type Error = crate::error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::util::to_json(value)
    }
}

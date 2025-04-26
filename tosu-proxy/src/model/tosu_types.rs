use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberName {
    pub number: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapTime {
    pub live: i64,
    #[serde(rename = "firstObject")]
    pub first_object: i64,
    #[serde(rename = "lastObject")]
    pub last_object: i64,
    #[serde(rename = "mp3Length")]
    pub mp3_length: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Beatmap {
    #[serde(rename = "isConvert")]
    pub is_convert: bool,
    pub time: BeatmapTime,
    pub status: NumberName,
    pub checksum: String,
    pub id: i64,
    pub set: i64,
    pub mode: NumberName,
    pub artist: String,
    #[serde(rename = "artistUnicode")]
    pub artist_unicode: String,
    pub title: String,
    #[serde(rename = "titleUnicode")]
    pub title_unicode: String,
    pub mapper: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    pub beatmap: String,
    pub background: String,
    pub audio: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Folders {
    pub game: String,
    pub skin: String,
    pub songs: String,
    pub beatmap: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TosuApi {
    pub beatmap: Beatmap,
    pub files: Files,
    pub folders: Folders,
}

impl TosuApi {
    pub fn print_audio_path(&self) -> PathBuf {
        Path::new(&self.folders.songs).join(&self.files.audio)
    }
}

impl TryFrom<&str> for TosuApi {
    type Error = crate::error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).or_else(|e| Err(e.into()))
    }
}

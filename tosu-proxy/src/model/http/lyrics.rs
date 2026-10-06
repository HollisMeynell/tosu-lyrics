use serde::Serialize;
use serde_json::Value;

use crate::lyric::LyricLine;
use crate::osu_source::OsuSongInfo;
use crate::service::lyric_content_service::Candidate;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongDto {
    pub bid: i64,
    pub sid: i64,
    pub title: String,
    pub artist: String,
    pub length: i32,
}

impl From<&OsuSongInfo> for SongDto {
    fn from(s: &OsuSongInfo) -> Self {
        Self {
            bid: s.bid,
            sid: s.sid,
            title: s.title_unicode.clone(),
            artist: s.artist_unicode.clone(),
            length: s.length,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLineDto {
    pub time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

impl From<&LyricLine> for LyricLineDto {
    fn from(line: &LyricLine) -> Self {
        Self {
            time: (line.time * 1000.0).round() as i64,
            origin: line.origin.clone(),
            translation: line.translation.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBinding {
    pub sid: i32,
    pub source: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentLyricResponse {
    pub song: SongDto,
    pub offset: i32,
    pub current: i32,
    pub next_time: i32,
    pub lyric: Vec<LyricLineDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceBinding>,
    pub lyric_state: &'static str,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsResponse {
    pub total: usize,
    pub items: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponse {
    pub source: String,
    pub key: String,
    pub line_count: usize,
    pub lines: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PutSourceResponse {
    pub ok: bool,
    pub source: Candidate,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSourceResponse {
    pub ok: bool,
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetResponse {
    pub ok: bool,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationCheckItem {
    pub source: String,
    pub key: String,
    pub has_translation: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationCheckResponse {
    pub items: Vec<TranslationCheckItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLyricResponse {
    pub ok: bool,
    pub lines: usize,
    pub song: UploadSongInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadSongInfo {
    pub bid: i32,
    pub sid: i32,
    pub title: String,
}

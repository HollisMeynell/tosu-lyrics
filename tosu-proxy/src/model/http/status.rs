use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub song: Option<StatusSong>,
    pub lyric: StatusLyric,
    pub offset: i32,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSong {
    pub bid: i64,
    pub sid: i64,
    pub title: String,
    pub artist: String,
    pub length: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLyric {
    pub loaded: bool,
    pub cleared: bool,
    pub line_count: usize,
    pub current: i32,
    pub next_time: i32,
}

impl StatusLyric {
    pub fn loaded(line_count: usize, current: i32, next_time: i32) -> Self {
        Self {
            loaded: true,
            cleared: false,
            line_count,
            current,
            next_time,
        }
    }

    pub fn empty(line_count: usize) -> Self {
        Self {
            loaded: false,
            cleared: true,
            line_count,
            current: -1,
            next_time: -1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSongResponse {
    pub song: StatusSong,
    pub offset: i32,
    pub current: i32,
    pub next_time: i32,
    pub lyric: Vec<crate::model::http::lyrics::LyricLineDto>,
}

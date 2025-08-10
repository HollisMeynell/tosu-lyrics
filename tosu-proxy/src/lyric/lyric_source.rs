use crate::error::{Error, Result};
use std::ops::Index;

use serde::{Deserialize, Serialize};

mod parse {
    use regex::Regex;
    use std::sync::LazyLock;

    /// parse time regex
    static REG_LYRIC_LINE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[(\d+):(\d+\.\d+)](.+)").unwrap());

    pub(super) struct LyricRawLine {
        // 秒
        pub(super) time: f32,
        pub(super) line: String,
    }

    impl From<LyricRawLine> for String {
        fn from(lyric: LyricRawLine) -> String {
            lyric.line
        }
    }

    fn parse_lyric_line(captures: &regex::Captures) -> LyricRawLine {
        let minutes: f32 = captures[1].parse().unwrap_or(0.0);
        let seconds: f32 = captures[2].parse().unwrap_or(0.0);
        let line = captures[3].trim().to_string();
        let time = minutes * 60.0 + seconds;
        LyricRawLine { time, line }
    }
    pub fn parse_lyric_text_raw(lyric_text: &str) -> Vec<LyricRawLine> {
        // tnnd rust 正则不支持回溯
        let mut line_buffer = String::new();
        let mut result = vec![];
        fn parse_line(text: &str, data: &mut Vec<LyricRawLine>) {
            let b = REG_LYRIC_LINE.captures(text);
            if b.is_none() {
                return;
            }
            let caps = b.unwrap();
            let line = parse_lyric_line(&caps);
            data.push(line);
        }
        for char in lyric_text.chars() {
            if char == '\n' {
                continue;
            }
            if char == '[' && !line_buffer.is_empty() {
                parse_line(&line_buffer, &mut result);
                line_buffer.clear();
            }
            line_buffer.push(char);
        }
        if !line_buffer.is_empty() {
            parse_line(&line_buffer, &mut result);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    // 秒
    pub time: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

#[derive(Debug)]
pub struct Lyric {
    lyrics: Vec<LyricLine>,
    // 秒
    end_time: f32,
    cursor: usize,
}

impl Default for Lyric {
    fn default() -> Self {
        Self {
            lyrics: Vec::new(),
            end_time: -1.0,
            cursor: 0,
        }
    }
}

impl Lyric {
    pub fn from_json_cache(json: &[u8]) -> Result<Self> {
        let lyrics: Vec<LyricLine> = serde_json::from_slice(json)?;
        let Some(end_time) = lyrics.last().map(|it| it.time) else {
            return Err(Error::from("缓存无效"));
        };

        let result = Self {
            lyrics,
            end_time,
            cursor: 0,
        };
        Ok(result)
    }

    pub fn to_json_cache(&self) -> Result<Vec<u8>> {
        if self.lyrics.is_empty() {
            return Err(Error::LyricParse("can not serialize empty lyric"));
        }
        let cache = serde_json::to_string(self.get_lyrics())?;
        Ok(cache.into_bytes())
    }

    pub fn parse(lyric: &str, trans: Option<&str>, title: Option<&str>) -> Result<Self> {
        use parse::*;
        if lyric.trim().is_empty() {
            return Err(Error::LyricParse("empty lyric"));
        }
        // parse origin lyric
        let mut lyric_lines = parse_lyric_text_raw(lyric);
        if lyric_lines.is_empty() {
            return Err(Error::LyricParse("lyric is empty"));
        }
        if let Some(title) = title {
            lyric_lines.insert(
                0,
                LyricRawLine {
                    time: 0f32,
                    line: title.to_string(),
                },
            )
        }

        let mut lyric = Self::default();
        for lr in lyric_lines {
            let lyric = lyric.get_line_mut(lr.time)?;
            lyric.origin = Some(lr.into())
        }
        lyric.cursor = 0;

        if trans.is_none() {
            return Ok(lyric);
        }

        let trans_lines = parse_lyric_text_raw(trans.unwrap());
        for tr in trans_lines {
            let lyric = lyric.get_line_mut(tr.time)?;
            let line = Some(tr.into());
            if lyric.origin.is_none() {
                lyric.origin = line
            } else {
                lyric.translation = line
            }
        }
        lyric.cursor = 0;
        Ok(lyric)
    }

    fn get_line_mut(&mut self, time: f32) -> Result<&mut LyricLine> {
        if time > self.end_time || self.lyrics.is_empty() {
            self.lyrics.push(LyricLine {
                time,
                origin: None,
                translation: None,
            });
            self.end_time = time;
            return self.lyrics.last_mut().ok_or(Error::Impossible);
        }

        if eq_f32(time, self.end_time) {
            return self.lyrics.last_mut().ok_or(Error::Impossible);
        }

        // no cursor
        if self.cursor >= self.lyrics.len() {
            let (index, line_time) = self
                .lyrics
                .iter()
                .enumerate()
                .rfind(|(_, v)| v.time <= time)
                .map(|(index, line)| (index, line.time))
                .unwrap_or((0usize, 999f32));
            self.cursor = index;
            return if eq_f32(time, line_time) {
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            } else {
                let index = index + 1;
                self.cursor += 1;
                self.lyrics.insert(
                    index,
                    LyricLine {
                        time,
                        origin: None,
                        translation: None,
                    },
                );
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            };
        }
        // has cursor
        let cursor_time = self.lyrics.get(self.cursor).unwrap().time;
        if eq_f32(cursor_time, time) {
            return self.lyrics.get_mut(self.cursor).ok_or(Error::Impossible);
        }
        let cursor = self.cursor;

        if cursor_time > time {
            let (index, line_time) = self.lyrics[0..cursor]
                .iter()
                .enumerate()
                .rfind(|(_, v)| v.time <= time)
                .map(|(i, v)| (i, v.time))
                .unwrap_or((0, 999f32));

            self.cursor = index;
            if !eq_f32(time, line_time) {
                let index = index + 1;
                self.cursor += 1;
                self.lyrics.insert(
                    index,
                    LyricLine {
                        time,
                        origin: None,
                        translation: None,
                    },
                );
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            } else {
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            }
        } else {
            let (index, line_time) = self.lyrics[cursor..]
                .iter()
                .enumerate()
                .find(|(_, v)| v.time >= time)
                .map(|(i, v)| (i, v.time))
                .unwrap_or_else(|| {
                    let last = self.lyrics.last().unwrap();
                    (self.lyrics.len() - 1, last.time)
                });
            self.cursor = index;
            if eq_f32(time, line_time) {
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            } else {
                self.end_time = time;
                self.lyrics.insert(
                    index,
                    LyricLine {
                        time,
                        origin: None,
                        translation: None,
                    },
                );
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            }
        }
    }

    /// `time` 时间, 秒
    pub fn find_line(&self, time: f32) -> Option<(usize, &LyricLine)> {
        let cursor_time = self.lyrics.get(self.cursor)?.time;
        if eq_f32(cursor_time, time) {
            let ly = self.lyrics.get(self.cursor)?;
            return Some((self.cursor, ly));
        }

        if cursor_time > time {
            let (index, line) = self.lyrics[0..self.cursor]
                .iter()
                .enumerate()
                .rfind(|(_, v)| v.time <= time)?;
            Some((index, line))
        } else {
            let (index, line) = self.lyrics[self.cursor..]
                .iter()
                .enumerate()
                .find(|(_, v)| v.time > time)
                .map(|(i, _)| (i - 1, &self.lyrics[self.cursor + i - 1]))
                .unwrap_or((self.lyrics.len() - 1, self.lyrics.last()?));
            Some((index, line))
        }
    }

    pub fn get_line_by_index(&self, index: usize) -> Option<&LyricLine> {
        if index > self.lyrics.len() {
            return None;
        }

        Some(self.lyrics.index(index))
    }

    pub fn get_lyrics(&self) -> &[LyricLine] {
        &self.lyrics
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
}

#[inline]
fn eq_f32(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

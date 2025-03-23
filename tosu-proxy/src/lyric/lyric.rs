use crate::error::{Error, Result};

use serde::{Deserialize, Serialize};

mod parse {
    use regex::Regex;
    use std::sync::LazyLock;

    /// split regex
    static REG_LINE_START: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?=\[.+:.+\])").unwrap());

    /// parse time regex
    static REG_LYRIC_LINE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[(\d+):(\d+\.\d+)](.+)").unwrap());

    pub(super) struct LyricRawLine {
        pub(super) time: f32,
        pub(super) line: String,
    }

    impl Into<String> for LyricRawLine {
        fn into(self) -> String {
            self.line
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
        REG_LINE_START
            .split(lyric_text)
            .filter_map(|line| REG_LYRIC_LINE.captures(line))
            .map(|caps| parse_lyric_line(&caps))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub time: f32,
    pub first: Option<String>,
    pub second: Option<String>,
}

pub struct Lyric {
    lyrics: Vec<LyricLine>,
    end_time: f32,
    cursor: usize,
}

impl Lyric {
    pub fn new() -> Self {
        Self {
            lyrics: Vec::new(),
            end_time: -1.0,
            cursor: 0,
        }
    }

    pub fn parse(&mut self, lyric: &str, trans: Option<&str>, title: Option<&str>) -> Result<()> {
        use parse::*;
        if lyric.trim().is_empty() {
            return Err(Error::LyricParse("empty lyric".to_string()));
        }
        // parse origin lyric
        let mut lyric_lines = parse_lyric_text_raw(lyric);
        if lyric_lines.is_empty() {
            return Err(Error::LyricParse("lyric is empty".to_string()));
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
        for lr in lyric_lines {
            let lyric = self.get_line_mut(lr.time)?;
            lyric.first = Some(lr.into())
        }
        self.cursor = 0;

        if trans.is_none() {
            return Ok(());
        }

        let trans_lines = parse_lyric_text_raw(trans.unwrap());
        for tr in trans_lines {
            let lyric = self.get_line_mut(tr.time)?;
            let line = Some(tr.into());
            if lyric.first.is_none() {
                lyric.first = line
            } else {
                lyric.second = line
            }
        }
        self.cursor = 0;
        Ok(())
    }

    fn get_line_mut(&mut self, time: f32) -> Result<&mut LyricLine> {
        if time > self.end_time || self.lyrics.is_empty() {
            self.lyrics.push(LyricLine {
                time,
                first: None,
                second: None,
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
                        first: None,
                        second: None,
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
                        first: None,
                        second: None,
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
                        first: None,
                        second: None,
                    },
                );
                self.lyrics.get_mut(index).ok_or(Error::Impossible)
            }
        }
    }

    pub fn find_line(&mut self, time: f32) -> Option<(usize, &LyricLine)> {
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

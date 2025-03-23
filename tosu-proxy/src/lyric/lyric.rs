use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct LyricError {
    message: String,
}

impl fmt::Display for LyricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for LyricError {}
static REG_LINE_START: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?=\[.+:.+\])").unwrap());
static REG_LYRIC_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(\d+):(\d+\.\d+)](.+)").unwrap());

fn parse_lyric_line(captures: &regex::Captures) -> LyricRawLine {
    let minutes: f64 = captures[1].parse().unwrap_or(0.0);
    let seconds: f64 = captures[2].parse().unwrap_or(0.0);
    let text = captures[3].trim().to_string();
    let time = minutes * 60.0 + seconds;
    LyricRawLine {
        time,
        first: text,
        second: None,
    }
}
pub fn parse_lyric_text_raw(lyric_text: &str) -> Vec<LyricRawLine> {
    REG_LINE_START
        .split(lyric_text)
        .filter_map(|line| REG_LYRIC_LINE.captures(line))
        .map(|caps| parse_lyric_line(&caps))
        .collect()
}

pub struct Lyric {
    lyrics: Vec<LyricRawLine>,
    end_time: f64,
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

    pub fn insert(&mut self, time: f64, text: &str, n: usize) -> usize {
        let text = text.trim();

        // 检查时间戳和歌词文本是否有效
        if time.is_nan() || time < 0.0 || text.is_empty() {
            return n + 1;
        }

        // 时间戳大于当前最大时间戳，直接插入到末尾
        if time > self.end_time {
            self.end_time = time;
            self.lyrics.push(LyricRawLine {
                time,
                first: text.to_string(),
                second: None,
            });
            return n + 1;
        }

        // 时间戳小于或等于当前最大时间戳，从指针 n 开始查找插入位置
        if n >= self.lyrics.len() {
            // 指针 n 超出数组范围，直接插入到末尾
            self.lyrics.push(LyricRawLine {
                time,
                first: text.to_string(),
                second: None,
            });
        } else if (self.lyrics[n].time - time).abs() < 1e-2 {
            // 时间戳接近（差值小于 1e-2），更新已有歌词 (翻译在前)
            let old_first = std::mem::replace(&mut self.lyrics[n].first, text.to_string());
            self.lyrics[n].second = Some(old_first);
        } else if self.lyrics[n].time > time + 1e-2 {
            // 时间戳小于指针 n 的时间戳，向前查找插入位置
            if n == 0 {
                // 直接插入到数组开头
                self.lyrics.insert(
                    n,
                    LyricRawLine {
                        time,
                        first: text.to_string(),
                        second: None,
                    },
                );
            } else {
                // 否则，向前查找合适的位置
                let mut pos = n;
                while pos > 0 && self.lyrics[pos - 1].time >= time + 1e-2 {
                    pos -= 1;
                }
                if (self.lyrics[pos].time - time).abs() < 1e-2 {
                    let old_first =
                        std::mem::replace(&mut self.lyrics[pos].first, text.to_string());
                    self.lyrics[pos].second = Some(old_first);
                } else {
                    self.lyrics.insert(
                        pos,
                        LyricRawLine {
                            time,
                            first: text.to_string(),
                            second: None,
                        },
                    );
                }
            }
        } else {
            // 时间戳大于指针 n 的时间戳，向后查找插入位置
            if n + 1 >= self.lyrics.len() {
                // n + 1 超出数组范围，直接插入到末尾
                self.lyrics.push(LyricRawLine {
                    time,
                    first: text.to_string(),
                    second: None,
                });
            } else {
                // 否则，向后查找合适的位置
                let mut pos = n;
                while pos < self.lyrics.len() - 1 && self.lyrics[pos + 1].time < time - 1e-2 {
                    pos += 1;
                }
                if (self.lyrics[pos + 1].time - time).abs() < 1e-2 {
                    let old_first =
                        std::mem::replace(&mut self.lyrics[pos + 1].first, text.to_string());
                    self.lyrics[pos + 1].second = Some(old_first);
                } else {
                    self.lyrics.insert(
                        pos + 1,
                        LyricRawLine {
                            time,
                            first: text.to_string(),
                            second: None,
                        },
                    );
                }
            }
        }
        n + 1
    }

    pub fn insert_all(
        &mut self,
        lyric: &str,
        trans: Option<&str>,
        title: Option<&str>,
    ) -> Result<(), LyricError> {
        if lyric.trim().is_empty() {
            return Err(LyricError {
                message: "Empty lyric".to_string(),
            });
        }

        // 解析原版歌词
        let lyric_lines = parse_lyric_text_raw(lyric);

        // 如果有标题且第一行不是时间戳0，插入标题
        if let Some(title) = title {
            if !lyric_lines.is_empty() && lyric_lines[0].time > 1e-2 {
                self.insert(0.0, title, 0);
            }
        }

        // 插入原版歌词
        for line in lyric_lines {
            self.insert(line.time, &line.first, self.cursor);
        }

        // 解析并插入翻译歌词
        if let Some(trans) = trans {
            if !trans.trim().is_empty() {
                let trans_lines = parse_lyric_text_raw(trans);
                let mut cursor = 0;
                for line in trans_lines {
                    cursor = self.insert(line.time, &line.first, cursor);
                }
            }
        }

        Ok(())
    }

    pub fn next_time(&self) -> f64 {
        if self.cursor >= self.lyrics.len() - 1 {
            0.0
        } else {
            self.lyrics[self.cursor + 1].time - self.lyrics[self.cursor].time
        }
    }

    pub fn jump(&mut self, time: f64) {
        if self.lyrics.is_empty() {
            return;
        }
        if time < 1.5 {
            self.cursor = 0;
            return;
        }

        let n = self.cursor;
        if self.lyrics[n].time < time {
            let next_time = self.lyrics.get(n + 1).map_or(99999.0, |l| l.time);
            if next_time > time {
                return;
            }
            let next_time2 = self.lyrics.get(n + 2).map_or(99999.0, |l| l.time);
            if next_time2 > time {
                self.cursor = n + 1;
                return;
            }
        }

        let mut low = 0;
        let mut high = self.lyrics.len() - 1;

        if self.lyrics[low].time > time {
            self.cursor = low;
            return;
        }

        if self.lyrics[high].time < time {
            self.cursor = high;
            return;
        }

        while low < high {
            let mid = (low + high) / 2;
            let smaller = self.lyrics[mid].time <= time;
            let bigger = self.lyrics[mid + 1].time > time;
            if smaller && bigger {
                self.cursor = mid;
                return;
            } else if smaller {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        self.cursor = low;
    }

    pub fn get_lyrics(&self) -> &[LyricRawLine] {
        &self.lyrics
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lyric_insert() {
        let mut lyric = Lyric::new();

        // 测试基本插入
        lyric.insert(0.0, "第一行", 0);
        lyric.insert(1.0, "第二行", 0);

        assert_eq!(lyric.get_lyrics().len(), 2);
        assert_eq!(lyric.get_lyrics()[0].first, "第一行");
        assert_eq!(lyric.get_lyrics()[1].first, "第二行");

        // 测试翻译插入
        lyric.insert(0.0, "First line", 0);
        assert_eq!(lyric.get_lyrics()[0].first, "First line");
        assert_eq!(lyric.get_lyrics()[0].second, Some("第一行".to_string()));
    }

    #[test]
    fn test_lyric_insert_all() {
        let mut lyric = Lyric::new();
        let origin = "[00:00.00]第一行\n[00:01.00]第二行";
        let trans = "[00:00.00]First line\n[00:01.00]Second line";

        lyric.insert_all(origin, Some(trans), None).unwrap();

        assert_eq!(lyric.get_lyrics().len(), 2);
        assert_eq!(lyric.get_lyrics()[0].first, "First line");
        assert_eq!(lyric.get_lyrics()[0].second, Some("第一行".to_string()));
        assert_eq!(lyric.get_lyrics()[1].first, "Second line");
        assert_eq!(lyric.get_lyrics()[1].second, Some("第二行".to_string()));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricRawLine {
    pub time: f64,
    pub first: String,
    pub second: Option<String>,
}

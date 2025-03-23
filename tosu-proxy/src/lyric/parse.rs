use regex::Regex;
use std::sync::LazyLock;
use serde::{Serialize, Deserialize};

/// time: ms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLyricLine {
    pub time: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricRawLine {
    pub time: f64,
    pub first: String,
    pub second: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLyricResult {
    pub lyric: String,
    pub trans: Option<String>,
}

static REG_LINE_START: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?=\[.+:.+\])").unwrap());
static REG_LYRIC_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[(\d+):(\d+\.\d+)](.+)").unwrap());

pub fn parse_lyric(captures: &regex::Captures) -> ParsedLyricLine {
    let minutes: u32 = captures[1].parse().unwrap_or(0);
    let seconds: f32 = captures[2].parse().unwrap_or(0.0);
    let text = captures[3].trim().to_string();
    let time = minutes * 60 + (seconds * 1000f32) as u32;
    ParsedLyricLine { time, text }
}

pub fn parse_lyric_text(lyric_text: &str) -> Vec<ParsedLyricLine> {
    REG_LINE_START
        .split(lyric_text)
        .filter_map(|line| REG_LYRIC_LINE.captures(line))
        .map(|caps| parse_lyric(&caps))
        .collect()
}

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

pub fn merge_lyrics(origin: &[LyricRawLine], trans: &[LyricRawLine]) -> Vec<LyricRawLine> {
    let mut result = origin.to_vec();
    
    for trans_line in trans {
        let time = trans_line.time;
        let mut found = false;
        
        // 查找相同时间戳的歌词
        for line in &mut result {
            if (line.time - time).abs() < 1e-2 {
                line.second = Some(trans_line.first.clone());
                found = true;
                break;
            }
        }
        
        // 如果没有找到相同时间戳的歌词，插入新的行
        if !found {
            let mut new_line = trans_line.clone();
            new_line.first = String::new(); // 原版歌词为空
            result.push(new_line);
        }
    }
    
    // 按时间排序
    result.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    result
}

pub fn parse_unified_lyric(lyric: &str, trans: Option<&str>) -> UnifiedLyricResult {
    UnifiedLyricResult {
        lyric: lyric.to_string(),
        trans: trans.map(|t| t.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lyric() {
        let lyric_text = "[00:00.00]Test lyric\n[00:01.00]Another line";
        let result = parse_lyric_text_raw(lyric_text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].time, 0.0);
        assert_eq!(result[1].time, 1.0);
        assert_eq!(result[0].first, "Test lyric");
        assert_eq!(result[1].first, "Another line");
    }

    #[test]
    fn test_merge_lyrics() {
        let origin = vec![
            LyricRawLine {
                time: 0.0,
                first: "原版歌词1".to_string(),
                second: None,
            },
            LyricRawLine {
                time: 1.0,
                first: "原版歌词2".to_string(),
                second: None,
            },
        ];

        let trans = vec![
            LyricRawLine {
                time: 0.0,
                first: "翻译歌词1".to_string(),
                second: None,
            },
            LyricRawLine {
                time: 1.0,
                first: "翻译歌词2".to_string(),
                second: None,
            },
        ];

        let merged = merge_lyrics(&origin, &trans);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].first, "原版歌词1");
        assert_eq!(merged[0].second, Some("翻译歌词1".to_string()));
        assert_eq!(merged[1].first, "原版歌词2");
        assert_eq!(merged[1].second, Some("翻译歌词2".to_string()));
    }
}


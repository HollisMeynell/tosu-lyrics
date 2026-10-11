use crate::error::{Error, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tokio::io::AsyncWriteExt;

static DISPLAY_NAME_CACHE: OnceLock<Mutex<HashMap<PathBuf, (String, String)>>> = OnceLock::new();

pub const FONT_DIR: &str = "font";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontEntry {
    pub name: String,
    pub file_name: String,
    pub size: u64,
    pub version: String,
    pub url: String,
}

fn percent_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            result.push(byte as char);
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}

pub fn version_of(path: &Path) -> (String, u64) {
    let Ok(meta) = std::fs::metadata(path) else {
        return (String::new(), 0);
    };
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0);
    (format!("{mtime}-{size}"), size)
}

fn font_dir() -> PathBuf {
    std::env::current_dir()
        .expect("cannot get current directory")
        .join(FONT_DIR)
}

fn display_name_for(path: &Path) -> String {
    let fallback = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let version = version_of(path).0;
    if version.is_empty() {
        return fallback;
    }

    let cache = DISPLAY_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some((cached_version, name)) = guard.get(path)
        && *cached_version == version
    {
        return name.clone();
    }

    let Ok(bytes) = std::fs::read(path) else {
        return fallback;
    };
    let name = font_display_name(&bytes).unwrap_or_else(|| fallback.clone());
    if let Ok(mut guard) = cache.lock() {
        guard.insert(path.to_path_buf(), (version, name.clone()));
    }
    name
}

pub fn list_fonts() -> Vec<FontEntry> {
    let mut entries = Vec::new();
    let dir = font_dir();

    if !dir.is_dir() {
        return entries;
    }

    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return entries;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(
            ext.to_lowercase().as_str(),
            "otf" | "ttf" | "woff" | "woff2"
        ) {
            continue;
        }
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let (version, size) = version_of(&path);
        let name = display_name_for(&path);
        let encoded_name = percent_encode(&name);
        entries.push(FontEntry {
            name,
            file_name,
            size,
            version: version.clone(),
            url: format!("/api/font/{encoded_name}?v={version}"),
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

pub fn find_font_path(name: &str) -> Option<PathBuf> {
    let dir = font_dir();
    if !dir.is_dir() {
        return None;
    }
    let read_dir = std::fs::read_dir(&dir).ok()?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let display = display_name_for(&path);
        if display == name {
            return Some(path);
        }
    }
    None
}

pub async fn load(name: &str) -> Option<Vec<u8>> {
    let path = find_font_path(name)?;
    tokio::fs::read(&path).await.ok()
}

pub async fn save(bytes: &[u8]) -> Result<FontEntry> {
    if bytes.is_empty() {
        return Err(Error::Runtime("invalid_param:字体文件为空".into()));
    }
    if !looks_like_font(bytes) {
        return Err(Error::Runtime(
            "invalid_param:不是可识别的字体文件（支持 ttf / otf / woff / woff2）".into(),
        ));
    }

    let name = font_display_name(bytes)
        .ok_or_else(|| Error::Runtime("invalid_param:无法解析字体名称".into()))?;

    let ext = match bytes.get(..4) {
        Some(b"wOFF") => "woff",
        Some(b"wOF2") => "woff2",
        Some(b"OTTO") => "otf",
        _ => "ttf",
    };

    let dir = font_dir();
    tokio::fs::create_dir_all(&dir).await?;

    let safe_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let file_name = format!("{safe_name}.{ext}");
    let path = dir.join(&file_name);

    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(bytes).await?;
    file.flush().await?;
    file.sync_all().await?;

    let (version, size) = version_of(&path);
    let encoded_name = percent_encode(&name);
    Ok(FontEntry {
        name,
        file_name,
        size,
        version: version.clone(),
        url: format!("/api/font/{encoded_name}?v={version}"),
    })
}

fn looks_like_font(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let tag = &bytes[..4];
    matches!(
        tag,
        [0x00, 0x01, 0x00, 0x00]       // TrueType
            | [0x4F, 0x54, 0x54, 0x4F] // 'OTTO' PostScript
            | [0x74, 0x72, 0x75, 0x65] // 'true'
            | [0x77, 0x4F, 0x46, 0x46] // 'wOFF'
            | [0x77, 0x4F, 0x46, 0x32] // 'wOF2'
    )
}

pub fn font_display_name(bytes: &[u8]) -> Option<String> {
    let num_tables = u16::from_be_bytes([*bytes.get(4)?, *bytes.get(5)?]) as usize;

    let mut name_table: Option<&[u8]> = None;
    for i in 0..num_tables {
        let rec = 12 + i * 16;
        if bytes.get(rec..rec + 4)? != b"name" {
            continue;
        }
        let offset = u32::from_be_bytes(bytes.get(rec + 8..rec + 12)?.try_into().ok()?) as usize;
        let length = u32::from_be_bytes(bytes.get(rec + 12..rec + 16)?.try_into().ok()?) as usize;
        name_table = bytes.get(offset..offset.checked_add(length)?);
        break;
    }
    let table = name_table?;

    let count = u16::from_be_bytes([*table.get(2)?, *table.get(3)?]) as usize;
    let storage = u16::from_be_bytes([*table.get(4)?, *table.get(5)?]) as usize;

    for want_id in [4u16, 1u16] {
        let mut mac_fallback: Option<String> = None;
        for i in 0..count {
            let rec = 6 + i * 12;
            let platform = u16::from_be_bytes([*table.get(rec)?, *table.get(rec + 1)?]);
            let encoding = u16::from_be_bytes([*table.get(rec + 2)?, *table.get(rec + 3)?]);
            let name_id = u16::from_be_bytes([*table.get(rec + 6)?, *table.get(rec + 7)?]);
            if name_id != want_id {
                continue;
            }
            let length = u16::from_be_bytes([*table.get(rec + 8)?, *table.get(rec + 9)?]) as usize;
            let offset =
                u16::from_be_bytes([*table.get(rec + 10)?, *table.get(rec + 11)?]) as usize;
            let start = storage.checked_add(offset)?;
            let Some(raw) = table.get(start..start.checked_add(length)?) else {
                continue;
            };
            let decoded = match (platform, encoding) {
                (3, _) => decode_utf16_be(raw),
                (1, 0) => Some(raw.iter().map(|b| *b as char).collect()),
                _ => None,
            };
            let Some(text) = decoded else { continue };
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            if platform == 3 {
                return Some(text.to_string());
            }
            if mac_fallback.is_none() {
                mac_fallback = Some(text.to_string());
            }
        }
        if mac_fallback.is_some() {
            return mac_fallback;
        }
    }
    None
}

fn decode_utf16_be(raw: &[u8]) -> Option<String> {
    if !raw.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    let text = String::from_utf16(&units).ok()?;
    Some(text.replace('\0', ""))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn font_magic_detection() {
        assert!(looks_like_font(&[0x00, 0x01, 0x00, 0x00, 0xAA]));
        assert!(looks_like_font(b"OTTO____"));
        assert!(looks_like_font(b"wOFF____"));
        assert!(looks_like_font(b"wOF2____"));
        assert!(!looks_like_font(b"PK\x03\x04"), "zip 不是字体");
        assert!(!looks_like_font(&[0x00, 0x01]), "太短");
        assert!(!looks_like_font(b""), "空");
    }
}

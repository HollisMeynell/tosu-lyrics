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

const FONT_NAME_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn percent_encode(input: &str) -> String {
    percent_encoding::utf8_percent_encode(input, FONT_NAME_ENCODE_SET).to_string()
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
            "invalid_param:不是可识别的字体文件（支持 ttf / otf）".into(),
        ));
    }

    let name = font_display_name(bytes).ok_or_else(|| {
        Error::Runtime("invalid_param:无法解析字体名称（上传暂支持 ttf / otf）".into())
    })?;

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
    let face = ttf_parser::RawFace::parse(bytes, 0).ok()?;
    let table = ttf_parser::name::Table::parse(face.table(ttf_parser::Tag::from_bytes(b"name"))?)?;
    for want_id in [ttf_parser::name_id::FULL_NAME, ttf_parser::name_id::FAMILY] {
        for name in table.names {
            if name.name_id != want_id {
                continue;
            }
            let text = name.to_string().or_else(|| {
                if name.platform_id == ttf_parser::PlatformId::Macintosh
                    && name.encoding_id == 0
                    && name.name.is_ascii()
                {
                    std::str::from_utf8(name.name).ok().map(str::to_owned)
                } else {
                    None
                }
            });
            let Some(text) = text else { continue };
            let text = text.replace('\0', "");
            let text = text.trim();
            if !text.is_empty() {
                return Some(text.to_owned());
            }
        }
    }
    None
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

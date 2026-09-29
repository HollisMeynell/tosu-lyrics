use crate::error::{Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontKind {
    Main,
    Sub,
}

impl FontKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "main" => Ok(Self::Main),
            "sub" => Ok(Self::Sub),
            other => Err(Error::Runtime(format!(
                "invalid_param:字体种类只能是 main / sub, 收到 {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Sub => "sub",
        }
    }

    /// 与 utils/fonts.ts 的约定一致
    fn file_name(self) -> &'static str {
        match self {
            Self::Main => "LRC.otf",
            Self::Sub => "tLRC.otf",
        }
    }

    pub fn family(self) -> &'static str {
        match self {
            Self::Main => "LRC",
            Self::Sub => "LRC-Sub",
        }
    }

    pub fn path(self) -> PathBuf {
        let dir = std::env::current_dir().expect("cannot get current directory");
        dir.join(self.file_name())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub kind: String,
    pub family: String,
    pub exists: bool,
    pub version: String,
    pub size: u64,
    pub url: String,
}

/// mtime(ms) + size，覆盖写后必然变化，且跨重启稳定
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

pub fn info(kind: FontKind) -> FontInfo {
    let path = kind.path();
    let (version, size) = version_of(&path);
    let exists = !version.is_empty();
    FontInfo {
        kind: kind.as_str().to_string(),
        family: kind.family().to_string(),
        exists,
        url: if exists {
            format!("/api/font/{}?v={}", kind.as_str(), version)
        } else {
            String::new()
        },
        version,
        size,
    }
}

pub fn all_info() -> Vec<FontInfo> {
    vec![info(FontKind::Main), info(FontKind::Sub)]
}

/// 以 create（截断）打开，连续上传不同大小的字体不会留下尾部残留
pub async fn save(kind: FontKind, bytes: &[u8]) -> Result<FontInfo> {
    if bytes.is_empty() {
        return Err(Error::Runtime("invalid_param:字体文件为空".into()));
    }
    if !looks_like_font(bytes) {
        return Err(Error::Runtime(
            "invalid_param:不是可识别的字体文件（支持 ttf / otf / woff / woff2）".into(),
        ));
    }

    let path = kind.path();
    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(bytes).await?;
    file.flush().await?;
    file.sync_all().await?;
    Ok(info(kind))
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

pub async fn load(kind: FontKind) -> Option<Vec<u8>> {
    tokio::fs::read(kind.path()).await.ok()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn kind_parsing() {
        assert_eq!(FontKind::parse("main").unwrap(), FontKind::Main);
        assert_eq!(FontKind::parse("sub").unwrap(), FontKind::Sub);
        assert!(FontKind::parse("other").is_err());
    }

    #[test]
    fn main_and_sub_use_different_files() {
        assert_ne!(
            FontKind::Main.file_name(),
            FontKind::Sub.file_name(),
            "主副字体必须是两个独立资源"
        );
        assert_ne!(FontKind::Main.family(), FontKind::Sub.family());
    }

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

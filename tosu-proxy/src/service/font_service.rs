//! 字体资源服务（B-07）。
//!
//! 主字体与副字体是**两个独立资源**，各自有自己的文件和版本。
//!
//! **版本化**：版本号由"文件修改时间 + 文件大小"派生，不需要额外的表 ——
//! 覆盖写之后 mtime 必然变化，因此版本号自然变化；重启后从磁盘读出来还是同一个值。
//! 展示端用 `?v=<版本>` 请求字体，版本一变 URL 就变，浏览器不会继续用旧字体。
//!
//! **修 R10**：旧实现 `get_font_file()` 在文件已存在时用 `File::open`（只读）
//! 再去 `io::copy`，第二次上传必然失败并 `expect` 掉；而且它写的是名为 `font`
//! 的文件，展示端读的却是 `/LRC.otf` —— 上传的字体从来没被用上。
//! 现在一律以 `create`（截断）打开，并按种类写入展示端真正会读的路径。

use crate::error::{Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// 字体种类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontKind {
    /// 主字体
    Main,
    /// 副字体
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

    /// 展示端实际读取的文件名。
    ///
    /// 保留 `LRC.otf` 这个既有名字，避免改动展示页既有的静态资源约定；
    /// 副字体用 `tLRC.otf`（与 `utils/fonts.ts` 里已有的约定一致）。
    fn file_name(self) -> &'static str {
        match self {
            Self::Main => "LRC.otf",
            Self::Sub => "tLRC.otf",
        }
    }

    /// FontFace 家族名
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
    /// 派生版本号；文件不存在时为空串
    pub version: String,
    pub size: u64,
    /// 供展示端直接使用的 URL（含版本参数）
    pub url: String,
}

/// 版本号 = mtime(ms) + size。覆盖写之后必然变化，且跨重启稳定。
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

/// 覆盖写入字体。**以 create 打开（截断）**，因此连续上传不同大小的字体不会
/// 留下尾部残留，也不会像旧实现那样在第二次上传时因只读句柄失败。
pub async fn save(kind: FontKind, bytes: &[u8]) -> Result<FontInfo> {
    if bytes.is_empty() {
        return Err(Error::Runtime("invalid_param:字体文件为空".into()));
    }
    // 基本格式判断：至少要是 sfnt/woff 家族，避免把随便一个文件传上来
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

/// 字体魔数判断
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

/// 读取字体字节；不存在返回 `None`
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

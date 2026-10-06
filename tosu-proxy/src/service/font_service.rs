use crate::error::{Error, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tokio::io::AsyncWriteExt;

/// 字体显示名缓存：路径 -> (版本, 解析出的 displayName)
static DISPLAY_NAME_CACHE: OnceLock<Mutex<HashMap<PathBuf, (String, String)>>> = OnceLock::new();

/// **上传字体**的存放目录（相对工作目录）。
///
/// **为什么必须独立于默认字体**：早期版本把上传字体直接写成工作目录的
/// `LRC.otf` / `tLRC.otf` —— 而这两个路径正是随包默认字体（单文件发行时由
/// `ensure_runtime_dir` 从内嵌资源释放）。于是"上传一次字体"就等于把默认字体
/// 文件永久覆盖掉：此后选择"默认字体"加载到的其实是上传字体，重启也回不来。
/// 上传落到独立目录后，两者从**文件路径**上就彻底分开，互不覆盖。
pub const UPLOAD_DIR: &str = "uploaded";

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

    /// **随包默认字体**的文件名（`static/LRC.otf`、内嵌资源里的同名文件）。
    ///
    /// 同时被静态回退路由使用（`/lyrics/LRC.otf`、`/lyrics/tLRC.otf`），
    /// 保证"默认字体在哪"只有一个说法。
    /// 上传字体**不再**写到这里，见 `UPLOAD_DIR` / `upload_path()`。
    pub fn file_name(self) -> &'static str {
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

    /// 上传字体的文件路径：`<工作目录>/uploaded/LRC.otf`。
    ///
    /// 与随包默认字体（`static/LRC.otf`、内嵌 `LRC.otf`）完全分离，
    /// 上传只会覆盖上一次的上传，永远不会碰到默认字体资源。
    pub fn upload_path(self) -> PathBuf {
        let dir = std::env::current_dir().expect("cannot get current directory");
        dir.join(UPLOAD_DIR).join(self.file_name())
    }
}

/// 历史兼容：把早期写在 `./LRC.otf` / `./tLRC.otf`（与默认字体同路径）里的
/// **上传字体**搬到新的上传目录，返回实际迁移的文件名（供调用方打日志）。
///
/// `is_program_default(name, bytes)` 由调用方判断"这份字节是不是程序自带的默认字体"：
/// 是 → 该文件只是释放出来的默认资源，不是上传字体，不搬。
///
/// 只做复制（不删原文件）：搬完之后由程序资源释放流程把默认字体写回原路径，
/// 既保住了用户上传的字体，又把被覆盖的默认字体自愈回来。
pub fn migrate_legacy_uploaded_fonts(
    dir: &Path,
    is_program_default: impl Fn(&str, &[u8]) -> bool,
) -> Vec<String> {
    let mut migrated = Vec::new();
    for kind in [FontKind::Main, FontKind::Sub] {
        let name = kind.file_name();
        let legacy = dir.join(name);
        let target = dir.join(UPLOAD_DIR).join(name);
        if target.exists() || !legacy.is_file() {
            continue;
        }
        let Ok(bytes) = std::fs::read(&legacy) else {
            continue;
        };
        if is_program_default(name, &bytes) {
            continue;
        }
        if std::fs::create_dir_all(dir.join(UPLOAD_DIR)).is_err() {
            continue;
        }
        if std::fs::copy(&legacy, &target).is_ok() {
            migrated.push(name.to_string());
        }
    }
    migrated
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub kind: String,
    /// **FontFace 名**（`LRC` / `LRC-Sub`）：CSS 与字体注册用的就是它
    pub family: String,
    /// 字体文件内部的真实名称（`name` 表 nameID 4 → 1），**仅供 UI 显示**。
    /// 解析不到时回落为 `family`，不会为空 —— UI 名与 FontFace family 必须分开。
    pub display_name: String,
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
    let path = kind.upload_path();
    let (version, size) = version_of(&path);
    let exists = !version.is_empty();
    FontInfo {
        kind: kind.as_str().to_string(),
        family: kind.family().to_string(),
        display_name: display_name_for(&path, kind),
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

/// UI 显示名：读字体文件 `name` 表里的真实名称；文件不存在或解析失败时回落 family。
fn display_name_for(path: &Path, kind: FontKind) -> String {
    let fallback = kind.family().to_string();
    let version = version_of(path).0;
    if version.is_empty() {
        return fallback;
    }

    // **版本化缓存**：解析需要整份读取字体文件（主 + 副实测合计约 16.7 MB），而
    // `/api/font/info` 会在 `/lyrics` 挂载时经 loadFont() 被调用；每次都读盘解析
    // 会让该接口慢到 7~8 ms（约为 /api/status 的 9 倍），属于关键路径上的白开销。
    // 以 mtime+size 作版本号，只有字体真被覆盖时才重新解析。
    let cache = DISPLAY_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some((cached_version, name)) = guard.get(path) {
            if *cached_version == version {
                return name.clone();
            }
        }
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

/// 从 OTF/TTF 的 `name` 表取字体内部名称。
///
/// 优先级：nameID **4（Full name）→ 1（Family）**；同一 nameID 内优先
/// **platform 3（Windows，UTF-16BE）**，其次 platform 1/encoding 0（Mac Roman）。
/// 纯字节解析，不引入任何新依赖；结构异常时返回 None 由调用方回落。
fn font_display_name(bytes: &[u8]) -> Option<String> {
    // sfnt 头：version(4) + numTables(2)，随后 numTables 条 16 字节 table record
    let num_tables = u16::from_be_bytes([*bytes.get(4)?, *bytes.get(5)?]) as usize;

    // 定位 name 表：record = tag(4) + checksum(4) + offset(4) + length(4)
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

    // name 表头：format(2) + count(2) + stringOffset(2)
    let count = u16::from_be_bytes([*table.get(2)?, *table.get(3)?]) as usize;
    let storage = u16::from_be_bytes([*table.get(4)?, *table.get(5)?]) as usize;

    for want_id in [4u16, 1u16] {
        let mut mac_fallback: Option<String> = None;
        for i in 0..count {
            // 每条 name record：platform(2) encoding(2) language(2) nameID(2) length(2) offset(2)
            let rec = 6 + i * 12;
            let platform = u16::from_be_bytes([*table.get(rec)?, *table.get(rec + 1)?]);
            let encoding = u16::from_be_bytes([*table.get(rec + 2)?, *table.get(rec + 3)?]);
            let name_id = u16::from_be_bytes([*table.get(rec + 6)?, *table.get(rec + 7)?]);
            if name_id != want_id {
                continue;
            }
            let length = u16::from_be_bytes([*table.get(rec + 8)?, *table.get(rec + 9)?]) as usize;
            let offset = u16::from_be_bytes([*table.get(rec + 10)?, *table.get(rec + 11)?]) as usize;
            let start = storage.checked_add(offset)?;
            let Some(raw) = table.get(start..start.checked_add(length)?) else {
                continue;
            };
            let decoded = match (platform, encoding) {
                // Windows / Unicode BMP：UTF-16BE
                (3, _) => decode_utf16_be(raw),
                // Mac Roman：近似按 Latin-1 读，够用于显示
                (1, 0) => Some(raw.iter().map(|b| *b as char).collect()),
                _ => None,
            };
            let Some(text) = decoded else { continue };
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            // Windows 平台最可靠，直接采用；Mac 的先记下来备用
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
    if raw.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = raw
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    let text = String::from_utf16(&units).ok()?;
    // 去掉字体名里偶见的控制字符
    Some(text.replace('\0', ""))
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

    let path = kind.upload_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
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
    tokio::fs::read(kind.upload_path()).await.ok()
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
        assert_ne!(
            FontKind::Main.upload_path(),
            FontKind::Sub.upload_path(),
            "主副上传字体必须是两个独立文件"
        );
    }

    /// 回归防线：上传字体路径**不能**是默认字体的路径。
    ///
    /// 历史上上传直接写工作目录的 `LRC.otf`，把随包默认字体永久覆盖掉，
    /// 「默认字体」于是变成上传字体 —— 这条断言保证不再回到那个状态。
    #[test]
    fn uploaded_font_never_shares_path_with_default_font() {
        for kind in [FontKind::Main, FontKind::Sub] {
            let upload = kind.upload_path();
            let parent = upload
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string());
            assert_eq!(
                parent.as_deref(),
                Some(UPLOAD_DIR),
                "上传字体必须落在独立的 {UPLOAD_DIR}/ 目录，而不是工作目录根"
            );
            assert_eq!(
                upload.file_name().and_then(|n| n.to_str()),
                Some(kind.file_name()),
                "上传目录里的文件名与默认字体同名，只是目录不同"
            );
        }
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

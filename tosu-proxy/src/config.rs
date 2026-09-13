use sea_orm::sqlx::types::chrono;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, stdin};
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;
use tracing::{Event, Level, error, info};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub static CONFIG_FRONTEND: &str = "lyrics/{**path}";
pub static CONFIG_ENDPOINT_WEBSOCKET: &str = "ws";
pub static CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT: &str = "setter";
pub static CONFIG_ENDPOINT_FONT: &str = "font";
pub static CONFIG_ENDPOINT_FONT_UPLOAD: &str = "upload";
pub static CONFIG_ENDPOINT_FONT_DOWNLOAD: &str = "download";
pub static CONFIG_ENDPOINT_AUDIO_LEN: &str = "audio/len";
pub static CONFIG_ENDPOINT_LYRIC: &str = "lyric";
pub static CONFIG_ENDPOINT_LYRIC_UPLOAD: &str = "upload";
// 管理 HTTP(新增): 只读状态查询 + 展示控制, 不依赖控制台改造即可用 HTTP 独立验证
pub static CONFIG_ENDPOINT_STATUS: &str = "status";
pub static CONFIG_ENDPOINT_LYRICS: &str = "lyrics";
pub static CONFIG_ENDPOINT_LYRICS_CURRENT: &str = "current";
pub static CONFIG_ENDPOINT_LYRICS_SEARCH_RESULTS: &str = "search-results";
pub static CONFIG_ENDPOINT_LYRICS_SEARCH: &str = "search";
pub static CONFIG_ENDPOINT_LYRICS_PREVIEW: &str = "preview";
pub static CONFIG_ENDPOINT_LYRICS_SOURCE: &str = "source";
pub static CONFIG_ENDPOINT_LYRICS_OFFSET: &str = "offset";
pub static CONFIG_ENDPOINT_LYRICS_UPLOAD: &str = "upload";
pub static CONFIG_ENDPOINT_LYRICS_TRANSLATION_CHECK: &str = "translation-check";
pub static CONFIG_ENDPOINT_DISPLAY: &str = "display";
pub static CONFIG_ENDPOINT_DISPLAY_CLEAR: &str = "clear";
pub static CONFIG_ENDPOINT_SETTINGS: &str = "settings";
pub static CONFIG_ENDPOINT_BLOCKS: &str = "blocks";
pub static CONFIG_ENDPOINT_CLIENTS: &str = "clients";
pub static CONFIG_ENDPOINT_CLIENTS_BLINK: &str = "blink";
pub static CONFIG_ENDPOINT_CLIENTS_SETTINGS: &str = "settings";
pub static CONFIG_ENDPOINT_CACHE: &str = "cache";
pub static CONFIG_ENDPOINT_CACHE_COUNT: &str = "count";
pub static CONFIG_ENDPOINT_CACHE_CLEANUP: &str = "cleanup";

static CONFIG_PATH: &str = "config.json5";
#[derive(Debug, Deserialize, Serialize)]
pub struct TosuConfig {
    pub url: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: String,
    #[serde(rename = "log", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    pub port: u16,
    pub database: String,
    /// 歌词缓存 TTL（小时）。缺省 30 天；填 0 表示不过期。
    #[serde(default)]
    pub lyric_cache_ttl_hours: Option<i64>,
    pub tosu: Option<TosuConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: "0.0.0.0".to_string(),
            log_level: None,
            port: 41280,
            database: "sqlite://lyric.db?mode=rwc".to_string(),
            lyric_cache_ttl_hours: None,
            tosu: Some(TosuConfig {
                url: "ws://127.0.0.1:24050/websocket/v2".to_string(),
            }),
        }
    }
}

impl Config {
    pub fn init_logger(&self) {
        let level = if let Some(level) = &self.log_level {
            Level::from_str(level)
                .expect("无法处理的配置 `log`, 仅支持 error, warn, info, debug, trace")
        } else if cfg!(debug_assertions) {
            Level::DEBUG
        } else {
            Level::INFO
        };

        tracing_subscriber::fmt()
            .compact()
            .with_timer(TimeFormat)
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_max_level(level)
            .with_test_writer()
            .init();
    }
}

struct TimeFormat;

impl tracing_subscriber::fmt::time::FormatTime for TimeFormat {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();
        now.format("%y-%m-%d %H:%M:%S").write_to(w)?;
        let ms = now.timestamp_subsec_millis() / 100;
        write!(w, ".{ms}")
    }
}

pub static GLOBAL_CONFIG: LazyLock<Config> = LazyLock::new(load_config);

fn load_config() -> Config {
    use config::FileFormat;
    let config_path = Path::new(CONFIG_PATH);
    if !config_path.exists() {
        return create_default_config(config_path);
    }

    let config = config::Config::builder()
        .add_source(config::File::new(CONFIG_PATH, FileFormat::Json5))
        .set_default("server", "0.0.0.0")
        .and_then(|b| b.set_default("port", 41280))
        .and_then(|b| b.set_default("database", "sqlite://lyric.db?mode=rwc"))
        .and_then(|b| b.build())
        .and_then(|c| c.try_deserialize::<Config>())
        .expect("配置加载失败");

    info!("配置加载成功");
    config
}

fn create_default_config(config_path: &Path) -> Config {
    let default_config = Config::default();

    match serde_json::to_string_pretty(&default_config) {
        Ok(config_str) => {
            info!("未找到配置文件, 正在生成默认配置, 请查看编辑后, 重启程序.");

            if let Err(err) = fs::write(config_path, config_str) {
                error!("无法创建配置文件: {}", err);
                info!("继续使用内存中的默认配置");
                return default_config;
            }

            info!("按任意键退出...");
            wait_for_key_press();
            std::process::exit(0);
        }
        Err(err) => {
            error!("无法序列化默认配置: {}", err);
            default_config
        }
    }
}

fn wait_for_key_press() {
    let mut buffer: [u8; 1] = [0; 1];
    let _ = stdin().read_exact(&mut buffer);
}

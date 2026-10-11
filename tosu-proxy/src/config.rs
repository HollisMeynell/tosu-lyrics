use config::FileFormat;
use nu_ansi_term::{Color, Style};
use sea_orm::sqlx::types::chrono;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{IsTerminal, Read, stdin, stdout};
use std::path::Path;
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};
use std::time::Instant;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber, error, info};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub static CONFIG_ENDPOINT_WEBSOCKET: &str = "ws";
pub static CONFIG_ENDPOINT_WEBSOCKET_NO_LYRIC_POINT: &str = "setter";
pub static CONFIG_ENDPOINT_FONT: &str = "font";
pub static CONFIG_ENDPOINT_FONT_LIST: &str = "list";
pub static CONFIG_ENDPOINT_FONT_UPLOAD: &str = "upload";
pub static CONFIG_ENDPOINT_AUDIO_LEN: &str = "audio/len";
pub static CONFIG_ENDPOINT_LYRIC: &str = "lyric";
pub static CONFIG_ENDPOINT_LYRIC_UPLOAD: &str = "upload";
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

static LOG_START: OnceLock<Instant> = OnceLock::new();

const SOURCE_COLOR: Color = Color::Rgb(255, 102, 170);
const TOSU_COLOR: Color = Color::Fixed(12);
const TIME_COLOR: Color = Color::Rgb(170, 170, 170);

impl Config {
    pub fn init_logger(&self) {
        LOG_START.get_or_init(Instant::now);

        let _ = tracing_subscriber::fmt()
            .event_format(SourceFormat {
                ansi: ansi_enabled(),
            })
            .with_writer(stdout)
            .try_init();
    }
}

fn elapsed() -> String {
    let duration = LOG_START.get().map(Instant::elapsed).unwrap_or_default();
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60,
        duration.subsec_millis()
    )
}

#[derive(Default)]
struct Message {
    text: String,
    raw: bool,
}

impl Message {
    fn as_str(&self) -> &str {
        if self.raw {
            return &self.text;
        }
        self.text.trim_matches('"')
    }
}

impl Visit for Message {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() != "message" {
            return;
        }
        self.text = value.to_owned();
        self.raw = true;
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() != "message" {
            return;
        }
        self.text = format!("{value:?}");
        self.raw = false;
    }
}

struct SourceFormat {
    ansi: bool,
}

impl SourceFormat {
    fn style(&self, color: Color) -> Style {
        if self.ansi {
            return color.normal();
        }
        Style::default()
    }

    fn prefix(&self, target: &str, level: &Level, stamp: &str) -> String {
        let separator = if self.ansi { "│" } else { "|" };
        if target == "tosu" {
            let style = self.style(TOSU_COLOR);
            return format!("{} {}", style.paint("tosu  "), style.paint(separator));
        }

        let level_color = match *level {
            Level::ERROR => Color::Rgb(255, 85, 85),
            Level::WARN => Color::Rgb(255, 184, 108),
            _ => Color::Rgb(80, 250, 123),
        };
        let source_style = self.style(SOURCE_COLOR);
        let level_separator = if self.ansi { "┃" } else { "  |" };
        format!(
            "{} {}         {}  {} ",
            source_style.paint("lyrics"),
            source_style.paint(separator),
            self.style(level_color).paint(level_separator),
            self.style(TIME_COLOR).paint(stamp),
        )
    }
}

impl<S, N> FormatEvent<S, N> for SourceFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let mut message = Message::default();
        event.record(&mut message);
        let prefix = self.prefix(metadata.target(), metadata.level(), &elapsed());

        for line in message.as_str().split('\n') {
            writeln!(writer, "{prefix} {line}")?;
        }
        Ok(())
    }
}

fn ansi_enabled() -> bool {
    if !stdout().is_terminal() {
        return false;
    }

    #[cfg(windows)]
    {
        nu_ansi_term::enable_ansi_support().is_ok()
    }
    #[cfg(not(windows))]
    {
        true
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

    let config_str = match serde_json::to_string_pretty(&default_config) {
        Ok(config_str) => config_str,
        Err(err) => {
            error!("无法序列化默认配置: {}", err);
            return default_config;
        }
    };

    info!("未找到配置文件, 正在生成默认配置, 请查看编辑后, 重启程序.");
    if let Err(err) = fs::write(config_path, config_str) {
        error!("无法创建配置文件: {}", err);
        info!("继续使用内存中的默认配置");
        return default_config;
    }

    info!("按任意键退出...");
    default_config
}

fn wait_for_key_press() {
    let mut buffer: [u8; 1] = [0; 1];
    let _ = stdin().read_exact(&mut buffer);
}

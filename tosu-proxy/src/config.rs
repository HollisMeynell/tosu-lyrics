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
        use std::sync::OnceLock;
        use std::time::Instant;
        use tracing::field::{Field, Visit};
        use tracing_subscriber::fmt::format::Writer;
        use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
        use tracing_subscriber::registry::LookupSpan;

        // 与 tosu 控制台一致的布局（去掉 tracing 默认的时间戳/INFO/模块/行号）：
        //   lyrics │         │ 00:00:00.123  消息     （中间一段留空，9 字符，与 tosu 的 `4.26.2  ` 对齐）
        //   tosu   │ 4.26.2  │ 00:00:00.449  消息     （中间一段原样保留 tosu 自身输出）
        // 级别只用第二根竖线的颜色表达：INFO 绿、WARN 黄、ERROR 红；状态线用粗竖线 ┃(U+2503)，
        let ansi = ansi_enabled();

        static START: OnceLock<Instant> = OnceLock::new();
        let _ = START.set(Instant::now());

        fn elapsed() -> String {
            let d = START.get().map(|s| s.elapsed()).unwrap_or_default();
            let secs = d.as_secs();
            format!(
                "{:02}:{:02}:{:02}.{:03}",
                secs / 3600,
                (secs % 3600) / 60,
                secs % 60,
                d.subsec_millis()
            )
        }

        const PINK: &str = "\u{1b}[38;2;255;102;170m";
        const BLUE: &str = "\u{1b}[94m";
        const GREEN: &str = "\u{1b}[38;2;80;250;123m";
        const YELLOW: &str = "\u{1b}[38;2;255;184;108m";
        const RED: &str = "\u{1b}[38;2;255;85;85m";
        const GRAY: &str = "\u{1b}[38;2;170;170;170m";
        const RESET: &str = "\u{1b}[0m";

        struct Message {
            text: String,
            raw: bool,
        }

        impl Visit for Message {
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.text = value.to_string();
                    self.raw = true;
                }
            }
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.text = format!("{value:?}");
                    self.raw = false;
                }
            }
        }

        struct SourceFormat {
            ansi: bool,
        }

        impl<S, N> FormatEvent<S, N> for SourceFormat
        where
            S: tracing::Subscriber + for<'a> LookupSpan<'a>,
            N: for<'a> FormatFields<'a> + 'static,
        {
            fn format_event(
                &self,
                _ctx: &FmtContext<'_, S, N>,
                mut w: Writer<'_>,
                event: &tracing::Event<'_>,
            ) -> std::fmt::Result {
                let meta = event.metadata();
                // `target: "tosu"` = 转发的 tosu 子进程原始输出
                let is_tosu = meta.target() == "tosu";

                let mut message = Message {
                    text: String::new(),
                    raw: false,
                };
                event.record(&mut message);
                let text = if message.raw {
                    message.text.clone()
                } else {
                    message.text.trim_matches('"').to_string()
                };
                let lines: Vec<&str> = text.split('\n').collect();

                let level_color = match *meta.level() {
                    tracing::Level::ERROR => RED,
                    tracing::Level::WARN => YELLOW,
                    _ => GREEN,
                };
                let stamp = elapsed();
                // 中段宽度：默认 9（使时间戳落在第 20 列），若已实测到 tosu 的时间戳列则与之对齐
                // 固定字符列，直接依据 tosu logger 的字符串拼接布局（packages/common/utils/logger.ts:194-199）：
                //   console.log(version, " ┃ ", time + " ", msg) 以【单空格】连接各参数，故 tosu 原始行可见为：
                //     "4.26.2" ␣ ␣ ┃ ␣ ␣ "00:00:00.403" ␣ ␣ msg
                //   即版本号后 2 空格、状态线后 2 空格，时间戳落在 0 基第 20 列（1 基第 21 列）。
                // 我方 tosu 行前缀 "tosu   │ " 为 9 列 → tosu 的 ┃ 落在第 18 列、时间戳第 21 列。
                // lyrics 前缀 6(标签)+1(空格)+1(│)=8 列，故版本栏留空必须为 9 字符（等价 " 4.26.2  " 的宽度），
                // 使 ┃ 同样落在第 18 列；其后 2 空格使时间戳同样落在第 21 列。
                let mid = 9usize;

                for (index, line) in lines.iter().enumerate() {
                    if self.ansi {
                        if is_tosu {
                            // tosu 自带版本号/时间戳/颜色，原样保留，只加左侧来源标签
                            write!(w, "{BLUE}{:<6}{RESET} {BLUE}\u{2502}{RESET} {line}", "tosu")?;
                        } else {
                            write!(
                                w,
                                "{PINK}{:<6}{RESET} {PINK}\u{2502}{RESET}{:mid$}{level_color}\u{2503}{RESET}  {GRAY}{stamp}{RESET}  {line}",
                                "lyrics",
                                "",
                                mid = mid
                            )?;
                        }
                    } else if is_tosu {
                        write!(w, "{:<6} | {line}", "tosu")?;
                    } else {
                        write!(
                            w,
                            "{:<6} | {:mid$} |  {stamp}  {line}",
                            "lyrics",
                            "",
                            mid = mid
                        )?;
                    }
                    if index + 1 < lines.len() {
                        writeln!(w)?;
                    }
                }
                writeln!(w)
            }
        }

        let _ = tracing_subscriber::fmt()
            .event_format(SourceFormat { ansi })
            .with_writer(std::io::stdout)
            .try_init();
    }
}

/// 是否给日志输出 ANSI 颜色转义。
///
/// 原先这里写死 `true`: Windows 上新开的 conhost 默认**没有**打开
/// `ENABLE_VIRTUAL_TERMINAL_PROCESSING`, 转义序列会被原样打印出来, 日志里
/// 就混进 `[2m` / `[33m` 这类可见字符。而从 PowerShell 启动时能正常显示颜色,
/// 是因为 exe 继承了宿主进程已经启用的 VT 模式。
///
/// 策略:
/// - stdout 不是终端(重定向到文件/管道) -> 关闭, 避免污染日志文件
/// - Windows: 读控制台模式, 已启用 VT 就直接用彩色; 未启用则尝试启用一次
///   (成功仍保留彩色), 只有启用失败(旧系统/非控制台)才关掉 ANSI
/// - 其它平台: 终端即认为支持
fn ansi_enabled() -> bool {
    use std::io::IsTerminal;

    if !std::io::stdout().is_terminal() {
        return false;
    }

    #[cfg(windows)]
    {
        win_console::enable_virtual_terminal()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// 直接声明 kernel32 的三个控制台函数, 避免为一次模式检查引入额外依赖。
#[cfg(windows)]
mod win_console {
    use std::ffi::c_void;

    /// `(DWORD)-11`, stdout 的标准句柄编号
    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
    /// 打开后控制台才会把 ANSI 转义序列当控制指令处理
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(n_std_handle: u32) -> *mut c_void;
        fn GetConsoleMode(h_console_handle: *mut c_void, lp_mode: *mut u32) -> i32;
        fn SetConsoleMode(h_console_handle: *mut c_void, dw_mode: u32) -> i32;
    }

    /// 控制台已支持 VT 返回 `true`; 未启用则尝试启用, 成功同样返回 `true`;
    /// 不是控制台句柄或启用失败返回 `false`。
    pub(super) fn enable_virtual_terminal() -> bool {
        // SAFETY: 只调用 kernel32 的稳定导出, 句柄由 GetStdHandle 返回并在本次调用内
        // 使用, 不跨线程保存; mode 是本栈变量, 三个函数不会保留其地址。
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle.is_null() || handle as isize == -1 {
                return false;
            }
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                // 不是控制台(例如 stdout 已被重定向到管道)
                return false;
            }
            if mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0 {
                return true;
            }
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
        }
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

            // 首次启动：默认配置已在上面写入磁盘，这里【直接返回它】让本次进程继续初始化，
            // 与第二次启动行为一致（不再 exit，避免"首次启动无日志、无监听、需重启"）。
            default_config
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

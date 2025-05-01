use std::fs;
use std::io::{Read, stdin};
use std::path::Path;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tracing::{info, error};

pub static CONFIG_ENDPOINT_WEBSOCKET: &str = "ws";
static CONFIG_PATH: &str = "config.json5";
#[derive(Debug, Deserialize, Serialize)]
pub struct TosuConfig {
    pub url: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub server: String,
    pub port: u16,
    pub database: String,
    pub tosu: Option<TosuConfig>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: "0.0.0.0".to_string(),
            port: 41280,
            database: "sqlite://lyric.db?mode=rwc".to_string(),
            tosu: Some(TosuConfig {
                url: "ws://127.0.0.1:24050/websocket/v2".to_string(),
            }),
        }
    }
}

pub static GLOBAL_CONFIG: LazyLock<Settings> = LazyLock::new(|| load_config());

fn load_config() -> Settings {
    use config::{Config, FileFormat};
    let config_path = Path::new(CONFIG_PATH);
    if !config_path.exists() {
        return create_default_config(config_path);
    }

    match Config::builder()
        .add_source(config::File::new(CONFIG_PATH, FileFormat::Json5))
        .set_default("server", "0.0.0.0")
        .and_then(|b| b.set_default("port", 41280))
        .and_then(|b| b.set_default("database", "sqlite://lyric.db?mode=rwc"))
        .and_then(|b| b.build())
        .and_then(|c| c.try_deserialize::<Settings>())
    {
        Ok(config) => {
            info!("配置加载成功");
            config
        }
        Err(err) => {
            error!("配置加载失败: {}", err);
            info!("使用默认配置");
            Settings::default()
        }
    }
}

fn create_default_config(config_path: &Path) -> Settings {
    let default_config = Settings::default();

    match serde_json::to_string_pretty(&default_config) {
        Ok(config_str) => {
            info!("未找到配置文件，正在生成默认配置。生成后请重启程序。（你可以编辑它）");

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

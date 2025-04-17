use serde_json::Value;
use std::fs;
use std::io::{Read, Write, stdin};
use std::path::PathBuf;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tracing::info;

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
                url: "ws://127.0.0.1:24050".to_string(),
            }),
        }
    }
}

pub static GLOBAL_CONFIG: LazyLock<Settings> = LazyLock::new(|| {
    use config::{Config, FileFormat};
    use std::path::Path;
    let config_path = Path::new(CONFIG_PATH);
    if !config_path.exists() {
        let default_config = Settings::default();
        let default_config_str = serde_json::to_string_pretty(&default_config).unwrap();
        info!(
            "Config file not found. Generating default configuration. Please edit and restart the program."
        );
        fs::write(config_path, default_config_str).expect("can not create config file");
        info!("Press any key to exit...");
        wait_for_key_press();
        std::process::exit(0);
    }
    let config = Config::builder()
        .add_source(config::File::new(CONFIG_PATH, FileFormat::Json5))
        .set_default("server", "0.0.0.0")
        .unwrap()
        .set_default("port", 41280)
        .unwrap()
        .set_default("database", "sqlite://lyric.db?mode=rwc")
        .unwrap()
        .build()
        .expect("can not parse config file!")
        .try_deserialize::<Settings>()
        .unwrap();
    info!("config loaded successfully.");
    config
});

fn wait_for_key_press() {
    let mut buffer: [u8; 1] = [0; 1];
    let _ = stdin().read_exact(&mut buffer);
}

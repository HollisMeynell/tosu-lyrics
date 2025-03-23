use crate::get_local_path;
use actix_web::{web, HttpResponse};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tracing::log::{log, Level};
/******************* new **********************/

pub static CONFIG_ENDPOINT_WEBSOCKET: &str = "ws";
static CONFIG_PATH: &str = "config.json5";

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub server: String,
    pub port: u16,
    pub database: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: "0.0.0.0".to_string(),
            port: 41280,
            database: "sqlite://lyric.db?mode=rwc".to_string(),
        }
    }
}

pub static GLOBAL_CONFIG: LazyLock<Settings> = LazyLock::new(|| {
    use config::{Config, FileFormat};
    use std::path::Path;
    let config_path = Path::new(CONFIG_PATH);
    if !config_path.exists() {
        let default_config = Settings::default();
        let default_config_str = serde_json::to_string(&default_config).unwrap();
        log!(Level::Info, "config file not exists, create default file");
        fs::write(config_path, default_config_str).expect("can not create config file");
        return default_config;
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
    log!(Level::Info, "load config ok.");
    config
});

/******************* old **********************/

static LOCAL_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = match get_local_path() {
        None => {
            panic!("can not read local file")
        }
        Some(path) => path,
    };
    path.push("config.json");
    path
});

fn save_file(val: String) {
    let path = LOCAL_PATH.as_path();
    if let Ok(mut file) = fs::File::create(path) {
        let _ = file.write(val.as_bytes());
    } else {
        panic!("can not write to local file")
    }
}

fn read_file() -> String {
    let path = LOCAL_PATH.as_path();
    fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string())
}

pub async fn handler_put(req: web::Json<Value>) -> HttpResponse {
    save_file(req.to_string());
    HttpResponse::Ok().json(req)
}
pub async fn handler_get() -> HttpResponse {
    let txt = read_file();
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(txt)
}

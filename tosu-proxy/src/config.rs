use crate::get_local_path;
use actix_web::{web, HttpResponse};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

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

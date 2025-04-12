use actix_files::{Files, NamedFile};
use actix_web::{HttpResponse, Result};
use actix_web::dev::{ServiceRequest, ServiceResponse, fn_service};
use std::path::Path;
use std::sync::LazyLock;

static STATIC_FILE_PATHS: [&str; 3] = [
    "./static/lyrics/index.html",
    "./static/index.html",
    "./index.html",
];

fn existing_files() -> Option<&'static str> {
    for path in STATIC_FILE_PATHS {
        if Path::new(path).is_file() {
            return Some(path);
        }
    }
    None
}

static STATIC_FILE: LazyLock<&'static str> = LazyLock::new(|| existing_files().unwrap());

async fn fallback(req: ServiceRequest) -> Result<ServiceResponse> {
    let (req, _) = req.into_parts();
    let file = NamedFile::open_async(*STATIC_FILE).await?;
    let res = file.into_response(&req);
    Ok(ServiceResponse::new(req, res))
}

pub fn handle() -> Files {
    let path = match existing_files() {
        None => {
            println!("can not found static files, exit...");
            panic!();
        }
        Some(path) => {
            let len = path.len();
            let mut path = path.to_string();
            path.truncate(len - 10);
            path
        }
    };
    Files::new("/lyrics", path)
        .index_file("index.html")
        .default_handler(fn_service(fallback))
}

pub async fn index() -> HttpResponse {
    HttpResponse::MovedPermanently()
        .append_header(("Location", "/lyrics"))
        .finish()
}
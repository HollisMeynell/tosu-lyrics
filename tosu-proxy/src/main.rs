use actix_files::NamedFile;
use actix_web::body::BoxBody;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::process::{Child, Command as TokioCommand};

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::new()
});

static LOCAL_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = std::env::current_dir().unwrap();
    path.push("static");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    } else {
        if !path.is_dir() {
            let _ = fs::remove_file(&path);
            let _ = fs::create_dir_all(&path);
        }
    }
    path
});

#[derive(Deserialize)]
struct ApiRequest {
    url: String,
    method: Option<String>,
    header: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
}

#[derive(Serialize)]
struct ApiResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

async fn proxy_handler(req: web::Json<ApiRequest>) -> impl Responder {
    let req = req.into_inner();
    let url = &req.url;
    let method = req.method.as_deref().unwrap_or("GET");
    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => CLIENT.get(url),
        "POST" => CLIENT.post(url),
        "PUT" => CLIENT.put(url),
        "DELETE" => CLIENT.delete(url),
        "PATCH" => CLIENT.patch(url),
        _ => return HttpResponse::BadRequest().body("Unsupported HTTP method")
    };

    if let Some(header) = &req.header {
        for (key, val) in header {
            request_builder = request_builder.header(key, val);
        }
    }

    if let Some(body) = &req.body {
        request_builder = request_builder.body(body.clone());
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response.text().await.unwrap_or_default();

            HttpResponse::Ok().json(ApiResponse {
                status,
                headers,
                body,
            })
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("Request failed: {:?}", err))
        }
    }
}

enum FileEnum {
    Dir(Vec<PathBuf>),
    Jump,
    Work(PathBuf),
    None,
}
impl FileEnum {
    fn from_path_str(path_str: &str) -> Self {
        use FileEnum::*;
        let mut path = LOCAL_PATH.clone();
        path.push(path_str);

        if path.is_file() {
            return FileEnum::from_path(&path);
        }

        if path.is_dir() {
            let index = path.join("index.html");
            return if index.is_file() {
                Jump
            } else {
                FileEnum::from_dir(&path)
            };
        }

        None
    }

    fn from_path(path: &Path) -> Self {
        if let Ok(abs) = path.canonicalize() {
            Self::Work(abs)
        } else {
            Self::None
        }
    }

    fn from_dir(dir: &Path) -> Self {
        let dir = fs::read_dir(dir);
        if dir.is_err() { return Self::None; }
        let dir = dir.unwrap();
        let mut dirs = Vec::new();
        for actix_files in dir {
            match actix_files {
                Ok(f) => {
                    dirs.push(f.path())
                }
                Err(_) => { continue }
            };
        }
        Self::Dir(dirs)
    }
}

enum DefaultResponse {
    File(NamedFile),
    Response(HttpResponse),
}

impl Responder for DefaultResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            DefaultResponse::File(data) => {
                data.respond_to(req)
            }
            DefaultResponse::Response(data) => {
                data
            }
        }
    }
}

async fn default_handler(path: web::Path<String>) -> impl Responder {
    let path = path.as_str();
    let file = FileEnum::from_path_str(path);
    match file {
        FileEnum::Dir(paths) => {
            let mut str = String::new();
            for path in paths {
                let name = path.file_name().and_then(OsStr::to_str).unwrap_or("???");
                str.push_str(&format!(r#"<a href="./{name}">{name}</a></br>"#));
            }
            DefaultResponse::Response(HttpResponse::Ok().body(str))
        }
        FileEnum::Jump => {
            let mut url = String::new();
            if path.is_empty() {
                url.push_str("/index.html");
            } else {
                url.push('/');
                url.push_str(path);
                if !path.ends_with('/') {
                    url.push('/');
                }
                url.push_str("index.html");
            }
            DefaultResponse::Response(
                HttpResponse::Found()
                    .append_header(("Location", url))
                    .finish()
            )
        }
        FileEnum::Work(path) => {
            match NamedFile::open(path) {
                Ok(actix_files) => {
                    DefaultResponse::File(actix_files)
                }
                Err(_) => {
                    DefaultResponse::Response(HttpResponse::BadRequest().body("not found"))
                }
            }
        }
        FileEnum::None => {
            DefaultResponse::Response(HttpResponse::BadRequest().body("not found"))
        }
    }
}

fn start_tosu() -> Option<Child> {
    let path = std::env::current_dir().unwrap();
    let dir = path.read_dir();
    if dir.is_err() { return None; }
    for item in dir.unwrap() {
        if item.is_err() { continue; }
        let item = item.unwrap();
        if let Some(name) = item.file_name().to_str() {
            if name == "tosu" || name == "tosu.exe" {
                println!("find tosu, start...");
                let start = TokioCommand::new(item.path()).spawn();
                if start.is_err() {
                    println!("start failed");
                } else {
                    return Some(start.unwrap());
                }
            }
        }
    }
    None
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const PORT: u16 = 41280;

    let tosu = start_tosu();

    let url = format!("http://127.0.0.1:{PORT}");
    println!("application started!");
    println!("start: {url}");
    let _ = webbrowser::open(&url);

    HttpServer::new(|| {
        let api = web::resource("/api/proxy").route(web::post().to(proxy_handler));
        let default = web::resource("/{path:.*}").route(web::get().to(default_handler));
        App::new()
            .service(api)
            .service(default)
    })
        .bind(("0.0.0.0", PORT))?
        .run().await?;

    if let Some(mut tosu) = tosu {
        let _ = tosu.kill();
    }

    println!("shutdown!");

    Ok(())
}

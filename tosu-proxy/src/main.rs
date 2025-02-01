mod config;
mod proxy;
mod websocket;

use actix_web::{web, App, HttpServer};
use std::path::PathBuf;
use tokio::process::{Child, Command as TokioCommand};

const ENV_PORT: &str = "TOSU_PROXY_PORT";
const DEFAULT_PORT: u16 = 41280;

pub fn get_local_path() -> Option<PathBuf> {
    let path = std::env::current_dir().unwrap();
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

fn start_tosu() -> Option<Child> {
    let dir = get_local_path();
    if dir.is_none() {
        return None;
    }
    let dir = match dir?.read_dir() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    for item in dir {
        if item.is_err() {
            continue;
        }
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

fn str_to_port(port_str: &str, default: u16) -> u16 {
    port_str.parse::<u16>().unwrap_or_else(|_| {
        println!("[{port_str}] is unable to port");
        default
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use std::env;

    let port: u16 = env::var(ENV_PORT)
        .map(|port_str| str_to_port(&port_str, DEFAULT_PORT))
        .unwrap_or_else(|_| DEFAULT_PORT);

    let tosu = start_tosu();

    let url = format!("http://127.0.0.1:{port}");
    println!("application started!\n{url}\n");

    HttpServer::new(|| {
        let proxy = web::resource("/api/proxy").route(web::post().to(proxy::handler));
        let websocket = web::resource("/api/ws").route(web::get().to(websocket::handle));
        let config = web::resource("/api/config")
            .route(web::put().to(config::handler_put))
            .route(web::get().to(config::handler_get));

        App::new().service(proxy).service(websocket).service(config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    if let Some(mut tosu) = tosu {
        let _ = tosu.kill();
    }

    println!("shutdown!");

    Ok(())
}

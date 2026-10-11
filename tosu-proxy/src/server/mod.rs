mod audio;
mod blocks;
mod cache;
mod clients;
mod file;
mod font;
mod lyric;
mod lyrics;
pub mod response;
mod settings;
mod status;
mod websocket;

use crate::config::GLOBAL_CONFIG;
use crate::error::*;
use crate::server::audio::get_audio_route;
use crate::server::blocks::get_blocks_route;
use crate::server::cache::get_cache_route;
use crate::server::clients::get_clients_route;
use crate::server::file::get_file_route;
use crate::server::font::get_font_route;
use crate::server::lyric::get_lyric_route;
use crate::server::lyrics::get_lyrics_route as get_content_lyrics_route;
use crate::server::settings::get_settings_route;
use crate::server::status::{get_display_route, get_status_route};
use crate::server::websocket::get_ws_route;
use reqwest::Url;
use salvo::prelude::{Redirect, Router, Server, TcpListener};
use salvo::server::ServerHandle;
use salvo::{Listener, Response, handler};
use std::sync::OnceLock;
use tracing::{info, warn};

static SERVER_HANDLE: OnceLock<ServerHandle> = OnceLock::new();
pub use websocket::ALL_SESSIONS;

#[handler]
async fn root_redirect(res: &mut Response) {
    res.render(Redirect::permanent("/lyrics"));
}

// 字体可达数 MiB，需超过框架默认请求体上限。
pub(crate) const MAX_UPLOAD_BODY: usize = 32 * 1024 * 1024;

pub async fn start_server() {
    spawn_tosu_linked();
    use_local_temp_dir();
    salvo::http::request::set_global_secure_max_size(MAX_UPLOAD_BODY);
    let api_router = Router::with_path("api")
        .push(get_font_route())
        .push(get_audio_route())
        .push(get_lyric_route())
        .push(get_status_route())
        .push(get_content_lyrics_route())
        .push(get_display_route())
        .push(get_settings_route())
        .push(get_blocks_route())
        .push(get_cache_route())
        .push(get_clients_route());
    let router = Router::new()
        .get(root_redirect)
        .push(get_ws_route())
        .push(api_router)
        .push(get_file_route());
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    info!("server start: http://127.0.0.1:{}", GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    open_controller_page();
    let server = Server::new(acceptor);
    let handle = server.handle();
    SERVER_HANDLE
        .set(handle)
        .map_err(|_| Error::Impossible)
        .expect("?");
    info!("web 服务器初始化完成");
    server.serve(router).await;
}

#[derive(rust_embed::RustEmbed)]
#[folder = "../embed"]
pub(crate) struct Assets;

// 配置、数据库和上传文件仍使用磁盘目录；前端资源由 Assets 直接提供。
pub fn ensure_runtime_dir() -> Vec<String> {
    let mut warn_message = vec![];

    let Ok(exe) = std::env::current_exe() else {
        warn_message.push("无法确定程序路径，跳过运行目录初始化".to_string());
        return warn_message;
    };

    let Some(base) = exe.parent() else {
        warn_message.push("无法确定程序目录，跳过运行目录初始化".to_string());
        return warn_message;
    };

    let dir = base.join("lyrics");
    if let Err(err) = std::fs::create_dir_all(&dir) {
        warn_message.push(format!("无法创建 {}: {err}", dir.display()));
        return warn_message;
    }

    if let Err(err) = std::env::set_current_dir(&dir) {
        warn_message.push(format!("无法切换到运行目录 {}: {err}", dir.display()));
    } else {
        warn_message.push(format!("运行目录: {}", dir.display()));
    }

    warn_message
}

fn tosu_already_running(port: u16) -> bool {
    let probe = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    for _ in 0..6 {
        if std::net::TcpStream::connect_timeout(&probe, std::time::Duration::from_millis(250))
            .is_ok()
        {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    false
}
fn tosu_port(url: &str) -> Option<u16> {
    let url = Url::parse(url).ok()?;
    url.port_or_known_default().filter(|port| *port != 0)
}

fn spawn_tosu_linked() {
    let Some(tosu_config) = GLOBAL_CONFIG.tosu.as_ref() else {
        return;
    };
    let port = tosu_port(&tosu_config.url).unwrap_or_else(|| {
        warn!("无法从 tosu.url 解析端口，使用默认端口 24050");
        24050
    });
    if tosu_already_running(port) {
        info!("检测到 tosu 已在运行（127.0.0.1:{port}），跳过自动启动");
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };

    let tosu_exe = if cfg!(target_os = "windows") {
        "tosu.exe"
    } else {
        "tosu"
    };

    let tosu = dir.join(tosu_exe);
    if !tosu.is_file() {
        info!("同目录未找到 tosu，按独立模式运行");
        return;
    }

    let mut cmd = std::process::Command::new(&tosu);

    match cmd
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            info!("已启动同目录 tosu: {}）", dir.display());
            if let Some(out) = child.stdout.take() {
                std::thread::spawn(move || forward_output(out, "tosu"));
            }
            if let Some(err) = child.stderr.take() {
                std::thread::spawn(move || forward_output(err, "tosu"));
            }
            std::thread::spawn(move || {
                let _ = child.wait();
                info!("tosu 已退出");
            });
        }
        Err(err) => warn!("启动 tosu 失败: {err}"),
    }
}

fn forward_output<R: std::io::Read + Send + 'static>(reader: R, tag: &'static str) {
    use std::io::BufRead;
    for line in std::io::BufReader::new(reader).lines() {
        match line {
            Ok(text) => info!(target: "tosu", "{text}"),
            Err(_) => break,
        }
    }
}
fn use_local_temp_dir() {
    let Ok(dir) = std::env::current_dir().map(|dir| dir.join("temp")) else {
        warn!("无法确定程序目录，上传临时文件将继续使用系统 %TEMP%");
        return;
    };
    if let Err(err) = std::fs::create_dir_all(&dir) {
        warn!("无法创建上传临时目录 {}: {err}", dir.display());
        return;
    }
    unsafe {
        std::env::set_var("TMP", &dir);
        std::env::set_var("TEMP", &dir);
    }
    info!("上传临时目录: {}", dir.display());
}

pub fn close_server() {
    if let Some(handle) = SERVER_HANDLE.get() {
        handle.stop_graceful(None);
    }
}

fn open_controller_page() {
    let url = format!(
        "http://127.0.0.1:{}/lyrics/controller/content",
        GLOBAL_CONFIG.port
    );
    #[cfg(target_os = "linux")]
    let _ = open_in_browser(&url);

    #[cfg(not(target_os = "linux"))]
    match open_in_browser(&url) {
        Ok(()) => info!("已用默认浏览器打开 {url}"),
        Err(err) => warn!("无法自动打开浏览器({url}): {err}"),
    }
}

#[cfg(target_os = "windows")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "linux")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::tosu_port;

    #[test]
    fn parses_websocket_ports() {
        assert_eq!(tosu_port("ws://127.0.0.1:25000/websocket/v2"), Some(25000));
        assert_eq!(tosu_port("wss://localhost:24443/websocket/v2"), Some(24443));
        assert_eq!(tosu_port("ws://[::1]:25000/websocket/v2"), Some(25000));
        assert_eq!(tosu_port("ws://localhost/websocket/v2"), Some(80));
        assert_eq!(tosu_port("wss://localhost/websocket/v2"), Some(443));
    }

    #[test]
    fn rejects_invalid_websocket_ports() {
        for url in [
            "invalid",
            "ws://localhost:99999",
            "ws://localhost:0",
            "http://localhost:25000",
        ] {
            assert_eq!(tosu_port(url), None, "{url}");
        }
    }
}

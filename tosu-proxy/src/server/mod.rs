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
use salvo::prelude::Redirect;
use salvo::server::ServerHandle;
use salvo::{Response, handler};
use std::sync::OnceLock;
use tracing::{info, warn};

static SERVER_HANDLE: OnceLock<ServerHandle> = OnceLock::new();
pub use websocket::ALL_SESSIONS;

#[handler]
async fn root_redirect(res: &mut Response) {
    res.render(Redirect::permanent("/lyrics"));
}

/// 上传体积上限：32 MiB。
///
/// **为什么需要显式设置**：salvo 默认的请求体上限远小于字体文件
/// （实测 5 MB 的 LRC.otf 会在 448 KB 处被截断，服务端回 400、
/// 浏览器侧表现为 `TypeError: Failed to fetch`）。
/// 字体动辄几 MB，必须放宽；歌词文本用不到这么大，但多留余量没有副作用。
pub(crate) const MAX_UPLOAD_BODY: usize = 32 * 1024 * 1024;

pub async fn start_server() {
    use salvo::prelude::*;
    ensure_runtime_dir();
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

/// 内嵌程序资源（路径 → 内容）：这些是**程序资源**，可由程序维护/补齐。
///
/// 由 `rust-embed` 在编译时从仓库根目录 `embed/` 嵌入。
/// `embed/` 由 `just assemble-embed` 在 `cargo build` 之前组装。
#[derive(rust_embed::RustEmbed)]
#[folder = "../embed"]
pub(crate) struct Assets;

/// 取一份内嵌的程序自带资源（默认字体等）的副本，没有则 `None`。
fn embedded_resource(rel: &str) -> Option<Vec<u8>> {
    Assets::get(rel).map(|f| f.data.to_vec())
}

/// 单文件发行：确保 exe 同目录的 `lyrics/` 运行目录存在，释放默认资源。
///
/// - 无 `lyrics/` → 创建
/// - 释放内嵌的默认字体到 `font/` 目录（仅当不存在时）
/// - 迁移旧的 `uploaded/` 目录到 `font/`
/// - **绝不触碰** `lyric.db` / `config.json5`
/// - 最后把工作目录切到 `lyrics/`
pub fn ensure_runtime_dir() {
    let Ok(exe) = std::env::current_exe() else {
        warn!("无法确定程序路径，跳过 lyrics/ 资源检查");
        return;
    };
    let Some(base) = exe.parent() else {
        warn!("无法确定程序目录，跳过 lyrics/ 资源检查");
        return;
    };
    let dir = base.join("lyrics");
    if let Err(err) = std::fs::create_dir_all(&dir) {
        warn!("无法创建 {}: {err}", dir.display());
        return;
    }

    // 创建字体和临时目录
    let font_dir = dir.join(crate::service::font_service::FONT_DIR);
    let _ = std::fs::create_dir_all(&font_dir);
    let temp_dir = dir.join("temp");
    let _ = std::fs::create_dir_all(&temp_dir);

    // 迁移旧的 uploaded/ 目录到 font/
    crate::service::font_service::migrate_uploaded_to_font(&dir);

    // 释放 ffprobe（如果内嵌了且目标不存在）
    let ffprobe_name = if cfg!(target_os = "windows") {
        "ffprobe.exe"
    } else {
        "ffprobe"
    };
    let ffprobe_target = dir.join(ffprobe_name);
    if !ffprobe_target.exists()
        && let Some(bytes) = embedded_resource(ffprobe_name)
    {
        match std::fs::write(&ffprobe_target, &bytes) {
            Ok(()) => {
                info!("已释放: {}", ffprobe_target.display());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &ffprobe_target,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
            }
            Err(err) => warn!("释放 ffprobe 失败 {}: {err}", ffprobe_target.display()),
        }
    }

    // 删除旧的 resources.json（如果存在）
    let manifest = dir.join("resources.json");
    if manifest.exists() {
        let _ = std::fs::remove_file(&manifest);
    }

    if let Err(err) = std::env::set_current_dir(&dir) {
        warn!("无法切换到运行目录 {}: {err}", dir.display());
    } else {
        info!("运行目录: {}", dir.display());
    }
}

/// 自动联动同目录的 `tosu.exe`。
///
/// - 若 127.0.0.1:24050 已在监听 → 认为 tosu 已运行，**不重复启动**
/// - 同目录没有 `tosu.exe` → 按独立模式运行（不影响主程序）
/// - 启动时 `current_dir` 取 tosu.exe 所在目录，保证其相对路径/配置/DLL 依赖正确
/// - 非阻塞：不等待、不 kill；tosu 之后退出不会影响本进程
/// - 输出以 `[tosu]` 前缀转发到本进程日志，不吞日志
/// - 不改动 tosu.exe，也不影响用户手动启动 tosu
///   单次探测不够可靠（tosu 可能刚启动、尚未完成监听），因此做多次重试。
///   只有在**确认没有** tosu 时才会启动新实例，避免 EADDRINUSE。
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
fn spawn_tosu_linked() {
    const TOSU_PORT: u16 = 24050;
    let probe = std::net::SocketAddr::from(([127, 0, 0, 1], TOSU_PORT));
    if tosu_already_running(TOSU_PORT) {
        info!("检测到 tosu 已在运行（127.0.0.1:{TOSU_PORT}），跳过自动启动");
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    let tosu = dir.join("tosu.exe");
    if !tosu.is_file() {
        info!("同目录未找到 tosu.exe，按独立模式运行");
        return;
    }

    let mut cmd = std::process::Command::new(&tosu);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        /// 不为子进程创建新的控制台窗口
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // 刻意不设置 CREATE_NO_WINDOW：让 tosu 附着到本进程控制台，
        // 从而「关闭控制台窗口 / Ctrl+C」会同时传递给 tosu，避免其残留。
    }

    match cmd
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            info!("已启动同目录 tosu.exe（工作目录 {}）", dir.display());
            if let Some(out) = child.stdout.take() {
                std::thread::spawn(move || forward_output(out, "tosu"));
            }
            if let Some(err) = child.stderr.take() {
                std::thread::spawn(move || forward_output(err, "tosu"));
            }
            // 只回收退出状态，不阻塞主线程、不终止子进程
            std::thread::spawn(move || {
                let _ = child.wait();
                info!("tosu 已退出（不影响主程序运行）");
            });
        }
        Err(err) => warn!("启动 tosu.exe 失败（不影响主程序）: {err}"),
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
    // edition 2024 下 set_var 是 unsafe：这里在启动最早期、尚未起任何工作线程时
    // 调用，只影响本进程后续创建的 multipart 临时文件。
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

/// 用系统默认浏览器打开"歌词内容控制"页。
///
/// 调用点在 `TcpListener::bind()` 成功之后 —— 那时端口已经可以接受连接，
/// 不会出现浏览器比服务先起来的连不上问题。端口取自配置，不写死。
/// 项目原本没有打开浏览器的机制，这里用各平台自带的命令实现，不引入依赖。
fn open_controller_page() {
    let url = format!(
        "http://127.0.0.1:{}/lyrics/controller/content",
        GLOBAL_CONFIG.port
    );
    match open_in_browser(&url) {
        Ok(()) => info!("已用默认浏览器打开 {url}"),
        Err(err) => warn!("无法自动打开浏览器({url}): {err}"),
    }
}

#[cfg(target_os = "windows")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    /// CREATE_NO_WINDOW：不为新进程创建控制台窗口。
    ///
    /// 没有它时 `Command::new("cmd")` 会在屏幕上**闪出一个控制台黑框** —— 这正是
    /// "当前版本会弹窗、旧版本不会"的直接原因（旧版完全没有这段代码）。
    /// 只隐藏窗口，不改动任何安全策略，自动打开浏览器的功能保持不变。
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // 中间的空参数是 `start` 的窗口标题占位符：缺了它，带引号的 URL 会被当作标题
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
        .spawn()
        .map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
}

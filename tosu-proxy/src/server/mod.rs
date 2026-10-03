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
use crate::server::font::{get_font_route, get_static_font_route};
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
        // 必须在静态目录之前：否则 /lyrics/LRC.otf 找不到时会命中 StaticDir 的
        // index.html 回退，浏览器把 HTML 当字体解析（OTS parsing error）
        .push(get_static_font_route())
        .push(get_file_route())
        .push(api_router);
    let listener_url = format!("{}:{}", GLOBAL_CONFIG.server, GLOBAL_CONFIG.port);
    info!("server start: http://127.0.0.1:{}", GLOBAL_CONFIG.port);
    let acceptor = TcpListener::new(listener_url).bind().await;
    // 监听套接字已就绪，这时拉起浏览器不会再出现"打开过早、连不上"的问题
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

/// 把上传用的临时目录指到程序自身目录下的 `temp/`。
///
/// **为什么需要**：salvo 解析 multipart 时会先把内容写进
/// `tempfile::Builder::new().prefix("salvo_http_multipart").tempdir()`
/// （salvo_core `http/form.rs`），而它取的是 `std::env::temp_dir()`，也就是
/// 用户级 `%TEMP%`。这台机器上该目录不可写，实测返回
/// `I/O error: 拒绝访问。 (os error 5)`，于是**任何**字体上传都会失败：
/// 小文件在读完 body 后才失败，表现为 400 + 错误体；5 MB 的文件在 body 发完
/// 之前服务端就已断开，浏览器只看到 `ERR_CONNECTION_ABORTED`。
/// 换到程序自己目录下即可绕开，且不依赖外部环境。
/// 程序资源版本：资源内容变化时递增，用于判断 `lyrics/` 内程序资源是否需要补齐。
///
/// v3：上传字体改到 `uploaded/`，默认字体不再被上传覆盖 —— 递增一次版本，
/// 让已有安装把被覆盖的默认字体资源（`./LRC.otf` / `./tLRC.otf`）重新释放回来。
const RESOURCE_VERSION: &str = "3";

/// 内嵌程序资源（路径 → 内容）：这些是**程序资源**，可由程序维护/补齐。
include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));
const EMBEDDED_RESOURCES: &[(&str, &[u8])] = EMBEDDED;

/// 取一份内嵌的程序自带资源（默认字体等），没有则 `None`。
pub fn embedded_resource(rel: &str) -> Option<&'static [u8]> {
    EMBEDDED_RESOURCES
        .iter()
        .find(|(name, _)| *name == rel)
        .map(|(_, bytes)| *bytes)
}

/// 用户数据 / 用户配置：**绝不写入、绝不覆盖**。
const USER_FILES: &[&str] = &["lyric.db", "config.json5"];

/// 单文件发行：确保 exe 同目录的 `lyrics/` 运行目录存在且程序资源完整。
///
/// - 无 `lyrics/` → 创建
/// - 程序资源缺失或版本过期 → 只补齐该文件（不全量覆盖）
/// - **绝不触碰** `lyric.db` / `config.json5`
/// - 最后把工作目录切到 `lyrics/`：既有相对路径（config.json5 / lyric.db /
///   static / temp）自然全部落在 `lyrics/` 内，无需改动其它模块。
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
    let manifest = dir.join("resources.json");
    let version_ok = std::fs::read_to_string(&manifest)
        .map(|text| text.contains(RESOURCE_VERSION))
        .unwrap_or(false);

    // 先做历史兼容：早期上传字体写在 `./LRC.otf` / `./tLRC.otf`（与默认字体同路径），
    // 会被下面的资源释放覆盖掉。先把这类**上传字体**搬到 `uploaded/`，再释放默认字体，
    // 用户上传的字体不会丢，被覆盖的默认字体也回到原位。
    for name in crate::service::font_service::migrate_legacy_uploaded_fonts(&dir, |name, bytes| {
        embedded_resource(name).map(|default| default == bytes).unwrap_or(false)
    }) {
        info!(
            "已把历史上传字体 {name} 迁移到 {}/",
            crate::service::font_service::UPLOAD_DIR
        );
    }

    for (rel, bytes) in EMBEDDED_RESOURCES {
        if USER_FILES.contains(rel) {
            continue;
        }
        let path = dir.join(rel);
        if version_ok && path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, bytes) {
            Ok(()) => info!("已释放程序资源: {}", path.display()),
            Err(err) => warn!("释放程序资源失败 {}: {err}", path.display()),
        }
    }
    if !version_ok {
        let body = format!("{{\n  \"version\": \"{RESOURCE_VERSION}\",\n  \"resources\": [\"LRC.otf\", \"tLRC.otf\"]\n}}\n");
        let _ = std::fs::write(&manifest, body);
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
/// 判断 127.0.0.1:port 是否已有正常运行的 tosu。
///
/// 单次探测不够可靠（tosu 可能刚启动、尚未完成监听），因此做多次重试。
/// 只有在**确认没有** tosu 时才会启动新实例，避免 EADDRINUSE。
fn tosu_already_running(port: u16) -> bool {
    let probe = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    for _ in 0..6 {
        if std::net::TcpStream::connect_timeout(&probe, std::time::Duration::from_millis(250)).is_ok() {
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

/// 把子进程输出逐行转发到本进程日志，带来源前缀。
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

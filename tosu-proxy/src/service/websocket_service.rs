//! （B-11）WS 管理入口已移除。
//!
//! 这里原本是 23 个 WS 管理命令的实现：setClear / setFont / getFont /
//! setFontSize / getFontSize / setAlignment / getAlignment / setColor / getColor /
//! setTranslationMain / getTranslationMain / setSecondShow / getSecondShow /
//! getLyricList / setLyricSource / getAllLyric / setBlock / getBlockList /
//! setUnblock / getCacheCount / setCacheClean / getLyricOffset / setLyricOffset。
//!
//! 它们的 HTTP 对等能力已经全部就位，因此整个入口被**删除**，
//! 而不是"内部转发到 HTTP 继续保留" —— 那只会留下一条隐藏的管理通道。
//!
//! 现在的分工：
//! - **管理**：一律走 HTTP（`/api/settings`、`/api/blocks`、`/api/cache`、
//!   `/api/lyrics/*`、`/api/display/clear`、`/api/clients/*`）
//! - **WS**：只做展示事件传输 —— 歌词推送、清屏、样式广播、定向 blink。
//!
//! 因此本文件不再存在任何"接收管理请求"的代码路径；
//! `server/websocket.rs` 也不再调用任何 dispatch 函数。

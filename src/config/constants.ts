
/**
 * 新版管理 HTTP 的基址。
 *
 * 用相对路径：开发时 vite 已把 `/api` 代理到后端，生产时页面由后端自身提供，
 * 两种情况都不需要写死主机与端口。
 */
export const BACKEND_API_BASE = "/api";

// 新版后端展示 WS 的真实路由是根路径 `/ws`(不是 /api/ws)。
// - 开发: 页面由 vite dev server 提供, 后端在默认端口, 显式指向它
// - 生产: 页面由后端自身提供, 直接用当前 host, 改端口/OBS 刷新都不会失联
const BACKEND_DEV_HOST = "127.0.0.1:41280";
const wsHost = import.meta.env.DEV ? BACKEND_DEV_HOST : window.location.host;
const wsScheme = window.location.protocol === "https:" ? "wss" : "ws";
export const BACKEND_WEBSOCKET_URL = `${wsScheme}://${wsHost}/ws`;

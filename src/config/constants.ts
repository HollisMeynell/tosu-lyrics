
// 相对路径：开发时 vite 代理 /api，生产时页面由后端自身提供，无需写死主机端口。
export const BACKEND_API_BASE = "/api";

// WS 路由是根路径 `/ws`（不是 /api/ws）。
// 开发: vite dev server 提供页面，后端在默认端口，显式指向它。
// 生产: 页面由后端自身提供，直接用当前 host。
const BACKEND_DEV_HOST = "127.0.0.1:41280";
const wsHost = import.meta.env.DEV ? BACKEND_DEV_HOST : window.location.host;
const wsScheme = window.location.protocol === "https:" ? "wss" : "ws";
export const BACKEND_WEBSOCKET_URL = `${wsScheme}://${wsHost}/ws`;

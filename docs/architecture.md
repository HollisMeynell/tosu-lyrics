<!-- markdownlint-disable MD028 MD033 -->
# 架构

```
                    ┌──────────────────────────────────────┐
   osu! ──► tosu ──►│  osu-lyric 后端 (Rust, :41280)        │
            (WS,    │  · 歌曲识别 / 歌词匹配 / 缓存         │
             :24050)│  · 黑名单 · 偏移 · 来源绑定           │
                    │  · 字体资源 (版本化)                  │
                    └───────┬──────────────────────┬───────┘
                            │                      │
                  WS 推送    │                      │  HTTP 管理
                  (展示事件) │                      │  (查询 / 修改)
                            ▼                      ▼
                    ┌───────────────┐      ┌────────────────┐
                    │  LyricsBox    │      │  Controller    │
                    │  /lyrics      │      │  /lyrics/      │
                    │  （OBS 浏览器源）│      │  controller/*  │
                    └───────────────┘      └────────────────┘
```

**职责边界（改造后的核心约定）**

| 通道 | 职责 |
|---|---|
| **HTTP `/api/*`** | **全部管理操作**：读取状态、修改设置、黑名单、缓存、歌词来源、上传、字体、客户端 |
| **WS `/ws`** | **只做展示事件传输**：后端把歌词、清屏、样式、闪烁**推给**展示端 |

WS **不是**管理通道 —— 客户端通过 WS 发送的任何内容都不会改变业务状态。展示端只接收、不发送。

---

## 路由

### 页面

| 路径 | 说明 |
|---|---|
| `/lyrics` | **歌词展示页**（加入 OBS 浏览器源的地址） |
| `/lyrics/controller/client` | 在线展示端 |
| `/lyrics/controller/content` | 歌词内容（当前歌词 / 搜索 / 预览 / 来源 / 偏移 / 清屏） |
| `/lyrics/controller/textstyle` | 文字样式（颜色 / 字号 / 字体 / 对齐 / 翻译优先 / 副歌词） |
| `/lyrics/controller/shadow` | 阴影（主 / 副独立） |
| `/lyrics/controller/blackList` | 黑名单 |
| `/lyrics/controller/cacheManager` | 歌词缓存 |
| `/lyrics/controller/upload` | LRC 上传与字体资源 |

> 访问 `/` 会永久重定向到 `/lyrics`。
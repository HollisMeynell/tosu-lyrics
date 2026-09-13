<!-- markdownlint-disable MD028 MD033 -->
<h1 align="center">osu! 歌词显示器</h1>

通过 [tosu](https://github.com/tosuapp/tosu) 获取 osu! 当前播放的歌曲信息，
由 Rust 后端从网易云 / QQ 音乐等平台匹配并解析歌词，推送到网页显示，作为 OBS 的浏览器源使用。

> **安全声明**：本项目所有代码均已开源。如对安全性有疑虑，可自行查看代码并编译。

---

## 架构

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

WS **不是**管理通道 —— 客户端通过 WS 发送的任何内容都不会改变业务状态（旧版 23 个
WS 管理命令已在 B-11 中彻底移除）。展示端只接收、不发送。

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

### HTTP API

全部挂在 `/api` 下，成功返回结构化 JSON，失败统一为
`{"error":{"code":"...","message":"..."}}` 并配合语义化状态码。

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/status` | 当前歌曲、歌词状态、生效偏移、是否被屏蔽 |
| GET | `/api/lyrics/current` | 当前歌词 + 身份 + 偏移 + 来源绑定 + `lyricState` |
| GET | `/api/lyrics/search-results` | 已有候选（每项带自己的 `source` / `key`） |
| POST | `/api/lyrics/search` | 主动搜索（空 body = 按当前歌曲） |
| GET | `/api/lyrics/preview?source=&key=` | 预览候选，**不改变播放** |
| PUT / DELETE | `/api/lyrics/source` | 应用来源绑定 / 恢复自动匹配 |
| PUT | `/api/lyrics/offset` | 设置偏移（毫秒） |
| POST | `/api/lyrics/upload` | 上传 LRC（multipart），绑定到发起时歌曲的 `sid` |
| POST | `/api/display/clear` | 清空所有展示端（持续清屏，直到换歌 / 换源 / 上传） |
| GET / PATCH | `/api/settings` | 展示设置（颜色 / 字号 / 字体 / 对齐 / 翻译优先 / 副歌词 / 阴影） |
| GET / POST | `/api/blocks` | 黑名单列表 / 新增（幂等） |
| PATCH / DELETE | `/api/blocks/{id}` | 修改备注与名称 / 删除单条 |
| DELETE | `/api/blocks` | 清空（幂等） |
| GET | `/api/cache?page=&size=&q=` | 缓存分页 + 标题过滤 |
| GET | `/api/cache/count` | 缓存总数 |
| DELETE | `/api/cache/{bid}` | 删除单条缓存 |
| DELETE | `/api/cache?title=` | 按标题删除缓存 |
| DELETE | `/api/cache` | 清空缓存 |
| POST | `/api/cache/cleanup` | 清理过期条目 |
| GET | `/api/clients` | 在线展示端列表 |
| POST | `/api/clients/{id}/blink` | 定向闪烁（`id` 可为会话 key 或自报身份） |
| POST | `/api/clients/blink` | 全部展示端闪烁 |
| GET | `/api/font/info` | 主 / 副字体版本信息 |
| GET / POST | `/api/font/{kind}` | 下载 / 上传覆盖字体（`kind` = `main` / `sub`） |

### WebSocket

| 路径 | 方向 | 说明 |
|---|---|---|
| `/ws` | 后端 → 前端 | 歌词推送、`setColor` / `setFont` / `setFontSize` / `setAlignment` / `setTranslationMain` / `setSecondShow` / `setShadow` / `setClear` / `setBlink` |
| `/ws?id=<名字>` | — | 展示端**自报稳定身份**，便于定向操作与识别（可选） |

---

## 构建

依赖：Rust（stable）、Node.js + pnpm、[just](https://github.com/casey/just)（可选）。

```bash
just build          # = build-dist + build-backend + copy-backend
```

或分步：

```bash
# 前端
pnpm i
pnpm build          # 产物在 dist/

# 后端
cd tosu-proxy
cargo build -r --bin osu-lyric --features=new
```

> ⚠️ **`vite build` 会清空 `dist/`**，而 `dist/` 里还放着后端二进制、`config.json5`、
> 数据库、字体等运行产物。构建前端前请先备份这些文件，或使用
> `npx vite build --outDir dist-new` 构建到临时目录后再并入。

---

## 运行

1. 下载 [tosu](https://github.com/tosuapp/tosu/releases) 并解压到任意目录（**无需运行 `tosu.exe`**）。
2. 把构建产物放到同一目录：
   - `osu-lyric.exe`（后端）
   - `dist/` 里的前端内容（`index.html` / `assets/` / `LRC.otf` 等）
   - 首次运行会自动生成 `config.json5` 与 `lyric.db`
3. 运行 `osu-lyric.exe`。
4. 打开 `http://127.0.0.1:41280/lyrics` 即为展示页；控制台在同源的 `/lyrics/controller/*`。

### 配置 `config.json5`

```json5
{
    server: "127.0.0.1",
    log: "info",
    port: 41280,
    database: "sqlite://lyric.db?mode=rwc",
    tosu: {
        url: "ws://127.0.0.1:24050/websocket/v2"
    },
    // 歌词缓存有效期（小时）。不填默认 30 天；填 0 表示不过期。
    lyricCacheTtlHours: 720
}
```

| 字段 | 说明 |
|---|---|
| `server` / `port` | 后端监听地址与端口（默认 `127.0.0.1:41280`） |
| `log` | 日志级别：`trace` / `debug` / `info` / `warn` / `error` |
| `database` | SQLite 连接串 |
| `tosu.url` | tosu 的 v2 WebSocket 地址 |
| `lyricCacheTtlHours` | 歌词缓存 TTL，缺省 30 天，`0` 为不过期 |

### 在 OBS 中使用

把 `http://127.0.0.1:41280/lyrics` 添加为浏览器源。建议宽 1200、高 300。
夜间模式下若希望背景透明，在自定义 CSS 中加入：

```css
.dark body { background-color: rgba(0, 0, 0, 0); }
```

---

## 数据持久化

后端用单个 SQLite 文件（默认 `lyric.db`）保存几类彼此独立的数据：

| 表 | 内容 | 能否随意删除 |
|---|---|---|
| `lyric_cache` | 歌词缓存（带 TTL） | ✅ 纯缓存，删了会重新联网取 |
| `lyric_binding` | **来源绑定**（按 `sid` 归属） | ❌ 用户数据 |
| `lyric_config` | 单曲**偏移** | ❌ 用户数据 |
| `lyric_block` | **黑名单**（`bid` / `sid` / `title` 三种作用域） | ❌ 用户数据 |
| `config` | 展示设置（JSON） | ❌ 用户数据 |

**删缓存不会影响来源绑定、偏移或黑名单。** 旧数据库会在启动时自动迁移
（补充新增列、把旧的屏蔽规则搬进 `lyric_block`），迁移是幂等的。

### 身份约定

| 标识 | 用途 |
|---|---|
| `bid` | **当前播放的具体谱面** —— 播放上下文、异步结果的新旧判断 |
| `sid` | **歌曲级歌词资源归属** —— 来源绑定、缓存、LRC 上传 |
| `title` | 仅用于展示与"标题级黑名单规则"，**不参与判等** |
| generation | 异步代际：换歌 / 清屏会让在途的搜索与取词结果失效 |

---

## 开发

```bash
# 前端热更新（已配置 /api 代理到 127.0.0.1:41280）
pnpm dev

# 后端测试
cd tosu-proxy && cargo test --features=new

# 后端 lint（新增代码不应引入 warning）
cd tosu-proxy && cargo clippy --features=new
```

### 自动化验证

仓库外的 `../_probe/` 下有一套基于假 tosu 的端到端测试，覆盖后端 HTTP、
展示 WS 推送、以及用 CDP 驱动真实浏览器点击 Controller 页面的全链路。

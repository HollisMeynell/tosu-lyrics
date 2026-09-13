# osu-lyric 后端

Rust 后端：连接 tosu 获取 osu! 当前歌曲，匹配并解析歌词，通过 HTTP 与 WebSocket 对外提供服务。

> 本文件描述的是**改造后**的接口。旧版基于 WS 的 23 个管理命令已全部移除，
> 不要再参考历史文档里的 `setter` / `getAllLyric` / `setBlock` 等 WS 管理协议。

---

## 职责边界

| 通道 | 用途 |
|---|---|
| **HTTP `/api/*`** | 全部**管理**操作：查询状态、修改设置、黑名单、缓存、歌词来源、上传、字体、客户端 |
| **WebSocket `/ws`** | **只做展示事件传输**：后端把歌词与展示事件推给展示端 |

WS **不是**管理通道。客户端通过 WS 发送的任何内容都只被记录，不会改变业务状态。

---

## HTTP 接口

统一约定：

- 路径挂在 `/api` 下
- 时间单位一律为**毫秒**（唯一年长例外见各接口说明）
- 成功返回结构化 JSON；mutation 返回**最终服务端状态**
- 失败统一为 `{"error":{"code":"...","message":"..."}}` + 语义化状态码

错误码：`invalid_param`(400) / `no_song`(409) / `no_lyric`(404|422) /
`not_found`(404) / `write_failed`(500) / `song_changed`(409) / `source_failed`(502) / `internal`(500)

### 状态与展示

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/status` | 当前歌曲、歌词状态、生效偏移、`blocked` |
| POST | `/api/display/clear` | 清空所有展示端。语义为**持续清屏**，直到换歌 / 换源 / 上传歌词 |

`/api/status` 的 `blocked` 与 `lyric.loaded=false` 是两回事：前者是用户显式拉黑，
后者是源里没有歌词。

### 歌词内容

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/lyrics/current` | 当前歌词 + 身份 + 偏移 + `source` + `lyricState` |
| GET | `/api/lyrics/search-results` | 已有候选 |
| POST | `/api/lyrics/search` | 主动搜索；空 body 表示按当前歌曲搜索 |
| GET | `/api/lyrics/preview?source=&key=` | 预览候选，**不改变播放状态** |
| PUT | `/api/lyrics/source` | 应用来源绑定（幂等） |
| DELETE | `/api/lyrics/source` | 恢复自动匹配 |
| PUT | `/api/lyrics/offset` | 设置偏移，返回最终生效值（±30s 内） |
| POST | `/api/lyrics/upload` | 上传 LRC（multipart 或原始 body） |

**`/api/lyrics/current` 的契约**：没有播放中的歌 → 404 `no_song`；
有歌但没有可用歌词 → **200** + `lyric: null` + `lyricState`
（`ok` / `blocked` / `none`）。没歌词是正常瞬时状态，不是错误。

**异步正确性**：所有联网操作（搜索 / 取词 / 应用来源 / 上传）在发起时取一份
"代际 + 歌曲身份"，提交前重新校验；不匹配返回 **409 `song_changed`**，结果整份丢弃。

### 设置

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/settings` | 读取完整设置（含默认值） |
| PATCH | `/api/settings` | 局部更新；校验 + 落库成功后才广播 |

字段：`textColor` / `fontSize` / `font`（均为 `{first, second}`）、`alignment`、
`translationMain`、`secondShow`、`shadow`（`{first, second}`，每项含
`enable` / `inset` / `color` / `offset`）。

### 黑名单

| 方法 | 路径 | 说明 |
|---|---|---|
| GET / POST | `/api/blocks` | 列表 / 新增（幂等：同一 `(scope,value)` 只更新元数据） |
| PATCH / DELETE | `/api/blocks/{id}` | 修改名称与备注 / 删除单条 |
| DELETE | `/api/blocks` | 清空 |

**作用域**：`bid`（单谱面）/ `sid`（谱面集）/ `title`（标题）。
一条规则只属于其中之一，**不做跨作用域回退** —— 标题相同的另一首歌不会被误伤。
拉黑当前播放的歌会立即清屏；解除后立即重新加载。

### 缓存

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/cache?page=&size=&q=` | 分页 + 标题过滤（`page` 从 1 开始，`size` 夹到 1..=200） |
| GET | `/api/cache/count` | 总数 |
| DELETE | `/api/cache/{bid}` | 删除单条（幂等） |
| DELETE | `/api/cache?title=` | 按标题模糊删除 |
| DELETE | `/api/cache` | 清空 |
| POST | `/api/cache/cleanup` | 清理过期条目 |

TTL 默认 30 天，配置 `lyricCacheTtlHours` 可覆盖，`0` 表示不过期。
**删缓存不会触碰来源绑定 / 偏移 / 黑名单** —— 它们分属不同的表。

### 在线展示端

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/clients` | 在线展示端列表 |
| POST | `/api/clients/{id}/blink` | 定向闪烁；`id` 可以是会话 key 或自报身份 |
| POST | `/api/clients/blink` | 全部展示端闪烁 |

**在线会话 ≠ 持久身份**：`id` 是每次连接随机生成的会话标识；
客户端可通过 `ws://host/ws?id=obs-main` **自报稳定身份**，重连后仍是它。

### 字体资源

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/font/info` | 主 / 副字体的版本、大小、URL |
| GET | `/api/font/{kind}` | 下载（`kind` = `main` / `sub`） |
| POST | `/api/font/{kind}` | 上传并覆盖 |

**版本化**：版本号由 `文件 mtime + 大小` 派生，覆盖写后必然变化，重启后仍从磁盘读出同一值。
展示端用 `/api/font/main?v=<版本>` 加载，版本变则 URL 变，浏览器不会继续用旧字体。
带 `v` 时返回 `immutable` 长缓存，不带 `v` 时返回 `no-cache`。

---

## WebSocket

**路径**：`/ws`（根路径，**不是** `/api/ws`）

前端**只接收，不发送**。后端推送的消息：

```jsonc
// 歌词推送
{ "type": "lyric", "lyric": [{ "origin": "...", "translation": "..." }],
  "current": 3, "nextTime": 4200, "sequence": "down" }

// 展示事件
{ "type": "setting", "key": "setColor", "value": { "first": "#fff", "second": "#e0e0e0" } }
```

展示事件 key：`setColor` / `setFont` / `setFontSize` / `setAlignment` /
`setTranslationMain` / `setSecondShow` / `setShadow` / `setClear` / `setBlink`。

新展示端接入时会立刻收到一次完整的样式快照与歌词快照（无歌词时下发 `setClear`），
因此 OBS 刷新或断线重连不需要额外处理。

可选参数 `?id=<名字>` 自报身份，便于在 `/api/clients` 中识别与定向操作。

---

## 数据与身份约定

### 表

| 表 | 内容 |
|---|---|
| `lyric_cache` | 歌词缓存（带 `updated_at`，受 TTL 约束） |
| `lyric_binding` | 来源绑定，**按 `sid` 归属** |
| `lyric_config` | 单曲偏移，**严格按 `bid` 精确匹配** |
| `lyric_block` | 黑名单规则（`scope` + `value`，复合唯一索引） |
| `config` | 展示设置（一行 JSON） |

启动时 `init_all_table_and_migrate()` 会建表、建索引并做幂等迁移
（补 `lyric_cache.updated_at`、补 `lyric_block.reason`、把旧 `lyric_config.disable`
行搬进 `lyric_block`）。旧库可直接被新版本打开。

### 身份

| 标识 | 用途 |
|---|---|
| `bid` | 当前播放的**具体谱面** —— 播放上下文与异步新旧判断 |
| `sid` | **歌曲级歌词资源**归属 —— 来源绑定 / 缓存 / LRC 上传 |
| `title` | 仅展示与标题级黑名单规则，**不参与判等** |
| generation | 异步代际，换歌与清屏会让在途结果失效 |

---

## 构建与运行

```bash
cargo build -r --bin osu-lyric --features=new
cargo test --features=new
cargo clippy --features=new
```

运行目录需要 `config.json5` 与前端产物（`index.html` / `assets/`），
首次启动会自动创建 `config.json5` 与 `lyric.db`。详见仓库根目录的 README。

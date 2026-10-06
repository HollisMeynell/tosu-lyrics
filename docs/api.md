<!-- markdownlint-disable MD028 -->
# HTTP API

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
| GET | `/api/font/list` | 字体库列表 |
| POST | `/api/font/upload` | 上传字体（multipart） |
| GET | `/api/font/{name}` | 按名称下载字体 |

---

# WebSocket

| 路径 | 方向 | 说明 |
|---|---|---|
| `/ws` | 后端 → 前端 | 歌词推送、`setColor` / `setFont` / `setFontSize` / `setAlignment` / `setTranslationMain` / `setSecondShow` / `setShadow` / `setClear` / `setBlink` |
| `/ws?id=<名字>` | — | 展示端**自报稳定身份**，便于定向操作与识别（可选） |
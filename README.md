<!-- markdownlint-disable MD033 -->
<h1 align="center">osu! 歌词显示器</h1>

通过 [tosu](https://github.com/tosuapp/tosu) 获取 osu! 当前播放的歌曲信息，
由 Rust 后端从网易云 / QQ 音乐等平台匹配并解析歌词，推送到网页显示，作为 OBS 的浏览器源使用。

> **安全声明**：本项目所有代码均已开源。如对安全性有疑虑，可自行查看代码并编译。

详细文档见 [`docs/`](docs/) 目录：[架构](docs/architecture.md) · [接口](docs/api.md) · [数据表](docs/data.md)

---

## 运行

1. 下载 [tosu](https://github.com/tosuapp/tosu/releases) 并解压到任意目录（**无需运行 `tosu.exe`**）。
2. 把 `osu-lyric.exe`（后端）放到 tosu 同目录。
3. 运行 `osu-lyric.exe`，首次运行会自动生成 `config.json5` 与 `lyric.db`。
4. 打开 `http://127.0.0.1:41280/lyrics` 即为展示页；控制台在同源的 `/lyrics/controller/*`。

## 配置 `config.json5`

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

## 在 OBS 中使用

把 `http://127.0.0.1:41280/lyrics` 添加为浏览器源。建议宽 1200、高 300。
夜间模式下若希望背景透明，在自定义 CSS 中加入：

```css
.dark body { background-color: rgba(0, 0, 0, 0); }
```
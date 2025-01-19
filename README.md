<!-- markdownlint-disable MD033 -->
<h1 align="center">tosu 歌词显示器</h1>

## 介绍

本项目通过 [tosu](https://github.com/tosuapp/tosu) 获取 osu 当前播放的歌曲信息，并借助一个简单的 [代理工具](tosu-proxy) 从网易云音乐获取歌词，最终将歌词显示在网页上。

> [!IMPORTANT]
> **跨域请求说明**：
>
> - 您可以通过在启动 OBS 时添加参数 `--disable-web-security` 来启用跨域请求。此时无需使用代理工具，但需要在 URL 末尾添加 `?cors=true` 以启用直接请求。
> - 请注意，此操作会降低 OBS 的安全性，如果您不了解其影响，请勿使用此方法。
> - 如果歌词始终无法显示，说明您的 OBS 不支持此方法，请使用代理工具。

> [!TIP]
> **最佳使用方式**：
>
> - 将本工具添加到 OBS 的浏览器源中，建议设置宽为 1200，高为 300。

## 使用方法

1. 下载并解压 release 中的文件到 tosu 的根目录。
2. 运行 `tosu-proxy.exe`。
3. 访问 [http://127.0.0.1:41280/lyrics/index.html](http://127.0.0.1:41280/lyrics/index.html)

> [!IMPORTANT]
> **注意事项**：
>
> - 由于浏览器的安全策略限制，如果不使用 OBS 的跨域功能，则必须使用 `tosu-proxy` 作为代理，否则无法获取歌词。
> - 本项目所有代码均已开源，如果您对安全性有疑虑，可以自行查看代码并编译。
> - 请确保 `tosu-proxy` 和 `tosu` 位于同一目录下，静态文件应放置在 `[tosu根目录]/static/lyrics` 目录中。
> - `tosu-proxy` 仅作为额外的代理工具，不会影响 `tosu` 或其他插件的正常使用。
> - 其他插件可通过访问 [`http://127.0.0.1:24050`](http://127.0.0.1:24050) 正常使用。
> - 您可以通过替换 `tLRC.otf` 和 `oLRC.otf` 文件来更改字体。支持 `ttf`、`woff` 等字体格式，但需将文件名严格更改为 `tLRC.otf` 或 `oLRC.otf`。

## 自行编译

### 代理工具

1. 确保已安装 Rust 编译环境。
2. 进入 [`tosu-proxy`](tosu-proxy) 目录。
3. 执行 `cargo build --release`，编译结果将生成在 `target/release` 目录下。

### 歌词页面

1. 确保已安装 Node.js 环境。
2. 执行 `npm install` 安装依赖。
3. 执行 `npm run build` 进行编译，编译结果将生成在 `dist` 目录下。
4. 将 `dist` 目录下的文件复制到 `tosu` 的 `static/lyrics` 目录中。

## 致谢

- 本项目的原型和样式改进灵感来自 [@EmitPots](https://github.com/EmitPots)。
- 参考项目：[LyricDisplayerPlugin](https://github.com/OsuSync/LyricDisplayerPlugin)。
- 感谢所有贡献者和用户的支持！

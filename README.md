<!-- markdownlint-disable MD028 MD033 -->
<h1 align="center">tosu 歌词显示器</h1>

## 介绍

本项目通过 [tosu](https://github.com/tosuapp/tosu) 获取 osu 当前播放的歌曲信息，并借助一个简单的 [代理工具](tosu-proxy) 从网易云音乐等多个平台获取歌词信息，解析后在网页中显示出来，作为 OBS 的浏览器源使用。

## 使用

1. 下载 [tosu](https://github.com/tosuapp/tosu/releases) 并解压到任意目录（例如 `/AppData/Roaming` 或 `Program Files`等）。**注意：无需运行 `tosu.exe`。**
2. 下载 [release](https://github.com/HollisMeynell/tosu-lyrics/releases/) 中的压缩包并将文件解压到 tosu 的根目录。
3. 运行 `tosu-proxy.exe`。本过程将会自动打开浏览器并访问 41280 端口和 24050 端口。
4. 41280端口用于显示本项目，24050端口启动的 tosu 面板可选择性关闭。
5. 你也可以手动访问[http://127.0.0.1:41280/lyrics/index.html](http://127.0.0.1:41280/lyrics/index.html)。
6. 将该地址添加到 OBS 的浏览器源中，建议设置宽为 1200，高为 300。

> [!IMPORTANT]
> **跨域请求说明**：
>
> - 您可以通过在启动 OBS 时添加参数 `--disable-web-security` 来启用跨域请求。此时无需使用代理工具 `tosu-proxy` ，直接访问即可。但需要在 URL 末尾添加 `?cors=true` 以启用直接请求。
> - 请注意，此操作会降低 OBS 的安全性，如果您不了解其影响，请勿使用此方法。
> - 如果歌词始终无法显示，说明您的 OBS 不支持此方法，请使用代理工具。
> - 由于浏览器的安全策略限制，如果不使用 OBS 的跨域功能，则必须使用 `tosu-proxy` 作为代理，否则无法获取歌词。

> [!IMPORTANT]
> **安全声明**:

> - 本项目所有代码均已开源，如果您对安全性有疑虑，可以自行查看代码并编译。

> [!TIP]
>
> - 文件结构: 确保 `tosu-proxy.exe` 和 `tosu.exe` 位于项目根目录，静态文件位于 `<tosu根目录>/static/lyrics` 目录中。
> - 隔离说明: `tosu-proxy` 仅作为额外的代理工具，不影响 `tosu` 或其他插件的正常使用。可访问 [`http://127.0.0.1:24050`](http://127.0.0.1:24050) 使用他们。
> - 字体修改: 您可以通过修改 `LRC.otf` 更改全部字体, 或者添加 `tLRC.otf` (对应主歌词) 和 `oLRC.otf` (对应副歌词) 来更改字体。
> - 自定义字体支持 `ttf`、`woff` 等字体格式，但需将文件名严格填写为 `LRC.otf`, `tLRC.otf` 或 `oLRC.otf` (即 'xxx.ttf' => 'LRC.otf')。

## 如何更新

> [!IMPORTANT]
> 如果已经使用了旧版本, 仅需要更新 `index.html` 文件即可, 其他的文件无需重复下载

> [!TIP]
> 如果旧版本出现异常的歌词，请更新后清理缓存来解决。
>
> 操作方法：向 obs 浏览器源中 url 末尾添加`?clear-cache=true`参数，并确定，刷新缓存后将参数去除

## 开发

### 代理工具

确保已安装 Rust 编译环境。

    ```bash
    cd tosu-proxy # 进入代理工具目录
    cargo build --release # 编译代理工具
    ```

### 歌词页面

确保已安装 Node.js 环境。

    ```bash
    cd /<path-to-root> # 进入项目目录
    npm install # 安装依赖
    npm run dev # 启动开发服务器
    npm run build # 编译
    ```

可将 `dist` 目录下的文件复制到 `tosu` 的 `static/lyrics` 目录。

## 致谢

- 本项目的原型和样式改进灵感来自 [@EmitPots](https://github.com/EmitPots)。
- 参考项目：[LyricDisplayerPlugin](https://github.com/OsuSync/LyricDisplayerPlugin)。
- 感谢所有贡献者和用户的支持！🙌

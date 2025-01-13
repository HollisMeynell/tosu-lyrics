<h1>
    <p align="center">
        tosu 歌词显示器
    </p>
</h1>

## 介绍

这是一个借助 [tosu](https://github.com/tosuapp/tosu) 获取 osu 当前播放的歌曲, 然后通过一个简单的 [代理小工具](tosu-proxy), 请求网易云的歌词, 显示在网页上

> [!IMPORTANT]
> 可以通过在启动 obs 添加参数 `--disable-web-security` 来启用跨域请求, 
> 此时可以无需使用代理, 此时需要将 url 末尾添加 `?cors=true` 来启用直接请求
> 
> 请先测试您的 obs 是否支持, 如果一直不出现歌词, 说明您的 obs 不支持, 请使用代理
> 
> 启动此参数会导致 obs 安全性降低, 如果不懂请不要使用此方法

> [!TIP]
> 最佳用途是添加到 obs 的浏览器源中（宽1200 高300）

## 使用方法

将 release 中的文件下载并解压到 tosu 的目录下, 并运行 `tosu-proxy.exe`

访问 [`http://127.0.0.1:41280/lyrics/index.html`](http://127.0.0.1:41280/lyrics/index.html)

> [!IMPORTANT]
> 受限于浏览器安全策略, 如果不使用 obs 的跨域, 必须使用 `tosu-proxy` 作为代理, 否则无法获取到歌词
> 
> 所有代码均开源, 如果不放心可以自行查看代码并编译
> 
> 请确保 `tosu-proxy` 和 `tosu` 在同一目录下, 静态文件位于 `[tosu根目录]/static/lyrics` 目录
>
> 仅运行 `tosu-proxy`, `tosu-proxy` 仅作为一个额外的代理工具, 不影响 `tosu` 的正常使用, 同时也不影响其他插件的使用
> 
> 其他插件访问 [`http://127.0.0.1:24050`](http://127.0.0.1:24050) 即可
> 
> 可以通过替换文件 `tLRC.otf` 与 `oLRC.otf` 来更改字体, 支持 `tff` / `woff` 等其他类型的字体, 但是要改一下后缀名到 `tLRC.otf` 或 `oLRC.otf`,
> 请确保文件名严格一致

## 自编译

### 代理小工具
- 准备好 rust 编译环境
- 切换到 [`tosu-proxy`](tosu-proxy) 目录
- 执行 `cargo build --release`, 编译完结果 `target/release` 目录下

### 歌词页面
- 准备好 nodejs 环境
- 执行 `npm install` 安装依赖
- 执行 `npm run build` 编译, 编译完结果 `dist` 目录下
- 将 `dist` 目录下的文件复制到 `tosu` 的 `static/lyrics` 目录下

## 感谢

本项目项目原型以及样式改进来自 [@EmitPots](https://github.com/EmitPots)

参照项目 [LyricDisplayerPlugin](https://github.com/OsuSync/LyricDisplayerPlugin)

非常感谢


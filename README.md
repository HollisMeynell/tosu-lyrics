<!-- markdownlint-disable MD028 MD033 -->
<h1 align="center">tosu 歌词显示器</h1>

## 介绍

本项目通过 [tosu](https://github.com/tosuapp/tosu) 获取 osu 当前播放的歌曲信息，并借助一个简单的 [代理工具](tosu-proxy)
从网易云音乐等多个平台获取歌词信息，解析后在网页中显示出来，作为 OBS 的浏览器源使用。

> **安全声明**:
>
> - 本项目所有代码均已开源，如果您对安全性有疑虑，可以自行查看代码并编译。

## TODO

- [ ] Controller
  - [x] 查看当前播放歌曲的歌词
  - [x] 提供单句的复制
  - [x] 文字色彩修改(支持主歌词、翻译歌词独立色彩)
  - [x] 选择是否将翻译作为主歌词显示
  - [x] 选择显示翻译与否
  - [x] 在外部控制后，OBS内自动更新内容变化( 通过ws广播实现 )
  - [x] 夜间模式
  - [ ] 查看当前曲名在各源下的所有搜索结果
  - [ ] 支持指定某单曲对应为其他 源/搜索结果 对应的歌词
  - [ ] 对指定名单进行管理
  - [ ] 加强对源的搜索精确度，减少指定歌词的次数以提升用户体验
  - [ ] 屏蔽单曲、某标题对应的全部曲目的歌词
  - [ ] 手动上传新歌词
  - [ ] 修改单曲的歌词细节
  - [ ] 对歌词缓存进行删除操作
  - [ ] 对黑名单进行管理
  - [ ] 字体修改
  - [ ] 左右对齐、居中对齐
  - [ ] 外部浏览器完全基于 OBS 浏览器源的内容，而非重复请求 TosuApi，实现内容的完全绑定

## 使用

### 下载、启动

1. 下载 [tosu](https://github.com/tosuapp/tosu/releases) 并解压到任意目录（例如 `/AppData/Roaming` 或 `Program Files`等）。**注意：无需运行 `tosu.exe`。**
2. 下载 [release](https://github.com/HollisMeynell/tosu-lyrics/releases/) 中的压缩包并将文件解压到 tosu 的根目录。
3. 运行 `tosu-proxy.exe`。
4. 41280 端口用于显示本项目，24050 端口用于 tosu。(可以通过环境变量 `TOSU_PROXY_PORT` 来指定本项目的端口, tosu 端口也可以被修改, 请参阅对应设置)

### 显示

将该地址添加到 OBS 的浏览器源中。建议设置宽为 1200，高为 300, 在自定义CSS中添加 `.dark body {background-color: rgba(0, 0, 0, 0);}` 用于在夜间模式将背景透明化。

### 控制

在外部浏览器访问[http://127.0.0.1:24050/lyrics/index.html](http://127.0.0.1:24050/lyrics/index.html)。
在 OBS 的浏览器的`交互`中、外部浏览器中，均可以通过

- ctrl + alt + t
- 三指轻点屏幕

来切换控制面板的打开状态。
默认关闭以提升观众体验。

> [!IMPORTANT]
> **跨域说明**：
>
> - **由于 obs 跨域环境限制很多, 你必须启动 `tosu-proxy`。**

> [!TIP]
>
> - 文件结构: 非必要情况下尽量使 `tosu-proxy.exe` 和 `tosu.exe` 位于项目根目录，静态文件位于 `<tosu根目录>/static/lyrics` 目录中，否则需要手动启动两个程序。
> - 隔离说明: `tosu-proxy` 不影响 `tosu` 或其他插件的正常使用。可访问 [`http://127.0.0.1:24050`](http://127.0.0.1:24050) 使用他们。
> - 字体修改: 您可以通过修改 `LRC.otf` 更改全部字体, 或者添加 `tLRC.otf` (对应主歌词) 和 `oLRC.otf` (对应副歌词) 来更改字体。
> - 自定义字体支持 `ttf`、`woff` 等字体格式，但需将文件名严格填写为 `LRC.otf`, `tLRC.otf` 或 `oLRC.otf` (即 'xxx.ttf' => 'LRC.otf')。

## 如何更新

- 如果已经使用了旧版本, 仅需要更新 `index.html` 文件即可, 其他的文件无需重复下载
- 如果旧版本出现异常的歌词，请在更新后清理缓存来解决。
    操作方法：向 obs 浏览器源中 url 末尾添加`?clear-cache=true`参数，并确定，刷新缓存后将参数去除

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
    # 访问5173端口需要附带参数 /?controller=true 以开启控制面板
    npm run build # 编译
    ```

可将 `dist` 目录下的文件复制到 `tosu` 的 `static/lyrics` 目录。

## 致谢

- 本项目的原型和样式改进灵感来自 [@EmitPots](https://github.com/EmitPots)。
- 参考项目：[LyricDisplayerPlugin](https://github.com/OsuSync/LyricDisplayerPlugin)。
- 感谢所有贡献者和用户的支持！🙌

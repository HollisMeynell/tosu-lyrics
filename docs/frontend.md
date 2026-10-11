# 前端技术栈与开发规范

## 技术统计

版本来自当前 `package.json`；含 `^` 的依赖是声明范围，实际解析版本以 `pnpm-lock.yaml` 为准。

| 类别     | 技术/声明版本                                                                   | 用途                                |
| -------- | ------------------------------------------------------------------------------- | ----------------------------------- |
| 语言     | TypeScript 5.8.3、TSX、ES Modules                                               | 严格类型检查、Solid JSX             |
| UI 框架  | solid-js 1.9.6                                                                  | 组件、响应式 signal/store、生命周期 |
| 路由     | @solidjs/router ^0.15.3                                                         | 页面路由、懒加载                    |
| 构建     | Vite 6.3.5、vite-plugin-solid 2.11.8                                            | 开发服务、Solid 编译、静态构建      |
| 样式     | Tailwind CSS 4.1.11、@tailwindcss/vite 4.1.11、CSS                              | 样式与 Vite 集成                    |
| 通信     | 浏览器 fetch、reconnecting-websocket ^4.4.0                                     | HTTP 管理接口、WS 自动重连          |
| 代码检查 | ESLint 9.39.2、@eslint/js 9.32.0、typescript-eslint parser 8.52.0/plugin 8.46.2 | JS/TS 推荐规则                      |
| 格式化   | Prettier 3.6.2                                                                  | 统一代码格式                        |
| 构建分析 | rollup-plugin-visualizer ^6.0.3                                                 | 生成体积分析报告                    |
| 辅助类型 | @types/node 25.0.3、globals 17.0.0                                              | 构建配置类型、ESLint 浏览器全局变量 |

当前未声明前端单元测试框架或 `test` 脚本；状态管理使用 Solid 自身能力，UI 组件在仓库内维护。

## 目录与职责

以下路径以仓库根目录为基准。

| 路径                                           | 职责                                       |
| ---------------------------------------------- | ------------------------------------------ |
| `src/main.tsx` / `src/App.tsx`                 | 应用入口与顶层组件                         |
| `src/routes`                                   | 页面路由、布局、懒加载                     |
| `src/pages/LyricsBox`                          | 展示页及控制台共用的歌词视图               |
| `src/pages/Controller/ControlTools`            | 内容、样式、黑名单、缓存、客户端等控制面板 |
| `src/components/ui/` / `src/components/error/` | 可复用 UI 与错误展示                       |
| `src/services`                                 | 按业务组织的 HTTP 调用与错误处理           |
| `src/api`                                      | WebSocket 连接封装、展示消息模型           |
| `src/hooks`                                    | 业务数据加载、交互与初始化逻辑             |
| `src/stores`                                   | 歌词、设置、字体模式共享状态               |
| `src/types`                                    | 共享类型与 DTO                             |
| `src/utils`                                    | 歌词解析、滚动、字体、参数等工具           |
| `src/config/constants.ts`                      | API 基础路径、WS 地址                      |
| `src/assets/Icons`                             | 仓库内 TSX 图标组件                        |
| `src/index.css`                                | 全局样式与字体相关样式                     |

## 格式化与检查

以下命令在仓库根目录执行：

```bash
pnpm install --frozen-lockfile
pnpm exec prettier --write src/pages/LyricsBox/index.tsx
pnpm format
pnpm lint
pnpm exec tsc --noEmit
pnpm build
```

单文件命令中的路径替换为实际修改文件。`pnpm format` 格式化 `src` 内脚本列出的扩展名，额外传入 `--end-of-line auto`；不覆盖根配置或 Markdown。文档可使用 `pnpm exec prettier --check "docs/**/*.md"` 检查，`--write` 整理。

Prettier 约定：4 空格缩进、分号、双引号、ES5 尾逗号、80 字符打印宽度、对象括号内空格。ESLint 使用 JS/TS 推荐规则。`pnpm build` 仅执行 Vite，不能替代 `tsc --noEmit`；构建分析插件配置了 `open: true`，可能自动打开报告。

开发启动命令为 `pnpm dev`；`pnpm preview` 只预览前端产物，管理接口和 WS 仍需要后端。

## 代码风格与组织

本节作为后续维护参考，新增和修改代码时均应遵守。

- 组件文件与页面目录采用 PascalCase，如 `FontPicker.tsx`、`LyricsBox/`；页面/模块入口沿用 `index.tsx`。
- Hook 使用 `useXxx.ts`，服务 `xxxService.ts`，状态 `xxxStore.ts`，工具使用 camelCase，如 `parseLyrics.ts`；沿用现有目录名，避免仅改大小写引发跨平台问题。
- 组件、类型、接口使用 PascalCase；函数与变量使用 camelCase；全局常量沿用 UPPER_SNAKE_CASE。
- 谨慎添加注释，非必要不留注释。不要逐行解释代码做了什么，也不要写“本次修改了什么”、修复过程或版本对比；修改记录放在提交说明中。
- 可以按需保留类型、接口或结构体字段的含义、单位与取值约束，函数的职责、参数和返回约定，以及生命周期、协议、兼容性等注意事项。函数说明应概括调用者需要知道的契约，避免复述实现步骤；名称和类型已足够清楚时不再补注释。
- 注释应简洁、准确，代码变化时同步更新或删除过时说明，不为增加注释数量而写注释。
- 按职责分层并按业务拆分目录和文件：数据模型放在 `types/` 或 `api/model.ts`，HTTP 调用放在 `services/`，业务交互与生命周期逻辑放在 `hooks/`，共享状态放在 `stores/`，页面和 UI 放在 `pages/`、`components/`，通用纯函数放在 `utils/`。
- 不把模型定义、请求、状态管理和页面渲染全部堆在同一个组件或文件中。根据职责、复杂度和复用需求拆分，优先沿用现有目录；仅在需要时新增模块，避免无必要的分层和碎片化文件。
- 新依赖先检查已有 Solid、路由、样式、WS 库及 `utils/`、`components/ui/`；不为已有能力再引入另一套状态库、UI 库或通信库。

## 开发边界

- HTTP 请求放在 `services/`，组件通过服务和 Hook 使用；共享状态放在 `stores/`，协议类型放在 `api/model.ts` 或 `types/`，避免各面板复制类型和请求逻辑。
- 使用 Solid 的响应式和生命周期，清理监听器、定时器和连接；不要套用 React 的渲染与 Hook 语义。
- 管理写操作通过 HTTP；以返回的服务端最终状态更新界面。正常的“无歌词”状态与接口失败分别处理，WS 重连后使用服务端快照恢复展示。
- `/lyrics` 与控制台预览共用 `LyricsBox` 和歌词状态；不要另建一套歌词推进逻辑。
- 更改路径时同步核对 `vite.config.ts` 的 `base: "/lyrics"`、`src/routes/index.tsx` 和后端静态资源路由；更改 `@/` 别名时同时修改 Vite 和 `tsconfig.json`。
- 开发 HTTP 代理在 `vite.config.ts` 指向 `127.0.0.1:41280`；开发 WS 在 `src/config/constants.ts` 单独指定同一主机。修改端口需要同时调整两处与后端配置。生产 HTTP 用 `/api`，WS 从页面协议和 host 推导。
- 修改展示消息时联动后端 `model/websocket/`、前端消息类型及处理逻辑；字段默认值、单位和可选性必须一致。

提交前对相关改动运行格式检查、lint、类型检查和构建；涉及显示或通信时，验证展示页、控制台预览、断线重连和对应管理操作。现有检查若失败，应记录实际错误，不能把命令已执行当作通过。

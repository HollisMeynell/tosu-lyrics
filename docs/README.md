# 项目概览与共同开发规范

本仓库通过 tosu 获取 osu! 播放状态，由 Rust 后端匹配、缓存并推送歌词，供 SolidJS 前端展示和管理，可作为 OBS 浏览器源使用。

本文依据仓库当前源码与配置整理；“开发规范”是后续修改应遵守的约定，不代表已有自动化规则全部覆盖。

## 文档导航

- [前端技术栈与开发规范](frontend.md)
- [后端技术栈与开发规范](backend.md)
- 业务文档：[架构](architecture.md)、[接口](api.md)、[数据表](data.md)。接口细节与文档不一致时先核对源码，再同步更新文档。

## 技术栈与服务边界

| 部分     | 技术                                                                 | 职责                                                    |
| -------- | -------------------------------------------------------------------- | ------------------------------------------------------- |
| 前端     | TypeScript、SolidJS、Solid Router、Vite、Tailwind CSS                | 歌词展示、控制台交互、调用管理接口、接收展示事件        |
| 后端     | Rust 2024、Tokio、Salvo、SeaORM、SQLite                              | 播放状态接入、歌词获取与解析、持久化、HTTP/WS、资源服务 |
| 外部服务 | tosu、网易云音乐、QQ 音乐                                            | 播放信息与在线歌词来源                                  |
| 工程工具 | pnpm、Cargo、Prettier、ESLint、rustfmt、Clippy、just、GitHub Actions | 依赖管理、质量检查、构建与发布                          |

仓库目录名含 `react`，但当前前端框架是 **SolidJS**；`package.json` 的名称为 `lyrics-solid`，不应引入 React 的组件或 Hooks 用法。

- `/lyrics` 为展示页，`/lyrics/controller/*` 为控制台。
- HTTP `/api/*` 承担管理操作；WebSocket `/ws` 推送歌词、样式、清屏等展示事件，不承担管理写操作。
- 开发时 Vite 提供页面、代理 `/api`；生产时后端同时提供接口和内嵌前端资源。
- 数据库、在线歌词请求和业务状态由后端维护，前端不直接访问数据库或音乐平台。

## 仓库结构

本文中的文件路径以仓库根目录为基准；Markdown 链接相对当前文档。

```text
src/                    前端源码，细分见前端文档
public/                 Vite 原样复制的公共资源
static/                 仓库内字体资源
tosu-proxy/             Rust 后端主 crate
  src/                  路由、服务、模型、数据库、歌词来源
  lyric-macro/          本地过程宏 crate
  lib/                  辅助程序资源（按实际平台使用）
docs/                   技术栈、开发规范、架构、接口与数据说明
.github/                构建发布 workflow、Dependabot 配置
.vscode/settings.json   VS Code 拼写词表
justfile                前后端构建与打包入口
embed/                  打包时组装、编译时嵌入的资源
dist/、release/          构建/发布产物
```

`node_modules`、`tosu-proxy/target/`、`embed/`、`dist/`、`release/` 是依赖或生成目录，不作为源码修改入口。`Plan.md`、`DEV_LOG.md` 是规划和历史记录，使用前需核实当前实现。

## 共同代码规范

- 注释保持简洁，**非必要不留注释**；不设注释覆盖率或行数要求。不重复代码含义，不添加逐行解释、文件名注释、修改流水账或大段分隔符。
- 只为不直观的业务约束、兼容原因、并发条件和安全前提补充必要说明；代码变化时同步修改或删除过时注释。Rust `unsafe` 的安全依据应保留。
- 引入新依赖前先检查 `package.json`、主/过程宏 `Cargo.toml`、锁文件和现有工具函数，确认是否已有同类能力。能复用现有库就复用；新增时说明用途和现有实现不足，并同步对应锁文件。
- 修改围绕当前任务，遵循所在模块的命名和组织方式；格式化优先限定修改文件，避免无关的全仓重排。
- 对协议、数据格式、配置含义或构建流程的修改，同步更新对应文档；检查范围按改动影响选择，不能以“构建成功”代替类型检查或业务验证。

## 关键配置与工具对应

| 文件                                                                      | 工具/作用                                          | 修改约定                                                   |
| ------------------------------------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------- |
| `package.json` / `pnpm-lock.yaml`                                         | pnpm 脚本、依赖与解析版本                          | 使用 pnpm 更新；不混入 npm/yarn 锁文件                     |
| `pnpm-workspace.yaml`                                                     | pnpm 依赖构建许可                                  | 与 `package.json` 内 `pnpm.onlyBuiltDependencies` 一并核对 |
| `.prettierrc`                                                             | Prettier 格式规则                                  | 风格变更需说明影响，不顺带全仓格式化                       |
| `eslint.config.js`                                                        | ESLint flat config、TS 推荐规则                    | 调整规则需说明原因，避免整体关闭检查                       |
| `tsconfig.json`                                                           | TS 严格检查、Solid JSX、路径别名                   | 与 Vite 的别名及 JSX 编译配置保持一致                      |
| `vite.config.ts`                                                          | Solid/Tailwind 插件、基础路径、开发代理、构建分析  | 路径/端口变化需联动前后端                                  |
| `tosu-proxy/Cargo.toml` / `tosu-proxy/Cargo.lock`                         | Cargo 主 crate 依赖、二进制入口、发布优化          | 默认编译 `src/main.rs`，二进制名统一为 `tosu-lyrics`       |
| `tosu-proxy/lyric-macro/Cargo.toml` / `tosu-proxy/lyric-macro/Cargo.lock` | 本地过程宏 crate 的依赖                            | 修改宏依赖时核对独立 crate 与主 crate 构建                 |
| `tosu-proxy/default.config.json5`                                         | 后端配置示例                                       | 与 `tosu-proxy/src/config.rs` 的字段和默认值同步           |
| 运行目录 `config.json5`                                                   | 后端实例运行配置                                   | 属于实例数据，勿用构建产物覆盖                             |
| `justfile`                                                                | 格式化、检查、构建、资源组装、打包                 | 核对删除范围、资源路径和发布文件名                         |
| `.github/workflows/release.yaml`                                          | `master` 构建预热缓存，`v*` 标签发布 Linux/Windows | 分支构建只打包，标签构建才创建 Release                     |
| `.github/workflows/release-test.yml`                                      | 手动测试发布                                       | 使用 `just package ./release`，与正式发布统一打包入口      |
| `.github/dependabot.yml`                                                  | npm/Cargo 每日依赖更新                             | 升级后仍需本地检查与兼容验证                               |
| `.vscode/settings.json`                                                   | cSpell 自定义词表                                  | 当前未配置自动格式化                                       |
| `.gitignore` / `.gitattributes`                                           | 产物忽略/文件属性                                  | 不将运行库、个人配置或生成产物纳入源码                     |

当前没有仓库级 `.editorconfig`、`rustfmt.toml` 或固定 Rust toolchain 文件。CI 使用 Node.js 22、pnpm 10 和 Rust stable；这属于当前 CI 配置，不是仓库已声明的最低版本。

本地运行 pnpm 时提示 `package.json` 中的 `pnpm.onlyBuiltDependencies` 已被忽略；当前依赖构建许可另列在 `pnpm-workspace.yaml` 的 `allowBuilds` 中，调整时应核对所用 pnpm 版本的实际行为。

## 构建边界

在根目录使用 `just --list` 查看任务；`just package ./release` 会组装 `embed` 并构建单文件发行版，当前输出名为 `tosu-lyrics`（Windows 为 `.exe`）。`just build` 使用同一文件名。

`just assemble-embed` 会重建 `embed`，`just build-frontend` 会替换部分 `dist/` 内容；这些目录仅放生成资源。发布 workflow 的 UPX 与发布附件路径均使用 `tosu-lyrics`。

修改数据库与运行目录前，遵守[后端数据库规范](backend.md#数据库与运行数据边界)，勿将真实用户数据作为测试库。

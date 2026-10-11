# 后端技术栈与开发规范

## 技术统计

后端主 crate 为 `tosu-proxy`，二进制为 `tosu-lyrics`，入口为 `src/main.rs`；Rust edition 为 2024，不再定义 `new` feature，默认编译完整后端。下列版本来自主 crate 与过程宏的 `Cargo.toml`，实际版本以对应锁文件为准。

| 类别        | 技术/声明版本                                                                        | 用途                                    |
| ----------- | ------------------------------------------------------------------------------------ | --------------------------------------- |
| 异步运行时  | Tokio 1.50.0                                                                         | 任务、文件、进程、同步                  |
| HTTP 服务   | Salvo 0.89.3                                                                         | HTTP、WebSocket、静态资源               |
| HTTP 客户端 | reqwest 0.13.2                                                                       | 在线歌词平台请求                        |
| WS 客户端   | tokio-tungstenite / tungstenite 0.29.0                                               | 接入 tosu 播放状态                      |
| 数据访问    | SeaORM / sea-orm-migration 1.1.19、sea-schema 0.16.2                                 | 实体、连接池、迁移与 schema             |
| 默认数据库  | SQLite                                                                               | 本地持久化；当前迁移配置面向 SQLite     |
| 序列化      | serde 1.0.228、serde_json 1.0.149、bincode 2.0.1、serde_path_to_error 0.1.20         | 数据模型、JSON/二进制处理、反序列化诊断 |
| 配置        | config 0.15.22                                                                       | JSON5 配置加载                          |
| 日志与错误  | tracing / tracing-subscriber 0.3（tracing 为 0.1）、thiserror 2.0.18                 | 结构化日志、错误定义                    |
| 内嵌资源    | rust-embed 8.6.0                                                                     | 编译时嵌入仓库根目录的 `embed/`         |
| 异步辅助    | futures / futures-util 0.3.32、async-trait 0.1.89                                    | 异步组合与 trait                        |
| 工具        | regex 1.12.3、rand 0.10.0、paste 1.0.15                                              | 文本匹配、随机值、宏拼接                |
| 本地过程宏  | lyric-macro 0.0.1；syn 2.0.101、quote 1.0.40、proc-macro2 1.0.95、convert_case 0.8.0 | 语法解析、代码生成与命名转换            |

SeaORM 虽启用了 MySQL/PostgreSQL 驱动，当前迁移依赖只启用了 SQLite；不能据此视为其他数据库已具备完整兼容性。

## 目录与职责

以下路径相对仓库根目录的 `tosu-proxy/`：

| 路径                       | 职责                                           |
| -------------------------- | ---------------------------------------------- |
| `src/main.rs`              | 初始化顺序与服务入口                           |
| `src/lib.rs`               | 主 crate 模块导出                              |
| `src/config.rs`            | 配置、默认值、日志、路由常量                   |
| `src/server/`              | 路由、请求解析、响应、静态资源与 WS 会话       |
| `src/service/`             | 歌词、来源、设置、缓存、黑名单、字体等业务流程 |
| `src/model/http/`          | HTTP 数据模型                                  |
| `src/model/websocket/`     | 展示事件模型                                   |
| `src/model/shared/`        | 共享设置模型                                   |
| `src/database/connect.rs`  | 数据库连接与启动迁移                           |
| `src/database/entity/`     | SeaORM 实体、数据访问和旧数据迁移              |
| `src/database/migration/`  | 有序 schema 迁移                               |
| `src/osu_source/`          | tosu 播放信息接入                              |
| `src/lyric/` / `source/`   | 歌词模型、解析和网易云/QQ 来源实现             |
| `src/setting/`             | 展示设置初始化                                 |
| `src/error.rs` / `util.rs` | 公共错误与工具                                 |
| `lyric-macro/`             | 独立过程宏 crate                               |

## 格式化与检查

在仓库根目录执行：

```bash
cargo fmt --manifest-path tosu-proxy/Cargo.toml
cargo fmt --manifest-path tosu-proxy/lyric-macro/Cargo.toml
cargo fmt --manifest-path tosu-proxy/Cargo.toml -- --check
cargo clippy --manifest-path tosu-proxy/Cargo.toml
cargo test --manifest-path tosu-proxy/Cargo.toml
cargo build --manifest-path tosu-proxy/Cargo.toml -r --bin tosu-lyrics
```

过程宏是独立 crate，修改时单独格式化，并检查主 crate 编译。仓库未提供自定义 rustfmt 配置，使用工具默认规则。`just build-backend` 会执行主 crate 的格式化、Clippy 和 release 构建，不包含测试。

## 代码风格与组织

本节作为后续维护参考，新增和修改代码时均应遵守。

- 文件/模块、函数和变量用 snake_case；类型与 trait 用 PascalCase；常量用 UPPER_SNAKE_CASE。按现有风格使用 `xxx_service.rs`、`mod.rs`。
- 迁移文件沿用 `mYYYYMMDD_HHMMSS_description.rs`，按递增顺序在 `migration/mod.rs` 注册。
- Rust 内部字段沿用 snake_case；HTTP/WS 外部字段依据现有 serde 映射保持契约，不能假设所有模型均自动转换为 camelCase。
- 谨慎添加注释，非必要不留注释。不要逐行解释代码做了什么，也不要写“本次修改了什么”、修复过程或版本对比；修改记录放在提交说明中。
- 可以按需保留结构体字段的含义、单位与取值约束，函数的职责、参数和返回约定，以及并发、数据归属、兼容性等注意事项。函数说明应概括调用者需要知道的契约，避免复述实现步骤；名称和类型已足够清楚时不再补注释。`unsafe` 的必要安全依据应保留。
- 注释应简洁、准确，代码变化时同步更新或删除过时说明，不为增加注释数量而写注释。
- 按职责分层并按业务拆分目录和文件：HTTP/WS 数据模型放在 `model/`，路由和请求响应处理放在 `server/`，业务流程放在 `service/`，实体与数据访问放在 `database/entity/`，迁移放在 `database/migration/`，歌词来源适配放在 `lyric/source/`。
- 不把模型定义、路由、业务逻辑和数据库操作全部堆在同一个文件中。根据职责、复杂度和复用需求拆分，优先沿用现有目录；仅在需要时新增模块，避免无必要的分层和碎片化文件。
- 引入 crate 前先查主 crate、`lyric-macro` 与既有工具；复用已有 reqwest、Tokio、serde、SeaORM 等能力。新增依赖时同步更新对应锁文件。

## 服务与配置边界

- 路由负责入参、状态码与响应，业务决策放在 `service/`，数据库访问沿用 `entity/`；歌词平台适配放在 `lyric/source/`，避免跨层复制业务规则。
- HTTP 管理错误沿用统一响应与语义化状态码；管理 mutation 落库成功后才广播并返回最终状态。内部错误不要伪装成成功。
- WS `/ws` 为展示推送；新增管理功能继续走 `/api/*`。调整展示事件需同步前端类型和消费逻辑，并保持新连接获得完整快照。
- 联网取得歌词、切换来源等异步流程遵循既有 generation 与歌曲身份校验；提交前重新验证，过时结果不得覆盖当前歌曲。
- `bid` 为具体谱面，`sid` 为歌曲级资源归属；标题仅用于展示和标题作用域黑名单，不用于歌曲判等。黑名单的 `bid`、`sid`、`title` 作用域不可混用。
- 启动顺序必须先确定运行目录，再加载全局配置、初始化数据库、设置和服务。不要让配置/迁移落到启动 shell 的旧目录。
- 修改 `config.rs` 的字段、默认值或配置读取逻辑时，同步示例、README 和兼容处理。`GLOBAL_CONFIG` 是延迟初始化的全局配置，现有模式下配置修改后需重启。

当前源码默认监听 `0.0.0.0:41280`，与根 README 的 `127.0.0.1` 示例不同。缓存 TTL 字段在 `Config` 中为 `lyric_cache_ttl_hours`，没有 camelCase rename/alias；当前应使用该字段，不能直接沿用旧 README 的 `lyricCacheTtlHours`。缺省 TTL 为 30 天，`0` 为不过期；调整字段兼容性时应同时补测试和文档。

## 数据库与运行数据边界

运行数据位于可执行文件同目录的 `lyrics/` 下，默认库为该目录中的 `lyric.db`；配置为 `config.json5`，字体为 `font/`，上传临时文件为 `temp/`。相对数据库路径以这个运行目录为基准。

| 表              | 数据归属       | 注意事项                                                                                    |
| --------------- | -------------- | ------------------------------------------------------------------------------------------- |
| `lyric_cache`   | 歌词缓存       | 主键为 `bid`，带 `sid`，读取先按 bid、再按 sid 回退；`updated_at` 为毫秒时间戳，受 TTL 约束 |
| `lyric_binding` | 用户来源绑定   | 按 `sid` 保存，不能随缓存清理删除                                                           |
| `lyric_config`  | 用户单谱面偏移 | 严格按 `bid` 匹配                                                                           |
| `lyric_block`   | 用户黑名单     | `(scope, value)` 复合唯一索引，新增/迁移需处理重复数据                                      |
| `config`        | 用户展示设置   | 一行 JSON；新增字段需兼容已有设置与默认值                                                   |

- schema 修改通过新增 migration 完成，同步实体、DTO 和访问逻辑；不要直接改生产库或重写已执行迁移。迁移顺序由 `Migrator::migrations()` 决定。
- 启动先执行 `Migrator::up`，再执行旧 `lyric_config` 到黑名单的纯数据迁移；重复启动必须安全，不能重复复制或丢失原用户数据。
- 用独立测试库验证空库初始化、旧库升级、重复启动和数据保留。关键索引、字段回填、缓存格式变化需覆盖旧数据兼容路径。
- 修改真实库前先停止相关写入并完成备份；SQLite 的 WAL 状态需一并考虑，可使用一致性备份或关闭连接后复制。勿用真实用户库跑清空、回滚或迁移试验。
- 删除缓存仅操作 `lyric_cache`；来源绑定、偏移、黑名单与展示设置都是用户数据，不得连带删除。
- 资源组装和启动资源补齐不得覆盖 `config.json5`、数据库或用户字体；变更 `ensure_runtime_dir()` 时核对运行路径和旧字体目录迁移。
- `rust-embed` 从后端 crate 的 `../embed`（仓库根目录的 `embed/`）读取资源；前端资源有改动时先在根目录执行 `just assemble-embed`，再构建后端，或使用 `just package ./release`。

涉及业务逻辑时运行相关测试和 Clippy；涉及数据库、上传、WS 或异步切歌时，额外验证旧库迁移、上传限制、连接快照与过时结果丢弃。不要在未验证的情况下宣称支持新的数据库或平台。

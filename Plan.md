# 新版歌词服务与前端改造计划

> 评估日期：2026-09-10

## 最高优先级：按以下顺序执行

1. **前端接入新后端接口，停止依赖旧后端接口。** 优先打通新版歌词 WS 展示链路，切换连接地址、协议和数据来源；缺少管理 HTTP 的功能明确标记待接入，不回退旧接口，也不新增 WS 管理调用。
2. **补充新后端缺失的管理 HTTP 接口。** 将已有管理业务通过 HTTP 暴露，并补齐缺失能力；管理查询、修改、上传全部使用 HTTP。
3. **移除所有运行在 WS 上的管理接口，全部迁移到 HTTP。** 逐项迁移现有 23 个 WS 管理命令，删除新版后端的管理消息分发、setter 连接模式及前端 WS 管理调用封装，不保留管理兼容入口。
4. **重构前端管理界面，使用新后端 HTTP 接口。** 在 HTTP 管理能力补齐、WS 管理入口移除后，重构控制台页面、状态管理和交互，完成逐项接入与联调。

> 此顺序是任务排期的首要依据，优先于下文按功能划分的任务组。接入新版展示所必需的后端缺陷修复随第 1 项处理；控制台界面重构不能抢在接口迁移之前。
> WS 保留后端向展示页推送歌词、样式、清屏、测试效果等展示事件，以及连接所需的心跳；这些推送不属于待移除的管理请求接口。移除范围限于新版运行链路，旧后端仍不维护。

## 1. 范围与最终目标

- 只维护新版后端：`tosu-proxy/src/bin/new.rs`，二进制 `osu-lyric`，默认 feature 为 `new`。旧入口 `bin/old.rs` 与 `src/old/` 不纳入修复或兼容性验收。
- 新前端分为两个独立部分：歌词展示页通过 **WebSocket 接收后端推送**；控制台通过 **HTTP API 查询、修改和上传**。
- 后端负责连接 tosu、搜索与解析歌词、选源、时间定位、缓存和配置持久化；展示页只保存渲染所需的临时状态。
- 控制台操作由后端执行，必要时通过 WS 通知展示页，不再向另一个浏览器查询歌词、缓存或发送管理指令。
- 每项能力分别评估后端业务、接口和前端接入。旧界面、类型定义、数据库表、WS 封装存在，都不能直接等同于新版功能完成。

```text
tosu ──WS──> 新版 osu-lyric ──WS 推送──> /lyrics（歌词展示）
                    │
                    ├── QQ / 网易云搜索、解析、选源
                    ├── 数据库：缓存、样式、屏蔽规则、偏移
                    └── 字体资源
                    ▲
                    │ HTTP 查询 / 修改 / 上传
             /lyrics/controller（控制台及子路由）
```

规划默认沿用当前后端“一首当前歌曲 + 全局显示设置 + 多展示端广播”的模型。在线展示端与定向闪烁列入后续管理任务；每个客户端独立保存样式属于额外扩展，不是现有能力，也不阻塞核心闭环。

## 2. 当前结论与评估口径

**新版后端已有数据源、歌词搜索解析、数据库、WS 服务器及 23 个管理命令，但歌词推送有关键状态缺陷，大部分管理功能缺少 HTTP 接口。前端主要仍在运行旧链路，新 WS 封装尚未驱动页面。当前不能认定新版端到端功能完成。**

| 状态 | 含义 |
| --- | --- |
| 已有实现 | 存在实际代码及调用入口，后续仍需运行验收 |
| 部分 / 待修 | 有基础，但缺闭环、存在明确缺陷或能力受限 |
| 未实现 | 新版代码范围内未找到相应实现或入口 |
| 旧界面待迁移 | 有旧 UI / 数据流，新版功能尚未接通 |
| 不需要 | 由另一端负责，不构成该端功能缺失 |

不使用笼统完成百分比。任务中的 `[x]` 只表示本次盘点已完成；实现任务需在后续开发及验收后分别更新。

### 2.1 新旧边界与混用位置

| 区域 | 代码事实 | 后续处理 |
| --- | --- | --- |
| 新后端入口 | `Cargo.toml` 默认 `new`；`bin/new.rs` 初始化数据库、设置、业务、服务器 | 只沿新版调用链维护 |
| 新后端主体 | `server/`、`service/`、`osu_source/`、`lyric/`、`database/`、`model/`、`setting/` | 本次后端评估范围 |
| 新前端协议准备 | `src/api/model.ts` 有扁平 `type: lyric/setting`；`api/websocket.ts` 封装 23 个 setting key | 展示侧保留必要类型，控制侧转 HTTP |
| 新旧连接混用 | `stores/indexStore.ts` 顶层创建 `new Websocket(false)`，实际 setter 仍调用旧 `wsService` 和旧配置 HTTP，新实例无业务调用 | 移除导入即建连，按路由初始化 |
| 展示仍走旧数据链 | `LyricsBox` 创建 `TosuManager`，后者直连 tosu、调用 adapters、操作浏览器缓存 | 改为后端 WS 驱动，复用渲染而非原调度链 |
| 控制台仍走旧协议 | 内容、样式、黑名单、缓存、客户端页依赖旧 `defaultClient`、store 或 Config | UI 可复用，重接 HTTP 与服务端状态 |
| 全局初始化仍旧 | `initializeApp.ts` 初始化 IndexedDB、注册互查处理器，再请求 `/api/config` | 拆分展示和控制台初始化，移除旧业务处理器 |
| 路由存在但未独立 | `/lyrics`、`/lyrics/lyric`、`/lyrics/controller/*` 已定义；控制台同时挂载 `LyricsBox debug=true` | 独立控制台布局，避免展示组件的连接副作用 |

开发模式下 `LyricsBox` 的 `isDebug` 判断会让控制台预览继续创建 `TosuManager`。旧控制台还用“必须选中其他客户端”的遮罩阻止管理；新版全局管理应在没有 OBS 展示端在线时也可使用。

## 3. 展示链路与新版后端基础能力

本节证据编号见第 8 节。前端状态以“使用新版后端”为准。

| 功能 | 新后端评估 | 展示前端评估 | 剩余工作 / 依据 |
| --- | --- | --- | --- |
| 接入 tosu、切歌与时间更新 | 已有连接、超时、心跳、重连及 Song/Time 分发 | 旧页面仍自己连接 tosu，职责迁移未完成 | B1；移除浏览器直连，验收无歌曲、断线恢复和同音频不同谱面 |
| QQ / 网易云搜索与歌词获取 | 已有并行搜索、候选与歌词下载 | 不需要；旧 adapters 仍被调用 | B2；第三方接口当前可用性未联调 |
| 自动匹配准确性 | 部分：时长过滤为 15 秒容差，标题 / 歌手过滤直接返回 `true` | 不需要 | B2；补标题 / 作者匹配、unicode 空值回退、候选失败后继续尝试 |
| LRC 解析、原文 / 翻译合并 | 部分：有自写解析器，并非旧计划所写的 `lyrics_helper_rs` | 旧字段为 main/origin、first/second，新字段为 origin/translation，未适配 | B2、F2；补格式、排序、合并边界验证 |
| WS 连接与广播 | 已有会话表、断开清理、ping/pong、展示/setter 分类、广播与内部单播 | 新封装有重连库，但 URL 不匹配，歌词只打印 | B3、F1；接正确路由并写入展示 store |
| 按时间切行 | 部分 / 待修：`time_next` 已计算 current、nextTime、sequence 并广播 | 仍由旧 tosu 本地时间轴驱动 | B4；修首包、切歌、跳转和时间语义 |
| 首次进入 / 重连同步完整歌词和设置 | 未实现：连接时只登记会话，无当前状态快照 | 未实现快照恢复与广播设置处理 | B3、F1；必须按每个连接补发，不能仅全局切歌时发一次 |
| 清屏、无歌词、屏蔽时隐藏 | 部分：有 `setClear` 广播，无歌词与屏蔽尚未完整联动清理 | 未处理新版 setClear、current=-1 | B1、B4、F2；禁止残留上一首 |
| 暂停、进度跳转、偏移、末行 | 部分：按 tosu 时间定位，无完整暂停 / 播放状态协议 | 新版未接入，旧滚动依赖 `tosu.getNextTime()` | B4；统一剩余毫秒、暂停、反向跳转、nextTime=-1 |
| 多展示端同步 | 部分：广播已有，晚加入无首包补发 | 新版未实现 | B3；每连接快照、歌曲 / 歌词版本与增量同步 |
| 静态站点与深层路由 | 已有 `lyrics/{**path}`、index fallback、根路径重定向 | 路由与 Vite `/lyrics` base 已有 | B3、F3；生产构建后直达、刷新子路由仍需验收 |

### 3.1 实际路由与已有 WS 管理命令

| 当前入口 | 方法 / 协议 | 实际能力 | 限制 |
| --- | --- | --- | --- |
| `/ws` | WebSocket | 展示广播与 setting 管理命令 | **不是 `/api/ws`**；前端常量仍为 `/api/ws` |
| `/ws?setter=true` | WebSocket | 管理请求及部分响应，不接收展示广播 | 仅当前过渡实现，最终控制台不依赖；代码按 query 是否存在判断，`setter=false` 也归入 setter |
| `/api/audio/len?path=...` | GET | 获取音频时长 | 已有辅助接口，新展示不再自行请求音频计算时长 |
| `/api/font/upload` | POST multipart | 保存首个字体文件到工作目录 `font` | 已有处理器，重复上传有文件打开模式缺陷 |
| `/api/font/download` | GET | 下载保存的字体 | 已有处理器，但只有一个文件，无多字体资源管理 |
| `/api/lyric/upload` | POST multipart | 首个 UTF-8 文件解析后绑定当前歌并写缓存 | 已有处理器，即时推送遗漏；无时间标签的普通文本不受当前解析器支持 |
| `/api/config` | 无新版路由 | 旧 `configService` 仍在请求 | 旧调用不代表新版配置 API 已实现 |
| 其他管理 HTTP | 无新版路由 | 当前歌曲、设置、搜索、偏移、屏蔽、缓存、客户端管理 | 需要新增 / 迁移 |

依据 `server/mod.rs`：`/api` 下只挂 font、audio、lyric，WS 在根路由单独挂载。后端 README 的 HTTP 路径省略 `/api`；原 Plan 的 WS 路径与控制台通信方向也需纠正。

当前 WS 分发实际有 **23 个 key**：

- `setClear`。
- `set/getFont`、`set/getFontSize`、`set/getAlignment`、`set/getColor`、`set/getTranslationMain`、`set/getSecondShow`（12 个）。
- `getLyricList`、`setLyricSource`、`getAllLyric`。
- `setBlock`、`setUnblock`、`getBlockList`。
- `getCacheCount`、`setCacheClean`。
- `getLyricOffset`、`setLyricOffset`。

这些是实际入口，不代表全部可靠可用。`getLyricList` 只读当前搜索内存，不按请求参数重新搜索；`getAllLyric` 返回含 `time`（秒）的后端歌词行。响应保留原 key，`set_replay` 不生成 README 示例的 `rep...`。样式 setter 只广播，`setClear` 无直接应答，不能统一按“发送后等待 echo”使用。

## 4. 控制台功能盘点：业务、HTTP、前端分别评估

| 功能 | 新后端业务 / 数据层 | 新后端 HTTP | 前端情况 | 后续任务 |
| --- | --- | --- | --- | --- |
| 独立页面、导航、子路由 | 静态服务与 fallback 已有 | 页面请求已有 | 路由已有，但混挂 LyricsBox、旧初始化与选端遮罩 | F-01 |
| 当前歌曲、bid/sid、作者、播放 / 获取状态 | 部分：有内部 OsuSongInfo，无完整查询模型 | 未实现 | 旧内容页经其他客户端获取标题 / bid | B-02、B-05、F-04 |
| 完整歌词查看与刷新 | 已有 WS getAllLyric | 未实现 | CurrentLyrics 旧界面待迁移 | B-05、F-04 |
| 单行原文 / 翻译复制 | 不需要专门后端操作，提供歌词即可 | 复用歌词查询，尚缺 | 按钮已有，需从 first/second 转新版数据 | F-04 |
| 各来源搜索结果 | 部分：读取 music_cache；命中歌词缓存时可能无候选 | 未实现 | SearchResult 旧界面待迁移 | B-05、F-04 |
| 主动搜索 / 重新搜索 | 底层搜索已有，无管理层主动搜索入口 | 未实现 | 旧“搜索”按钮查询旧端已搜结果，不是新 HTTP 搜索 | B-05、F-04 |
| 按来源和 key 预览候选 | 底层 fetch_lyrics 已有，无独立预览服务 | 未实现 | 有旧查看动作，但 nowLyric 未实际渲染为预览内容 | B-05、F-04 |
| 应用候选 / 换源 | 部分：下载、缓存已有，刷新状态不完整，未持久化来源 key | 未实现 | 旧端传整个 Lyric；各行应用共用上次预览值，需绑定候选身份 | B-01、B-05、F-04 |
| 查看 / 恢复来源绑定 | 部分：只保留覆盖后歌词缓存，不能查询来源身份，清缓存会丢选择结果 | 未实现 | 未实现 | B-05、F-04 |
| 手动上传 LRC | 部分：上传、解析、当前歌绑定、缓存已有，即时刷新待修 | **已有** `/api/lyric/upload` | 未实现控制台入口 | B-01、B-07、F-06 |
| 清空当前展示 | 部分：WS 广播已有，清屏持续 / 恢复语义未定义 | 未实现 | 未见新版控制入口 | B-02、B-05、F-04 |
| 当前歌偏移查看 / 调整 / 归零 | 部分：WS、内存和数据库保存已有，写入错误忽略，依赖下一次时间更新生效 | 未实现 | 控件未实现，WS 方法封装不算页面 | B-01、B-05、F-04 |
| 主 / 副颜色 | 已有存取和广播，校验、默认值、错误处理待补 | 未实现 | 旧双颜色 UI / 渲染已有，仍旧协议保存 | B-03、F-03 |
| 对齐方式 | 已有 first/second 配置存取与广播 | 未实现 | 旧 UI / 渲染只有整体对齐，主副分别配置未适配 | B-03、F-03 |
| 翻译为主、副歌词显隐 | 已有数据库与 WS 广播 | 未实现 | 旧开关和渲染已有，新版读写 / 广播未接 | B-03、F-03 |
| 字体选择、主副独立字体 | 部分：WS first/second 可存取，无字体资源模型 | 设置接口未实现 | 仅整体选择，旧主副 FontFace helper 未接完整流程 | B-03、B-07、F-03、F-06 |
| 主 / 副字号 | 已有 WS 存取，类型与范围未校验 | 未实现 | 控件未实现，渲染字号硬编码 | B-03、F-03 |
| 上传 / 应用字体 | 部分：单文件上传下载已有，覆盖与刷新机制待修 | **已有** upload/download，多字体管理未实现 | 孤立 Upload.tsx 未被页面引用，仍加载旧 `/LRC.otf` 等路径 | B-07、F-06 |
| 歌词阴影 | 未实现后端设置项 | 未实现 | 有 Shadow store 字段，无完整控件，阴影 CSS 固定 | B-09、F-08 |
| 黑名单列表、当前歌拉黑 / 解除 | 部分：WS 与表已有，ID、重复操作、清屏恢复有缺陷 | 未实现 | 旧列表 / 当前标题拉黑待迁移，数据结构不同 | B-04、F-05 |
| 按谱面 / 谱面集 / 标题管理任意规则 | 部分：有 bid→title→sid 回退查询，无显式 scope，管理入口只操作当前歌 | 未实现 | 旧添加 / 删除 UI 已有，不能直接映射新版 | B-04、F-05 |
| 编辑屏蔽规则、清空列表 | 无独立规则编辑 / 清空业务入口 | 未实现 | 旧 UI 已有，本地更改不等于服务端保存 | B-04、F-05 |
| 黑名单备注与时间 | 未实现，表无 reason/timestamp | 未实现 | 旧表单 / 列已有，需要补后端或移除未支持字段 | B-10、F-05 |
| 缓存数量与清空 | 已有 count/delete_all 和 WS，清空 handler 忽略错误，不清播放内存 | 未实现 | 旧清空按钮已有，新计数未接 | B-06、F-05 |
| 缓存分页、搜索、单项 / 标题删除 | 部分：实体有标题 contains 查询、ORM 删除基础，无完整管理服务 | 未实现 | 旧分页、按 sid / 标题删除已有；StoredLyrics.tsx 为空 | B-06、F-05 |
| 缓存过期与自动清理 | **未实现**：表无时间字段，未找到 TTL 逻辑 | 未实现策略接口 | 未实现 | B-06、F-05；撤回原计划“过期已完成” |
| 在线展示端列表、定向闪烁测试 | 部分：内部会话 ID、表、单播已有，无列表接口 / blink 命令 | 未实现 | 旧 Client UI 已有，新后端不支持旧 online 协议 | B-08、F-07 |
| 每客户端独立设置 | 未实现，歌曲与配置全局共享 | 未实现 | 旧 target 互控不是新版独立配置 | 扩展待定，不阻塞核心交付 |
| 控制台夜间模式 | 不需要 | 不需要 | localStorage、组件已有，需脱离旧配置初始化 | F-01 |
| 请求失败、保存状态、重试、空状态 | 局部 HTTP 状态 / WS error 已有，无统一契约 | 只有辅助接口局部提供 | 零散 loading、console/alert 与空态，新流程未实现 | B-02～B-07、F-02～F-06 |

## 5. 代码缺陷与关键缺口（本次仅记录）

以下来自静态控制流，不是运行复现记录；后续应按触发条件补必要的回归验证。

| 编号 | 代码事实与影响 | 任务 |
| --- | --- | --- |
| R1 首包丢失 | `time_next` 在发送前置 `is_song_changed=false`，之后 find_line=None 或 now_index==index 会提前返回。初始 now_index=0，进入第 0 行即可吞掉完整歌词包，之后只发下标 | B-01 |
| R2 切歌状态残留 | `song_change` 调用的 `clear_cache` 主要清搜索任务 / 结果，不重置播放下标、时间窗口和旧歌词；屏蔽或搜索错误会提前返回，新歌可能被旧窗口挡住，旧歌词也可残留 | B-01、B-04 |
| R3 换源 / 上传刷新遗漏 | 两处 `_ = self.time_next(0)` 均未 await，Future 被丢弃；换源还未重置完整歌词标志 / 时间窗口。上传虽重置字段，仍要等后续 Time；即时刷新应按实际播放时间而非固定 0 | B-01、B-05、B-07 |
| R4 晚加入无快照 | `connect` 只登记会话；完整歌词标志属于全局而非每连接，OBS 刷新 / 中途进入无法可靠恢复 | B-02、F-02 |
| R5 时间和清屏语义 | next_time=下一行起点减当前行起点，行中跳转时不是剩余时长；窗口使用 <=end；首行前直接返回；current=-1、暂停 / 空歌、清屏恢复尚无完整状态流程 | B-01、B-02 |
| R6 黑名单错误 | 实体返回 (bid,sid,title)，服务按 (sid,bid,title) 解包；set_block 先 take 当前歌曲，状态相同时直接返回导致绑定丢失；拉黑无同步清屏 | B-04 |
| R7 作用域混用 | 一条配置混合 bid/sid/title/disable/offset，读取按 title/sid 回退；单曲偏移 / 屏蔽可能影响同名或同集谱面，保存 title 与查询 title_unicode 也不统一 | B-04、B-05 |
| R8 配置失败仍广播 | 样式保存 **有 await，但忽略 Result**，失败仍广播。启动设置 key 是 trans_main/align/show_second，WS 使用 translation-main/alignment/second-show，两套 key、默认值和内存状态未统一；新库 getter 可报缺失 | B-03 |
| R9 缓存缺陷 | 失效缓存只构造 delete_by_id(bid)，未 exec/await；sid 回退时还需使用命中记录的主键。清空 handler 忽略错误、仅广播清屏，未定义内存失效和重搜；没有 TTL | B-06 |
| R10 字体覆盖失败 | 字体存在时 `get_font_file` 使用只读 File::open，上传复制写入失败并 expect；需要可写 / 截断、错误响应和浏览器资源更新机制 | B-07 |
| R11 新前端封装不完整 | onLyric 只打印，无 echo 设置广播被忽略；请求队列不清理，error 不转失败；统一等 echo 但样式 / 清屏不向 setter 应答 | F-01～F-03；不继续扩建 WS 管理 RPC |
| R12 解析与搜索边界 | 正则要求小数时间和正文，非所有 LRC / 文本均可解析；get_line_mut 有切片 enumerate 局部下标当全局下标的路径，需验证翻译插入排序。标题 / 作者过滤占位，候选下载遇 `?` 提前退出 | B-01、B-05 |

还需检查：搜索 / 下载持有 `LYRIC_SERVICE` 全局锁、快速切歌取消和旧结果覆盖新歌。已有防抖 / 取消通道是基础，不代表并发正确性已验收。

## 6. 目标协议与 HTTP 清单（建议，尚未实现）

### 6.1 展示 WS 与状态约定

- 优先沿用真实 `/ws`，按当前 host 构造 ws/wss URL；开发代理同时覆盖 `/api` 与 `/ws`。若改路径，需同步改两端和文档。
- 每次建立 / 恢复连接下发完整快照：歌曲身份、歌词版本、完整列表、当前行、剩余时长、播放 / 获取 / 屏蔽状态、完整有效样式。
- 增量消息关联歌曲 / 歌词版本；切歌、换源、上传导致内容替换时发完整数据，避免将新下标应用到旧列表。
- 明确 current=-1 隐藏、nextTime=-1 末行；无歌曲、搜索中、无歌词、错误、屏蔽、暂停均有明确状态。WS 时间统一毫秒，HTTP 现有 time 秒值需统一或显式转换。
- 前端动画可以消费后端时间，不再保留 `LyricManager.jump/nextTime` 作为播放定位来源，也不在浏览器搜索、解析、持久缓存歌词。
- 控制台不建 setter WS，后端也移除 setter 模式和所有 WS 管理请求入口。HTTP 成功更新后由后端广播；控制台通过初始查询、操作后刷新、可取消的适度 HTTP 轮询获得状态。

### 6.2 控制台 HTTP 候选接口

以下路径为后续开发建议，除标注沿用者外均不存在。结构在 B-00 定稿，不直接照搬旧 Config / BlacklistItem。

| 候选接口 | 职责 | 依赖 |
| --- | --- | --- |
| GET `/api/status` | 当前歌曲、播放 / 获取状态、歌词版本、tosu 连接状态 | B-02 |
| GET/PATCH `/api/settings` | 完整默认设置、部分修改；明确主副字段、字号单位、校验 | B-03 |
| GET `/api/lyrics/current` | 完整歌词、有效偏移、已选来源与版本 | B-05 |
| GET `/api/lyrics/search-results`、POST `/api/lyrics/search` | 已有候选、主动按当前歌 / 输入条件搜索，返回关联身份 | B-05 |
| GET `/api/lyrics/preview?source=...&key=...` | 结构化候选预览，不改当前播放 | B-05 |
| PUT/DELETE `/api/lyrics/source` | 保存绑定 / 恢复自动匹配，校验歌曲与版本 | B-05 |
| PUT `/api/lyrics/offset` | 毫秒偏移 / 归零，返回最终有效值 | B-05 |
| POST `/api/display/clear` | 清屏，持续与恢复语义在 B-02 定稿 | B-02、B-05 |
| POST `/api/lyric/upload`（沿用） | 补歌曲 / 版本校验、结构化结果与即时推送 | B-07 |
| GET/POST `/api/blocks`、PATCH/DELETE `/api/blocks/{id}`、DELETE `/api/blocks` | 列表、增改删、清空，显式 bid/sid/title 作用域 | B-04 |
| GET `/api/cache`、GET `/api/cache/count` | 分页 / 搜索、total、稳定主键和数量 | B-06 |
| DELETE `/api/cache/{bid}`、POST `/api/cache/delete-by-title`、DELETE `/api/cache` | 单项、标题、全量删除，返回实际影响数 | B-06 |
| GET/PUT `/api/cache/policy` | TTL、关闭过期及自动清理策略 | B-06 |
| POST `/api/font/upload`、GET `/api/font/download`（沿用） | 修复单字体处理；独立主副资源另补资源 ID、列表 / 下载地址 | B-07 |
| GET `/api/clients`、POST `/api/clients/{id}/blink` | 展示端列表、HTTP 发起测试，后端 WS 单播 | B-08 |

共同要求：统一 JSON 错误结构与 HTTP 状态码；区分无当前歌、非法参数、源失败、写入失败。换源、上传、屏蔽、偏移携带目标歌曲 / 预期版本，切歌后拒绝过期操作。写入先验证并确认成功，再返回最终状态和广播；部分样式更新不能覆盖未提交的另一行配置。

## 7. 执行优先级、任务分组与验收条件

### 执行顺序与完成门槛

| 顺序 | 工作重点 | 对应任务 | 进入下一步的条件 |
| --- | --- | --- | --- |
| 1 | 前端接新后端，停止走旧接口 | F-01 的连接 / 初始化解耦、F-02；必要的 B-00～B-02 | 新版歌词推送能驱动展示，运行入口不再调用旧后端；缺管理 HTTP 的功能停留在明确待接入状态 |
| 2 | 补齐新后端管理 HTTP | B-03～B-07；增强管理对应 B-08～B-10 按功能范围补充 | 现有 23 个 WS 管理命令均有 HTTP 对应能力，核心管理 HTTP 完成独立验收；尚未实现的增强功能继续单独标注 |
| 3 | 全量移除 WS 管理入口与调用 | B-11、F-10 | WS 管理请求无法再查询或修改业务状态；前端无 setter 连接 / WS 管理封装；HTTP 操作仍能触发展示推送 |
| 4 | 重构管理界面并接新 HTTP | F-01 的控制台布局、F-03～F-08；F-09、D-01 收尾 | 控制台功能通过新 HTTP 完成查询与操作，展示通过 WS 同步，前后端分别验收后完成联调 |

第 1 项仅做接入所需的页面和初始化调整，不提前开展完整控制台重构。第 2 项可直接用 HTTP 请求独立验证，不以新管理界面完成为前提。下方 P0～P3 保留为功能任务组编号，不代表执行优先级；跨组排期以上表为准。

### 第 3 优先级专项目标：彻底移除 WS 管理接口

- [ ] **B-11 后端：迁移并删除全部 WS 管理入口。** 对第 3.1 节的 23 个 key 建立 HTTP 对照并验证功能等价，将可复用业务抽到服务层；删除 WS 入站管理分发、管理请求 / 响应处理和 setter 分类。不得将 WS 管理请求转发 HTTP 作为兼容层保留。保留独立的展示事件广播 / 单播能力。验收：23 项管理能力均可经 HTTP 调用；向普通 WS 连接或带 setter 参数的连接发送原管理请求，均不能查询管理数据或改变业务状态；HTTP 成功操作仍能正确推送展示变化。
- [ ] **F-10 前端：删除 WS 管理调用链。** 移除 `api/websocket.ts` 中管理 send/await 封装、`new Websocket(false)`、旧 `wsService` 管理互查 / 控制调用及只用于管理请求的模型；展示事件类型按接收职责保留。验收：运行入口无 WS 管理发送、echo 管理请求队列或 setter 连接；待重构控制台不调用旧协议临时维持功能，后续统一接新 HTTP。

### 任务组 P0：盘点与契约

- [x] **A-01** 确认新后端边界，追踪真实路由和业务调用链。
- [x] **A-02** 梳理前端混用与控制台清单，分列后端业务、HTTP、前端状态。
- [x] **A-03** 修正旧计划“WS 未开发”“缓存过期已完成”“控制台使用 WS”等结论。
- [ ] **B-00 后端：定稿数据与通信契约。** 输出 HTTP/WS 示例、默认值、时间单位、歌曲 / 版本身份、错误码、屏蔽作用域、来源绑定和缓存策略。验收：第 4 节每项都对应接口、本地职责或明确扩展归属，核心模型不存在待猜测字段。

### 任务组 P1：后端推歌词、前端展示、HTTP 样式控制的最小闭环

- [ ] **B-01 后端：修复歌词状态和时间推进。** 处理 R1～R3、R5、R12，统一切歌 / 换源 / 上传重置与按当前进度刷新；无歌词及时清空，异步结果不覆盖新歌。验收：第 0 行、首行前、行中进入、正反向跳转、末行、连续切歌、搜索失败均有确定结果，基础 LRC 原文 / 翻译排序正确。
- [ ] **B-02 后端：WS 快照与状态查询。** 补每连接首包、重连补发、版本、清屏 / 暂停 / 错误状态及 GET status。验收：任意时刻打开第二个展示页或重连均恢复相同有效歌词与样式；tosu 断开 / 恢复可识别且不残留旧状态。
- [ ] **B-03 后端：HTTP 设置与一致持久化。** 统一 key / 默认值，修 R8，实现 GET/PATCH、类型范围校验、部分更新、写入错误处理和 WS 广播。验收：新库可读完整默认值，重启保留修改，写失败不会返回成功或广播已保存状态。
- [ ] **F-01 前端：路由与初始化解耦。** 分开两种入口，控制台去掉自动挂载的旧 LyricsBox、选端遮罩、IndexedDB 和互查注册；消除导入即建旧 / 新双 WS，夜间模式单独初始化。验收：直接进入控制台子路由不连接 tosu、旧 WS 或 setter WS；没有 OBS 在线也能管理。
- [ ] **F-02 前端：新版歌词展示。** 实现 WS 接收层、数据校验、store、快照 / 增量 / 清屏 / 设置处理与连接生命周期；适配 origin/translation、时间单位，保留滚动和对齐效果。验收：展示只消费后端 WS，刷新切歌不串词，空歌词 / 暂停 / 末行不产生无效动画。
- [ ] **F-03 前端：HTTP 请求层与样式控制。** 初始查询、保存 / 失败状态，接主副颜色、字号、字体选择、对齐、翻译优先、显隐，操作后读回有效值。验收：控制台只发 HTTP，多展示页通过后端 WS 同步，刷新和后端重启保持设置。

依赖：第 1 优先级先完成 F-01 的接入解耦、F-02 及必要的 B-00/B-01/B-02；B-03 属于第 2 优先级，F-03 在第 3 优先级移除 WS 管理后进入第 4 优先级实施。本组完整闭环在控制台接入后验收，不作为其他管理 HTTP 开发的前置条件。

### 任务组 P2：控制台内容、屏蔽、缓存与上传管理

- [ ] **B-04 后端：屏蔽规则服务与 HTTP CRUD。** 修 R6/R7，明确 bid/sid/title 优先级和独立作用域并迁移数据；支持当前歌及任意规则增改删 / 清空、幂等操作、即时清屏与解除恢复。验收：ID 不互换，重复操作不丢绑定，单曲规则不意外扩展到同名曲，删规则不误删偏移。
- [ ] **B-05 后端：内容管理 HTTP。** 补当前歌词、候选、主动搜索、预览、来源应用 / 恢复、偏移、清屏；来源绑定独立于可清除缓存持久化，缓存命中也能主动搜候选；操作校验版本，偏移写入失败返回错误。验收：预览不改播放，应用立即同步全部展示端，重启 / 清缓存不丢绑定，切歌后旧操作被拒绝。
- [ ] **B-06 后端：缓存管理和过期。** 修 R9，补稳定分页、搜索、数量、单项 / 标题 / 全量删除；明确当前内存失效与重搜，增加时间字段、TTL、旧数据迁移与清理。验收：total 和删除结果正确，过期不命中，清缓存不删来源绑定、偏移或屏蔽规则。
- [ ] **B-07 后端：上传和字体资源。** 修 R3/R10，补歌词编码 / 格式错误、歌曲版本校验；字体重复覆盖、版本化加载与独立主副资源模型。验收：有效 LRC 按当前进度立即显示，非法文件不替换旧歌词；连续上传大小不同字体可用，主副独立应用并在重启后恢复。
- [ ] **F-04 前端：内容页 HTTP 接入。** 展示歌曲身份、完整歌词 / 复制、候选搜索、实际预览区、准确候选应用 / 来源恢复、偏移调整 / 归零、清屏，处理切歌冲突。验收：每个应用按钮对应本行候选，不能误用上次预览；所有操作有加载、结果和失败反馈。
- [ ] **F-05 前端：屏蔽与缓存页面。** 用服务端规则替换旧本地列表，接 CRUD / 清空、作用域、缓存 total / 分页 / 删除 / TTL；成功后刷新，失败不伪装已保存。验收：跨页面 / 浏览器 / 重启状态一致，删除末页后回退有效页；备注 / 时间若未持久化应移除或明确待实现。
- [ ] **F-06 前端：上传入口和字体应用。** 把孤立表单接入控制台，使用新 HTTP 结果和后端资源 URL / 版本加载 FontFace，支持独立主副字体。验收：不需手工放 `/LRC.otf`，失败留在页面可重试，成功在展示端可见。

依赖：新版展示接入后，B-04～B-07 按第 2 优先级先补 HTTP 并独立验收；第 3 优先级移除 WS 管理后，F-04～F-06 按第 4 优先级重构接入并联调。只完成一端不勾选整个功能。

### 任务组 P3：增强管理与迁移收尾

- [ ] **B-08 后端：在线展示端和定向闪烁。** 基于已有会话表增加 HTTP 列表 / blink，区分在线会话与持久身份，WS 单播测试消息。验收：仅目标端闪烁，离线返回明确错误，列表不含控制台 setter。
- [ ] **F-07 前端：客户端页接 HTTP。** 列表刷新 / 轮询、测试与离线反馈；选端仅用于明确的定向操作，不阻挡全局管理。验收：离线 / 重连后列表、选择和测试反馈正确。
- [ ] **B-09 后端：阴影设置。** 补模型、默认值、校验、持久化和 WS 同步。验收：修改 / 关闭阴影后重启保留有效值。
- [ ] **F-08 前端：阴影控件和渲染。** HTTP 接入并替换固定 CSS，提供主副效果预览。验收：修改后全部展示端同步，错误可见。
- [ ] **B-10 后端：黑名单元数据。** 若保留旧 UI 的备注 / 创建时间，补字段、迁移及 HTTP 返回 / 编辑。验收：跨浏览器 / 重启保留，编辑不改变规则身份。
- [ ] **F-09 前端：旧链路清理。** 新功能验收后移除或隔离无调用的 tosuManager、adapters、浏览器歌词缓存、旧 WS 消息 / 互查、旧 Config、旧解析和时间轴调用。验收：运行入口不再依赖 `/api/proxy`、旧 `/api/config`、tosu 直连、IndexedDB 歌词缓存或多余 WS；清理前先确认页面无剩余依赖。
- [ ] **D-01 文档与运行入口。** 更新两份 README 的新二进制、真实路由、HTTP/WS 职责、示例、配置和启动方式；核对 justfile 的构建产物与复制流程。只维护新版说明，不修旧后端。

扩展待定：每展示端独立样式 / 持久客户端身份、旧浏览器缓存迁移。若决定实施，分别新增后端模型 / 接口与前端任务，不混入全局控制的完成度。

### 最终验收（后续实施阶段执行）

- [ ] **后端独立：** 用 WS 客户端和 HTTP 请求覆盖首包、重连、切歌、进度跳转、暂停 / 恢复、无歌词、屏蔽、上传、换源、偏移，不依赖旧前端证明正确。
- [ ] **前端独立：** 用确定性 WS / HTTP 样例覆盖快照、增量、版本变化、错误、延迟、断线；控制台不发 WS 管理请求，展示不直连 tosu 或第三方。
- [ ] **WS 管理移除：** 逐项核对原 23 个管理命令的 HTTP 替代接口；检查后端入站分发、setter 模式、前端发送封装均已移除，向 WS 发送原管理请求无法执行管理操作，展示推送与心跳仍正常。
- [ ] **联调：** 实际 tosu + 新后端 + 控制台 + 至少两个展示窗口（含 OBS），验证 HTTP → 后端 → WS，刷新 / 重启不丢状态。
- [ ] **数据：** 新库默认值、旧库迁移、重启持久化、写入失败、TTL、清缓存后保留来源绑定和偏移。
- [ ] **部署：** 新后端构建、前端类型检查 / 构建通过；直接打开、刷新控制台深层路由正常，开发代理与生产路径一致。
- [ ] **回填状态：** 分别更新后端业务、HTTP、前端列，记录验证方式与遗留问题；只有两端和相应链路通过，才勾选完整功能。

## 8. 代码证据索引

代码变更后应按实际实现和验证结果更新本计划。

| 索引 | 主要代码 |
| --- | --- |
| B0 新旧边界 | [Cargo.toml](tosu-proxy/Cargo.toml)、[new.rs](tosu-proxy/src/bin/new.rs)、[lib.rs](tosu-proxy/src/lib.rs) |
| B1 tosu / 事件 | [tosu.rs](tosu-proxy/src/osu_source/tosu.rs) handle_tosu_message/update_song_info；[song_source_service.rs](tosu-proxy/src/service/song_source_service.rs) on_song_update/on_clean |
| B2 搜索 / 解析 | [source/mod.rs](tosu-proxy/src/lyric/source/mod.rs) search_lyrics/song_filter_*；[qq.rs](tosu-proxy/src/lyric/source/qq.rs)、[netease.rs](tosu-proxy/src/lyric/source/netease.rs)；[lyric_source.rs](tosu-proxy/src/lyric/lyric_source.rs) parse/get_line_mut/find_line |
| B3 路由 / WS | [server/mod.rs](tosu-proxy/src/server/mod.rs)、[config.rs](tosu-proxy/src/config.rs)、[server/websocket.rs](tosu-proxy/src/server/websocket.rs) connect/ALL_SESSIONS、[file.rs](tosu-proxy/src/server/file.rs) |
| B4 歌词 / 管理业务 | [lyric_service.rs](tosu-proxy/src/service/lyric_service.rs) song_change/time_next/clear_cache/set_song_by_key/set_manual_lyric/set_block/set_offset/get_all_block_list；[websocket_service.rs](tosu-proxy/src/service/websocket_service.rs) handle_setting/base_setter/set_cache_clean |
| B5 数据持久化 | [lyric_cache.rs](tosu-proxy/src/database/entity/lyric_cache.rs)、[lyric_config.rs](tosu-proxy/src/database/entity/lyric_config.rs)、[setting.rs](tosu-proxy/src/database/entity/setting.rs)、[connect.rs](tosu-proxy/src/database/connect.rs) |
| B6 设置 / 协议模型 | [model/setting.rs](tosu-proxy/src/model/setting.rs)、[setting/mod.rs](tosu-proxy/src/setting/mod.rs)、[设置宏](tosu-proxy/lyric-macro/src/lib.rs)、[WS lyric](tosu-proxy/src/model/websocket/lyric.rs)、[WS setting](tosu-proxy/src/model/websocket/setting/mod.rs) |
| B7 上传 / 辅助 HTTP | [font.rs](tosu-proxy/src/server/font.rs)、[lyric.rs](tosu-proxy/src/server/lyric.rs)、[audio.rs](tosu-proxy/src/server/audio.rs) |
| F1 新协议 / 混用 store | [api/model.ts](src/api/model.ts)、[api/websocket.ts](src/api/websocket.ts)、[indexStore.ts](src/stores/indexStore.ts)、[settingsStore.ts](src/stores/settingsStore.ts)、[constants.ts](src/config/constants.ts) |
| F2 展示 / 旧业务 | [LyricsBox](src/pages/LyricsBox/index.tsx)、[tosuManager.ts](src/services/managers/tosuManager.ts)、[旧 WS](src/services/webSocketService.ts)、[initializeApp.ts](src/hooks/initializeApp.ts)、[configService.ts](src/services/configService.ts) |
| F3 路由 / 控制台 | [App.tsx](src/App.tsx)、[routes](src/routes/index.tsx)、[Controller](src/pages/Controller/index.tsx)、[vite.config.ts](vite.config.ts) |
| F4 内容 | [CurrentLyrics](src/pages/Controller/ControlTools/Content/CurrentLyrics.tsx)、[SearchResult](src/pages/Controller/ControlTools/Content/SearchResult.tsx)、[StoredLyrics（空）](src/pages/Controller/ControlTools/Content/StoredLyrics.tsx) |
| F5 样式 / 上传 | [TextStyle](src/pages/Controller/ControlTools/TextStyle/index.tsx)、[Font](src/pages/Controller/ControlTools/TextStyle/Font.tsx)、[Upload（未接入）](src/components/ui/Upload.tsx)、[fonts.ts](src/utils/fonts.ts) |
| F6 管理页面 | [BlackList](src/pages/Controller/ControlTools/BlackList/BlackList.tsx)、[CacheManager](src/pages/Controller/ControlTools/CacheManager/index.tsx)、[Client](src/pages/Controller/ControlTools/Client/index.tsx) |

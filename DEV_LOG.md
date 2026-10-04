# DEV_LOG — 新版歌词核心开发日志

> 本文件记录**实际开发进度**，以实际代码为准。
> 当前计划见同目录的 `Plan.md`（**唯一有效的开发计划**）。
> 项目目录：`D:\.1yrics\tosu-lyrics-now`（仓库根目录）。
>
> 说明：历史记录里的路径 `D:\.1yrics\tosu-lyrics-dev` 是开发期间的目录名，
> 整理后等价于当前的 `tosu-lyrics-now`；**历史内容原样保留，未改写**。

---

## 当前状态

```
时间：2026-09-13 06:05
阶段：展示链路已实机验证；管理侧改为**纵向切片**推进，切片 A（B-03 设置 + TextStyle）已完成
状态：B-03 已可人工验收（挡住页面的旧遮罩已移除）；B-04～B-07 未开始

本轮（纵向切片）进度
- [x] **切片 A：B-03 设置 + Controller TextStyle** —— 已完成并验证
  - 新增 `GET/PATCH /api/settings`，统一设置模型，修复 R8（两套 key、写失败仍广播）
  - TextStyle 页走 HTTP；展示页通过 WS 收到变化并真正生效（字号 / 颜色 / 对齐 / 副歌词显隐）
  - 顺带修复"回菜单时在途换歌任务让旧歌词复活"的并发缺陷
- [x] **F-01 最小前置修复（为 B-03 人工验收）** —— 移除挡住全局管理页的旧 `clientSignal` 遮罩
  - 只改 `src/pages/Controller/index.tsx`；`/lyrics/controller/client` 保持原样留给 B-08
  - 见 §8 06:05 条
- [x] **异步取消专项验证（2026-09-13 06:20）** —— 10 项断言通过，取消机制无误伤，原策略未变
  - 新脚本 `_probe/async-cancel-test.mjs`；见 §8 06:20 条
  - 附带发现一个**既有**行为（非本轮引入）：串行处理导致快速切歌时中间会闪过上一首，
    最终态正确；根治属于 generation 校验（B-01/B-05）
- [x] **切片 B：B-04 黑名单** —— **后端 + 前端 + 全链路，全部完成并验证**
  - B-00 契约同步落地（歌曲身份 / generation / 黑名单 scope / cache 与 source 边界）
  - 新表 `lyric_block` + 旧库安全迁移；`lyric_config` 收敛为只存 offset
  - `GET/POST/PATCH/DELETE /api/blocks`（含 `{id}` 与清空），统一错误体、幂等
  - 修 R6（元组互换）、R7（作用域混用 + 逐级回退）
  - 验证：`_probe/blocks-api-test.mjs` 21 项断言全通过；旧库迁移实测通过；见 §9
  - 前端：`blocksService.ts`（HTTP CRUD）+ `useBlocks.ts`（服务端唯一真源）+ `BlackList.tsx` / `index.tsx` 重写
  - 全链路：真实浏览器点击 → HTTP → 后端 → 状态 → WS → LyricsBox 清屏 / 恢复，27 项断言通过
- [x] 切片 C：B-05 歌词内容管理 —— **后端 + 前端 + 全链路完成并验证**
- [x] 切片 D：B-06 缓存 + Controller CacheManager —— **完成并验证**
- [x] 切片 E：B-07 上传 / 字体 —— **后端完成并验证**（上传/字体页面属 F-06）

本轮遗留的已知行为（**不要当作已修复**）
- ⚠️ 快速切歌时中间可能闪过上一首：**本轮未复现**——因为 `song_change` 已改为
  "联网阶段不持锁 + 提交前校验 generation"，`_probe/async-cancel-test.mjs` 10 项断言
  在真正并发搜索下全部通过。但这只是自动化结论，**尚未做真人复测**。

仍未解决 / 已知问题（不变）
- ⚠️ 选曲界面预览时首行跑马灯反复触发（P2，前端）—— 调查暂停，不视为已修复
- ⏭️ QQ 无歌词响应的解码 WARN —— 本轮仍未处理
- ⏭️ LRC 上传真机验证 —— 仍未做
- ⏭️ `artist` 繁简 / 罗马字匹配影响面 —— 仍未验证

实机验证结论（2026-09-13 04:33，真实 osu! + tosu + osu-lyric + 浏览器/OBS）
- ✅ 正常播放时歌词显示与跟随   ✅ 暂停 / 恢复   ✅ 拖动 / 跳转播放进度
- ✅ 切歌：旧歌词清除、新歌词显示
- ✅ 返回 osu! 选曲界面后歌词清除
      （同时确认原协议未知项 ①：真机是**变体 A**，后端 `bid == 0` 清空判定成立，**无需修改**）
- ✅ 新开歌词页 / WS 重连恢复当前状态   ✅ 刷新歌词页恢复当前播放位置
- ✅ 基础 HTTP API
- ⏭️ LRC 上传：**暂时跳过**（用户决定，未测）

未解决项（**不要当作已完成**）
- ⚠️ **选曲界面预览时首行歌词横向滚动、且反复触发**（P2，前端）
  - 04:40 做过一次修复（把固定 `scrollWidth * 2` 换成按目标字号归一化），
    自动化回归测试 `_probe/scroll-test.mjs` 4 项断言通过
  - **但真人复测后现象依旧 → 该修复不构成完整根因，不标记为已修复**
  - 后续用真实组件 + headless Chrome 追踪（`_probe/scroll-trace.mjs`）确认：
    跑马灯是 `animation: scroll var(--time) linear infinite`，`--time = nextTime/1000`；
    在"首行之前"状态下 `nextTime` 很小（实测 0.76s）→ 动画**反复循环**。
    宽度判定与动画重启时序可能也参与，但**尚未定论**
  - **调查已暂停**。建议后续与「横向滚动与播放时间轴同步」一起重新设计，而不是继续打补丁
- ⏭️ Nhato - Virus Funk 的 QQ 歌词响应解码 WARN（P3，后端 QQ 源）：**本轮明确跳过**，
  不改 `qq.rs` / `QQLyricResponse` / 搜索流程与相关日志，保持现状（该曲是纯音乐，行为正确）

构建事故（已确认原因并恢复，2026-09-13 04:47）
- `npx vite build` 会**清空整个 `dist/`**（`vite.config.ts` 无 build 段 → `outDir=dist`、
  `emptyOutDir` 默认 true），把 `dist/osu-lyric.exe`、`config.json5`、`lyric.db` 一并删掉
- 已用 `cargo build -r` 重新产出并拷回；从 dist 启动验证 `/lyrics`、深层路由、assets、字体、`/api/status`、WS 全部正常
- 避免办法见 §8 事故条目；**未修改任何业务代码**

范围声明
- 本轮**没有修改后端业务逻辑**；前端只动过 2 个文件（新增 `src/utils/lyricScroll.ts`、
  `src/pages/LyricsBox/index.tsx` 改 2 处）
- 后端新增仅限 P3 第一片的 3 个只读/展示控制 HTTP 接口，以及 `model/tosu_types.rs` 的
  `#[cfg(test)]` 测试模块（无行为改动）

已完成：
- 第一阶段调查（Plan.md vs 实际代码差异、基线检查）
- P0/P1 后端：R1 / R2 / R3 / R4 / R5 全部修复
  - R3：换源 / 上传的 `_ = time_next(0)` 未 await -> 改为按实际进度 await 下发
  - R5：next_time 语义改为「剩余毫秒」；窗口判定改左闭右开
  - R1：首行之前不锁定下标，进入首行一定会补发正确 nextTime 的帧
  - R2：切歌时在 100ms 防抖之前立即清空旧状态并广播清屏
  - R4：连接快照补样式设置；无歌词时明确下发 setClear
- P1 前端：WS 地址自适应 host；歌词 store 批处理
- P3 第一片：`GET /api/status`、`GET /api/lyrics/current`、`POST /api/display/clear`
- 对照真实 tosu v4.26.2 校准假 tosu（150ms 轮询广播、真实音频时长、完整报文结构）
- 验证：Rust 单测 12 条（8 lyric_service + 4 真实报文解析）+ **44 项端到端断言**全部通过；
  debug / release 均构建通过；clippy 无新警告；tsc 通过

正在做：
- 无（跑马灯问题按用户要求暂停；其余无进行中任务）

阻塞：
- 无

下一步（继续纵向切片，按 Plan.md 的执行顺序第 2 项）
1. **切片 B：B-04 黑名单 + Controller BlackList**（修 R6/R7，`GET/POST/PATCH/DELETE /api/blocks`）
2. 切片 C：B-05 歌词内容管理 + Controller Content
3. 切片 D：B-06 缓存 + Controller CacheManager
4. 切片 E：B-07 上传 / 字体 + Controller
- 之后再进入第 3 项（移除 WS 管理入口 B-11 / F-10）与第 4 项（控制台其余页面）
- 跑马灯问题已转入 §6 问题 1 作为已知 P2 待办，**不作为当前主线**
- 真机仍未覆盖：① LRC 上传（用户暂跳过）② `artist` 繁简 / 罗马字匹配的影响面
```

> **重要**：`D:\.1yrics\tosu-lyrics-dev` **不是 git 仓库**（无 `.git`），无法 commit / diff / 回滚。
> 本次所有改动只能通过本文档追溯。建议尽早 `git init` 或至少备份一次目录。

---

## 0. 环境与基线事实

| 项目 | 实际情况 |
| --- | --- |
| git | **项目目录没有 `.git`**（`git status` 报 `not a git repository`）。无版本控制，无法查看提交历史，也无从「恢复已有工作」——所有改动只能靠本文档记录 |
| Rust | cargo 1.98.0 |
| Node | v22.12.0 / pnpm 9.15.2 |
| 后端入口 | `tosu-proxy/src/bin/new.rs`，二进制 `osu-lyric`，默认 feature `new` |
| 后端基线 | `cargo check --bin osu-lyric --features=new` → **exit 0**（仅一条 proc-macro-error2 future-incompat 警告） |
| 前端基线 | `npx tsc --noEmit` → exit 0；`npx vite build` → exit 0 |
| node_modules | 本次开始前**不存在**，已执行 `pnpm install`（exit 0） |
| dist/ | 不存在 |

---

## 1. Plan.md 与当前代码的差异（重要）

Plan.md 的评估日期是 2026-09-10，**部分结论已经被代码超越**。以实际代码为准：

| Plan.md 的说法 | 实际代码 | 结论 |
| --- | --- | --- |
| §2.1「`stores/indexStore.ts` 顶层创建 `new Websocket(false)`，实际 setter 仍调用旧 wsService」 | `indexStore.ts` **已无** `Websocket` 导入；`webSocketService.ts` 改为「Proxy 延迟创建」（`wsServiceInstance` 惰性化） | **已修复**，Plan 过时 |
| §3「展示仍走旧数据链：`LyricsBox` 创建 `TosuManager`」 | `LyricsBox/index.tsx` **已不导入** `tosuManager`；改用 `@/stores/lyricStore` 的 `lyrics/cursor/nextTime` | **已修复**，Plan 过时 |
| §3「首次进入 / 重连同步完整歌词和设置：未实现：连接时只登记会话」 | `server/websocket.rs:210-215` 已在 `connect` 时调用 `get_snapshot()` 并单发；`lyric_service.rs:289` 有 `get_snapshot()` | **部分实现**（歌词有，**样式设置无**） |
| §4「WS 连接与广播：新封装 URL 不匹配」 | `constants.ts` 的 `BACKEND_WEBSOCKET_URL` 已是 `http://127.0.0.1:41280/ws`，为真实路由 | **已修复**，Plan 过时 |
| R1「首包丢失」 | `lyric_service.rs` 已有 `push_now()`（歌词加载完立即下发完整帧），`clear_state()` 把 `now_index` 置为 `usize::MAX` | **部分修复**，见 §2 |
| R3「两处 `_ = self.time_next(0)` 均未 await」 | **仍然存在**：`lyric_service.rs:520`、`lyric_service.rs:541` | **未修复**，本次必修 |
| R5「窗口使用 <=end」 | **仍然存在**：`lyric_service.rs:316` `t <= self.current_lyric_end_time` | **未修复**，本次必修 |
| R4「晚加入无快照」 | 歌词快照已补；**每连接样式快照仍缺** | **部分修复** |
| R6~R12 | 逐条核对后确认**基本仍然存在**（黑名单 (bid,sid,title) 解包顺序、缓存 `delete_by_id` 未 exec、字体只读打开、设置两套 key 等） | **未修复**，本次不在范围内 |

### 关于 R7 / R8 的额外确认

- R8 的「两套 key」确认属实：
  - `setting/mod.rs` + `model/setting.rs` 的宏 `LyricSetting` 使用 DB key `trans_main` / `align` / `show_second`；
  - `service/websocket_service.rs` 的 `LyricSettingDatabaseKey` 使用 DB key `translation-main` / `alignment` / `second-show`。
  - `GLOBAL_SETTINGS` 除了自身模块外**没有任何调用点**（grep 确认），当前是死代码，不影响今晚链路。

### 关于部署路径的差异

`server/file.rs` 的静态根是 `["./static/lyrics", "./static", "./"]`，而 `justfile` 的 `build` 把前端产物放在 `./dist/`、后端二进制放在 `./dist/tosu-proxy`。
→ 直接从项目根运行 `./dist/tosu-proxy` 时，`lyrics/**` 会 fallback 到项目根的 `index.html`（开发版，引用 `/src/main.tsx`），**页面无法工作**。今晚通过 vite dev server 验证不阻塞；已记录，必要时补 `./dist` 静态根。

---

## 2. 本次开工时 P0/P1 的真实缺口（逐条核对代码后）

### R1 首包/首帧
**已基本修复**：`song_change` 走 `push_now()`（缓存命中）或 `try_source_lyric()` → `push_now()`（网络命中），都会下发含完整 `lyric` 数组的帧。
**残留问题**：`push_now` 在「首行之前」分支把窗口设成 `[0, first_start]`，而 `time_next` 的窗口判断用 `t <= end`；当播放时间正好到达 `first_start` 时会命中「时间未越过窗口」提前 return，且此时 `now_index(0) == index(0)`，第一行**永远不会**补发正确的 `nextTime` 帧 → 第一行的滚动动画时长错误。

### R2 切歌残留
`LyricService::song_change` 内部已 `clear_state()` + `broadcast_clear()` + `clear_cache()`。
**残留问题**：`song_source_service::on_song_update` 里 `song_change` 是在 `tokio::spawn` + `sleep(100ms)` **之后**才执行的。这 100ms 内，`on_time_update` 会用**新歌的时间**去驱动**旧歌的歌词**（`now_lyric` 尚未被清空），可能推出一帧错误下标。

### R3 换源/上传刷新
**未修复**：`lyric_service.rs:520` / `:541` 的 `_ = self.time_next(0);` 未 `.await`，Future 被直接丢弃，**即时刷新完全不生效**。
另外即使 await，`time_next(0)` 也是按时间 0 定位，而不是实际播放位置。

### R4 快照
歌词快照已有（`get_snapshot()` + `connect` 单发）。
**缺口**：快照不含样式设置；`now_lyric` 为 `None`（无歌词/清屏）时**什么都不发**，重连的展示页可能保留旧内容。

### R5 时间与清屏
- `next_time`：`time_next` 里算的是 `next_line_start - current_line_start`（整行时长），行中跳转时不是剩余时长 → 滚动动画时长错误。Plan §6.1 要求「统一剩余毫秒」。
- 窗口判断 `t <= end` 应为 `t < end`（见 R1）。
- `current = -1` / 暂停 / 空歌：后端目前不发送这类状态；清屏走 `setClear` 广播，前端 `handleSettingBroadcast` 已处理 → 清屏链路可用。

### R12 附带确认
`lyric/lyric_source.rs:233-239` `find_line` 的 `else` 分支把切片局部下标 `i-1` 当全局下标返回。当前 `Lyric::parse` 结尾会把 `cursor` 置 0（`lyric_source.rs:139`），因此该分支实际等价于正确结果，**暂时不触发**；但属于隐患，本次不动（避免扩大范围）。

---

## 3. 今晚范围界定

**做（P0/P1）**
- 后端：R3 未 await 的即时刷新；R5 `next_time` 剩余时长 + 窗口边界；R2 切歌立即清旧状态；R4 快照补样式 + 无歌词时补清屏。
- 前端：确认 WS 常量指向真实路由；`lyricStore` 对新版帧的处理；必要时最小修补。

**不做（明确延后）**
- Controller 重写 / 23 个 WS 管理命令迁移 / 管理 HTTP API
- 黑名单（R6/R7）、缓存（R9）、字体（R10）、两套 setting key 统一（R8）
- `find_line` 下标隐患（R12）
- UI 重构、旧链路（`tosuManager`/`adapters`/IndexedDB 缓存）清理

---

## 4. 本次未做（明确延后，不是遗漏）

| 项 | 原因 |
| --- | --- |
| HTTP 管理 API（B-03~B-07） | **部分已做**：只落地 `GET /api/status`、`GET /api/lyrics/current`、`POST /api/display/clear`（见 §8 03:50 条）。其余（设置、搜索 / 预览 / 换源、偏移、屏蔽、缓存、字体、客户端）仍待做 |
| Controller 接新版协议 | 同上。当前控制台仍走旧 `wsService` / `/api/config`（新后端没有该路由，会 404） |
| 黑名单 R6/R7、缓存 R9、字体 R10 | 用户明确要求本次不动 |
| 设置两套 key 统一（R8） | 同上。`LyricSetting`(trans_main/align/show_second) 与实际使用的 (translation-main/alignment/second-show) 仍然并存；`GLOBAL_SETTINGS` 目前是死代码 |
| `find_line` 切片局部下标隐患（R12） | 因为 `Lyric::parse` 结尾把 cursor 置 0，该分支当前等价于正确结果，暂不触发；改动风险大于收益 |
| 每展示端独立样式 / 客户端身份 | Plan 中即为扩展待定 |
| 清空判定改成基于 `state.number` | 待真机确认「回菜单时 tosu 是否把 beatmap 归零」后再决定，见 §6 04:02 条 |
| `artist` 繁简 / 罗马字匹配（B2） | 本次只记录复现，未改匹配算法（属 B2 范围） |

## 5. 真机测试结果（2026-09-13 04:33 完成）

真人实测（osu! + 浏览器/OBS + 后端 release）**核心功能全部正常**：

| 场景 | 结果 |
| --- | --- |
| 播放时歌词显示与跟随 | ✅ |
| 暂停 / 恢复 | ✅ |
| 拖动 / 跳转进度 | ✅ |
| 切歌清除旧歌词并显示新歌词 | ✅ |
| **回到选曲界面清除歌词** | ✅ **（同时确认了协议未知项 ①：真机是变体 A，`bid == 0` 判定成立，后端无需修改）** |
| 新开页面 / WS 重连恢复状态 | ✅ |
| 刷新页面恢复播放位置 | ✅ |
| 基础 HTTP API | ✅ |
| LRC 上传 | 未测（用户明确暂不处理） |

新发现两个问题（**仅定位，未改代码**）：见 §7 问题 1 / 问题 2。

## 6. 两个新问题的定位结论（2026-09-13 04:33，仅分析未修改）

### 问题 1：选曲界面预览时首行歌词出现横向滚动

- **涉及代码**：`src/pages/LyricsBox/index.tsx`
  `getMaxWidth()` L108-116、触发判定 L126、`setScroll` L150/152、
  滚动副作用 L157-170、`MainLyric` 的 `transition-[font-size]` L200 与 L204、`Index` L304
- **根因**：`getMaxWidth()` 在 2 个子元素时返回 `main.scrollWidth * 2`（L111）。
  这个 ×2 是**为"测量发生在 font-size 过渡刚开始、读到的是 1.5em 的宽度"而做的补偿**。
  但当该行**创建时就已是 active**（整份歌词列表一次性替换，`<li>` 全新创建且带 `font-size:3em`），
  不会有过渡，`scrollWidth` 已经是 3em 的宽度，再 ×2 就是**真实宽度的 2 倍**，
  于是 `maxWidth > lyricUL.clientWidth` 成立 → `setScroll(true)` → 跑马灯。
- **为什么预览会、正式播放不会**：tosu 在选曲预览与正式游玩时 `beatmap.id/set/title` **完全相同**
  （`osuInstance.ts` 进入 `selectPlay` 只重置 gameplay/resultScreen，**不动 menu**），
  所以后端 `update_song_info` 为 false → **进入谱面时不会重新下发完整歌词、不会广播清屏**。
  - 选曲预览：换歌触发 `setClear` → `lyrics=[]` → 歌词到达时**整份列表新建** → 当前行创建即 active → 走 ×2 分支 → 滚动
  - 正式游玩：歌词列表不变，只有 `cursor` 变化 → 该行由 inactive 过渡到 active → 测量到 ~1.5em → ×2 恰好还原 3em 宽度 → 不滚动
- **实测证据**（headless Chrome 复现真实首行 `通り過ぎる車窓に映った一瞬の君と` + 翻译，`_probe/scroll-test.html`）：

  | 窗口宽 | A. 创建即 active | B. inactive→active（过渡起点） |
  | --- | --- | --- |
  | 1920 | main=768 → max=1536，no | main=384 → max=768，no |
  | 1280 | max=1536 → **SCROLL** | max=768 → no |
  | 1024 | max=1536 → **SCROLL** | max=768 → no |
  | 800 | max=1536 → **SCROLL** | max=768 → no |
  | 640 | max=1536 → **SCROLL** | max=768 → **SCROLL** |

  同一行同一字号，两条路径测出的 `maxWidth` 相差整整 2 倍 —— 与根因一致。
- **归属**：**前端歌词组件状态缺陷**（不是 tosu 问题，不是后端时间/同步逻辑问题，不是 WS 时序问题）
- **第一次修复**：2026-09-13 04:40 实施（见 §8），但——⚠️ **真人复测后现象依旧，不构成完整根因**。
  该修复本身没有被证明是错的（自动化测试显示两条路径的宽度判定确实被拉平了），
  但它**没有解决用户实际看到的现象**，因此**不得标记为已修复**。
- **后续追踪（2026-09-13 05:00，已暂停）**：改用真实组件 + headless Chrome 追踪
  （`_probe/scroll-trace.mjs`）后确认了另一条独立机制：
  - 跑马灯是 `animation: scroll var(--time) linear infinite`（`src/index.css`），**本身就会无限循环**
  - `--time = nextTime / 1000`，而 `nextTime` 在"首行之前"状态下 = `首行起点 - t`，**很小**：
    实测预览首帧 `nextTime = 760ms` → `--time = 0.76s` → **约每 0.76 秒循环一次 = "反复滚动"**
  - 实测数据（窗口 800px，真实歌词首行 `通り過ぎる車窓に映った一瞬の君と`）：
    `ulClientWidth=759`、`mainScrollWidth=816`（已处于 3em 全尺寸）、`measured=816` →
    `816 > 759` 成立 → `animate-scroll` → `anim-START --time=0.76s` → `anim-ITER`
  - **宽度判定与"反复触发"是两件事**：前者决定"滚不滚"，后者决定"滚几次/多快"
  - 尚未定论的部分：为什么正式播放时同一句不触发（已排除"宽度测量不一致"是唯一原因）
- **当前处置**：⏸️ **调查已暂停**（用户要求）。记录为**已知 P2 / UI 问题**。
  建议后续与**「横向滚动与播放时间轴同步」**一起重新设计，而不是继续打补丁。
- **优先级**：P2（纯观感问题，不影响时间/同步正确性）

### 问题 2：QQ 歌词响应解码失败（Nhato - Virus Funk）

- **谱面**：`beatmapsets/2114086` → **Nhato - Virus Funk**，source `beatmania IIDX 27 HEROIC VERSE`，
  tags 含 `instrumental`；网易云对同名曲返回 `pureMusic: true` → **这首是纯音乐，本来就没有歌词**
- **复现**：对 `songmid=0024Wy6H0zZjnJ` 请求
  `c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg?...&nobase64=1`
  → HTTP 200，`Content-Type: text/html;charset=utf-8`，body 46 字节：
  `{"retcode":-1901,"code":-1901,"subcode":-1901}` —— **完全没有 `lyric` / `trans` 字段**
- **根因**：`lyric/qq.rs` 的 `QQLyricResponse` 把 `lyric` / `trans` 声明为**必填 `String`**，
  QQ 的"无歌词/不可用"响应里这两个 key 直接缺失 → serde 报缺字段 →
  `response.json()` 失败 → 打出 `error decoding response body`（qq.rs:135）。
  加 Cookie / 去 nobase64 / 换旧接口都无效（旧接口还会包一层 `MusicJsonCallback(...)`），
  是**该曲在 QQ 侧就没有可下发的歌词**，不是请求参数问题，也不是 QQ 改格式。
- **数据流**：网易云先试 → 命中 "Virus Funk / Nhato" 但 `pureMusic=true` → 空歌词，继续候选 → 无同曲候选 → `Ok(None)`
  → QQ 兜底 → 命中同一首 → `fetch_lyrics` 返回解码错误 → 打 warn → 空 `LyricResult` → 继续候选 → `Ok(None)`
  → `now_lyric = None` → **不显示歌词（对纯音乐而言这是正确结果）**
- **处置**：⏭️ **本轮明确跳过**（用户决定）。不改 QQ 歌词源、`QQLyricResponse`、搜索流程及相关日志，保持现状。
- **归属**：**属于当前项目代码的健壮性/可诊断性缺陷，不是功能缺陷**。
  最终行为正确；问题是 (a) 正常情况打 WARN 噪音，(b) 报错不含响应体，无法区分"没有歌词"与"QQ 真的改格式了"
- **最小修复方案（待定，未实施）**：`lyric`/`trans` 加 `#[serde(default)]`；解出后检查 `code != 0` 时按"无歌词"处理并降为 `debug!`；
  解码失败时把响应体前若干字节带进日志。**不动搜索/匹配主流程**。
- **优先级**：P3（不影响功能，只影响日志噪音与可诊断性）

## 7. 当时的验证脚本与步骤（已执行完毕，保留备查）

自动化已经覆盖了协议层，**只有浏览器渲染 + 真实 tosu 数据**必须人工确认。
请一次连续做完下面这一轮（约 5 分钟）：

**准备**
```
# 1. 构建
cd D:\.1yrics\tosu-lyrics-dev
pnpm install && pnpm build
cargo build -r --bin osu-lyric --features=new --manifest-path tosu-proxy/Cargo.toml
cp tosu-proxy/target/release/osu-lyric.exe dist/
# 2. 启动(必须在 dist 目录里跑)
cd dist && ./osu-lyric.exe          # 首次会生成 config.json5 并退出, 再跑一次即可
```
**注意**：二进制必须在 `dist/` 目录内运行（静态根是相对 CWD 的）。若不想构建后端，
也可以 `pnpm dev` 后用 `http://localhost:5173/lyrics` 测试（后端仍需单独运行）。

**验证清单**

| # | 操作 | 期望 |
| --- | --- | --- |
| 1 | 浏览器打开 `http://127.0.0.1:41280/lyrics` | 进入游戏播放歌曲后，**不用刷新**就出现歌词并跟随滚动 |
| 2 | 同一个 URL 再开一个窗口（模拟第二个展示端 / OBS 刷新） | 立刻显示与第一个窗口**相同的歌词行**，不是空白 |
| 3 | 在 osu! 里按 `Shift+←` 或拖动进度条跳到歌曲中段 | 歌词立刻跳到对应行，不卡在旧行 |
| 4 | 暂停播放 | 歌词停在当前行不动 |
| 5 | 恢复播放 | 从当前行继续跟随 |
| 6 | 切到另一首歌 | 立刻清空，然后显示新歌歌词，**不残留上一首** |
| 7 | 回到选歌菜单 | 歌词清空。**这一条同时是协议未知项**：若真机下没清空，说明 tosu 在主菜单仍上报上一张图，需要改后端的清空判定（见 §6 04:02 条与 `_probe/README.md`） |
| 8 | 在浏览器里按 `Ctrl+Shift+R` 刷新展示页 | 刷新后立刻恢复到当前歌词行（不是空白，不用等切歌） |
| 9 | 上传歌词：`curl -F "file=@某.lrc" http://127.0.0.1:41280/api/lyric/upload` | 按**当前播放位置**立即显示新歌词，不是跳回第一行 |

若第 1/2/8 项是空白：先看浏览器控制台是否有 WS 报错，再确认后端端口与页面地址一致。

**可选：一边测一边用 HTTP 观察后端状态**（不用开控制台）
```
curl http://127.0.0.1:41280/api/status
# {"song":{"bid":...,"title":"..."},"lyric":{"loaded":true,"current":12,"nextTime":3410},"offset":0}

curl http://127.0.0.1:41280/api/lyrics/current   # 完整歌词, time 单位毫秒
curl -X POST http://127.0.0.1:41280/api/display/clear   # 清屏(持续到切歌/换源/上传)
```
若 `"song":null` 但游戏里明明在放歌 → 说明后端没连上 tosu（看后端日志里有没有 `已成功连接到 Tosu`）。

## 8. 变更记录

### [2026-09-13 06:20] 专项验证：异步搜索 / 任务取消（on_clean 的 abort 是否有副作用）

- **目的**：确认上一轮为修「回菜单后旧歌词复活」而加的 `on_clean → abort` **没有动摇原有异步搜索策略**。
- **方法**：新增 `_probe/async-cancel-test.mjs`（假 tosu + **冷缓存** + 三首带歌词指纹的歌：
  Lemon=46行/`室屋光一郎`、アイネクライネ=43行/`蔦谷好位置`、夜明けのベル=38行/`通り過ぎる車窓`）。
  冷缓存是关键：热缓存会让搜索瞬间完成，根本测不到"搜索进行中切歌"。
- **代码层面的核对（原策略未变）**：
  - `on_song_update` 的 `take → abort 上一个 → 清空 → spawn 新任务 → replace handle` **一行未动**
  - 内层的 `cancel_tx / cancel_rx` 广播取消（用于并行 QQ/网娱搜索）**未动**
  - 本轮唯一的改动只是 `on_clean` 里**多了同样的一次 `take + abort`**
- **测试结果：10 项断言全部通过**

  | 场景 | 结果 |
  | --- | --- |
  | S1 A→B（A 搜索中切走） | 最终显示 B；最后一次清屏后只出现 B 的帧 → **A 的结果没有留下来** |
  | S2 A→B→C（快速连续） | 最终显示 C；最后一次清屏后只出现 C → **C 的有效任务没有被连续取消误伤** |
  | S3 A→B→A | A 重新加载成功（46 行）→ **被取消过的歌之后仍能正常加载** |
  | S4 搜索中回菜单 | 收到清屏；清屏后 **0 个 lyric 帧** → **旧歌词没有复活** |

  - 时间线佐证：A 在整个测试里被成功加载 **4 次**、B **4 次**、C **1 次**，
    说明反复取消不会"烧掉"某首歌的加载能力。
- **发现的既有行为（不是本轮引入，本轮不改）**：
  后端是**串行**处理切歌的——`song_change` 全程持有 `LYRIC_SERVICE` 锁做网络搜索，
  tosu 读取循环会卡在 `on_time_update` 上，**新的切歌消息要等当前搜索结束才被处理**。
  后果：快速连续切歌时，前一手的在途结果可能先落地、随后才被新歌的清屏覆盖，
  **最终状态始终正确，但中间会闪过上一首的歌词**。
  这属于"歌曲身份 / generation 校验"的范畴（Plan 里 B-01 / B-05 的并发要求），
  真正的修法是让异步结果带上歌曲身份、落地前校验，而不是只靠 abort。
- **回归**：`_probe/e2e.mjs` → 44 项断言全部通过（未受影响）。
- **结论**：取消机制工作正常，**没有误伤新任务**，也未改变原异步策略。

### [2026-09-13 06:05] F-01 最小前置修复：去掉挡住全局管理页的旧 clientSignal 遮罩

- **背景**：B-03 人工验收时打开 `/lyrics/controller/textstyle` 被「当前客户端不可用, 请选择客户端」挡住，
  而 `/lyrics/controller/client` 里没有任何可选客户端。
- **根因（代码依据）**：
  - `Controller/index.tsx` 的 `shouldShowMask()` = `!isClientPage && !wsService.clientSignal()`
  - `wsService.clientSignal()` 依赖 `onlineClients`，后者只由旧协议的 `command.type === "online"` 消息填充
    （`services/webSocketService.ts:103-112`）
  - **新后端从不产生这类消息**（`model/websocket/` 里没有 online/command 字段）
  - → `clientSignal()` 恒为 false → 除 client 页外一律盖遮罩；`Mask` 是 `absolute inset-0 z-100`，**真实用户点不动任何控件**
- **性质**：这是**旧 Controller 的残留**。新版后端是全局单一状态（一首当前歌 + 一份全局设置），
  不存在"其他浏览器客户端"概念，该前置条件对全局管理页没有意义。属 Plan 中 F-01「控制台去掉…选端遮罩」的范围。
- **改动（只动 1 个文件）**：`src/pages/Controller/index.tsx`
  - 删除 `shouldShowMask()` / `jumpToClient()` / `Tips` 与 `<Show>` 包裹的 `<Mask>` 渲染，并清理随之无用的
    `Button` / `Mask` / `Show` / `useNavigate` / `wsService` 导入
  - 留下的注释说明为什么移除，以及 `/lyrics/controller/client` 保持原样留给 B-08 / F-07
- **明确未做**：`/lyrics/controller/client` 页面**一行未改**，仍是我们不需要的新客户端管理；
  WS 管理接口、setter 模式、旧 `wsService` / `configService` 全部保留（B-11 / F-10 不动）。
- **测试**
  - `npx tsc --noEmit` exit 0；`npx vite build` exit 0
  - `_probe/controller-e2e.mjs` → **13 项断言全部通过**，其中新增：
    - 「TextStyle 页不再出现『当前客户端不可用』遮罩」
    - 「TextStyle 页没有覆盖层挡住操作」
    - 对齐方式改为用 **CDP 真实鼠标事件**（`Input.dispatchMouseEvent`）点击，不再用 `element.click()`
  - B-03 链路回归确认：真实鼠标点击「左对齐」→ 后端 `alignment=left` → 展示页 `<ul>` 变 `flex-start`；
    颜色 / 字号 / 副歌词显隐同样全部生效 ✅
  - 旁路确认：`/lyrics/controller/client` 仍能正常渲染（root 内 82 个节点，标题与说明文案都在）
  - `dist` 又被 `vite build` 清空，已重新拷回 release exe + config，`/lyrics/`、`/lyrics/lyric`、`/api/status`、`/api/settings` 均 200
- **顺带发现的既有噪音（未处理）**：控制台页面在浏览器控制台会刷
  `WebSocket message error: TypeError: Cannot read properties of undefined (reading 'type')`——
  旧 `webSocketService` 连的是新后端 `/ws`，收到 `{type:"lyric"}` 后按旧协议读 `command.type` 报错。
  属于新协议与旧封装的既有不兼容（F-10 范围），只影响控制台日志，**不影响页面渲染与本轮功能**。
- **测试方法上的教训（已修正）**：上一轮的浏览器自动化用 `element.click()` 程序化点击，
  **会绕过遮罩**，所以没发现这个问题。现已改为 CDP 真实鼠标事件，这类"能不能点得动"的问题以后能被自动发现。

### [2026-09-13 05:45] 切片 A 完成：B-03 设置 + Controller TextStyle（纵向闭环）

本轮改为**纵向切片**推进：一个后端能力 → HTTP API → 对应 Controller 页面 → 后端执行 → WS 广播 → LyricsBox 生效。
切片 A 已完成并**用真实浏览器联调验证**。

- **后端（修 R8：两套 setting key 统一）**
  - 新增 `model/setting.rs`：`LyricSettings`（`textColor` / `fontSize` / `font` 各为主副 `Pair`，加 `alignment` /
    `translationMain` / `secondShow`）+ `LyricSettingsPatch`（局部更新）+ `SettingKey`（WS 广播 key 映射）。
  - **整份设置以一条数据库记录保存**（`SettingEntity` key = `settings`），取代原先
    `trans_main`/`align`/`show_second` 与 `translation-main`/`alignment`/`second-show` 两套 key。
  - `setting/mod.rs` 改为加载该结构；数据库里的值非法时回退默认值而不是 panic。
  - 新增 `service/setting_service.rs`：`current_settings` / `patch_settings` / `broadcast_settings` /
    `send_settings_snapshot`，统一走 **校验 → 落库 → 更新内存 → 广播**；
    **写库失败直接返回错误，不更新内存、不广播**（原 R8 的"保存失败仍广播"）。
  - 旧 WS 管理命令（`setColor`/`getColor`/`setFontSize`/`setAlignment`/`setTranslationMain`/`setSecondShow` 等）
    改为**委托同一份状态**，只广播变化项；兼容"标量"与 `{first, second}` 两种旧写法。
  - 新增 `server/response.rs`：统一错误结构与错误码常量（`invalid_param` / `no_song` / `no_lyric` /
    `not_found` / `write_failed` / `song_changed` / `source_failed` / `internal`）。
- **HTTP API（新增）**：`GET /api/settings`、`PATCH /api/settings`
  （局部更新、类型与范围校验：颜色 `#rgb/#rrggbb/#rrggbbaa`、字号 0.5~12、对齐 left/center/right；
  返回**最终服务端状态**）
- **前端**
  - `services/settingsService.ts`（新）：走相对路径 `/api/settings`（开发走 vite 代理、生产同源），
    统一 `ApiError`；`config/constants.ts` 新增 `BACKEND_API_BASE`。
  - `hooks/useSettings.ts`（新）：loading / saving / error 状态机；成功用服务端返回值覆盖本地状态，失败保留原值。
  - `stores/settingsStore.ts`：新增 `fontSize` / `secondFont` 信号与 `applySettings()`（唯一的写入口）。
  - `TextStyle` 页 + `TextColor` / `Font` / 新增 `FontSize`：全部改为 HTTP，
    `CustomColorSelector` 改为受控组件（拖动只预览，松手才提交）。
  - `LyricsBox`：字号来自设置（active = `fontSize`，非 active 保持 2:1），主副字体分别应用到两个 `<p>`；
    `utils/lyricScroll.ts` 的 `measureLineWidth` 增加主副目标字号参数（默认 3/2，行为不变）。
  - `indexStore` 的样式 setter 去掉旧 `wsService` / `configService` 写入（黑名单等未迁移功能保持原样）。
  - `initializeApp`：旧 `/api/config` 失败不再中断控制台初始化（否则夜间模式都设置不上）。
- **顺带修复的并发缺陷**：`on_clean`（回到菜单）现在会**取消在途的换歌任务**。
  否则"搜索还没回来就回菜单"时，那个任务稍后会把 `now_lyric` 写回去并推帧，让已清空的歌词复活。
- **测试**
  - Rust：`cargo test --lib` → model 12 passed + service 11 passed（新增 6 条设置相关用例）
  - `cargo clippy`：仅剩 2 条既有警告（`lyric/source/mod.rs`），无新增
  - `npx tsc --noEmit` exit 0；`npx vite build` exit 0
  - `_probe/settings-api-test.mjs`（新）→ **17 项断言全部通过**：连接快照 6 项、GET/PATCH、
    局部更新不覆盖其他字段、只广播变化项、4 类非法值被拒、校验失败不污染状态、空/非法请求体
  - **持久化**：写入自定义值 → 重启后端 → `GET /api/settings` 返回相同值 ✅
  - **WS 与 HTTP 一致性**：WS `setAlignment(left)` 后 HTTP 读到 left；HTTP 改 right 后 WS `getAlignment` 读到 right ✅
  - `_probe/e2e.mjs` → 44 项断言全部通过（改了一处断言：G2 改为只统计**清屏之后**的帧，
    原来把上一个场景在途的帧也算进去了）
  - `_probe/controller-e2e.mjs`（新，真实浏览器 + 真实后端 + 假 tosu）→ **11 项断言全部通过**：
    Controller 点击「左对齐」→ 后端 `alignment=left` → 展示页 `<ul>` 变 `flex-start`；
    改主歌词颜色 → 展示页当前行颜色变 `rgb(255,0,0)`；改字号 4 → 展示页当前行 `4em`；
    关掉副歌词 → 展示页 `<p>` 从 88 个降到 46 个；非法值被 400 拒绝
- **dist 恢复**：本轮 `vite build` 再次清空了 dist（已知问题），已重新 release 构建并拷回
  exe + config，验证 `/lyrics/`、`/lyrics/lyric`、assets、`/api/status`、`/api/settings` 均 200
- **未做（按用户要求）**：B-11 / F-10 未动；setter WS 模式、23 个 WS 管理 key、旧
  `wsService` / `configService` / `tosuManager` 全部保留（黑名单相关的旧写入路径保持原样）

### [2026-09-13 05:15] 文档同步（Plan.md + DEV_LOG.md）

- **完成**：按本轮实际情况更新 `DEV_LOG.md`（当前状态 / §6 问题 1 结论 / 本节）与 `Plan.md`
  （§2 结论、§3.1 接口表、§7 任务状态与验收状态、新增《本轮进度》小节）
- **未修改任何代码**，也未修改 `Plan.md` / `DEV_LOG.md` 以外的任何文件
- **记录原则**：只更新状态与新增记录，不删除历史；未完成项一律不勾选

### [2026-09-13 05:00] 横向滚动问题二次调查（结论：暂停，不视为已修复）

- **触发**：真人复测反馈 04:40 的修复**没有解决实际问题**，且现象更具体 ——
  "不是只滚动一次，而是会横向滚动好几次 / 反复触发"
- **本轮做了什么**：搭了「真实组件 + headless Chrome」追踪环境（**组件代码一行未改**，全部从外部观测）
  - `_probe/harness/entry.tsx`：渲染真实 `LyricsBox`，把 store 接口挂到 window
  - `_probe/build-harness.mjs`：用 vite JS API 打包（避免往项目里加文件）
  - `_probe/scroll-trace.mjs`：CDP 驱动 headless Chrome，监听
    `animationstart` / `animationiteration` / style / class 变更，并复刻后端 `frame_at` + `time_next` 窗口逻辑
  - 依赖 `_probe/harness/node_modules`（指向项目 node_modules 的 junction）
- **确认到的机制**（窗口 800px，真实歌词 + 真实编译后 CSS）：
  - `ulClientWidth=759`、`mainScrollWidth=816`（**已处于 3em 全尺寸**）、`measured=816` → 超宽成立
  - `animate-scroll` 生效，`animation: scroll var(--time) linear infinite`，`--time=0.76s`
  - `nextTime=760ms` 来自**"首行之前"**状态（首行在 960ms）→ `--time` 极小 → **动画约每 0.76 秒循环一次**
  - 即：**"滚不滚"由宽度判定决定，"反复滚"由 `--time`（= nextTime/1000）+ `infinite` 决定**，是两件事
- **仍未定论**：正式播放时同一句不触发滚动 —— 已排除"宽度测量不一致是唯一原因"
- **处置**：⏸️ 用户要求**暂停调查**。不标记为已修复，转为已知 P2 / UI 问题，
  建议后续与「横向滚动与播放时间轴同步」一并重新设计
- **保留的自动化测试**（**仍然有效，只是不足以覆盖真实现象**）：
  - `_probe/scroll-test.mjs`：4 项断言通过 —— 两条路径宽度一致 / 滚动判定一致 /
    修复前公式确实不一致（测试有检出能力）/ 窄窗口下仍会滚动
  - `_probe/e2e.mjs`：44 项断言通过（后端链路，与本次前端改动无关）
  - `npx tsc --noEmit` exit 0；`npx vite build` exit 0

### [2026-09-13 04:47] 事故：`vite build` 清空 dist 导致 dist\osu-lyric.exe 被删（已恢复）

- **结论**：**是 `npx vite build` 清空 `dist/` 导致的**，不是编译失败、不是被杀毒删除、也不是文件系统问题。
- **原因**：`vite.config.ts` **没有 `build` 段** → Vite 取默认 `outDir = "dist"`；
  而 `emptyOutDir` 在「outDir 位于项目根内」时**默认为 `true`**。
  于是每次 `vite build` 都会**先清空整个 `dist/`** 再写入自己的产物。
  `osu-lyric.exe` / `config.json5` / `lyric.db` 都不是 Vite 的产物 → 被一并删除。
- **证据**：
  - `dist/` 目录 mtime = `09-13 04:39`，正是问题 1 修复后那次 `npx vite build` 的时间
  - 清空后 dist 里剩下的**恰好都是 Vite 会重新生成的东西**：`index.html`、`assets/`（Vite 产物），
    以及 `LRC.otf` / `osu.svg`（来自 `public/`，时间戳仍是 `09-09 23:42`，证明是清空后从 `public/` 重新拷贝的）
  - `tosu-proxy/target/release/osu-lyric.exe` **一直存在**，所以没有任何东西真正丢失，
    只是 `dist/` 里的那份副本被删了（`lyric.db` 是歌词缓存，可重新生成）
- **恢复步骤（未改任何业务代码）**：
  1. `cargo build -r --bin osu-lyric --features=new` → exit 0（04:46）
  2. `cp tosu-proxy/target/release/osu-lyric.exe dist/`
  3. 重建 `dist/config.json5`（内容与之前一致，即应用默认配置 + `log: "info"`）
- **为什么没有直接跑 justfile**：
  - 本机**没有安装 `just`**（`which just` 找不到）
  - `build-backend` 会跑 `cargo fmt` + `cargo clippy --fix --allow-dirty`，**会改写源码**，
    与本次「不要修改业务代码」冲突，故只取其等价的核心两步
  - 另外 `copy-backend` 在 Windows 上是坏的：`cp ./tosu-proxy/target/release/osu-lyric ./dist/tosu-proxy`
    源文件少 `.exe`、目标也没有扩展名
- **恢复后验证（从 `dist/` 目录启动 `osu-lyric.exe`）**：

  | 请求 | 结果 |
  | --- | --- |
  | `/lyrics/` | 200 |
  | `/lyrics/lyric`（深层路由） | 200 |
  | `/lyrics/assets/index-*.js` | 200（75481 字节） |
  | `/lyrics/LRC.otf` | 200 |
  | `/api/status` | 200，返回 JSON |
  | `ws://127.0.0.1:41280/ws` | 接入即收到 `setClear` 快照 |

- **顺带观察（不影响本次结论）**：验证时用户的 tosu 正在运行，`/api/status` 返回了
  `song: {bid:32006, sid:5889, length:267360, title:"", artist:""}` ——
  **bid 非 0 但 title/artist 为空**。此时 `lyric.cleared=true`，不显示歌词，无害。
  这属于 tosu 菜单态的一种中间状态，与已确认的「回菜单清屏」不冲突，仅记录备查。
- **避免再次发生的建议**：
  1. **构建顺序纪律（零改动）**：`dist/` 同时放前端产物和后端二进制时，**拷贝 exe 必须是最后一步**。
     justfile 的 `just build`（build-dist → build-backend → copy-backend）顺序本身是对的，
     出事的是**单独重跑 `pnpm build`**。改完前端要么跑完整套，要么跑完立刻把 exe 拷回去。
  2. **给 Vite 关掉 `emptyOutDir`（1 行配置，最省事）**：`vite.config.ts` 加 `build: { emptyOutDir: false }`，
     `vite build` 就再也不会碰 dist 里的非 Vite 文件。
     代价：前端旧版本带 hash 的 `assets/` 会累积不清理（只占空间，不影响运行）。
     **这是构建配置改动，本次未擅自修改，留给用户决定。**
  3. **物理隔离（最干净，但要动后端）**：把 exe 与 config 移出 `dist/`（如放 `run/`），
     Vite 独占 `dist/`。但 `server/file.rs` 的静态根是相对 CWD 的，需要改后端才能配合 → 本轮不做。
  4. **兜底认知**：`tosu-proxy/target/release/osu-lyric.exe` 是唯一真源，
     `dist/` 里那份随时可拷回，本次就是这样恢复的。

### [2026-09-13 04:40] 修复问题 1：选曲预览误触发横向滚动（P2）

- **改动范围**：**只动前端 2 个文件**，后端 / QQ 歌词源 / 搜索流程 / 时间同步逻辑均未触碰
  - 新增 `src/utils/lyricScroll.ts`（`measureLineWidth`）
  - `src/pages/LyricsBox/index.tsx`：`updateScroll` 里原来的 `getMaxWidth()` 闭包整个替换成
    `const maxWidth = measureLineWidth(p);`（含新增一行 import）
- **修复方式**：把固定 `scrollWidth * 2` 换成**按「目标(active)字号 / 当前实际字号」比例补偿**：

  | 场景 | 当前实际字号 | 补偿系数 | 结果 |
  | --- | --- | --- | --- |
  | 行创建时就已 active（无过渡） | 3em(48px) | 1 | 768 |
  | 由 inactive 过渡过来（过渡起点） | 1.5em(24px) | 2 | 384×2 = 768 |

  → 两条路径收敛到同一个宽度，滚动判定一致。
  副歌词原本完全没有补偿，现按同样方式折算到 active 字号（2em），语义与原来的 `max` 一致。
  目标字号以常量 `ACTIVE_MAIN_EM = 3` / `ACTIVE_SECOND_EM = 2` 写在文件顶部，
  与 `MainLyric` / `SecondLyric` 的 `font-size` 对应。
- **明确未做**：没有禁用横向滚动；没有改 CSS / 视觉；没有动 `nextTime`、`--offset`、动画时长；
  没有改歌词时间或同步逻辑；没有改后端。
- **行为变化说明（有意的）**：
  1. 只有 1 个子元素（无翻译）的行，原来**不做任何补偿**，与有翻译的行不一致；
     现在两者都折算到 active 字号 → 少数原本"过渡中量到偏小、因此没滚"的行会正常滚动。
     这正是"同一行不该因 DOM 复用历史而表现不同"的要求。
  2. 判定从"取决于该行是新建还是复用"变为"取决于过渡结束后的最终宽度"，即确定性行为。
- **测试结果**：
  - `npx tsc --noEmit` → exit 0
  - `npx vite build` → exit 0
  - `node _probe/e2e.mjs` → **44 项断言全部通过**（0 失败 0 跳过）
  - `node _probe/scroll-test.mjs`（新增回归测试）→ **4 项断言全部通过**
- **回归测试**：`_probe/scroll-test.mjs`
  - 把**真实的** `src/utils/lyricScroll.ts` 用本地 tsc 转译后放进 headless Chrome 里跑，
    因此测的是实际代码而不是复制品，不会随源码漂移
  - 用真实歌词首行（安田みずほ - 夜明けのベルが鳴る）在 5 种窗口宽度 × 有/无副歌词共 10 组下，
    分别构造「创建即 active」与「inactive→active 过渡中」两种路径
  - 实测对比（修复前 → 修复后）：

    | 窗口宽 | 修复后 A/B | 修复前 A/B |
    | --- | --- | --- |
    | 1920 | 768 / 768 同 | 1536 / 768 **异** |
    | 1280 | 768 / 768 同 | 1536 / 768 **异** |
    | 1024 | 768 / 768 同 | 1536 / 768 **异** |
    | 800 | 768 / 768 同 | 1536 / 768 **异** |
    | 640 | 768 / 768 同 | 1536 / 768 **异** |

  - 断言：① 两路径宽度一致 ② 两路径滚动判定一致 ③ **修复前公式确实不一致（证明测试有检出能力）**
    ④ 窄窗口下仍会滚动（没有为了修 bug 而禁用滚动）

### [2026-09-13 04:02] 对照真实 tosu v4.26.2 校准假 tosu 与协议

参考源码：`D:\.1yrics\tosu-4.26.2`（**只读，未修改任何文件**；注意目录名没有 `v`，实际是 `tosu-4.26.2`）。
完整对照表见 `D:\.1yrics\_probe\README.md`。

- **核对文件**：`packages/server/utils/socket.ts`（WS 广播循环）、`router/socket.ts`（路由）、
  `packages/tosu/src/api/utils/buildResultV2.ts`（v2 报文组装，权威）、`api/types/v2.ts`（类型）、
  `states/global.ts`、`states/menu.ts`、`memory/stable.ts`、`common/utils/config.ts`、`common/enums/osu.ts`

- **发现的真实协议事实（与假 tosu 原实现不一致的地方，均已修 fixture）**：

  | # | 真实 tosu | 原 fake tosu 的假设 |
  | --- | --- | --- |
  | 1 | `/websocket/v2` 是 **150ms 轮询整包广播**（`pollRate` 默认 150、min 100），与状态是否变化无关 | 只在脚本显式 push 时发一包（事件驱动）→ **后端在真实 6.7 包/秒 下的稳态从未被测过** |
  | 2 | `game.paused` 由 tosu 用 `previousPlayTime === playTime` 自行推导 | 完全没有 `game` 字段 |
  | 3 | `folders.songs`(绝对) + `folders.beatmap`(相对) + `files.audio` 拼出音频真实路径 | 三者皆空串 → `read_audio_length` 恒失败 → **`length` 恒为 -1，按时长筛选候选的分支从未被覆盖** |
  | 4 | `beatmap.time.live = global.playTime`，**毫秒**（`buildResultSC.ts:115` 除以 1000 转秒可证） | 已是毫秒，一致 |
  | 5 | `lastObject` = 整图长度、`firstObject` = 首个物件起点、`mp3Length` = 菜单读到的 mp3 时长，均毫秒 | 全是 0 |
  | 6 | `artist/title` 是 ASCII 版，`artistUnicode/titleUnicode` 才是原文 | 两者相同 → 没验证过后端是否真的用 Unicode 字段搜索 |
  | 7 | 普通客户端**换图无防抖**（500ms 只对 tourney 生效） | 一致 |
  | 8 | 顶层固定 17 个 key，`rankedPlay` 在 stable 上恒 `undefined`→**key 被省略** | 只有 3 个 key |
  | 9 | **未 commit 谱面前，menu 字段是 `undefined`，key 直接被省略** | 菜单发的是零值/空串 |

- **fixture 改动**（`_probe/e2e.mjs`）：
  1. 假 tosu 改为 150ms 定时整包广播；`live` 随墙钟推进；`paused` 按 tosu 同样公式推导
  2. 新增 `_probe/run/media/`：按每首歌**真实时长**生成静音 WAV，让后端 ffprobe 读到真实 `length`，
     从而真正走到「按时长筛选候选」这条路径（已断言 `J2 length === 120000`）
  3. 补齐 v2 顶层结构（`game/client/server/state/session/settings/profile/play/leaderboard/performance/resultsScreen/directPath/tourney`）
  4. 菜单 fixture 改为**省略**未赋值字段（贴近全新 tosu），覆盖「字段缺失」解析路径
  5. `title`/`artist`(ASCII) 与 `titleUnicode`/`artistUnicode` 故意取不同值
  6. 新增「连续播放按真实速率」用例：6 秒内收到的帧数必须≈跨过的换行数（若窗口判断失效会变成 ~40 帧）
  7. 新增「暂停 + 恢复跨行」用例（真实 tosu 暂停时仍在广播）

- **后端新增（仅测试，无行为改动）**：`model/tosu_types.rs` 的 `#[cfg(test)] mod test`，
  用真实形状报文锁定 4 条解析契约。其中一条澄清了原先不确定的问题：
  **serde 对 `Option<T>` 字段「key 缺失」时取 `None`，不会丢弃整条报文**
  （若会报错，真实 tosu 任意一次字段未就绪都会让后端整包丢弃）。结论：后端结构体对
  多余字段、缺失可选字段都安全。

- **测试结果**：
  - `cargo test --lib model::` → 含 4 条新用例全部通过；`cargo test --lib service::` → 8 passed；clippy 无新警告
  - `node e2e.mjs`（清空缓存、走真实网络搜索）→ **44 项断言全部通过，0 跳过**

- **必须真机确认的两点**：
  - ✅ **已由真人测试确认（2026-09-13 04:33）**：① 回到 osu! 选曲界面歌词能正常清除 —— 说明真机是**变体 A**（tosu 在选曲界面会把 beatmap 归零），后端的 `bid == 0` 清空判定成立，**不需要改**。
  - ⏳ ② `artist` 繁简/罗马字匹配仍未确认影响面（自动搜索失败时用户能否察觉）。
- 以下为当时的原始记录（保留，注意 ① 已被实测推翻"需要改"的推测）：
  1. **回到菜单时 tosu 到底报什么**。源码推断是「保留上一张图」（`states/menu.ts` 只在 commit 时赋值、
     从不重置；`memory/stable.ts` 的 `menu()` 在 checksum 未变或 filename 不以 `.osu` 结尾时直接 return）。
     但后端作者注释说 2025-10-04 后观察到 tosu 会下发「无歌曲事件」，后端据此用 `bid == 0` 判清空。
     实测变体 B（保留上一张图）下后端**不会清屏**（仍报 `song="Lemon"`, `current=0`）。
     若真机是变体 B，需求 7「回菜单清屏」不成立，需要改清空判定。→ 已写成 `e2e.mjs` 的 `[INFO]` 段。
     **不确定**：主菜单会加载 osu! 主题曲这个伪谱面（`menu.folder === '.'`），可能正是 `id == 0` 的来源。
  2. **`artist` 匹配跨不过繁简 / 罗马字**。`artist_score()` 是「归一化后 contains」，实测
     `周杰倫` vs `周杰伦`、`米津玄師` vs `Kenshi Yonezu` 都不匹配。后果是首选候选 artist 对不上时
     `search_lyrics` 会**直接放弃整个源**（网易云、QQ 都放弃）→ 该歌永远拿不到歌词。
     这是 Plan 里 B2「补标题 / 作者匹配」的已知 TODO，非本次引入；已在 e2e 末尾 `[INFO]` 段复现记录。

- **顺带观察（未处理）**：`cargo test --lib model::` 会触发 `model::setting::test::test_default`，
  它在 CWD 生成 `tosu-proxy/config.json5` 并调用 `process::exit(0)`（`wait_for_key_press` 遇 stdin EOF）。
  该文件本身在 `.gitignore` 里（`/config.json5`），内容就是默认配置，无害。

### [2026-09-13 03:50] P3 第一片：管理 HTTP 接口（只读查询 + 展示控制）

核心链路已验证稳定，按用户给定的优先级顺序开始 P3。**只做最小、可独立验证的一片**，
不碰 Controller / 黑名单 / 缓存 / 字体。

- **完成**：新增 3 个 HTTP 接口（挂在 `/api` 下，纯新增路由，不影响既有功能）

  | 接口 | 作用 |
  | --- | --- |
  | `GET /api/status` | 当前歌曲(bid/sid/标题/歌手/时长) + 歌词状态(loaded/cleared/lineCount/current/nextTime) + 有效 offset |
  | `GET /api/lyrics/current` | 完整歌词（`time` 单位**毫秒**）+ offset + current + nextTime |
  | `POST /api/display/clear` | 清空所有展示端（广播 `setClear`） |

- **约定（B-00 的开端）**：
  - 错误统一为 `{"error":{"code":"...","message":"..."}}` + 对应状态码；已实现 `no_song` / `no_lyric`
  - 时间单位：**新的 HTTP 接口与展示 WS 统一用毫秒**；旧 WS `getAllLyric` 仍是秒，待 B-11 移除
- **配套改动**：
  - `LyricService` 新增 `current_frame()`（按最近进度算帧）、`get_now_song()`、`clear_display()`、`song_clean()`；
    `FrameTime` 对外只读暴露 `current` / `next_time`
  - `on_clean`（回菜单）改用 `song_clean()`，清展示的同时**忘掉当前歌曲** →
    修正了「已经在菜单里，`/api/status` 却还报着上一首歌」的问题
  - WS `setClear` 与 HTTP 清屏统一走 `clear_display()`：**只清展示、保留歌曲信息**，
    语义为「持续清屏，切歌/换源/上传后恢复」
- **修改文件**：`server/status.rs`（新增）、`server/mod.rs`、`config.rs`、
  `service/lyric_service.rs`、`service/song_source_service.rs`、`service/websocket_service.rs`
- **测试结果**：
  - `cargo check` / `cargo clippy`（新增代码无警告）
  - `cargo test --lib service::` → 8 passed
  - `node e2e.mjs` → **39 项断言全部通过**（新增 J1~J13 覆盖 3 个接口 + 错误契约 + 清屏广播）
  - `npx tsc --noEmit` → exit 0
- **已知问题**：
  - 「持续清屏」的**恢复**目前只靠切歌/换源/上传；同一首歌内不会自动恢复（B-02 定稿）
  - 尚未做：`PUT /api/lyrics/offset`、搜索/预览/换源、`/api/cache*`、`/api/settings`、`/api/clients`
  - 控制台仍未接这些接口（F-03/F-04 属于第 4 优先级）
- **下一步**：人工真机验证（§5），之后继续 P3 剩余接口

### [2026-09-13 03:36] release 构建验证

- **完成**：`cargo build -r --bin osu-lyric --features=new` → **exit 0**（1m12s，LTO + opt-level=z，
  产物 `target/release/osu-lyric.exe` 11.5 MB）
- **测试结果**：debug 与 release 均构建通过；`cargo clippy`（本文件）无警告；8 个单测通过；26 项 e2e 断言通过
- **当前状态**：代码改动到此为止，等待人工真机验证
- **下一步**：见 §5；之后按 Plan 顺序进入 P3（HTTP 管理 API）

### [2026-09-13 03:16] P1 后端：R1 / R2 / R3 / R5 修复 + 8 个单测

- **完成**：
  1. **R3 修复**：`set_song_by_key` / `set_manual_lyric` 中的 `_ = self.time_next(0);`（Future 被丢弃、完全不生效）改为 `self.push_now(self.now_time).await`，按**实际播放位置**而不是固定 0 立即下发完整帧。
  2. **R5 修复**：
     - `next_time` 语义统一为「距离下一行开始的**剩余毫秒**」（原来是整行时长），末行 -1；
     - 窗口判定由 `t <= end` 改为 `t < end`（左闭右开），行边界与「首行之前」窗口都能正确结束；
     - 新增 `BEFORE_FIRST_LINE` 常量表示「首行之前」窗口，该状态下**不锁定下标**，播放头进入首行时一定会补发一帧带正确 `nextTime` 的帧（原 R1 的残留缺陷）。
  3. **R1/R2 修复**：`song_source_service::on_song_update` 在 100ms 防抖**之前**立即 `clear_state() + broadcast_clear()`，消除「新歌时间驱动旧歌词」的窗口。
  4. **新增 `now_time` 字段**：`time_next` 每次都会记录最近播放进度，供换源 / 上传 / 快照使用。
  5. **重构**：把帧计算抽成纯函数 `LyricService::frame_at(&Lyric, t) -> FrameTime` + `apply_frame`，`push_now` 与 `time_next` 共用，逻辑只有一份。
  6. `set_block(true)` 补上 `clear_state() + broadcast_clear()`（原来只 `now_lyric.take()`，会留下旧下标且不清屏）。
- **修改文件**：`tosu-proxy/src/service/lyric_service.rs`、`tosu-proxy/src/service/song_source_service.rs`
- **为什么**：R3 是「换源/上传不刷新」的直接原因；R5 的窗口右闭与整行 `next_time` 会导致首行与行中跳转的滚动动画错误；R2 的 100ms 窗口会推出错误下标。
- **测试结果**：`cargo check --bin osu-lyric --features=new` exit 0；`cargo test --lib --features=new service::lyric_service` → **8 passed; 0 failed**（覆盖首行前 / 首行边界 / 行中剩余时长 / 前后跳转 / 末行 / 清空窗口失效 / 按最新进度刷新）
- **已知问题**：`Lyric::find_line` 的切片局部下标隐患（R12）仍未处理，暂不触发
- **下一步**：R4 连接快照（样式 + 无歌词时的清屏）

### [2026-09-13 03:17] P1 后端：R4 连接快照补全

- **完成**：
  1. 新增 `service::send_settings_snapshot(session_key)`：展示端连接后立即下发已保存的 6 项样式（`color` / `font` / `font-size` / `alignment` / `translation-main` / `second-show`），映射为前端已消费的广播 key（`setColor` / `setFont` / `setFontSize` / `setAlignment` / `setTranslationMain` / `setSecondShow`）。
  2. 主副类样式（颜色/字体/字号/对齐）存储为单值时，补全成前端期望的 `{first, second}` 结构。
  3. `server/websocket.rs` 的 `connect`：先发样式快照，再发歌词快照；**后端当前无歌词时改为单发 `setClear`**，避免 OBS 刷新 / 重连后残留上一首内容。
- **修改文件**：`tosu-proxy/src/service/websocket_service.rs`、`tosu-proxy/src/service/mod.rs`、`tosu-proxy/src/server/websocket.rs`
- **为什么**：R4 要求「每次建立/恢复连接下发完整快照」，原实现只有歌词且无歌词时什么都不发。
- **测试结果**：`cargo check --bin osu-lyric --features=new` exit 0
- **已知问题**：样式快照的数据来源是 `SettingEntity`，而当前**只有 WS 管理命令会写这些 key**；由于管理命令前端还没接新版协议，DB 里很可能为空 → 该快照目前多数情况不发送任何内容（属 B-03 范围，本次不展开）
- **下一步**：前端 WS 接收侧核对与修补

### [2026-09-13 03:33] 收尾：快照按当前进度重算 + 移除只写不读的 last_frame

- **完成**：
  1. `get_snapshot()` 不再重放 `last_frame`，而是按 `now_time + offset` 用 `frame_at()` **重新计算** `current / next_time`。
     原来晚加入的展示端拿到的是「上一帧推送时算出的 nextTime」，可能已经过时几秒，滚动动画时长会偏长。
  2. 删掉 `LyricPayload` 的 `last_frame` 字段——`get_snapshot` 改完后它变成只写不读的死状态。
  3. 发送处 `ws_lyric.clone().into()` 改为 `ws_lyric.into()`（`From<LyricPayload>` 是按值转换，不需要 clone）。
- **修改文件**：`tosu-proxy/src/service/lyric_service.rs`
- **为什么**：写出「当前状态」就应该按当前时刻算，而不是重放历史帧；死状态会在后续被误当成"最新帧"使用
- **测试结果**：`cargo clippy`（本文件无警告）；`cargo test --lib service::` → 8 passed；
  `node e2e.mjs` → **全部通过**（快照段 E1~E4 仍为 `current=2`，与在线端一致）
- **下一步**：release 构建 + 人工真机验证

### [2026-09-13 03:19] 前端：WS 地址 + 歌词 store

- **完成**：
  1. `BACKEND_WEBSOCKET_URL` 由写死的 `ws://127.0.0.1:41280/ws` 改为自适应：
     开发（`import.meta.env.DEV`）指向 `127.0.0.1:41280`，生产用 `window.location.host`。
     → 后端改端口、OBS 用 `127.0.0.1:<port>/lyrics` 都能连上；协议按 `https` 选 `ws/wss`。
  2. `lyricStore.applyLyricEvent` / `clearLyrics` 用 `batch()` 包住。
     WS 回调不是 Solid 事件处理器，不会自动批处理；不 batch 会让 lyrics / cursor / nextTime
     各自触发一次滚动副作用，产生一次错误滚动。
  3. `applyLyricEvent` 增加下标越界保护（越界时保持现状，不滚动到不存在的行）。
  4. 去掉 `LyricsBox` 里每次滚动都打印的 `console.log`（OBS 控制台噪音）。
- **修改文件**：`src/config/constants.ts`、`src/stores/lyricStore.ts`、`src/pages/LyricsBox/index.tsx`
- **为什么**：前两条分别是「OBS/生产环境连不上 WS」和「切歌时滚动错位」的真实风险点
- **测试结果**：`npx tsc --noEmit` exit 0；`npx vite build` exit 0
- **已知问题**：控制台页（`/lyrics/controller`）仍走旧链路，未接新版协议（本次范围外）

### [2026-09-13 03:29] 自动化端到端验证（不依赖 osu! / 浏览器）

为了不把验证工作推给人工，在**项目目录外**搭了一套可重复运行的验证环境：
`D:\.1yrics\_probe\`

| 文件 | 作用 |
| --- | --- |
| `ws-probe.mjs` | 纯 WS 客户端，验证连接快照 / 换源链路 / getAllLyric |
| `e2e.mjs` | **假 tosu 服务器 + 展示端**，脚本驱动切歌/播放/暂停/跳转/上传 |
| `run/` | 后端运行目录（自带 `config.json5`，端口 41280，tosu 指向假服务器） |

**假 tosu**：按 `model/tosu_types.rs` 的 `TosuApi` 结构发消息，因此可以在没有 osu! 的情况下
驱动整条链路。歌词来自真实 QQ / 网易云接口。

**验证结果：26 项断言全部通过**

| 对应需求 | 断言 |
| --- | --- |
| 7 自动搜索→QQ/网易云→匹配→LRC→解析→显示 | B1~B3（46 行歌词，含 origin） |
| 4 跳转跟随 | C1（0→14）、C4（14→7 反向）、C5（sequence=up） |
| 3 播放/暂停 | C3（nextTime=1410 剩余毫秒）、D1（时间不变时 0 帧） |
| 6 上传歌词即时刷新 | I1~I5（12 行；播放头在 20s → **current=2**，不是 0；nextTime=10000） |
| 1 / 5 新开 & OBS 刷新拿到完整状态 | E1~E4（中途接入立即拿到 12 行且 current 一致）；A1/H1（无歌时接入收到清屏） |
| 2 切歌不残留 | F1/F2（立即广播清屏）、F3/F4（新歌 71 行、从头部开始） |
| 清屏 | G1/G2（回菜单清屏且不再推帧） |

**怎么跑**（后端需先运行）：
```
cd D:\.1yrics\_probe\run
D:\.1yrics\tosu-lyrics-dev\tosu-proxy\target\debug\osu-lyric.exe
# 另开一个终端
cd D:\.1yrics\_probe && node e2e.mjs
```
> 想要跑「真实网络搜索」路径时先 `rm run/lyric.db`；不删则命中缓存，跑得更快但不再验证搜索。

**顺带确认的两件事**：
- 网易云 `/api/song/lyric` 对**部分歌曲**返回空歌词（如 "Clair De Lune" id 1434822376），
  对正常曲目（如 "Lemon"）正常返回。这是接口本身的可用性差异，不是代码缺陷。
- QQ 对纯音乐返回「此歌曲为没有填词的纯音乐」，代码已正确过滤并继续尝试下一个候选。

### [2026-09-13 03:31] 部署路径确认（无需改代码）

`server/file.rs` 的静态根是 `["./static/lyrics", "./static", "./"]`，是**相对 CWD** 的。
因此正确用法是**在 `dist/` 目录里运行二进制**：

```
just build            # 前端 -> dist/, 后端 -> dist/tosu-proxy
cd dist && ./tosu-proxy
```

已实测（端口 41290）：

| 请求 | 结果 |
| --- | --- |
| `/` | 308 → `/lyrics` |
| `/lyrics` | 302 → `/lyrics/` |
| `/lyrics/` | 200，返回 dist/index.html |
| `/lyrics/lyric`（深层路由） | 200（SPA fallback） |
| `/lyrics/assets/index-*.js` | 200，75414 字节 |
| `/lyrics/LRC.otf` | 200 |

**未修改 `file.rs`**：从 `dist/` 运行即可，不需要加静态根。

### [2026-09-13 03:12] 第一阶段：调查完成

- **完成**：阅读 Plan.md；盘点后端 22 个模块文件、前端 60 个源文件；确认 git 缺失；运行基线检查。
- **修改文件**：无（仅新建本文件）
- **测试结果**：`cargo check --bin osu-lyric --features=new` exit 0；`npx tsc --noEmit` exit 0；`npx vite build` exit 0；`pnpm install` exit 0
- **为什么**：先建立可信基线，避免在未知状态下改动
- **已知问题**：见 §2
- **下一步**：开始 P0/P1 后端修复

---

## 9. 2026-09-13 B-00 契约 + B-04 黑名单后端

### 9.1 B-00：本轮真正需要的五条契约

不以"大重构"方式推进，只收口后续 B-04～B-07 必须依赖的部分。

**① 歌曲身份（谁算同一首歌）**
新增 `SongIdent { bid, sid, title }`（`service/lyric_service.rs`）。
判定**只认 `bid`**；`title` 仅用于展示与"标题级黑名单规则"，绝不用于判等。
理由：旧 `find_first` 按 `bid → title → sid` 回退，导致标题相同的另一首歌继承别人的屏蔽与偏移。

**② generation / stale request（本轮最关键的正确性改动）**
`LyricService` 新增单调递增的 `generation`，换歌 / 回菜单 / 清屏各自增一次。

旧结构的问题：`song_change` 全程持有 `LYRIC_SERVICE` 锁，把"联网搜索"也包在锁里。
后果有两个——快速切歌被串行化；且 abort 只是尽力而为，任务可能在 abort 生效前就已算完并在等锁。

改动（`song_change` 拆成三段）：
- 阶段 1 `begin_song`：**短暂持锁**，换代、记身份、清屏、查黑名单 / 偏移 / 缓存
- 阶段 2 `search_sources`：**完全不持锁**做网络搜索，结果只返回值，**不写共享状态**
- 阶段 3 `commit_search`：重新持锁，先 `generation_matches` + 身份双重校验，通过才提交

`abort` 保留（`BEFORE_HANDLE` / `cancel_tx` 未删），但**不再作为唯一正确性保证**。

**③ 黑名单 scope**
一条规则只属于 `bid` / `sid` / `title` 之一，读取时**不做跨作用域回退**。
`bid` 命中即命中；`title` 规则只在用户显式创建过同名标题规则时才生效。

**④ lyric source identity 与 cache 解耦**
`lyric_block`（规则）与 `lyric_config`（offset）**分表**。
删规则不碰偏移，改偏移不碰规则。B-05 的"来源绑定"将复用同一原则（尚未实现）。

**⑤ cache / source 持久化边界**
`lyric_cache` = 纯缓存（可删）；`lyric_block` / `lyric_config` = 用户数据（不随缓存删除）。

### 9.2 B-04：schema 拆分与迁移

- 新表 `lyric_block(id, scope, value, title, sid, created_at)`，
  复合唯一索引 `idx_lyric_block_scope_value`（`DeriveEntityModel` 的 `indexed` 只给单列非唯一索引，手工补）
- `lyric_config` 收敛为**只存 offset**；`disable` 列保留仅为兼容旧库，恒为 false，不参与判断
- 启动时 `init_all_table_and_migrate()`：建表 → 建索引 → 把 `disable=true` 行搬进 `lyric_block`
  （`bid` 作用域）→ 清标志 → 删除既无偏移也无用的空行。**幂等**

**实测**（手工构造旧 schema 库，两行：`(1110001,1,0)` 与 `(1110002,0,250)`）：
```
lyric_block:  [(1, 'bid', '1110001', 'Lemon', 910001)]
lyric_config: [(1110002, 910002, 'Lemon', 0, 250)]
日志: 黑名单迁移完成: 1 条旧屏蔽规则 -> lyric_block, 清理 1 条空配置行
```
拉黑的行进了新表，**有偏移的行连同 250ms 完整保留** —— 这正是 R7 要修的东西。

### 9.3 B-04：HTTP 接口

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/blocks` | `{total, items[]}`，item 含 `id/scope/value/title/sid/createdAt` |
| POST | `/api/blocks` | 幂等；同一 `(scope,value)` 重复添加只更新标题 |
| PATCH | `/api/blocks/{id}` | 只改展示标题；不存在 → 404 `not_found` |
| DELETE | `/api/blocks/{id}` | 不存在 → 404 `not_found` |
| DELETE | `/api/blocks` | 幂等清空，返回 `{removed}` |

错误体统一为 `{"error":{"code","message"}}`。
新增 `BlockError { InvalidParam, NotFound, Internal }` —— 用类型而不是"字符串前缀 + 猜"
来让 HTTP 层映射状态码（早期草稿用过 `__not_found__:` 前缀，已废弃）。

副作用集中在 `block_service::apply_current_song_effects()`：拉黑当前歌 → 清屏；
解除 → 立即 `reload_current()`。**不接收调用方传参**，直接读当前身份，
因此对"清空整个列表"这类批量操作同样成立。

旧 WS `setBlock` / `unsetBlock` / `getBlock` **保留未删**，改为读写新表。
`set_unblock` 必须在**释放服务锁之后**才调用 `reload_current()`，否则自锁死。

### 9.4 验证结果

| 套件 | 结果 |
|---|---|
| `cargo test --features new` | 36 passed（原 25，新增 11） |
| `cargo check` / `cargo clippy` | 我的新增代码 0 warning（`lyric/source/mod.rs` 的 2 条为既有，未动） |
| `_probe/blocks-api-test.mjs` | **21/21 通过** |
| `_probe/async-cancel-test.mjs` | **10/10 通过**（锁重构后回归） |
| `_probe/e2e.mjs` | **44/44 通过**（核心展示链路无回归） |
| 旧库迁移实测 | 通过，offset 未丢失 |

`blocks-api-test.mjs` 覆盖：增删改查、重复添加幂等、清空幂等、
不存在资源 404、未定义 scope / 非数字 bid 被拒、
**同名不同 bid 不互相误伤**、拉黑即刻清屏、解除即刻恢复、缓存清理不动黑名单。

### 9.5 本轮踩到的坑（供后续参考）

- `use crate::error::Result` 会遮住 `sea_orm` prelude 里 derive 宏依赖的 `Result` 别名，
  实体文件必须用全限定 `crate::error::Result`（沿用 `lyric_cache.rs` 的写法）
- 实体上不要定义 `find_by_id` / `delete_by_id` 同名方法 —— 会盖住 `EntityTrait` 造成**自递归**
  （本轮已改名为 `get_by_id` / `remove_by_id`）
- `match` 里用 const 做模式时，**没 import 的常量会退化成绑定模式**（变成 catch-all）。
  `SCOPE_SID` 漏 import 时编译器用 `non_snake_case` 警告暴露了它 —— 这是真 bug，不是风格问题
- 交叉验证：`Service` 锁里**不能**调用会再次取同一把锁的辅助函数（`reload_current` / `now_ident`）

### 9.6 本轮**未完成**（不要当作已完成）

- **B-04 前端**：`BlackList.tsx` / `BlackList/index.tsx` 仍走 `blacklistStore` 与旧 WS 协议，
  未接 `/api/blocks`。因此"Controller → HTTP → 后端 → 状态 → WS → LyricsBox"全链路
  **在本轮只完成了后半段**。
- **B-05 / B-06 / B-07：完全未开始**（歌词内容接口、缓存管理、上传与字体）。
  其中 B-07 的 R10（字体覆盖写）、B-06 的分页 / TTL 均未触碰。
- 未做真实浏览器 / CDP 验证（B-04 后端用 HTTP 断言覆盖，尚未经 Controller 点击验证）。

---

## 10. 2026-09-13 B-04 前端 + 全链路（续 §9）

### 10.1 新增前端文件

| 文件 | 职责 |
|---|---|
| `src/services/blocksService.ts` | `/api/blocks` 的 HTTP 客户端，含 `BlockRule` 类型 |
| `src/services/statusService.ts` | `GET /api/status` 客户端（当前歌曲 / 歌词状态 / 偏移 / blocked） |
| `src/hooks/useBlocks.ts` | 黑名单控制器：写操作成功后**强制重新拉取**，本地不留副本 |
| `src/pages/.../BlackList/BlackList.tsx` | 重写为服务端数据驱动 |
| `src/pages/.../BlackList/index.tsx` | "当前播放"改走 HTTP，"添加到黑名单"点击时现查 |

`Select` 组件新增 `clearable` 属性（黑名单作用域不允许清空成空值）。

### 10.2 两个真实缺陷（都是自动化测试抓出来的，不是设计推测）

**① "添加到黑名单"按钮永远点不动**
初版把按钮写成 `disabled={saving() || !song()}`，而 `song()` 只在 `onMount` 拉一次。
用户先进控制台、后开始放歌时，`song()` 始终是 null → 按钮永久禁用。
修复：**点击时现查一次 `/api/status`**，以点击那一刻的后端状态为准。
这同时消除了"拉黑了一首已经切走的歌"的可能。

**② `blocked` 状态会过期**
初版在 `LyricService` 上缓存了 `is_blocked_now`，只在 `begin_song` 时刷新。
规则在**歌曲播放途中**被增删时，该字段立刻变成过期真相：
`/api/status` 报告 `blocked=false`，而展示端其实已经被清屏 —— 同一件事两个说法。
修复：**删掉这个缓存字段**，`/api/status` 每次现查 `blocked_rule()`。
（`blocked_rule` 只碰数据库、不取服务锁，因此是廉价查询。）

> 教训：这类"派生状态被缓存"的字段是 bug 温床。B-05/B-06 同样不要缓存
> "当前歌是否被屏蔽 / 是否有歌词"这类可以从权威表直接算出来的东西。

### 10.3 `/api/status` 新增 `blocked`

```
{ song, lyric, offset, blocked }
```

`blocked` 与 `lyric.loaded=false` 是两回事：前者是用户显式拉黑，后者是源里没有歌词。
前端（和测试）需要能区分这两者。

### 10.4 验证

| 套件 | 结果 |
|---|---|
| `npx tsc --noEmit` | 通过 |
| `npx vite build` | 通过（构建到临时目录后并入 `dist/`，`osu-lyric.exe` / `config.json5` / `lyric.db` 均保留） |
| `_probe/controller-blacklist-e2e.mjs`（新） | **27/27 通过**，含真实鼠标事件 |
| `_probe/blocks-api-test.mjs` | 21/21 |
| `_probe/async-cancel-test.mjs` | 10/10 |
| `_probe/e2e.mjs` | 44/44 |
| 重启持久化 | 通过（规则跨重启保留） |

`controller-blacklist-e2e.mjs` 用 **CDP `Input.dispatchMouseEvent` 真实指针事件**
（不是 `element.click()`）点击"添加到黑名单"与行内"删除"，
覆盖：正常添加 / 删除 / 重复添加幂等 / 重复删除 404 / 拉黑即刻清屏 /
解除即刻恢复 / WS 重连后仍是清屏 / 清空幂等 / **同名不同 bid 不误伤** /
**显式 title 规则确实同时屏蔽两首同名歌**。

> 经验沿用：`element.click()` 会绕过遮罩，上一轮已经因此漏掉过一个真实遮挡问题。
> 涉及"人能不能点得动"的验证一律用真实指针事件。

### 10.5 构建注意（重复提醒）

`vite.config.ts` 没有 `build.outDir`，默认输出到 `dist/` 且 `emptyOutDir` 为 true，
**会删掉 `dist/osu-lyric.exe` / `config.json5` / `lyric.db`**。
本轮与上轮均用 `npx vite build --outDir dist-new` 构建到临时目录再并入，未修改 `vite.config.ts`。

### 10.6 仍未处理（不属于 B-04）

- 旧 `blacklistStore` 仍被 `tosuManager.ts`（浏览器直连 tosu 的旧路径）与
  `initializeApp.ts` 的旧 WS 消息处理器引用 —— 属 **F-09 / F-10** 清理范围
- 旧 WS `setBlock` / `unsetBlock` / `getBlock` 仍保留 —— 属 **B-11** 删除范围

---

## 11. 2026-09-13 B-05 歌词内容管理（后端）

### 11.1 新增接口

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/lyrics/current` | 当前歌词 + 身份 + 偏移 + `source` + `blocked` + `lyricState` |
| GET | `/api/lyrics/search-results` | 已有候选，每项带自己的 `source/key` |
| POST | `/api/lyrics/search` | 主动搜索；空 body = 按当前歌搜 |
| GET | `/api/lyrics/preview?source=&key=` | 预览，**不改播放** |
| PUT | `/api/lyrics/source` | 应用来源绑定 |
| DELETE | `/api/lyrics/source` | 恢复自动匹配 |
| PUT | `/api/lyrics/offset` | 设置偏移（±30s 校验） |

### 11.2 契约变更（有意为之，已同步改测试）

`GET /api/lyrics/current` 旧行为：有歌但暂时没歌词 → **404 `no_lyric`**。
B-05 改为：只有**没有播放中的歌**才是 404 `no_song`；有歌但没歌词返回
**200 + `lyric: null` + `lyricState`**。

原因：没歌词是**正常且瞬时的状态**（搜索中 / 已清屏 / 被拉黑 / 该源没有），
不是错误；而且恰在这种状态下 Controller 最需要读 `blocked` 与 `source` ——
旧契约会把这两个字段一起藏进错误响应里。

`lyricState` 取值 `ok` / `blocked` / `none`，让前端不必靠 `lyric == null` 猜原因。
`_probe/e2e.mjs` 的 J14 已改写为验证新契约，并在注释里写明为什么改，
原有要保证的行为（无歌词不得伪装成有歌词）保留。

### 11.3 来源绑定：按 sid，且与缓存分离

新表 `lyric_binding(sid PK, source_type, source_key, title, artist, bid, created_at)`。

- 归属按 **sid**（歌曲级歌词资源）：同谱面集的 Easy / Hard 共享同一份绑定
- 与 `lyric_cache` **完全独立**：删缓存不删绑定，删绑定不影响缓存与偏移
- 应用来源时**先落绑定再提交歌词**：即使随后因代际失效没提交，
  下次进这首歌仍按用户选的源取词，选择不会丢

### 11.4 异步正确性

所有联网操作（搜索 / 取词 / 应用来源）在**发起时**取 `ticket()`（代际 + 身份），
提交前重新校验；不匹配返回 `STALE_REQUEST`，HTTP 层转 **409 `song_changed`**。
`abort` 保留但不再是唯一保证。

**本轮修的一个真实竞态**：代际原本只在防抖结束、`song_change` 真正开始时才自增，
于是 `on_song_update` 观察到切歌之后、防抖结束之前的这 100ms 窗口里，
一个已发出的搜索仍会拿**旧代际**通过校验并写入新歌状态。
修复：在 `on_song_update` 观察到变化的那一刻就调用 `invalidate_async()` 换代。

### 11.5 验证状态（**重要，勿高估**）

- 从**全新数据库**起步时：`_probe/lyrics-content-test.mjs` **39 项断言全部通过**
  （含预览不改播放、绑定按 sid、409 拒绝陈旧搜索、offset 立即生效并推帧、
  A→B→C / A→B→A 最终态正确、clear 后不复活）
- **该套件有硬前置**：必须先清空 `lyric_cache` **与** `lyric_binding` 再重启后端。
  根因已定位：来源绑定跨运行持久化（这是它应有的行为），一旦某首歌被绑定过别的源，
  那份歌词就写进了 `lyric_cache`；下次进这首歌会**直接命中缓存**、不再走自动匹配，
  于是基于歌词指纹（SIGN）的断言全部失效。
  **只清绑定是不够的** —— 缓存里留着的正是上一次绑定的那份歌词。
  已把该前置写进测试文件头部注释（与 `async-cancel-test.mjs` 同一写法）。
- 满足前置时该套件 **39 项全部通过**；不满足时出现指纹类伪失败。
  失败信息（`lines=undefined`）本身**不足以区分"真故障"和"状态不干净"** ——
  这是该套件目前最大的可用性缺陷，后续应让它自己检测并报出前置未满足。
- `清屏之后没有任何完整歌词帧复活` 曾在 1 次运行中报 1 帧，之后未复现，**未定位**。
- 结论：B-05 接口层在干净状态下 39 项通过。
- **前端已于同日完成**（见 §12），全链路 22 项通过，B-05 标记完成。

### 11.6 未完成

- **Controller Content 页面未迁移**（`CurrentLyrics.tsx` / `SearchResult.tsx` 仍走旧 WS）
- 预览 / 候选页面未接入；无法确认"每个候选按钮绑定自己的身份"这一前端要求
- `_probe/lyrics-content-test.mjs` 的幂等性未解决

---

## 12. 2026-09-13 B-05 前端 + 全链路

### 12.1 前端文件

| 文件 | 职责 |
|---|---|
| `src/services/lyricsContentService.ts` | `/api/lyrics/*` 全部 7 个接口的 HTTP 客户端 |
| `src/hooks/useLyricsContent.ts` | 内容控制器：当前歌词 / 搜索 / 预览 / 应用 / 恢复 / 偏移 |
| `Content/index.tsx` | 重写，onMount 走 HTTP |
| `Content/CurrentLyrics.tsx` | 重写：时间轴列表 + 复制 + 偏移输入 + 绑定状态 + `lyricState` 提示 |
| `Content/SearchResult.tsx` | 重写：搜索 / 候选 / 逐候选预览与应用 |

### 12.2 关键设计：候选身份不共享

F-04 明确点名的缺陷是"当前所有应用按钮实际上都使用上一次 preview 的对象"。
这里从数据结构上排除该可能：

- 预览结果存在 `Record<"source|key", PreviewDto>` 里，**每个候选查自己的那一份**
- `preview(candidate)` / `apply(candidate)` 接收完整候选对象，从它身上取 `(source, key)`，
  函数内部**不读任何"上一次"的变量**
- 页面上不存在"当前预览"这样的单一信号

CDP 测试用两个不同候选分别点预览，断言两个预览块身份互不相同，
且都出现在候选列表里 —— 直接针对该缺陷。

### 12.3 验证

`_probe/controller-content-e2e.mjs`（新）**22/22 通过**，真实鼠标事件：

页面加载 → 身份与后端一致 → 主动搜索出 16 个候选 →
两个候选各自预览且身份互不相同 → **预览不改播放、不改绑定** →
应用第一个候选 → 后端绑定 key 与所点候选一致 → 展示端内容变化 →
偏移写入后端 → 恢复自动匹配 → 绑定清除且展示端恢复。

### 12.4 `lyrics-content-test.mjs` 前置已写明

该套件依赖"干净状态"，根因已写入文件头：来源绑定持久化 → 命中缓存 →
不再走自动匹配 → 歌词指纹失效。**只清绑定不够，必须连缓存一起清。**
这是测试前置问题，未因此修改任何产品语义。

### 12.5 B-05 未做

- `StoredLyrics.tsx` 为空文件，本轮未处理（属 B-06 缓存范围）
- 预览只显示前 12 行，未做完整预览面板（够用即可，非缺陷）

---

## 13. 2026-09-13 B-06 缓存管理与 TTL

### 13.1 后端

`lyric_cache` 新增 `updated_at`（毫秒时间戳），用于 TTL。
**旧的 `init_entity!` 只建表不改表**，因此新增 `migrate_cache_updated_at()`：
`PRAGMA table_info` 查列 → 缺失则 `ALTER TABLE ADD COLUMN` → 把 `updated_at = 0`
的历史行回填为迁移时刻（避免一启动就全被判过期）。幂等，实测见 13.4。

接口（`server/cache.rs`）：

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/cache?page=&size=&q=` | 分页 + 标题过滤；`size` 夹到 1..=200 |
| GET | `/api/cache/count` | 总数 |
| DELETE | `/api/cache/{bid}` | 删除单条（幂等，`removed` 0/1） |
| DELETE | `/api/cache?title=` | 按标题模糊删除 |
| DELETE | `/api/cache` | 清空全部 |
| POST | `/api/cache/cleanup` | 清理过期条目 |

TTL 默认 30 天，配置 `lyricCacheTtlHours` 可覆盖，填 `0` 表示不过期。

### 13.2 修 R9

R9 描述的两处：
- **失效缓存只构造 delete 没 exec/await** —— 已改为 `delete_by_bid(cache.bid).await`
  并把结果记进日志。注意用的是**命中记录自己的 bid**，而不是当前歌的 bid：
  sid 回退命中时两者可能不同，删错行的旧写法会留下永不失效的条目。
- **清空 handler 忽略错误、未定义内存失效与重搜** —— 现在明确语义：
  **删缓存不影响正在播放的歌词**（歌词已在内存），缓存只影响"下次进这首歌要不要联网"。
  这样删除不会打断展示，也不会出现"删了缓存反而把当前歌词清掉"。

### 13.3 边界（B-00 契约 ⑤）

`cache_service` 只操作 `lyric_cache`。来源绑定 / 偏移 / 黑名单分属另外三张表，
**本模块没有任何代码路径会触碰它们**。测试直接断言了这一点（见 13.4）。

### 13.4 验证

| 项目 | 结果 |
|---|---|
| `cargo test` | **43 passed**（+3：TTL 默认值、分页算术、size 夹取） |
| `_probe/cache-api-test.mjs`（新） | **34 项全部通过** |
| `_probe/cache-ttl-test.mjs`（新） | **9 项全部通过** |
| `_probe/controller-cache-e2e.mjs`（新） | **13 项全部通过**（真实鼠标事件） |
| 旧库列迁移 | 实测通过：`lyric_cache 已补充 updated_at 列` + `回填 1 条历史条目的 updated_at` |

关键验证点：
- **未过期命中**：`updated_at` 保持不变（证明真的走了缓存）
- **过期不命中**：把 `updated_at` 改成 40 天前后重新进歌，`updated_at` 被刷新
  （证明重新联网取词），列表里该条 `expired: true`
- **边界**：删单条 / 按标题删 / 全量清空之后，来源绑定、偏移 777、黑名单 1 条**全部原样保留**
- 删缓存后当前歌词仍可用（内存态未被清）
- 清缓存 + 重新进歌仍能拿到歌词

### 13.5 本轮踩到的坑

`GET /api/cache/count` 与 `/{bid}` 都是 `cache` 的子路由，**push 顺序必须是
count / cleanup 在 `{bid}` 之前**，否则 `count` 会被当成 bid 解析。

### 13.6 未做

- 缓存条目的"当前正在播放"标记（非必须）
- 缓存体积上限 / 自动淘汰（Plan 未要求）

---

## 14. 2026-09-13 B-07 LRC 上传与字体资源

### 14.1 修 R10：字体覆盖

旧 `get_font_file()` 在文件**已存在**时用 `File::open`（只读）再 `io::copy`，
第二次上传必然失败并 `expect` 掉；而且它写的是名为 `font` 的文件，
展示端读的却是 `/LRC.otf` —— **上传的字体从来没被用上**。

现在：一律以 `File::create`（截断）打开，按种类写到展示端真正会读的路径。
测试直接断言"第二次上传后读回的字节数**精确等于**第二次的大小且内容一致"，
既证明没有失败，也证明没有尾部残留。

### 14.2 字体接口（`server/font.rs` + `service/font_service.rs`）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/font/info` | 主/副字体的版本、大小、URL |
| GET | `/api/font/{kind}` | 下载（kind = main / sub） |
| POST | `/api/font/{kind}` | 上传并覆盖 |
| POST | `/api/font/upload` | 旧入口保留，默认主字体 |
| GET | `/api/font/download` | 旧入口保留 |

**版本化**：版本号 = `mtime(ms)-size`，不需要额外表 ——
覆盖写后 mtime 必变，重启后从磁盘读出来仍是同一值。
展示端用 `/api/font/main?v=<版本>` 加载：版本变 → URL 变 → `FontFace` 重新拉取，
浏览器不会继续用旧字体。带 `v` 时回 `immutable` 长缓存，不带 `v` 时回 `no-cache`。

主/副是两个独立资源（不同文件、不同家族名 `LRC` / `LRC-Sub`、独立版本）。

### 14.3 LRC 上传（`POST /api/lyrics/upload`）

- 统一错误体；**非法文件不替换旧歌词**
- 校验链：空 → 体积上限 1 MiB → 去 BOM → UTF-8 → 解析 → **至少要有一行时间标签**
  （最后一条是新增的：能"解析成功"但没有时间标签的文本不是 LRC）
- 提交前校验 `sid`：上传绑定到**发起时那首歌的 sid**，切到别的歌后不落到新歌上
- 有效歌词按当前播放进度 `push_now`，不回到第 0 行

### 14.4 本轮踩到的两个坑

1. **响应头插到了 Request 上**。旧代码 `req.headers_mut()` 是给 `NamedFile::send`
   用的；我改用 `res.body()` 之后必须插到 `res.headers_mut()`，
   否则 ETag / Cache-Control 全部丢失（测试直接暴露了 etag=null）。
2. **路由前缀不一致**。上传原本挂在 `/api/lyric/upload`（单数），
   而其它内容接口都在 `/api/lyrics/*`（复数）。请求打到不存在的路径时
   salvo 提前关连接，客户端表现为 `write ECONNABORTED` —— 报错信息完全指不到根因。
   现已统一到 `/api/lyrics/upload`，旧路径保留。

### 14.5 验证

`_probe/upload-font-test.mjs`（新）**30 项全部通过**，覆盖：
连续覆盖上传（R10）→ 版本号变化 → 字节精确相等 → ETag/缓存头 →
主副独立 → 非法字体被拒且不破坏原文件 → LRC 上传立即生效 →
BOM 容忍 → 无时间标签 / 非 UTF-8 / 空文件全部被拒且旧歌词原样保留 →
上传绑定属于发起时的 sid（切歌后不落到新歌、回来仍是自己的）→ 字体版本跨重启稳定。

`cargo test` **46 passed**，clippy 新增代码 0 warning。

### 14.6 未做

- 字体 / 上传的**页面**（F-06）；展示端已改为读取版本化 URL
- 上传的 LRC 没有单独持久化到 `lyric_binding`（它按 sid 落在缓存里）

---

## 15. 2026-09-13 B-08 在线展示端与定向 blink

### 15.1 会话模型：在线会话 ≠ 持久身份

`WebsocketSession` 的条目从裸 `ClientType` 升级为 `SessionEntry`：

| 字段 | 含义 |
|---|---|
| `key` | **会话标识**，每次连接随机生成，重连就换 |
| `identity` | 客户端通过 `?id=obs-main` **自报的稳定身份**，可为空（匿名会话） |
| `connected_at` / `user_agent` | 展示与排查用 |

这是 B-08 要求"区分在线会话和持久身份"的落地：定向既可以按会话 key，
也可以按身份；重连后 key 变了、身份不变。

### 15.2 接口

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/clients` | 在线**展示端**列表（不含 setter / 管理连接） |
| POST | `/api/clients/{id}/blink` | 定向闪烁；`id` 可以是会话 key 或自报身份 |
| POST | `/api/clients/blink` | 全部展示端闪烁 |

blink 复用**展示事件通道**（`SettingPayload` 的 `setBlink`，与 `setClear` 同类），
没有引入新的管理消息类型 —— WS 在本项目里只做展示事件传输。

### 15.3 只有展示端进列表

`display_clients()` 过滤 `ClientType::Client`。setter / 管理连接没有画面，
让它们出现在"在线客户端"里既没意义也会误导用户。

### 15.4 验证

`_probe/clients-blink-test.mjs`（新）**24 项全部通过**：
列表只含展示端（setter 不在）→ 匿名/具名身份正确 → **定向 blink 只到目标、
其它展示端与 setter 都收不到** → 按会话 id 定向 → 离线目标 404 `not_found` →
blink 全部 → 断开后从列表消失 → **重连后身份保持而会话 key 变化** →
重连后仍可按身份定向 → 展示链路（歌词/设置快照）未受影响。

### 15.5 未做

- 展示端**视觉上的**闪烁效果（属于 F-07 客户端页 / 展示端 UI）
- 客户端页（F-07）

---

## 16. 2026-09-13 B-09 阴影设置

### 16.1 模型

`ShadowSettings { enable, inset, color, offset }`，**沿用前端既有的 `Shadow` 形状**
（原来叫 `type` 的字段在服务端叫 `offset`），作为 `Pair<ShadowSettings>` 进入
`LyricSettings` —— 与颜色/字号/字体保持一致的主副模型，没有另起设计系统。

- WS key：`setShadow`（`ALL_SETTING_KEYS` 从 6 项变 7 项）
- 默认值：关闭、非 inset、`#000000`、无偏移
- 校验：颜色复用 `validate_color`；偏移必须是 1~3 个带单位（px/em/rem）的长度值或空串

### 16.2 展示端

`LyricsBox` 里原来写死的 `drop-shadow-[5px_5px_3px_rgba(0,0,0,1)]` 改为由设置驱动：
`shadowFilter("first" | "second")` 生成 CSS `filter`，**未启用时返回 `undefined`**
（不写 filter，保持"完全没有阴影"的原始外观，而不是写一个空 filter）。

### 16.3 验证

`_probe/shadow-test.mjs`（新）**16 项全部通过**：
默认值 → PATCH 成功且返回最终状态 → **广播恰好一次**（先扣掉接入时的设置快照）→
广播内容含主副两份 → 再次修改再广播一次 → 非法 offset / 颜色被拒且不污染 →
**CDP 实测展示端 DOM 的 `filter` 真的带上了颜色与偏移** → 关闭后 filter 被清掉 →
**重启后端后设置完全一致**。

> 测试踩坑：展示端接入时后端会先下发一次**完整设置快照**（含 `setShadow`），
> 那是"当前状态"不是"这次变更"。断言必须扣掉快照再计数，否则永远多一条。

---

## 17. 2026-09-13 B-09 补充 / B-10 黑名单元数据

（B-09 详见 §16。）

### 17.1 B-10：reason 元数据

结论：**旧 UI 一直有"原因"输入框，但它从来没有被持久化过** ——
填了之后刷新页面就没了。这不是"UI 不需要"，而是"功能是假的"。
因此补上真正的存储，而不是把输入框删掉。

- `lyric_block` 新增 `reason` 列；`migrate_block_reason()` 幂等补列（PRAGMA → ALTER）
- `POST /api/blocks` 接受 `reason`；`PATCH /api/blocks/{id}` 可只改 `reason`
- `update_title` 改为 `update_meta(id, title, reason)` —— **作用域与取值不可改**，
  因此规则身份不变，也不需要重算当前歌是否被屏蔽
- 前端 BlackList 恢复"备注"列与表单字段（现在是真的会存下来）

### 17.2 验证

- 接口：创建带 reason → 列表回读 → 只 PATCH reason → **scope/value 未被改动**
- 重启：`scope=sid value=910003 reason="persist me"` 重启后完全一致
- **B-04 回归**：`controller-blacklist-e2e.mjs` 27 项全部通过（B-10 没有破坏 B-04 闭环）
- `cargo test` 50 passed（`lyric::tests::test_parse_lyric` 是**联网**测试，
  在 QQ 被频繁请求时会失败，属既有测试的环境依赖，非本轮引入）

---

## 18. 2026-09-13 B-11 彻底移除 WS 管理入口

### 18.1 23 个管理 key 逐个核对

枚举来源是 `handle_setting` 里的 `key_matcher!`，共 **23 个**：

setClear / setFont / getFont / setFontSize / getFontSize / setAlignment / getAlignment /
setColor / getColor / setTranslationMain / getTranslationMain / setSecondShow / getSecondShow /
getLyricList / setLyricSource / getAllLyric / setBlock / getBlockList / setUnblock /
getCacheCount / setCacheClean / getLyricOffset / setLyricOffset

**不是"按文件搜索"判断，而是逐个实测 HTTP 对等能力**（curl 逐条打）：

| 旧 WS key | HTTP 对等 |
|---|---|
| setClear | POST `/api/display/clear` |
| setFont / getFont | PATCH / GET `/api/settings`（font） |
| setFontSize / getFontSize | PATCH / GET `/api/settings`（fontSize） |
| setAlignment / getAlignment | PATCH / GET `/api/settings`（alignment） |
| setColor / getColor | PATCH / GET `/api/settings`（textColor） |
| setTranslationMain / getTranslationMain | PATCH / GET `/api/settings`（translationMain） |
| setSecondShow / getSecondShow | PATCH / GET `/api/settings`（secondShow） |
| getLyricList | GET `/api/lyrics/search-results` |
| setLyricSource | PUT `/api/lyrics/source` |
| getAllLyric | GET `/api/lyrics/current` |
| setBlock | POST `/api/blocks` |
| getBlockList | GET `/api/blocks` |
| setUnblock | DELETE `/api/blocks/{id}` |
| getCacheCount | GET `/api/cache/count` |
| setCacheClean | DELETE `/api/cache` |
| getLyricOffset | GET `/api/status`（offset） |
| setLyricOffset | PUT `/api/lyrics/offset` |

（核对时 `getAllLyric` 一度显示 404 —— 那是**没有歌在播**时的语义化 `no_song`，
起假 tosu 后返回 200。记下来以免下次误判。）

### 18.2 删除内容

- `service/websocket_service.rs` 的**全部实现**（`handle_setting`、`key_matcher!`、
  23 个 handler、`WebsocketResult`）删除，文件只留一份说明"这里曾经是什么、为什么删除"
- `server/websocket.rs` 的 `on_ws_message` 不再 dispatch 到任何管理函数 ——
  客户端上行文本只记一条 debug 日志
- `ClientType` 枚举删除：**所有 WS 连接都是展示端**。`?setter=true` 仍然被接受，
  但**不再有任何特权**（管理通道没了，它也就没有存在意义）

**没有**做"内部转发到 HTTP 继续保留"，也没留兼容模式或隐藏通道。

### 18.3 保留内容（展示传输）

`send_to_all_client` / `send_to_one_client` / `send_message`、歌词推送、`setClear`、
样式广播、`setBlink` 单播、心跳 ping/pong、接入快照 —— 全部保留。

### 18.4 验证

`_probe/ws-management-removed-test.mjs`（新）**13 项全部通过**：

- 逐个发送 12 个旧管理 key（取值刻意与当前不同）后：
  **样式未变、黑名单未变、缓存未被清空、偏移未变、歌曲仍在下屏**
- 没有收到任何带 `echo` / `error` 的回包（说明根本没有被当作管理请求处理）
- `?setter=true` 连接发送管理命令同样无效
- HTTP 侧：清屏 / 改样式 / blink 仍然正确触发展示端 WS

`clients-blink-test.mjs` 已同步更新到 B-11 的新契约（旧断言"setter 不出现在展示端列表里"
在 B-11 后不再成立，改为"带 ?setter=true 的连接现在也是普通展示端"），**更新后 24 项通过**。

### 18.5 本轮踩到的两个坑（都曾把测试搞红，均已回退）

为了让"清屏"压住在途换歌，我一度加了 `cleared_ident` 抑制机制，并顺带在
`on_song_update` 里提前更新 `now_ident`。结果：

1. `clear_source()`（恢复自动匹配）会先 `clear_display` 记下当前歌，
   紧接着的 `reload_current()` 又被自己压住 —— **自己挡自己**
2. 提前更新 `now_ident` 会让它与 `now_save_cache` **短暂不一致**，
   而多处代码假设这两个字段同步变化

两者都**超出 B-11 的范围**（B-11 只负责删 WS 管理），且引入了新的相互影响，
因此**全部回退**到 B-05 已验证的行为。"清屏与在途换歌的时序"作为一个
**已知未决问题**保留（见 §11.5 / §19）。

> 教训：`now_ident` 与 `now_save_cache` 必须**原子更新**；
> 想在两者之间插入"半更新"状态，就会在别处炸出难以定位的问题。

---

## 19. 2026-09-13 阶段收尾状态（B-04 → B-11）

### 19.1 已完成

| 阶段 | 状态 |
|---|---|
| B-04 黑名单（后端 + 前端 + 全链路） | ✅ 完成并验证 |
| B-05 歌词内容（后端 + 前端 + 全链路） | ✅ 完成并验证 |
| B-06 缓存（后端 + 前端 + TTL + 迁移） | ✅ 完成并验证 |
| B-07 LRC 上传 + 字体（后端；页面属 F-06） | ✅ 完成并验证 |
| B-08 在线展示端 + 定向 blink | ✅ 完成并验证 |
| B-09 阴影设置 | ✅ 完成并验证 |
| B-10 黑名单备注元数据 | ✅ 完成并验证 |
| B-11 移除 WS 管理入口 | ✅ 完成并验证 |
| F-03 ~ F-10、D-01 | ❌ **未开始** |

### 19.2 回归结果（后端探针全量）

在**干净状态**（清空 `lyric_cache` + `lyric_binding` 后重启）下：

| 套件 | 结果 |
|---|---|
| e2e.mjs | 全部通过 |
| blocks-api-test.mjs | 全部通过 |
| async-cancel-test.mjs | 全部通过 |
| clients-blink-test.mjs | 全部通过 |
| shadow-test.mjs | 全部通过 |
| upload-font-test.mjs | 全部通过 |
| cache-api-test.mjs | 全部通过 |
| lyrics-content-test.mjs | **2 项失败**（见 19.3） |
| ws-management-removed-test.mjs | 全部通过 |
| controller-blacklist-e2e.mjs / controller-content-e2e.mjs / controller-cache-e2e.mjs | 全部通过 |

### 19.3 两个**已定性为脏状态**的失败（附证据）

- `async-cancel-test` 在连续跑其它套件后报 1 项失败（`ident=unknown(57行)`）。
  **证据**：清空 `lyric_cache` + `lyric_binding` 并重启后，同一脚本**全部通过**。
  57 行 ≠ 指纹期望的 46 行，说明是缓存里存着**另一个来源**的歌词，不是代码缺陷。
  按用户给定的分类，这属于**脏状态**。
- `lyrics-content-test` 仍有 2 项失败：该套件**自身前半段会应用一份来源绑定**，
  于是后半段基于歌词指纹的断言失效。已知根因，尚未在套件内部自愈。

### 19.4 未解决 / 已知问题

- **清屏与在途换歌的时序**：切歌刚发出（防抖 100ms 内）随即清屏时，
  那次换歌可能在清屏之后拿到新代际并提交，于是清屏后仍冒出一帧。
  只比对代际无法区分"清屏前的换歌"与"清屏后的换歌"。**未修复，未美化**。
- ⚠️ 选曲预览跑马灯反复触发 —— 仍暂停，**未修复**
- ⏭️ QQ 解码 WARN —— 未动 `qq.rs`
- `lyric::tests::test_parse_lyric` 是**联网测试**（请求 c.y.qq.com），
  与其它测试并发跑时会因 runtime/网络抖动失败；**单独跑必过**。非本轮引入。

---

## 20. 2026-09-13 F-03 / F-04

### 20.1 F-03 HTTP 请求层与样式控制 —— 已完成（收尾）

TextStyle 在 B-03 时已整体转向 HTTP，本轮核对确认：
`index.tsx` 及 `TextColor / Font / FontSize` **没有任何 `wsService` / `defaultClient` 引用**；
主副颜色、字号、字体、对齐、翻译优先、副歌词显隐 6 项全部经 `PATCH /api/settings`；
`useSettings` 保证"成功用服务端返回值覆盖本地、失败保留原值并显示错误"。

验证：`_probe/controller-e2e.mjs`（CDP 真实鼠标事件）**全部通过**。

**未做**：主/副字体分别选择（现 UI 一次设两处）。这是既有 UI 限制，非缺陷，
按"不做无关 UI 重构"保持原样。

### 20.2 F-04 Content 页面 —— 已完成

B-05 时 Content 页已 HTTP 化，本轮补齐两处缺口：

- **清屏**：新增 `clearDisplay()`（`POST /api/display/clear`）+ Content 页按钮。
  清屏是**展示控制**不是样式设置，语义为"持续清屏直到换歌 / 换源 / 上传歌词"。
- **偏移归零**：偏移输入框旁新增「归零」按钮（`updateOffset(0)`）。

### 20.3 本轮修正的**测试**问题（都是断言不可靠，不是产品缺陷）

`controller-content-e2e.mjs` 的"应用来源后展示端内容变化"一条反复伪失败，
逐层排查后确认是**断言方式**的问题，产品行为正确：

1. 只比对展示端 DOM 文本的**前 400 字符** —— 不同来源开头往往相同，差异在中后段；
2. 改用整段哈希后暴露出第二个坑：**展示端只渲染视口内的一个窗口**
   （实测可见文本仅 117 字符），DOM 文本本身就不是可靠判据；
3. 第一个候选有可能**恰好就是自动匹配到的那一个**，"内容变了"这种断言天然失去意义。

最终改为断言真正的不变量：
**应用之后的当前歌词，必须与该候选自己的 preview 内容逐行一致** ——
这条既证明了 apply 生效，也证明了"应用的是所点候选"。
（实测 `current=57行 preview=57行` 完全一致。）

另外为该套件补了**开跑前清场**（`DELETE /lyrics/source` + offset 归零）：
来源绑定跨运行持久化，否则"应用第一个候选"可能恰好是已绑定的那一个。

### 20.4 状态

| 阶段 | 状态 |
|---|---|
| F-03 HTTP 请求层与样式控制 | ✅ 完成 |
| F-04 Content 页面 | ✅ 完成 |
| F-05 Blacklist + Cache 页面 | ⏳ 本轮未做（B-04/B-06 时两页已 HTTP 化，待核对收尾） |
| F-06 Upload / Font 页面 | ⏳ 未做 |
| F-07 Client 页面 | ⏳ 未做（B-11 后仍依赖已删除的旧 WS 管理，**当前是坏的**） |
| F-08 Shadow 页面 | ⏳ 未做 |
| F-09 / F-10 / D-01 | ⏳ 未做 |

> 说明：B-04 / B-06 阶段已经把 BlackList 页与 CacheManager 页改成 HTTP 驱动的服务端真源
> （见 §10、§13），因此 F-05 的主体工作**可能已完成**，需要在下一轮核对确认后再判定。

---

## 21. 2026-09-13 F-05 ～ F-08

### 21.1 F-05 Blacklist + Cache 页面 —— 已完成（核对确认）

两页在 B-04 / B-06 时已随 HTTP 化完成，本轮核对：

- `BlackList/` 与 `CacheManager/` 目录下 **没有任何 `wsService` / `defaultClient` 引用**
- 数据全部来自 `GET /api/blocks` / `GET /api/cache`，写操作后**重新拉取服务端状态**
- 失败以红色横幅显式呈现，不会显示成"已保存"
- 分页越界会自动回退到最后一页（F-05 明确要求）
- 作用域 / total / 搜索 / TTL / 单条·标题·全量删除 均已具备

验证：`controller-blacklist-e2e.mjs` **全通过**、`controller-cache-e2e.mjs` **全通过**。

### 21.2 F-06 上传与字体页面 —— 已完成（新建页面）

原来只有一个孤立的 `<form action="/api/font/upload">`，**不显示任何结果**，
且字体必须用户手工把文件放进运行目录。新增：

- `services/uploadService.ts`：`uploadLrc` / `fetchFontInfo` / `uploadFontFile`
- `pages/Controller/ControlTools/Upload/index.tsx`，路由 `/lyrics/controller/upload` + 侧栏入口

页面内容：
- **LRC 上传**：显示当前歌曲（来自 `/api/status`），上传后按当前进度立即刷新
- **字体**：主 / 副各一个上传框，显示 `版本 / 字节数 / 家族` 与**后端版本化 URL**，
  并**实取一次**做自检（字节数比对），确认展示端确实能加载
- 失败显示为可重试错误；重新选择同一个文件不会被 input 缓存挡住

验证：`controller-upload-e2e.mjs`（新）**全通过** —— 覆盖
LRC 上传成功（后端歌词换成上传内容）→ 非法 LRC 被拒且**不替换旧歌词** →
字体上传后版本变化、大小精确、**主副互不影响** → 页面自检显示可加载 →
**重复覆盖**同一字体版本再次变化且无残留。

### 21.3 F-07 Client 页面 —— 已完成（修复 B-11 造成的损坏）

B-11 删除 WS 管理入口后，这一页依赖的 `getOnlineClients` / `blinkOtherClient` /
`setDefaultClient` 全部失效，页面是坏的。重写为：

- `services/clientsService.ts`：`fetchClients` / `blinkClient`
- 列表来自 `GET /api/clients`（**只含展示端**），显示自报身份或匿名标识 + 连接时间
- 定向闪烁 `POST /api/clients/{id}/blink`；目标离线返回 404 时视为**正常结果**
  （提示"已离线，列表已刷新"），不是报错
- **4 秒轮询**刷新在线状态；选中的端离线时自动清空选择
- **本页只做定向操作**：不再把"必须先选客户端"变成其它管理页的前置条件

验证：`controller-client-e2e.mjs`（新）**全通过** —— 列表出现自报身份端 →
真实鼠标选中 → 定向 blink **只到目标、其它端收不到** → 断开后列表自动移除 →
对离线目标 404 → 重连后重新出现且**仍能定向**。

> 测试踩坑：一开始用"整页文本是否含 obs-main"判断行是否存在，
> 结果上一句的通知文案「已向「obs-main」发送闪烁」把它自己撑住了。
> 改为只匹配**列表行**（含"连接于"字样的按钮）。

### 21.4 F-08 阴影页面 —— 已完成（新建页面）

新增 `pages/Controller/ControlTools/Shadow/index.tsx`，路由 `/lyrics/controller/shadow` + 侧栏入口。

- 主 / 副**各一份**编辑器：启用开关、inset 开关、颜色、偏移（带预设按钮）
- 走 `/api/settings` 的 `shadow` 字段（B-09 的 `Pair<ShadowSettings>`）
- 本地**草稿态**：只有服务端确认后才写回 store；失败保留用户输入并显式报错
- 页内预览与展示端用**同一套 CSS filter 语义**
- 重新打开页面按服务端状态回填（勾选 / 偏移 / 颜色）

验证：`controller-shadow-e2e.mjs`（新）**全通过** —— 真实鼠标勾选启用、
点偏移预设、改颜色、点保存 → 后端保存且**副歌词未被连带改动** →
**展示端 DOM 的 filter 真的带上了 drop-shadow 与偏移** → 副歌词仍无 filter →
关闭后展示端 filter 被清掉 → 新开页面按服务端状态回填 → 非法偏移被拒且不污染状态。

---

## 22. 本轮阶段总结（F-03 ～ F-08）

| 阶段 | 状态 | 验证方式 |
|---|---|---|
| F-03 HTTP 请求层与样式控制 | ✅ 完成 | `controller-e2e.mjs`（CDP）全通过 |
| F-04 Content 页面 | ✅ 完成 | `controller-content-e2e.mjs`（CDP）全通过 |
| F-05 Blacklist + Cache 页面 | ✅ 完成（核对） | 两个 CDP 套件全通过 |
| F-06 上传与字体页面 | ✅ 完成（新建） | `controller-upload-e2e.mjs`（CDP）全通过 |
| F-07 Client 页面 | ✅ 完成（修复） | `controller-client-e2e.mjs`（CDP）全通过 |
| F-08 阴影页面 | ✅ 完成（新建） | `controller-shadow-e2e.mjs`（CDP）全通过 |

**剩余阶段**：F-09（旧链路清理）、F-10（删除前端 WS 管理调用链）、D-01（文档与运行入口）。
本轮按要求**不启动**这三项。

### 22.1 本轮新增 / 重写的文件

- 新增服务：`uploadService.ts`、`clientsService.ts`
- 新增页面：`ControlTools/Upload/index.tsx`、`ControlTools/Shadow/index.tsx`
- 重写页面：`ControlTools/Client/index.tsx`
- 补充：`Content/CurrentLyrics.tsx`（清屏 / 偏移归零）、`statusService.ts`（`clearDisplay`）、
  `useLyricsContent.ts`（`clear`）、`routes/index.tsx` 与 Controller 侧栏（两个新路由）

### 22.2 已知未决（本轮**未**处理，按要求记录后继续）

- **清屏与在途换歌的时序**（§19.4）
- 选曲预览跑马灯反复触发 —— 仍暂停
- QQ 解码 WARN —— 未动 `qq.rs`
- 前端仍有个别旧链路待 F-09/F-10 清理（`initializeApp` 的旧消息处理器、
  `tosuManager` 的浏览器直连 tosu 路径、`blacklistStore` 的旧用途）

---

## 23. 2026-09-13 F-09 / F-10 旧链路清理与 WS 管理调用链删除

### 23.1 先查依赖，再删除

用引用搜索建立完整依赖图，确认整条旧链**只被自己人引用**之后才动手：

| 模块 | 谁在引用 | 处置 |
|---|---|---|
| `services/webSocketService.ts` | 仅 `stores/indexStore.ts` | 删除 |
| `services/configService.ts` | 仅 `stores/indexStore.ts` | 删除 |
| `stores/blacklistStore.ts` | 仅 `stores/indexStore.ts` | 删除 |
| `stores/indexStore.ts` | `initializeApp` / `DarkModeToggle` / `LyricsBox` | 删除（见 23.2） |
| `services/managers/tosuManager.ts` | 仅 `initializeApp.ts` | 删除 |
| `adapters/`（含 netease / qq） | 仅 `tosuManager` | 删除 |
| `utils/cache.ts`（IndexedDB 歌词缓存） | initializeApp / tosuManager / webSocketService | 删除 |
| `utils/request.ts`（`/api/proxy`） | 无引用 | 删除 |
| `services/managers/lyricManager.ts` | 无引用 | 删除 |
| `types/{wsTypes,wsLyricTypes,tosuTypes}.ts` | 无引用 | 删除 |
| `utils/{parseLyrics,parseParams}.ts` | 无引用 | 删除 |

`config/constants.ts` 里的 `BACKEND_CONFIG_URL`（旧 `/api/config`）、`PROXY_URL`
（旧 `/api/proxy`）、`AUDIO_URL`、`WS_URL`（浏览器直连 tosu `24050`）、
`WS_QUERY_TIMEOUT`、`TIME_DIFF_FILTER`、`SEARCH_MUSIC_URL`、`GET_LYRIC_URL`
（浏览器侧歌词搜索）**全部删除** —— 它们只服务于已被后端取代的前端业务链路。
文件从 56 行缩到 17 行，只剩新版的 `/api` 基址与 `/ws` 地址。

### 23.2 三处"消费者"的迁移（不是简单删除）

`indexStore` 被三个地方使用，逐个改到新版真源：

1. **`LyricsBox`**：原来读 `store.getState.settings.*`（一份 indexStore 维护的
   设置快照）。改为**直接读 `settingsStore` 的信号**（`textColor()` /
   `useTranslationAsMain()` / `showSecond()`）—— 少一层副本，也就少一处会不同步的状态。
2. **`DarkModeToggle`**：原来调 `store.toggleDarkMode`。夜间模式本来就定义在
   `settingsStore`（`darkMode` 信号），现在把 `toggleDarkMode` /
   `initializeDarkMode` 一并放到那里，组件直接用。
3. **`initializeApp`**：整体重写，删掉 `initializeLegacy`。

### 23.3 `initializeApp` 重写后的职责

```
initializeApp()
  ├─ initializeDarkMode()            // 两端都需要
  └─ 非 /lyrics/controller 路径才 connectBackend()   // 展示端才接 WS
```

- 展示端（LyricsBox）：接后端 WS，只做两件事 —— 收歌词推送、收设置广播
- 控制台（Controller）：**不建立 WS 连接**，管理数据全部来自 HTTP

删除的旧代码：IndexedDB 缓存注册与清理、`/api/config` 拉取、
`wsService.registerHandler` 的 7 个旧消息处理器（text-color / use-main-translation /
add-black-list / delete-black-list / showSecond / alignment / blink-lyric）。

### 23.4 F-10：`api/websocket.ts` 改为**只接收**

改造前这个类同时承担"接收展示推送"和"WS 管理 RPC"两件事，后者包含：
`sendData`、`sendSetting`、`actions` echo 请求队列、`genKey`、
`constructor(isClient)` 的 `?setter=true` 分支，以及 20 多个管理方法
（`getFont` / `setFont` / `getBlockList` / `setCacheClean` / `getCacheCount` /
`getLyricOffset` / `setLyricOffset` …）。

**全部删除**，只保留：连接、`setLyricHandler`、`setSettingHandler`、`close`。
构造参数也去掉了 —— 不再有"接受管理命令的连接"这种模式。

`api/model.ts` 的 `WebsocketSettingTypeMap` 从 24 项收到 **9 项**，只留展示事件：
`setClear` / `setFont` / `setFontSize` / `setAlignment` / `setColor` /
`setTranslationMain` / `setSecondShow` / `setBlink` / `setShadow`。
`SongInfoKey` / `SongInfoList` / `BlockItem` 三个只服务于 WS 管理请求的类型一并删除。
`echo` 字段保留为**兼容读取**并注明"后端不会再产生它"。
`utils/helpers.ts` 的 `generateRandomString`（只被 echo 队列用来生成关联键）删除。

### 23.5 验证

| 项目 | 结果 |
|---|---|
| 旧链路运行时依赖搜索 | `/api/proxy` 0 处、`24050/websocket` 0 处、`indexedDB` 0 处、`new Websocket(false)` 0 处、`registerHandler` 0 处、`registerQueryHandler` 0 处 —— 全部为 0；`/api/config`、`setter=true`、`sendSetting` 仅剩**注释里说明历史**的 4 处 |
| 旧管理 WS key 作为发送方 | 0 处 |
| `npx tsc --noEmit` | 通过 |
| `npx vite build` | 通过 |
| `e2e.mjs`（tosu → backend → WS → 展示） | **全部通过** |
| `controller-e2e.mjs`（HTTP → backend → WS → 展示） | **全部通过** |
| `controller-content-e2e.mjs` | **全部通过** |

> 展示 WS 链路、样式/清屏/blink 等**后端主动推送**的事件全部保留且实测正常。

---

## 24. 2026-09-13 D-01 文档与运行入口收尾

### 24.1 README 重写

两份 README **按当前真实代码重写**，不再照搬过期描述：

**根 `README.md`**：架构图与职责边界、页面路由、**完整 HTTP 路由表**、
WS 路由与消息格式、构建（含 `vite build` 会清空 `dist/` 的警告）、运行、
`config.json5` 字段说明、OBS 用法、数据持久化与身份约定、开发与自动化验证。

**`tosu-proxy/README.md`**：原文件描述的是**旧 WS 管理协议**
（`?setter=true`、`getAllLyric`、`setBlock` 等）。已整体重写为：
HTTP 接口逐条说明 + 错误码表 + 各接口语义要点（`lyricState`、409 `song_changed`、
作用域不回退、TTL、版本化字体、会话 vs 持久身份）、WS 只接收的约定、
数据表与身份约定、构建运行。

两份都**只描述新版**，并明确标注旧版 23 个 WS 管理命令已移除。

### 24.2 `justfile` 修正

原文件两个问题：
- `copy-backend` 复制的是 `./dist/tosu-proxy`，而真实二进制叫 **`osu-lyric.exe`**
- `build-dist` 直接 `pnpm build`，**会清空 `dist/`**，把运行产物（二进制、
  `config.json5`、`lyric.db`、字体）一起删掉

改为：前端构建到 `dist-new` 再并入 `dist/`，后端二进制按真实名字复制，
理由写进文件头注释。

### 24.3 验收中发现并修复的问题

**`dist/osu-lyric.exe` 是过期产物**（06:03 的构建，早于 B-06）。
后果：生产目录里的后端缺少 `/api/blocks`、`/api/cache`、`/api/clients`、
`/api/font/*` 等路由（全部 404），而源码与 `target/` 里都是好的。
已用当前构建覆盖，并把这一步固化进 `just copy-backend`。

> 这正是"检查运行产物"该抓的东西 —— 只跑源码测试不会暴露它。

---

## 25. 最终自动验收

### 25.1 构建与静态检查

| 项目 | 结果 |
|---|---|
| `cargo test --features=new` | **50 passed** |
| `cargo clippy --features=new` | 新增代码 **0 warning**（仅 `lyric/source/mod.rs` 2 条既有 + 依赖提示） |
| `npx tsc --noEmit` | 通过 |
| `npx vite build` | 通过（0 error） |

### 25.2 端到端回归（11 个套件，全部通过）

```
e2e                        全部通过   <- tosu -> backend -> WS -> 展示
blocks-api-test            全部通过
clients-blink-test         全部通过
cache-api-test             全部通过
controller-e2e             全部通过   <- HTTP -> backend -> WS -> 展示（样式）
controller-blacklist-e2e   全部通过
controller-content-e2e     全部通过
controller-cache-e2e       全部通过
controller-client-e2e      全部通过
controller-upload-e2e      全部通过
controller-shadow-e2e      全部通过
```

### 25.3 生产运行产物与入口

`dist/` 完整：`osu-lyric.exe`（已更新为当前构建）、`config.json5`、
`lyric.db`、`LRC.otf`、`osu.svg`、`ffprobe`、`index.html`、`assets/`。

**生产 bundle 里旧链路命中数全部为 0**：
`24050`（浏览器直连 tosu）= 0、`api/proxy` = 0、`api/config` = 0、
`indexedDB` = 0、`setter=true` = 0。

生产启动实测：

```
页面   /lyrics -> 302 -> /lyrics/ -> 200
       controller 下 7 个路由全部 200
API    /api/status /api/settings /api/blocks /api/cache /api/cache/count
       /api/clients /api/font/info 全部 200
       /api/lyrics/current -> 404 {"code":"no_song"}   无歌时的语义化 404，正确
```

### 25.4 验收重点逐条确认

| # | 项目 | 结论 |
|---|---|---|
| 1 | LyricsBox：tosu -> backend -> WS -> display | 通过（`e2e`） |
| 2 | Controller：HTTP -> backend -> WS -> display | 通过（4 个 controller 套件） |
| 3 | 7 个管理页面全部走 HTTP | 通过（路由均 200，各自 CDP 套件通过） |
| 4 | 不存在 WS 管理入口或 setter 管理发送 | 通过（后端 B-11 已删；前端只接收） |
| 5 | 旧浏览器业务链路已清除或隔离 | 通过（生产 bundle 命中数全 0） |
| 6 | 构建与运行产物完整 | 通过（并修好一处过期二进制） |
| 7 | Plan_now / DEV_LOG 与实际状态一致 | 通过 |

---

## 26. 主线完成

B-04 ～ B-11、F-03 ～ F-10、D-01 **全部完成**。

**剩余已知但明确暂缓的问题**（本轮按要求未处理）：

1. **清屏与在途换歌的时序**（见 §19.4）：切歌刚发出、防抖窗口内随即清屏时，
   那次换歌可能在清屏之后拿到新代际并提交，于是清屏后仍冒出一帧。
   只比对代际无法区分"清屏前的换歌"与"清屏后的换歌"。**未修复、未美化。**
2. **选曲预览跑马灯反复触发** —— 调查仍暂停，**未修复**。
3. **QQ 歌词响应解码 WARN** —— 未动 `qq.rs`。
4. **`artist` 繁简 / 罗马字匹配** —— 未处理（已知自动匹配边界）。
5. `lyric::tests::test_parse_lyric` 是**联网测试**（请求 `c.y.qq.com`），
   与其它测试并发时会因网络/运行时抖动偶发失败；单独执行必过。非本轮引入。
6. `lyrics-content-test.mjs` / `async-cancel-test.mjs` 对缓存与绑定状态有前置要求，
   连续跑多个套件后会因脏状态出现伪失败（清空 `lyric_cache` + `lyric_binding`
   并重启后全部通过）。根因已在文件头注明。

**未开始**：无。主线阶段已全部完成。

---

## 27. 2026-09-13 功能增强 M1 ～ M8（主线完成之后）

主线（B-04～B-11 / F-03～F-10 / D-01）已完成后的一轮 UI 与功能增强。
**未改动已冻结的主线架构**：HTTP 管理 + WS 展示广播、bid/sid/generation 契约均不变。

### 27.1 M1 在线展示端：全局 + 单独客户端

**先修了一个真实 bug。** 需求里说"手动点 blink 没反应"，排查后确认不是点击没绑定，
而是 **F-09 删除旧链时把 `setBlink` 的接收端一并删掉了、却没有补上**：
请求发出去了、后端也单播了，展示端收到后**静默忽略**。
修复：在 `handleSettingBroadcast` 里补 `setBlink` → 调用 `lyricBlink()`。

新增能力（后端 `POST /api/clients/{id}/settings`）：

| 模式 | 触发 | 样式改动 | 测试按钮 |
|---|---|---|---|
| 全局 | 未选中任何端 | `PATCH /api/settings`（持久 + 广播全部） | 全部端闪烁 |
| 单独 | 选中某个端 | `POST /api/clients/{id}/settings`（**只推给它，不落库**） | 只有它闪烁 |

「单独」是**即时调整**，不是每客户端持久配置 —— 后端不保存，该端重连后回到全局设置
（这一点在 UI 上明确写了）。选中的端离线时自动回到全局模式。

验证：`clients-global-vs-single-test.mjs`（16 项）与
`controller-client-enhance-e2e.mjs`（CDP 真实鼠标，含"展示端 visibility 真的翻转"）。

### 27.2 M2 歌词内容控制（阅读区）

**没有**把展示端的三行滚动搬到 Controller。阅读区显示**完整歌词**，新增：

- 「跟随当前行滚动」开关 —— 只影响这个阅读区的滚动位置，与展示端播放滚动无关
- 左列由"开始时间"改为**该行持续时间**（下一行开始 − 本行开始）；
  最后一行没有下一行时显示 `—`，**不编造数值**
- **拖动调整 offset**：拖动歌词内容上下移动，1px = 10ms，向下拖 = 歌词晚出现 = offset 减小；
  拖动过程只更新本地预览，松手才写回；与手动输入共用同一个 `updateOffset`

### 27.3 M3 搜索结果

- **时长差**：显示"候选 − 当前"（正=蓝 / 负=红 / 0=中性）。没有在播歌曲时后端返回 0，
  不产生 NaN
- **统一排序**：后端按"标题匹配分优先，其次时长差绝对值"排序，**不再按来源分组**
  （旧行为是 QQ 永远在前、网易云永远在后）。实测前 6 项来源交错
- **翻译标记**：每行左侧显示"有翻译 / 无翻译 / 读取中"，数据来自**新增的
  `POST /api/lyrics/translation-check`**——真实取词判断，不是猜测（并发上限 8）
- **预览同时显示原文与翻译**
- 候选身份隔离与 generation 防护保持不变，未改动

### 27.4 M4 Shadow 简化

模型从 `{enable, inset, color, offset(string)}` 改为
`{enable, color, blur, offsetX, offsetY}`：去掉 inset，偏移拆成 X/Y，增加模糊。

UI：去掉「启用阴影」勾选框（改为「开启/关闭阴影」按钮）、去掉 inset、**去掉保存按钮**
（任何调整立即 `PATCH` 生效）。范围：偏移 ±20px、模糊 0~30px（**相对字号比例的合理区间**），
后端二次校验，越界返回 400。

主 / 副仍独立；展示端 `drop-shadow(Xpx Ypx blur color)` 实测随滑块变化。

### 27.5 M5 恢复默认文字样式

逐项核对了 `tosu-lyrics-origin` 的 `stores/settingsStore.ts` 与 `pages/LyricsBox/index.tsx`：

| 项 | 原项目默认 |
|---|---|
| 主/副颜色 | `#ffffff` / `#e0e0e0` |
| 主/副字号（active） | 3em / 2em |
| 字体名 | 空（用随包字体） |
| 对齐 | center |
| 翻译优先 / 副歌词 | 均开启 |
| 阴影 | 关闭 |

与当前后端默认值**完全一致**。新增「恢复默认样式」按钮一次性 PATCH 这些字段，
**只覆盖文字样式**，不动其它状态。

### 27.6 M6 BlackList 默认 scope 改为 sid

新增项不再让用户选 scope（表单里的选择器删除），默认 **sid** ——
同一首歌的不同难度通常共享歌词，按 bid 屏蔽常只能挡住一张图。
scope 选择器移到「添加到黑名单」下面，作为**页面级**控制，
作用于本页所有新增操作；已有条目的作用域不受影响。

后端 bid/sid/title 模型未变，未引入 fallback。

### 27.7 M7 Cache 刷新按钮

新增「刷新缓存」按钮：只重新拉一次 `GET /api/cache` 并刷新本页。
不修改缓存、不动 TTL、不删除、不重建。

### 27.8 M8 字体上传修复 + 拖拽

**根因是一个真实 bug**：salvo 的请求体大小限制。实测 5.25 MB 的
`D:/.1yrics/type/LRC.otf` 在 **448 KB 处被截断**，服务端回 400，
浏览器侧表现为 `TypeError: Failed to fetch`。

修复：`salvo::http::request::set_global_secure_max_size(32 MiB)`。
修复前 curl 实测 `HTTP 400 size_upload=458752`；修复后
`HTTP 200 uploaded=5251235`，落盘文件精确 5251024 字节。

拖拽：主 / 副字体与 LRC 区域都支持 drag & drop。
**文件选择与拖拽走同一个 upload handler**（`uploadLrcFile` / `uploadFontFor`），
不存在两套实现。

### 27.9 本轮验证汇总

| 套件 | 结果 |
|---|---|
| `cargo test --features=new` | **50 passed** |
| `cargo clippy --features=new` | 本轮新增代码 0 warning |
| `npx tsc --noEmit` / `npx vite build` | 通过 / 0 error |
| e2e / controller-e2e / controller-content-e2e | 全部通过 |
| controller-cache-e2e / controller-shadow-e2e（已更新到新模型） | 全部通过 |
| controller-blacklist-e2e（已更新到 sid 默认） | 全部通过 |
| clients-blink-test / clients-global-vs-single-test | 全部通过 |
| controller-client-enhance-e2e（CDP） | 全部通过 |

**因契约变更而更新的既有测试**（保留原本验证的行为，只改断言口径）：
- `controller-blacklist-e2e`：新增项作用域由 bid 改为 sid
- `controller-shadow-e2e`：shadow 字段与"无保存按钮"的交互方式
- `controller-client-enhance-e2e`：blink 判定阈值改为"至少翻转一次"并说明原因

### 27.10 本轮**未完成 / 未做**

- **M1 的"单独客户端调整"未覆盖全部样式项**：目前支持颜色 / 字号 / 对齐；
  字体、阴影、副歌词显隐仍需在**全局**模式下到对应页面调整
  （前端已明确写出这一点，不是隐性缺失）
- **M2 的拖动区域**是一个小的"⇕ 拖动这里调整偏移"控件，
  不是"拖动整块歌词"（后者会与文本选择、滚动冲突）
- 按用户要求**未处理**：marquee、quick change + clear 时序、QQ WARN、
  artist 繁简匹配、LRC 上传逻辑

---

## 28. 2026-09-13 最终小修复 + Git 前整理

### 28.1 阴影默认值统一（两处修复）

**问题**：主 / 副歌词阴影默认是**关闭**，且定义分散在四处，容易出现
"新装开启、点「恢复默认样式」后又关掉"这类自相矛盾。

**修复**：

| # | 位置 | 改动 |
|---|---|---|
| ① | `tosu-proxy/src/model/setting.rs` | `ShadowSettings::default().enable` → `true` |
| ① | `src/stores/settingsStore.ts` | `DEFAULT_SHADOW.enable` → `true` |
| ② | 同上两处 | 颜色确认均为 `#000000`（原本一致，本轮显式核对并加注释说明四处必须同步） |

「恢复默认样式」直接复用前端 `DEFAULT_SHADOW`，`LyricsBox` 的 filter 由 `enable` 驱动 ——
因此只需改这两处即可让四处一致，按钮与展示端逻辑未改动。

新增后端单测 `both_shadow_defaults_enabled_and_black` 锁住契约，
并把 `shadow_defaults` 的断言由"默认关闭"改为"默认开启"。

**验证**：`_probe/shadow-default-test.mjs`（新）**10 项全通过**：
全新库主 / 副默认 `enable=true` + `#000000` → 展示端**实际渲染**出
`drop-shadow(rgb(0, 0, 0) 2px 2px 3px)`（主、副都有）→ 改成"关闭 + 红/绿"后
点「恢复默认样式」→ 主 / 副都回到开启 + 纯黑 → 展示端重新出现阴影。

> 过程中踩到一个坑：第一次跑该测试时前端 bundle 是旧的（重建了 `dist/` 却忘了同步到
> `_probe/run/`），于是"恢复默认"仍写入 `enable:false`。**产物同步也是验证链的一环**，
> 不能只看源码改对了。

### 28.2 ⚠️ 真人测试发现：当前歌词无法展示（**本轮未修复**）

真人测试反馈：Controller「歌词内容」页的**当前歌词展示无法显示**。

按要求本轮**只记录，不修复，不分析根因**：

- `Plan.md` §4.1 已列为明确待办，标记**未完成 / 待处理**
- 记录要点：自动化测试 `controller-content-e2e.mjs` 在当前代码上**通过**，
  说明这是自动化**未覆盖到**的场景 —— 不能用自动化结果否定真人反馈
- 后续排查方向（仅列出，未执行）：先区分是"后端返回空"还是"前端请求/渲染失败"

**不猜测根因。**

### 28.3 建立最终整理目录

`D:\.1yrics	osu-lyrics-dev\` → 复制为 `D:\.1yrics	osu-lyrics-now\`
（排除 `node_modules` / `tosu-proxy/target` / `.git` / `stats.html`）。

- `tosu-lyrics-dev` **保持原样**，作为已验证版本备份
- `tosu-lyrics-now` 作为**仓库根目录**，未多套一层项目目录
- `tosu-lyrics-origin` 只读，未修改

体积 5.9 GB → **29 MB**（去掉 Rust `target` 5.7 GB 与 `node_modules` 126 MB）。

### 28.4 清理的本机 / 编译产物

**`tosu-lyrics-now/`**：
- 删除 `dist/lyric.db` —— 测试运行数据（假歌 1110001 等），首次运行会自动重建
- 未复制 `stats.html`、`node_modules`、`tosu-proxy/target`

**`_probe/`**（在仓库根之外，属本机测试环境）：
- 删除 10 多个 `.tmp-chrome-*` 浏览器临时 profile（每个 20~35 MB）、
  `.tmp-upload-files`、全部 `*.log` / `*.out` / `frames.json`、
  `run/osu-lyric.exe`、`run/lyric.db`
- 2.9 GB → **59 MB**
- **保留**：全部 `.mjs` 测试脚本、`run/config.json5`、`run/media`（测试音频）、
  `harness/`（假 tosu 测试页）—— 这些是**正式测试**，不是临时生成物

**`dist/` 逐项判断**（未整目录删除）：

| 文件 | 判断 | 处置 |
|---|---|---|
| `assets/`、`index.html` | Vite 构建产物 | 保留（运行需要；已 gitignore） |
| `LRC.otf`、`osu.svg` | `public/` 的副本 | 保留（源码在 `public/`） |
| `ffprobe` | 后端从内嵌 blob **运行时提取** | 保留（`tosu-proxy/.gitignore` 已忽略） |
| `config.json5` | 运行时配置 | 保留 |
| `osu-lyric.exe` | 后端二进制 | 重新构建后覆盖（见 28.6） |
| `lyric.db` | **测试运行数据** | **删除** |

### 28.5 `.gitignore`

以 **origin 规则为基础**，补充本项目新增的本机 / 构建产物：

```
tosu-proxy/target/          # Rust 编译产物
tosu-proxy/config.json5     # 运行时配置（模板是 default.config.json5）
tosu-proxy/*.db
tosu-proxy/ffprobe          # 运行时提取
tosu-proxy/static
/dist                       # 构建 + 运行产物
*.db / *.db-journal / *.db-wal
stats.html
*.out / frames.json / trace.json   # 端到端测试临时输出
.vscode/*, .idea/, *.iml, .DS_Store, Thumbs.db, desktop.ini
```

**确认未忽略**（属正式内容）：`Cargo.lock`、`pnpm-lock.yaml`、
`tosu-proxy/default.config.json5`（配置模板）、`src/`、`public/osu.svg`、
`.github/`、`.prettierrc`、`eslint.config.js`、`tsconfig.json`、`vite.config.ts`。

> ⚠️ **需要确认**：origin 的规则含 `/public/*.otf`，即默认字体 `public/LRC.otf`
> （7.9 MB 二进制）**不进仓库**。本轮沿用 origin 未改，
> 但这样全新 clone 构建出的 `dist/` 不含默认字体，展示端会退化为浏览器兜底字体
> （可用「上传与字体」页补上）。若希望开箱即用，需要删掉这条规则。

### 28.6 构建

按 **前端 → 后端 → 复制 exe** 的顺序重建，避免 `dist/` 留下旧 exe：

```bash
npx vite build --outDir dist-new && 并入 dist/     # 1. 前端
cargo build -r --features=new                       # 2. 后端
cp tosu-proxy/target/release/osu-lyric.exe dist/    # 3. 覆盖 exe
```

---

# 29. 最终开发交接（dev → 当前工作区）

> 本章是本轮长周期开发的**最终交接记录**，写在 §1 ~ §28 之后。
> §1 ~ §28 记录的是更早阶段的开发过程，**历史内容原样保留、未改写**。
> 本章只描述一件事：**相对于 GitHub `dev` 分支基线，本轮实际改了什么**。

## 29.1 基线与对比方法

| 项 | 值 |
| --- | --- |
| 基线 | GitHub `dev` 分支 = 本地 `HEAD` = `72816a2 优化代码` |
| 远端 | `https://github.com/EmitPots/tosu-lyrics.git` |
| 对比方式 | `git diff HEAD`（工作区 vs dev）、`git diff --stat HEAD`、`git diff --name-status HEAD`、`git ls-files --others --exclude-standard` |
| 当前状态 | 改动**全部未提交**（工作区 + 未跟踪文件），没有 commit / push |
| 新增文件 | 10 个（`git diff` 不含未跟踪文件，随 §29.1 一并列出） |
| 删除文件 | **0 个**（`git diff --diff-filter=D` 为空） |

```bash
# 复现本节结论
cd D:\osu-lyric-github-ready
git diff --stat HEAD
git diff --name-status HEAD
git ls-files --others --exclude-standard
```

## 29.2 改动规模总览

```
38 files changed, 2643 insertions(+), 546 deletions(-)
```

分布：前端 `src/` 30 个文件、后端 `tosu-proxy/` 8 个文件，另有 10 个新增文件。
改动覆盖 9 个主题：**歌词行数、展示端加载提示、字体系统（含一次 Bug 修复）、
候选歌词时机、搜索匹配算法、单文件发行与启动链、日志、offset、控制台 UI 重排**。

| 主题 | 主要文件 |
| --- | --- |
| 歌词行数（lyricLines） | `src/utils/lyricLines.ts`(新)、`src/pages/.../TextStyle/LyricLines.tsx`(新)、`model/setting.rs`、`LyricsBox`、`settingsStore` |
| 展示端加载提示 | `lyricStore.ts`、`initializeApp.ts`、`LyricsBox`、`service/lyric_service.rs` |
| 字体系统 + Bug 修复 | `utils/fonts.ts`、`FontPicker.tsx`(新)、`fontModeStore.ts`(新)、`FontModeToggle.tsx`(新)、`service/font_service.rs`、`server/font.rs` |
| 候选歌词时机 | `service/lyric_service.rs`、`hooks/useLyricsContent.ts`、`Content/index.tsx` |
| 搜索匹配算法 | `lyric/source/mod.rs` |
| 单文件发行 + 启动链 | `server/mod.rs`、`bin/new.rs`、`build.rs`(新)、`config.rs`、`justfile`、`scripts/`(新) |
| 日志（tosu 风格） | `config.rs` |
| offset | `server/lyrics.rs`、`Content/CurrentLyrics.tsx` |
| 控制台 UI 重排 | `pages/Controller/index.tsx`、`routes/index.tsx`、`TextStyle/index.tsx`、`Shadow/index.tsx`、`Client/index.tsx` |

## 29.3 歌词行数 `lyricLines`（新增设置项，前后端对称）

**改了什么**：新增一个「歌词行数」设置项，允许 1/3/5/7/9/11/13/15 行，默认 3 行，
当前歌词始终落在可见窗口**正中间**。

**为什么改**：原先窗口固定 3 行（`h-[300px]`，位移写死 `-(cursor - 1) * 100`），
用户无法调整；同时还存在"界面改了行数、渲染不跟随"的风险，因为行数语义没有单一出处。

**最终实现**：把行数语义收敛到**唯一一份实现** `src/utils/lyricLines.ts`，
8 套行数不复制 8 套渲染代码，全部由共享计算驱动。

| 常量 / 函数 | 值 / 语义 |
| --- | --- |
| `LYRIC_LINE_OPTIONS` | `[1,3,5,7,9,11,13,15]`（**只允许奇数**） |
| `DEFAULT_LYRIC_LINES` / `MIN` / `MAX` | `3` / `1` / `15` |
| `LYRIC_LINE_HEIGHT` | `100`（必须与每个 `<li>` 实际高度一致） |
| `normalizeLyricLines(v)` | 缺失→3；非有限数→3；`<1`→1；`>15`→15；**偶数→相邻奇数（向上优先，15 封顶时向下）** |
| `visibleSideCount(lines)` | 上下各显示多少行：`(lines-1)/2` |
| `visibleWindowStart(cursor, lines)` | `cursor - visibleSideCount(lines)` |
| `windowTranslateY(cursor, lines)` | 列表纵向位移，让当前行居中 |
| `lineDistance(index, cursor)` | `abs(index - cursor)` |
| `lineLevel(distance)` | 距离 0→**0**；距离 1→**1**；距离 ≥2→**2**（只有 3 级，不随距离无限细分） |
| `LINE_LEVEL_FONT_SCALE` | `{0:1, 1:0.5, 2:0.5}` —— 层级 1 与 2 **相同**，15 行时不会越缩越小 |
| `lyricBoxHeight(lines)` | `lines × 100` |
| `CONTROLLER_VIEWPORT_LINES` | `3`（见 §29.4） |

**兼容性（硬性要求，勿破坏）**：
`font-size` 由原先的 `active ? size : size/2` 改为 `size * lineLevelScale(level)`。
由于层级 1 与 2 都是 `0.5`，**3 行模式下与改动前逐像素一致**（1 与 1/2）。
`windowTranslateY(3 行)` 等价于原来的 `-(cursor - 1) * 100`。

**前后端一致性**（两侧必须同步，改一处要改另一处）：

| 位置 | 内容 |
| --- | --- |
| 前端类型 | `SettingsDto.lyricLines: number`（`src/types/globalTypes.ts`） |
| 前端 store | `settingsStore` 的 `lyricLines` signal；`applySettings()` 经 `normalizeLyricLines` |
| 前端 UI | `TextStyle/LyricLines.tsx`（下拉，只列合法奇数） |
| 后端常量 | `MIN_LYRIC_LINES=1`、`MAX_LYRIC_LINES=15`、`DEFAULT_LYRIC_LINES=3` |
| 后端归一化 | `normalize_lyric_lines(i32)` —— 与前端同一套规则 |
| 后端校验 | `validate_lyric_lines()`：越界或偶数直接报 `invalid_param` |
| 后端落库 | `LyricSettings::load()` 读旧数据时也过一遍归一化 |
| WS 广播 | `SettingKey::LyricLines` → `ws_key() == "setLyricLines"`（标量，**不是 Pair**） |

> ⚠️ `ALL_SETTING_KEYS` 从 **7 项变为 8 项**。任何依赖该数组长度或顺序的代码都要一起看。
> 前端 `WebsocketSettingTypeMap` 已同步新增 `setLyricLines: number`
> 与 `setLyricLoading: boolean`。

**测试**：`model/setting.rs` 新增 `lyric_lines_normalization`（含 `-20..=40` 全区间
归一化后必为合法奇数的不变量断言）与 `lyric_lines_patch_roundtrip`
（PATCH → 落库 → 广播 key/value 全链路，并断言"只影响这一个设置"）。

## 29.4 Controller 固定 3 行 viewport（`/lyrics` 不受影响）

**改了什么**：`LyricsBox` 新增可选 prop `viewportLines`；Controller 顶部预览传
`CONTROLLER_VIEWPORT_LINES = 3`，`/lyrics` **不传**。

**为什么改**：Controller 的面板位置必须恒定。若预览高度跟随 `lyricLines` 变高，
用户把行数调到 15 行时，下方面板会被挤动，操作时控件"乱跳"。

**最终实现**：

| 函数 | 作用 |
| --- | --- |
| `boxLineCount(lines, viewportLines?)` | 窗口高度与居中位移所用行数：传了 viewport 就恒用它 |
| `windowLineCount(lines, viewportLines?)` | 允许显示的最大行数 = `min(设置, viewport)` |
| `clipped()` | Controller 下 `distance > visibleSide` 的行**整行隐藏** |

两个关键细节：

1. 超出范围的行用 **`visibility: hidden`（`invisible` 类）而不是 `display:none`** ——
   每行仍占满 100px，居中位移与相邻行位置完全不变（15 行时 Controller 显示的
   仍是与 3 行时相同的三行）。
2. `routes/index.tsx` 抽出 `ControllerLayout` 组件：顶部 `<LyricsBox viewportLines={3} />`
   + 下方 `<Controller>`，路由表因此更清晰（`/controller` 直接挂 layout）。

**结果**：`/lyrics` 行为与改动前完全一致（窗口高度 = `lyricLines`，按设置完整显示）；
Controller 面板位置永远与 3 行时一致，且两边共用同一个 `LyricsBox` 组件 ——
不存在两套渲染实现。

## 29.5 展示端加载提示（`.` 递增动画）

**改了什么**：换歌 / 搜索期间，展示端显示 `1 → 5` 个点递增的加载提示（每 0.5s 加一个，
到顶回到 1 个循环）；歌词一到就消失。

**为什么改**：搜索 + 取词实测需要 50ms ~ 2.5s（偶发触及 5s 超时）。此前这段时间
屏幕完全空白，用户无法区分"正在加载"和"没有歌词"。

**最终实现**：新增 WS 事件 `setLyricLoading`（bool），与 `setClear` 同级的纯广播。

- 置 `true`：`begin_song()` 换歌时立即广播。
- 置 `false`：**统一放在 `load_plan()` 最外层**，所以命中缓存 / 命中黑名单 /
  取词失败 / 没有歌词**任何一种结束方式**都不会让提示一直转下去；
  此外 `clear`（主动清屏）与 `song_clean`（回到菜单）也会广播 `false`。
- 展示端：`lyricStore` 新增独立的 `lyricLoading` signal（**不参与 `applyLyricEvent`
  的批处理**，避免影响正常歌词显示）。
- 渲染：`showLoading() = lyricLoading() && lyrics().length === 0` ——
  **只在"正在加载且还没有歌词"时出现，绝不覆盖正常歌词**。

> 关键设计：`broadcast_loading()` **不参与搜索时序、不改变任何请求并发**，
> 只是往 WS 播一条设置消息。
> 加载点样式：字号取主歌词的 `1.15em`（跟随用户主字号），`letter-spacing: 0.4em`
> 配 `margin-right: -0.4em` 抵消末点空白，保证三种对齐下整串都不偏移。

## 29.6 控制台接入展示端 WS（顶部预览与 `/lyrics` 完全一致）

**改了什么**：`initializeApp` 原先遇到 `/lyrics/controller` 直接 `return`（不建 WS）；
现在控制台**也**接入展示端推送，并以 `?id=controller` 自报身份。

**为什么改**：控制台顶部的歌词预览与加载动画必须与 `/lyrics` 完全一致。
两边共用同一份 `lyricStore` 与 `lyricLoading`，动画天然同步，不需要第二套实现。

**最终实现**：`Websocket` 构造函数新增可选 `identity`，拼成
`${BACKEND_WEBSOCKET_URL}?id=<encodeURIComponent(identity)>`。
该参数**只影响"在线展示端"列表里显示的名字**，不改变任何推送内容 ——
因此控制台不会与真正的展示端混淆。

## 29.7 字体与字号设置 UI 重做（主 / 副独立 + 共用字体模式）

**改了什么**：

1. 字体选择从"一个下拉同时设置主副"改为**主 / 副两个独立选择器**。
2. 新增「共用字体 / 分开字体」模式开关。
3. 新增 `FontPicker` 自绘下拉，替换原生 `<select>`。
4. 上传字体后立即重新注册 FontFace（原先只刷新列表，展示端仍用旧字体）。

**为什么改**：

- 后端契约本就支持主副独立字体，但 UI 只有一个选择器，属于功能未落地。
- **原生 `<select>` 的字体无法用 CSS 控制**：Tailwind preflight 只给
  `button,input,select,optgroup,textarea` 设了 `font: inherit`，**`<option>` 不在其中**；
  且 Windows 上 Chromium 的原生下拉弹层由**系统菜单**渲染，任何 CSS 都改不了它。
  所以字体选择框的文字会随机跳成微软雅黑。自绘之后收起/展开都是普通 DOM，UI 字体稳定。
- 上传后不重新注册 FontFace，会出现"上传成功但字形没变"。

**最终实现**：

- `FontPicker.tsx`（新）：受控下拉，`options/value/onChange/disabled`，
  点击外部关闭（`mousedown` 监听 + `onCleanup` 移除）。
- `fontModeStore.ts`（新）：模式存 **localStorage**（`osu-lyrics.font-mode`），
  **不新增后端设置字段** —— 因为实际渲染只由 `font.first` / `font.second` 决定，
  共用模式下这两个值本就相同，换浏览器/换展示端看到的字形天然一致；
  「模式」只影响控制台的编辑形态。
  切入共用前把当前分开设置备份到 `osu-lyrics.font-split-backup`，切回时恢复，
  **来回切换不会丢失用户原来选的两个字体**。
- `FontModeToggle.tsx`（新）：与「对齐方式」同款的 `ToggleNSwitch`。
- `FontModeToggle` 放在歌词字体设置**上方**；共用模式下副字体选择器隐藏，
  显示"共用字体模式：副歌词跟随主字体"。
- `applyMain()` 在共用模式下**同时写两侧**（`font.second = 共用值`），
  否则会出现"UI 共用、渲染却不共用"。
- `Font.tsx` 选项列表 = `默认字体` + `上传字体`（槽位**始终存在**，
  上传后显示后端解析出的真实字体名）+ 5 个系统字体（微软雅黑/宋体/仿宋/楷体/Arial）。
- `Upload/index.tsx`：上传成功后 `await loadFont()` 立即重新注册
  （`loadFont` 内部按版本号判断，同版本不会重复拉取）。

## 29.8 字体 Bug 修复：默认字体被上传字体覆盖

> 这是本轮最重要的一个缺陷修复，涉及**文件路径 / 资源来源 / 语义归一化 / 历史兼容**四层。

**根因**：早期版本把上传字体**直接写成工作目录的 `./LRC.otf` / `./tLRC.otf`**，
而这两个路径正是随包默认字体（单文件发行时由 `ensure_runtime_dir()` 从内嵌资源释放）。
于是"上传一次字体"就等于把默认字体文件**永久覆盖**：此后选"默认字体"加载到的
其实是上传字体，重启也回不来（`RESOURCE_VERSION` 没变 → 资源释放会跳过已存在文件）。
叠加第二个语义问题：空串 `""` 在旧逻辑里被当作"自动"（有上传就用上传），
于是 **UI 显示"默认字体"、实际渲染成上传字体**。

**最终实现（五条）**：

1. **默认字体与上传字体彻底分离**：上传改写到 `<工作目录>/uploaded/LRC.otf`
   （`FontKind::upload_path()`，`UPLOAD_DIR = "uploaded"`）。两者从**文件路径**上就分开。
2. **默认字体改用程序内嵌资源**：`read_static_font()` 顺序为
   `./static/lyrics` → `./static` → **内嵌程序资源**，**不再回落到 `./`**
   （回落到 `./` 就会重新引入本 Bug）。
3. **`normalizeFontCode()` 统一 UI 与渲染层语义**：
   `{"", "LRC.otf", "tLRC.otf"}` 一律归一为 `FONT_CODE_DEFAULT`（`"default"`）。
   `resolveFamily()` 与 `needsBuiltinFont()` 内部都先归一化，FontPicker 的选中项/显示名
   也用它 —— **UI 与渲染共用同一个函数**，分歧从根上消除。
4. **历史配置兼容 + 迁移**：见下 §29.9。
5. **`RESOURCE_VERSION` 2 → 3**：递增版本才让**已有安装**把被覆盖的默认字体
   重新释放回来。

**四套互相独立的 FontFace family**（上传侧名字保持不变，已验证的注册链路不受影响）：

| 常量 | 值 | 用途 |
| --- | --- | --- |
| `UPLOADED_MAIN_FAMILY` / `UPLOADED_SUB_FAMILY` | `LRC` / `LRC-Sub` | 上传字体 |
| `DEFAULT_MAIN_FAMILY` / `DEFAULT_SUB_FAMILY` | `LRC-Default` / `LRC-Sub-Default` | **随包**默认字体 |

> 拆开的原因：两者原先共用 `LRC` / `LRC-Sub`，"上传了字体"就等于把默认字体顶掉，
> UI 上无法做到「主 = LRC.otf，副 = Unifont-JP」这种组合。
> 拆开后两者可以**同时注册、并存**。
> 内置字体**按需注册**（`needsBuiltinFont`）：`LRC.otf` 约 8MB，
> 不该每次打开页面都下载。

**两个必须记住的坑**：

1. `/lyrics/LRC.otf` **必须由字体路由直接应答，且注册在静态目录之前**。
   静态目录配了 `fallback("index.html")` 供 SPA 路由用 —— 副作用是**任何找不到的资源
   都会返回 index.html**。浏览器把这段 HTML 当字体解析会报
   `OTS parsing error: invalid sfntVersion: 1008821359`（首四字节正是 `<!do`）。
2. 静态字体路由**不能**读 `font_service::load()` —— 那指向**上传字体**
   （`uploaded/LRC.otf`）；用它的话默认字体与上传字体就是同一份字节，**family 拆开也白拆**。

**UI 名与 FontFace family 分离**：`FontInfo` 新增 `displayName`（后端解析 OTF/TTF
`name` 表 nameID 4→1，platform 3 UTF-16BE 优先，Mac Roman 兜底），**仅供 UI 显示**。
解析结果按 `mtime+size` 做版本化缓存 —— 否则 `/api/font/info` 每次都要整份读盘解析
（主+副实测合计约 16.7MB），会让该接口慢到 7~8ms。

## 29.9 历史 `LRC.otf` / `tLRC.otf` 上传字体迁移

**问题**：老安装的工作目录里，`./LRC.otf` 可能**是用户上传的字体**（而不是默认字体）。
修复后默认字体会被重新释放到该路径 —— 若不先抢救，用户上传的字体就丢了。

**实现**：`migrate_legacy_uploaded_fonts(dir, is_program_default)`：

- 对 main / sub 各检查一次；**新位置已有上传（或没有历史文件）时不动**。
- 调用方传入 `is_program_default(name, bytes)` 判断"这份字节是不是程序自带的默认字体"：
  **是 → 只是释放出来的默认资源，不是上传字体，不搬**。
- **只做复制（不删原文件）**：搬完后资源释放流程把默认字体写回原路径，
  既保住用户上传的字体，又把被覆盖的默认字体**自愈**回来。

调用点在 `ensure_runtime_dir()` 中，**必须在资源释放循环之前**。

**`RESOURCE_VERSION` 机制**：释放前读 `lyrics/resources.json` 判断是否
`contains(RESOURCE_VERSION)`；一致且文件存在 → 跳过（不全量覆盖）；不一致 → 重新释放
并重写 manifest。`USER_FILES`（`lyric.db` / `config.json5`）在循环里 `continue`，
**绝不写入、绝不覆盖**。

**回归防线**：`font_service.rs` 的单测
`uploaded_font_never_shares_path_with_default_font` 断言上传路径的父目录
必须是 `uploaded`，保证不再回到"上传覆盖默认字体"的状态。**不要删这个测试。**

## 29.10 候选歌词与主歌词解耦（候选尽早可见）

**改了什么**：候选写入 `music_cache` 的时机从 `commit_search()`（排在取词之后）
**前移到搜索结果一到手**（新增 `publish_candidates()`）。

**为什么改**：原先候选列表必须等整个主歌词流程（含 `fetch_search_lyric`，
实测 50ms~2.5s）结束才可见，表现为"候选要等主歌词出来才出现"。

**最终实现**：`search_sources()` 一返回就 `publish_candidates(&plan, results.clone())`
（clone 只为把数据 move 进服务任务，几十条以内可忽略）。这样：

- 候选与主歌词**彻底解耦** —— 搜索完成即可被 `GET /api/lyrics/search-results` 读到；
- **主歌词匹配失败不影响候选显示**。

**竞态防护（与原先完全一致，两道校验）**：`generation_matches()` 代次校验 +
`now_ident` 当前歌曲身份校验 —— 保证 A→B→C 快速切歌时旧搜索结果不污染新歌。

**前端配合**：候选列表改由 `lyricLoading` 驱动（`true` 立即清空旧候选 → 消除
"上一首候选闪现"；`false` 读取并显示新候选），不再依赖 `sid` 轮询。
`loadCandidates()` 增加了结果缓存 + **在途 Promise 复用**（同一首歌只发一次请求）。

## 29.11 搜索匹配算法：跨写法识别

**改了什么**：`lyric/source/mod.rs` 的标题 / 歌手评分算法系统性放宽，
并新增 14 个纯逻辑单测。

**为什么改**：大量真实谱面（TitleUnicode / ArtistUnicode）因为**写法差异**被误判为
不相关，导致正确歌词被丢掉。典型三类：

| 反馈场景 | 原行为 | 现行为 |
| --- | --- | --- |
| `os-宇宙人(Asterisk Makina Remix)` 查 `os-宇宙人` | 60 分 | **85 分**（剥括号附录） |
| `Song - Remastered` 查 `Song` | 低分 | **85 分**（剥版本词） |
| `篠澤広（CV: 川村玲奈）` 对 `篠澤広` | artist 命中 0 → 被否 | **命中**（全角括号拆开） |

**最终实现**：

- 版本词表 `VERSION_TAG_RE` 扩充：`remaster(ed)` / `tv[ -]?size` / `short[ -]?ver` /
  `game[ -]?ver` / `full[ -]?ver` / `version` / `ver` / `inst` / `off[ -]?vocal` /
  `sped[ -]?up` 等（并修正一处正则坑：**不要把 `\.` 写进 `\b...\b` 之间**）。
- 新增 `core_title()`（剥版本词）与 `strip_bracket_content()`（剥成对括号及内容，
  支持 `()（）[]【】{}｛｝`）。二者覆盖不同情况：括号里常带**混音师名**
  （如 `Asterisk Makina`），那不是版本词，只靠剥版本词剥不掉。
- `title_score()` 新增 **85 分档**（核心标题一致）。不给 100 是为了让标注完全一致的
  候选仍然优先。
- artist 拆解重写：`ARTIST_SEPARATORS` 加入全角括号等 18 个字符；
  `ARTIST_JOINERS` 改为 `[" feat.", " feat ", " vs.", " vs ", " x ", " with ", " meets "]`
  —— **不拆裸 `x`/`X`**（会误伤 `Xceon` / `xi`），**不要求尾随空格**
  （`Xceon feat.森永真由美` 这种中日文写法 `feat.` 后直接接名字）。
  拆分前先统一小写再取索引（`to_lowercase` 可能改变字节长度，用原串下标会 panic）。
- 新增 `TITLE_STRONG_SCORE = 85`：标题强匹配时 **artist 只参与排序、不再否决候选**
  —— CV 标注 / 罗马字假名差异 / 平台只写合作者之一都很常见。
- `version_adjust()` 改为**非对称**处理三种情况：
  目标无版本词、候选有（平台自行标注 Remastered / TV Size）→ **中性（0）**，
  不再扣 10；目标要特定版本、候选没有 → `-10`；双方都有 → 有交集 `+10`、冲突 `-10`。
- 抓词失败后的回退范围 `is_same_song()`：标题归一化后完全一致、且时长不冲突时，
  **即使 artist 写法不同也视为同一首**（否则正确歌词会失去回退机会）。

**测试**（新增 14 个，全部离线、不联网）：标题满分 / 强匹配 / 85 分档 /
版本词覆盖面（含 `Veronica`、`Instinct` 不被误判）/ 主干标题辅助函数 /
无关标题仍低分 / CV 标注拆分 / 合作署名部分命中 / 连接词不拆散 `Xceon` /
artist 分数边界 / 版本调整非对称 / 同曲容忍 artist 写法。

## 29.12 检索并发策略调整

**改了什么**：跨源搜索从 `tokio::join!` 改为 `JoinSet`（每源一个独立 tokio 任务）；
单源内 title/artist 两次搜索从**串行**改为 `tokio::join!`。

**为什么改**（两处原因完全不同，不要混淆）：

- `search_raw()`：`tokio::join!` 只能在**同一个任务里轮流推进**，
  会把两个源的 **JSON 解析、正则评分等 CPU 工作串行化**，且任意一侧的同步工作
  都会拖住另一侧。改成独立任务后两路真正并行；单源 panic 由
  `AssertUnwindSafe(...).catch_unwind()` 捕获，**不影响另一个源**；
  调用方被取消时 `JoinSet` 随函数 drop，**子任务一并取消**。
- `search_all_music()` 短标题分支：原先 `search_music(title).await?` 之后再
  `search_music(&with_artist).await?`，单源要串行等两次网络往返
  （实测该分支让 `search_sources` 达到 **≈4.5s**，而单次搜索只要 ≈2.2s）。
  改成 join 后墙钟时间降到一次往返。

**保持不变**：请求参数、合并去重逻辑、错误传播顺序（先 title 后 artist）、返回结果。

## 29.13 单文件发行与启动链

**改了什么**：新增内嵌资源机制、运行目录自举、tosu 自动联动、自动打开控制台、
本地临时目录；首次启动不再退出。

**为什么改**：目标是**单文件发行** —— 一个 exe 内嵌前端产物与默认字体，
首次运行自动展开出运行目录，用户无需手工放文件。

**最终实现**：

| 组件 | 位置 | 说明 |
| --- | --- | --- |
| 资源收集 | `tosu-proxy/build.rs`（新） | 编译期递归扫描仓库根 `embed/`，生成 `embedded_assets.rs`（`EMBEDDED: &[(&str,&[u8])]`） |
| 资源引入 | `server/mod.rs` | `include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"))` |
| 运行目录 | `ensure_runtime_dir()` | 建 `exe同目录/lyrics/` → 迁移历史上传字体 → 按版本补齐程序资源 → **最后 `set_current_dir(lyrics/)`** |
| 用户数据保护 | `USER_FILES` | `lyric.db` / `config.json5` 永不写入、永不覆盖 |
| tosu 联动 | `spawn_tosu_linked()` | 见下 |
| 自动开浏览器 | `open_controller_page()` | 在 `TcpListener::bind()` **成功之后**调用，端口取自配置不写死 |
| 本地临时目录 | `use_local_temp_dir()` | 把 `TMP`/`TEMP` 指到程序目录下 `temp/` |

> ⚠️ **`ensure_runtime_dir()` 必须在 `init_logger()` / `init_database()` 之前**。
> `bin/new.rs` 里已把它放在**最先**：否则建库与迁移会作用在旧 cwd，
> 导致 `lyrics/lyric.db` 成为空库（`no such table: lyric_cache`），且首次启动需重启。

**首次启动不再退出**：`create_default_config()` 原先在写出默认配置后
`wait_for_key_press(); std::process::exit(0);`，导致"首次启动无日志、无监听、需重启"。
现改为**直接返回默认配置**，让本次进程继续初始化，与第二次启动行为一致。

**tosu 自动启动 / 检测 / 退出联动**（tosu 端口 `24050`）：

- **检测**：单次探测不可靠（tosu 可能刚启动尚未监听），因此**重试 6 次**，
  每次 `connect_timeout(250ms)` + `sleep(250ms)`。
- **不重复启动**：确认端口已在监听 → 打印日志并返回，避免 `EADDRINUSE`。
- **独立模式**：同目录没有 `tosu.exe` → 打印"按独立模式运行"，不影响主程序。
- **工作目录**：`current_dir(dir)` 取 tosu.exe 所在目录，保证其相对路径/配置/DLL 正确。
- **非阻塞**：不等待、不 kill；stdout/stderr 设为 `piped` 并逐行转发到本进程日志
  （`info!(target: "tosu", ...)`）。
- **退出联动**：**刻意不设 `CREATE_NO_WINDOW`** —— 让 tosu 附着到本进程控制台，
  于是"关闭控制台窗口 / Ctrl+C"会同时传递给 tosu，避免其残留。

**上传相关的三个"必须保留"的修复**（都是为了绕开真实环境限制）：

1. `MAX_UPLOAD_BODY = 32 MiB` + `set_global_secure_max_size(MAX_UPLOAD_BODY)`
   —— salvo 默认上限远小于字体文件（实测 5MB 的 LRC.otf 在 448KB 处被截断 →
   服务端 400、浏览器只看到 `TypeError: Failed to fetch`）。
2. `req.form_data_max_size(MAX_UPLOAD_BODY)` —— **显式指定本次解析上限**，
   不依赖 request 级/全局设置；salvo 默认 64KB，超限会在**响应发出前**中止读取。
3. `use_local_temp_dir()` —— salvo 解析 multipart 会先写 `std::env::temp_dir()`
   （用户级 `%TEMP%`），该目录在本机不可写（`os error 5`），于是**任何**字体上传
   都会失败。换到程序自己目录下即可绕开。

## 29.14 release 打包脚本

**新增**：`scripts/package-release.ps1`（5 步：前端构建 → 组装 `embed/` →
后端 release 构建 → 产出 `tosu-lyrics.exe` → 打印产物）与 `scripts/sign-windows.ps1`。

要点：

- 前端输出刻意放到 `tosu-proxy/target/dist-package`，**避免清空 `dist/` 里可能存在的
  运行时产物**（历史事故见 §28 与 §22 的 `vite build` 清空 dist）。
- `embed/` 组装内容：`index.html`、`assets/`、`osu.svg`、`static/LRC.otf`、
  `static/tLRC.otf`、`tosu-proxy/lib/ffprobe.exe`（重命名为 `ffprobe`）。
- 后端二进制 Cargo 名是 **`osu-lyric`**，打包时重命名为 **`tosu-lyrics.exe`**。
- 输出目录由 `-FinalDir` 参数指定，不传时默认写到仓库上一级的 `release/`。
- `sign-windows.ps1`：需要**正式代码签名证书**（`-PfxPath` 或
  `OSU_LYRIC_PFX_PATH` / `OSU_LYRIC_PFX_PASSWORD` / `OSU_LYRIC_TIMESTAMP_URL`）；
  **没有证书时明确说明原因并以非零码退出，不会生成假证书，也不会修改
  Defender / SmartScreen**。SmartScreen 的"发布者：未知"只取决于证书，改源码无法消除。
- `justfile`：`build-frontend` 新增把 `static/LRC.otf`、`static/tLRC.otf`
  复制到 `dist/static/`；新增 `@sign` recipe。

## 29.15 日志系统：tosu 风格终端输出

**改了什么**：`config.rs` 的 `init_logger()` 完全重写，用自定义 `FormatEvent` 替代
原先的 `.compact()` + 默认格式。

**为什么改**：目标是让本程序日志与 tosu 输出在同一个控制台里**列对齐、来源可辨**。

**最终实现**：两行前缀固定宽度对齐（状态线用粗竖线 `┃`，时间戳落在同一列）：

```
lyrics │         │ 00:00:00.123  消息
tosu   │ 4.26.2  │ 00:00:00.449  消息
```

- 级别**只用第二根竖线的颜色**表达（INFO 绿 / WARN 黄 / ERROR 红），不再打印
  时间戳、`INFO` 字样、模块路径与行号。
- `target: "tosu"` 的行（转发的 tosu 子进程输出）**原样保留**其自带版本号/时间戳/颜色，
  只加左侧来源标签 —— 所以**不吞 tosu 自己的格式**。
- 中段宽度 9 字符，依据 tosu `logger.ts` 的字符串拼接布局推算，使两侧时间戳列一致。

**ANSI 颜色的自适应**（`ansi_enabled()`）：原先写死 `true`，但 Windows 上新开的
conhost 默认**没有**启用 `ENABLE_VIRTUAL_TERMINAL_PROCESSING`，转义序列会被原样打印，
日志里就混进 `[2m` 这类可见字符（从 PowerShell 启动时正常，因为继承了宿主的 VT 模式）。
策略：stdout 非终端（重定向/管道）→ 关闭；Windows → 读控制台模式，已启用就用彩色，
未启用则**尝试启用一次**，只有启用失败才关掉；其它平台终端即支持。
为此直接声明了 kernel32 的三个函数（`GetStdHandle` / `GetConsoleMode` / `SetConsoleMode`），
**不引入额外依赖**。

> `config.rs` 中旧的 `TimeFormat` 与 `wait_for_key_press()` 现已不再被调用。

## 29.16 offset（歌词偏移）改动

**改了什么**：

1. **移除服务端 ±30 秒上限**（原 `put_offset` 会因 `|offset| > 30_000` 返回 400）。
2. `CurrentLyrics` 的高亮/滚动来源从 HTTP 快照改为 **WS 实时 `cursor`**。
3. 拖动时立即同步滚动与高亮；松手提交成功后**再刷新一次 `current`**。
4. 用直接设置列表 `scrollTop` 取代 `scrollIntoView`。

**为什么改**：

- 上限是人为限制，整首歌长度级别的偏移也合理，服务端只需原样保存与生效。
- 原先用 `c.current()?.current`（`/api/lyrics/current` 的一次性 HTTP 快照），
  **页面停留不动时永远不会刷新**，于是歌词不跟着播放滚动。
- 偏移提交后 `current` 行号还是拖动前那一份，高亮会跳回旧行。
- `scrollIntoView` 会**连带滚动所有可滚动祖先**，包括 Controller 外层的
  `overflow-y-auto` 容器 —— 表现就是"鼠标滚轮往下滚之后又跳回原位"。

**最终实现**：

- `activeIndex()`：默认取 WS 的 `cursor()`；拖动时以当前行开始时间为基准，
  加上偏移变化量做线性平移，再在现有行时间里找落点（**复用现有行时间，不新建时间轴**；
  偏移与行时间同为**毫秒**，可直接相减）。
- `scrollToIndex()`：`el.offsetTop - (box.clientHeight - el.clientHeight)/2`，
  只滚列表自身，**外层页面滚动位置完全不受影响**。
- 本页新增 `autoFollow` signal（替代原「跟随当前行滚动」复选框），
  预览搜索结果或用户停止跟随时不跟随。
- 时间显示新增 `formatTime()`：**毫秒 → 固定 `mm:ss`**（分钟允许超过 59）。
  ⚠️ `/api/lyrics/current` 与 `/api/lyrics/preview` 的 `time` 都是**毫秒**，
  必须先 `/1000` —— 直接把毫秒当秒用会得到几十倍偏大的分钟数。

## 29.17 其它改动

**缓存标题改记原文**（`lyric_service.rs::save_lyric`）：
写入值由 `this.title`（tosu 上报的 ascii / 罗马字标题）改为
**`this.title_unicode`（原文标题）**。原因：缓存页直接展示这个字段，
而搜索与黑名单本来就用原文标题，记 ascii 会让同一首歌在两处显示成不同名字。
**只改写入值** —— 不新增字段、不改表结构；`save` 的 upsert 会一并更新 `title`，
所以已缓存的歌在下次播放重新缓存时会自动变为原文标题。

**阴影默认值变更**：偏移由 `2,2` 改为 **`3,3`**（模糊仍为 3，颜色 `#000000`，默认开启）。
**前后端两处必须一致**：前端 `DEFAULT_SHADOW`（`stores/settingsStore.ts`）与
后端 `ShadowSettings::default()`（`model/setting.rs`），两处都有注释互相点名。
**已保存的用户自定义阴影不受影响**（它们存在设置里，不会回落到默认值）。

**控制台 UI 重排**：

- **路由合并**：`/lyrics/controller/upload` 与 `/shadow` 两个独立路由**删除**，
  阴影与上传并入「文字样式」页；导航项相应从 7 项减为 5 项。
- `TextStyle` 页改为**左右两栏**（`xl:flex-row`）：左侧文字样式设置
  （颜色 / 歌词行数 / 对齐 / 共用字体 / 主副字体 / 字号 / 开关），右侧字体资源与上传。
  设置项顺序也做了调整（颜色 → 歌词行数 → 对齐方式 → 共用字体 → 字体 → 字号 → 开关）。
- 各面板标题统一由 `text-2xl` 降为 **`text-xl`**（BlackList / CacheManager / FontSize /
  Client / Shadow / TextStyle）。
- `Shadow` 默认**收起**，标题点击展开（`expanded` signal）。
- `Client` 页改为左右两栏（左侧客户端列表 + 刷新/测试按钮，右侧样式调整），
  并把原生 `input[type=color]` 换成与文字样式页同一个 **`ColorSelector`**，
  保证两处视觉一致。
- **UI 字体隔离**：Controller 内容区与 `Select` 显式钉住
  `font-family: var(--font-sans, ui-sans-serif, system-ui, sans-serif)`。
  原因：动态的歌词 `font-family` 绝不能顺着继承链污染表单控件 ——
  原生 `<select>` 一旦继承到缺字的自定义字体就会整体回退成微软雅黑。
  作用范围仅限 Controller 内容区，`ControllerLayout` 里的 `<LyricsBox />` 在它之外，
  **歌词自身的字体不受影响**。
- `Shadow` 页移除了"预览文字 Preview"块与底部取值范围说明；
  `Shadow/index.tsx` 中 `DEFAULT_SHADOW` 导入随之不再需要。

**PATCH 串行化（重要并发修复）**：`useSettings.ts` 新增**模块级** `patchChain`。
原因：`saving()` 只是**本实例**的锁，而项目里有多个 `createSettingsController` 实例
（文字样式 / 阴影 / 在线展示端 / 共用字体开关），它们可以并发 PATCH；
而每个响应都是**完整的服务端设置快照**，两个 PATCH 同时在途时后到的响应可能携带
更旧的快照，把刚改好的字段覆盖回去 —— 这正是"改完字体再切对齐方式，字体被还原"的原因。
串行后响应按发送顺序应用，最后一次应用的就是包含全部改动的状态。

**`.gitignore`**：新增 `/embed/`（打包流程填充的内嵌资源目录，由 `build.rs` 扫描，可空）。
另：原先忽略 `public/*.otf` 的旧规则**已在本轮之前删除**（`public/LRC.otf` 是项目
正式运行所需的默认字体，不是编译缓存，全新 clone 后应可直接使用）。

## 29.18 兼容性、测试与注意事项

**前端 / 后端契约同步（改动设置项时必须成对修改）**：

| 位置 | 文件 |
| --- | --- |
| 前端类型 | `src/types/globalTypes.ts`（`SettingsDto` / `SettingsPatch`） |
| 前端 WS 类型 | `src/api/model.ts`（`WebsocketSettingTypeMap`） |
| 前端 store | `src/stores/settingsStore.ts` + `src/hooks/initializeApp.ts`（`handleSettingBroadcast`） |
| 后端模型 | `tosu-proxy/src/model/setting.rs`（`LyricSettings` / `Patch` / `SettingKey` / `ALL_SETTING_KEYS`） |

**旧数据兼容**：新增字段 `lyricLines` 在旧配置里不存在 → serde 回落默认 3，
且 `LyricSettings::load()` 会对读到的值再归一化一次。
`normalizeFontCode()` 兼容旧的 `""` / `"LRC.otf"` / `"tLRC.otf"` 字体选择码。

**测试**：

- `model/setting.rs` 新增 2 个（行数归一化 + PATCH 全链路）。
- `lyric/source/mod.rs` 新增 14 个（评分算法，全部离线）。
- `font_service.rs` 保留关键回归测试
  `uploaded_font_never_shares_path_with_default_font`。
- 后端可直接跑：`cd tosu-proxy && cargo check --features=new`（本轮实测 **exit 0**）；
  `cargo test --features=new` 共 64 个测试。
  注意其中 3 个依赖外网（`test_qq_lyric_source` / `test_netease_lyric_source` /
  `lyric/mod.rs::test_parse_lyric`），1 个硬编码了原作者机器的音频路径
  （`util.rs::test_get_audio_length`，只打印不断言）—— 离线环境下失败属正常。
- 前端无自动化测试；验证方式为 `pnpm build` 成功 + 实机联调。
  `pnpm exec tsc --noEmit` 当前有 **13 条历史存量错误**（本轮实测，`exit code 2`）。
  ⚠️ **注意与早期记录的差异**：§0 / §8 / §20 ~ §27 多处记载 `npx tsc --noEmit` **exit 0**，
  说明这 13 条是**早期阶段之后**才引入的存量问题，本文档此前**没有**任何章节记录过它们。

  13 条全部集中在遗留的旧协议链（`src/adapters/**`、`src/services/managers/**`、
  `src/services/configService.ts`、`src/services/webSocketService.ts`、
  `src/stores/indexStore.ts`、`src/utils/request.ts`），共同根因是它们仍引用
  `src/config/constants.ts` 重构（改为只导出 `BACKEND_API_BASE` + 自适应 WS）时
  被移除的旧常量（`SEARCH_MUSIC_URL` / `GET_LYRIC_URL` / `WS_QUERY_TIMEOUT` /
  `PROXY_URL` / `AUDIO_URL` / `WS_URL` / `BACKEND_CONFIG_URL` / `TIME_DIFF_FILTER`、
  以及 `@/utils/helpers` 的 `generateRandomString`），另有 `indexStore.ts` 两处
  `Pair<Shadow>` 与 `Shadow` 的类型不匹配。

  **本轮未修复、也不应顺手修复**（会牵动上面那串遗留模块，回归风险高于收益）；
  **本轮改动没有新增任何一条**。若要清理，应单独开一次改动并同步更新本节。
  注意 `pnpm build`（vite）**不做类型检查**，所以这 13 条不会阻塞构建 ——
  构建成功不代表类型干净。

**发布前检查清单**：

1. `pnpm build` 通过（vite 不做类型检查，构建成功 ≠ 类型干净）。
2. `pnpm exec tsc --noEmit` 仍为 13 条，未增加。
3. `cargo check --features=new` 通过。
4. **`embed/`、`tosu-proxy/target/`、`dist/` 均不存在** —— 仓库当前是"源码态"。
   直接 `cargo build` 得到的 exe 其 `EMBEDDED` 表是**空的**（`build.rs` 只 warn），
   不含前端与字体。单文件发行只有一条正经路径：
   `pwsh -File scripts/package-release.ps1`。
5. 改了任何内嵌资源（前端产物 / 默认字体 / 新增内嵌文件）都要**递增
   `RESOURCE_VERSION`**，否则已有安装不会更新资源。
6. `git status` 中不应出现 `node_modules`、`target`、`embed`、`lyric.db`、
   `config.json5`、`tosu.exe`、exe 产物、`stats.html` 等运行时数据与构建产物。

**不要顺手改动的地方**（冻结范围，改动风险高于收益）：

- `src/utils/lyricLines.ts` 的行数语义 —— 3 行模式下与历史**逐像素一致是硬性要求**。
- `LyricsBox` —— `/lyrics` 与 Controller 预览**共用同一次实现**，改一处影响两端。
- 字体 family 名 `LRC` / `LRC-Sub` —— 这是上传字体的注册名与 CSS 命中名，
  改名等于所有已上传字体失效。
- `font_service.rs` 的 `uploaded_font_never_shares_path_with_default_font` 测试。
- `server/font.rs` 的 `read_static_font`（决定"默认字体从哪来"）与
  `get_static_font_route()` 的注册位置（必须在静态目录之前）。
- `ensure_runtime_dir()` 内三步顺序：**迁移 → 释放资源 → 写 manifest**。
- WS 事件 key 字面量（`setLyricLines` / `setLyricLoading` 等）——
  已部署的 OBS 页面/浏览器缓存里跑的是旧 bundle，key 改名 = 老页面静默失效。
- `JoinSet` / `tokio::join!` 的各自用途（见 §29.12），两者**不可互换**。

## 29.19 本轮已确认的已知问题（未修，非本轮引入）

| # | 问题 | 位置 | 说明 |
| --- | --- | --- | --- |
| 1 | `static/LRC.otf`、`static/tLRC.otf`、`public/LRC.otf` **三份内容完全相同**（SHA256 均为 `C01078CF5E0CA166…EC0E`，各 7,959,920 B） | `static/`、`public/` | 副字体实际是主字体的副本，视觉上不会有区别。按字体系统设计，副字体本应使用另一款字体。**属既定现状，本轮未改动** |
| 2 | 配置项 `log` 实际不生效 | `config.rs` | `Config.log_level` 会被反序列化，但全仓库没有 `with_max_level` / `EnvFilter` 等应用点；`init_logger()` 未使用它。调成 `debug` 不会提高日志详细度 |
| 3 | CI 工作流与当前发行方式不搭 | `.github/workflows/release.yaml` | 它在 `v.*` tag 上构建 `--bin tosu-proxy --features=old` 并 upx + zip，与当前单文件 `tosu-lyrics.exe`（`--bin osu-lyric --features=new`）发行路径无关。**本轮未改动** |
| 4 | 文档漂移 | `README.md`、`tosu-proxy/README.md` | 仍提到已删除的 `/lyrics/controller/shadow`、`/upload` 路由；shadow 字段写成 `enable/inset/color/offset`（实际为 `enable/color/blur/offsetX/offsetY`）；称 offset 限 "±30s"（本轮已移除该限制）。**本轮未改写文档** |
| 5 | 空文件 | `src/pages/Controller/ControlTools/Content/StoredLyrics.tsx`、`tosu-proxy/src/service/websocket_service.rs` | 后者甚至未在 `service/mod.rs` 中声明 `mod`。无害，但会让人以为存在未完成功能 |
| 6 | Rust 侧死代码被 `[lints.rust] unused = "allow"` 掩盖 | 后端多处 | 未使用的函数/结构编译器不会提示，阅读时需自行留意 |
| 7 | 13 条 TypeScript baseline errors | 前端遗留模块（`adapters/**`、`services/managers/**`、`configService`、`webSocketService`、`indexStore`、`utils/request`） | 早期阶段之后引入的存量问题，本文档此前未记录（§0/§8/§20~§27 记载的仍是 `tsc --noEmit` exit 0）。共同根因是引用了 `config/constants.ts` 重构时移除的旧常量。**已知、本轮未修、不得再增加**；详见 §29.18 |
| 8 | 3 个测试依赖外网、1 个硬编码个人路径 | `lyric/mod.rs`、`lyric/source/mod.rs`、`util.rs` | 离线环境下 `cargo test` 会失败，非代码缺陷 |

## 29.20 本轮涉及的文件清单

**修改（38）**

```
.gitignore
justfile
src/api/model.ts
src/api/websocket.ts
src/components/ui/Select.tsx
src/components/ui/index.tsx
src/hooks/initializeApp.ts
src/hooks/useLyricsContent.ts
src/hooks/useSettings.ts
src/pages/Controller/ControlTools/BlackList/index.tsx
src/pages/Controller/ControlTools/CacheManager/index.tsx
src/pages/Controller/ControlTools/Client/index.tsx
src/pages/Controller/ControlTools/Content/CurrentLyrics.tsx
src/pages/Controller/ControlTools/Content/SearchResult.tsx
src/pages/Controller/ControlTools/Content/index.tsx
src/pages/Controller/ControlTools/Shadow/index.tsx
src/pages/Controller/ControlTools/TextStyle/Font.tsx
src/pages/Controller/ControlTools/TextStyle/FontSize.tsx
src/pages/Controller/ControlTools/TextStyle/TextColor.tsx
src/pages/Controller/ControlTools/TextStyle/index.tsx
src/pages/Controller/ControlTools/Upload/index.tsx
src/pages/Controller/index.tsx
src/pages/LyricsBox/index.tsx
src/routes/index.tsx
src/services/uploadService.ts
src/stores/lyricStore.ts
src/stores/settingsStore.ts
src/types/globalTypes.ts
src/utils/fonts.ts
tosu-proxy/src/bin/new.rs
tosu-proxy/src/config.rs
tosu-proxy/src/lyric/source/mod.rs
tosu-proxy/src/model/setting.rs
tosu-proxy/src/server/font.rs
tosu-proxy/src/server/lyrics.rs
tosu-proxy/src/server/mod.rs
tosu-proxy/src/service/font_service.rs
tosu-proxy/src/service/lyric_service.rs
```

**新增（10）**

```
scripts/package-release.ps1                     单文件 release 打包
scripts/sign-windows.ps1                        Authenticode 签名（可选）
static/LRC.otf                                  随包默认主字体（7.9 MB）
static/tLRC.otf                                 随包默认副字体（7.9 MB，当前与主字体字节相同）
tosu-proxy/build.rs                             扫描 embed/ 生成内嵌资源表
src/components/ui/FontPicker.tsx                自绘字体下拉
src/pages/Controller/ControlTools/TextStyle/LyricLines.tsx    歌词行数设置
src/pages/Controller/ControlTools/Upload/FontModeToggle.tsx   共用/分开字体开关
src/stores/fontModeStore.ts                     字体模式（localStorage）
src/utils/lyricLines.ts                         行数语义唯一实现
```

**删除（0）**

本轮**没有删除任何文件**。`routes/index.tsx` 中删除的只是 `/upload` 与 `/shadow`
两条**路由注册**，对应的组件文件 `Upload/index.tsx`、`Shadow/index.tsx` 仍保留
（现已并入「文字样式」页使用）。

## 31. 2026-10-05 默认字体恢复为系统字体 fallback

**问题**：`ecb4815` 的 `/lyrics` 在与 `07d3f91` 相同的默认设置下（`font = ""`、未上传
字体），歌词中文字与文字之间的**视觉间距变宽**。两版歌词 `<p>` 的计算 `letter-spacing`
都是 `normal`（全项目唯一带 `0.4em` 的是加载提示那个 `<p>`），**不是排版参数变化**。

**根因**（以 `07d3f91 → ecb4815` 的实际差异为依据）：`07d3f91` 的默认档是
`font() || undefined`，元素上**根本没有 `font-family`**，继承 `html` 的 preflight 值
→ 实际使用**系统回退字体**（Windows：拉丁 `Segoe UI` + 中日韩 `Microsoft YaHei`）；
该版本内置字体的 URL 是根路径 `/LRC.otf`（必然 404），从未注册成功。`ecb4815` 修好
字体路径与静态字体路由后，默认档 `resolveFamily("")` 落到 `LRC-Default`，
**实际改用内置 MiSans Bold**，字形墨迹与侧边距随之改变。中日韩 advance 两版都精确
等于 1 em（**字间距 pitch 未变**），但 MiSans Bold 的字形墨迹更窄（`中` 的侧边距
18.8% em，YaHei Bold 为 12.3%），相邻字之间的空白近乎翻倍 —— 这才是"字距看起来变宽"
的来源。

**修改（仅 `src/utils/fonts.ts` 一个文件）**：

1. 新增 `DEFAULT_SYSTEM_FAMILY = "var(--font-sans, ui-sans-serif, system-ui, sans-serif)"`，
   与 `07d3f91` 元素所继承到的 preflight 值同源（`--default-font-family: var(--font-sans)`）。
   **不新增任何字体文件。**
2. `resolveFamily()` 默认档：`builtin`（`LRC-Default` / `LRC-Sub-Default`）→ 系统回退字体栈。
3. `loadFont()` 中副字体的"没有上传"档同步改为系统回退字体栈（副歌词会优先取
   `loadedSubFamily()`，不同步会出现"主歌词系统字体、副歌词 MiSans"）。
4. `needsBuiltinFont()` 默认档改为 `false`：默认字体不再需要内置字体文件，
   不再为默认档白下约 8 MB 的 `LRC.otf`。

上传字体（`LRC` / `LRC-Sub`）、字体选择器、`LRC-Default` / `LRC-Sub-Default` 机制
（选"上传字体"但尚未上传时仍回落到内置字体）均保持可用。

**明确未改动**：`letter-spacing`、字号（`fontSize` / `lineLevelScale`）、`font-weight`、
`shadowFilter` 与阴影默认值、`transform` / 布局 / 窗口高度，以及 `LyricsBox`、
`FontPicker`、后端字体路由、`static/` 与 `public/` 下的字体文件。

**验证**：`eslint src/utils/fonts.ts` 通过；`tsc --noEmit` 无新增错误（仅既有 13 项，
分布在 `adapters/`、`services/managers/` 等引用已删除旧常量的文件）；打包后以真实模块
实测 `resolveFamily("")` 与 `loadedSubFamily()` 均返回系统回退字体栈，且无上传字体时
`loadFont()` 的 `FontFace` 构造次数为 **0**（确认不再加载内置字体）。

## 32. 2026-10-05 最终发行产物名称统一为 `tosu-lyrics.exe`

**仅打包命名，无任何功能代码改动。**

- 仓库内的 Cargo 产物仍是 `tosu-proxy/target/release/osu-lyric.exe`（`--bin osu-lyric`；
  `justfile` 的 `copy-backend`、`scripts/package-release.ps1`、`scripts/sign-windows.ps1`
  都按这个名字引用它），**一律不改动**，以免破坏既有打包链。
- 交付使用者的单文件发行产物统一命名为 **`tosu-lyrics.exe`**
  （与 `scripts/package-release.ps1` 第 [4/5] 步一致）。
- 本轮发布目录 `release-test/` 只保留 `tosu-lyrics.exe`，旧的 `osu-lyric.exe` 已删除。
- 已实测启动该产物：`GET /lyrics/` 正常返回并通过本次新前端（内嵌 `index-Np7zGYY7.js`）。

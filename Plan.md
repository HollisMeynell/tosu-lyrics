# 第二代前端开发规划

## 项目背景

旧版前端（SolidJS + TypeScript + TailwindCSS）直接连接 tosu WebSocket 获取歌曲信息，通过代理自行从 QQ/网易云搜索歌词，在浏览器中展示。新版后端（Rust `lyric-server`）接管了歌词搜索、缓存、配置持久化等职责，前端退化为纯展示 + 控制层。

> 架构原则：**后端处理数据，前端专注展示**。后端通过 WebSocket 下发歌词、接收控制指令；前端不再直接访问 tosu 或第三方歌词 API。

---

## 一、旧版架构梳理

### 1.1 技术栈

| 项目 | 技术 |
|------|------|
| 框架 | SolidJS 1.9 + TypeScript 5.8 |
| 样式 | TailwindCSS 4.1 |
| 构建 | Vite 6.3 |
| 路由 | @solidjs/router 0.15 |
| WebSocket | reconnecting-websocket 4.4 |
| 缓存 | IndexedDB（浏览器端） |

### 1.2 目录结构

```
src/
├── adapters/              # 歌词源适配器（QQ、网易云）
│   ├── index.ts
│   ├── lyricAdapter.ts    # 抽象基类
│   ├── netease/index.ts   # 网易云歌词
│   └── qq/index.ts        # QQ音乐歌词
├── api/
│   ├── model.ts           # 旧 WS 消息模型（将被替换）
│   └── websocket.ts       # 旧 WS 客户端（将被替换）
├── assets/Icons/          # SVG 图标组件
├── components/ui/         # 通用 UI 组件库
│   ├── Button.tsx
│   ├── Select.tsx
│   ├── ToggleSwitch.tsx
│   ├── ToggleNSwitch.tsx
│   ├── ToggleList.tsx
│   ├── ToggleListExtends.tsx
│   ├── DragPanel.tsx
│   ├── Mask.tsx
│   ├── DarkModeToggle.tsx
│   ├── CustomColorSelector.tsx
│   ├── Upload.tsx
│   └── index.tsx
├── config/constants.ts    # API URL、超时等常量
├── hooks/initializeApp.ts # 应用初始化入口
├── pages/
│   ├── LyricsBox/         # 歌词展示页面（OBS 浏览器源）
│   └── Controller/        # 控制面板
│       ├── ControlTools/
│       │   ├── BlackList/  # 黑名单管理
│       │   ├── CacheManager/ # 缓存管理
│       │   ├── Client/     # 客户端选择
│       │   ├── Content/    # 歌词内容控制
│       │   └── TextStyle/  # 文字样式（颜色、字体）
│       └── index.tsx       # 控制器主框架 + 导航
├── routes/                # 路由定义
├── services/
│   ├── configService.ts   # REST 配置存取
│   ├── webSocketService.ts # WebSocket 服务
│   └── managers/
│       ├── tosuManager.ts  # tosu 连接 + 歌词获取调度
│       └── lyricManager.ts # 歌词数据模型
├── stores/
│   │
├── indexStore│├── settingsStore│├── blacklistStore│   
├── types/│    
│   ├── globalTypes│    ├──lyricTypes│├── tosuTypes│  ├── wsTypes│    └── wsLyricTypes│    
│── utils/
└──── cache│(IndexedDB)── fonts.ts── helpers── parseLyrics│(LRC 解析)── parseParams│── request│(HTTP│代理请求)
```

### 1.3 数据流（旧版）

```
tosu WebSocket ──► tosuManager ──► LyricManager ──► LyricsBox（展示）
                       │
                       ├──► Adapters（QQ/网易云）──► HTTP 代理 ──► 第三方 API
                       │
                       └──► IndexedDB（缓存读写）

Controller ──► webSocketService ◄──► 其他客户端（OBS 端）
```

### 1.4 功能清单（旧版已完成）

- [x] 歌词展示（主歌词 + 翻译歌词）
- [x] 时间轴滚动、歌词闪烁
- [x] 文字色彩（主/副歌词独立色彩）
- [x] 翻译作为主歌词显示
- [x] 显示/隐藏副歌词
- [x] 左对齐 / 居中 / 右对齐
- [x] 夜间模式
- [x] 控制面板（Ctrl+Alt+T / 三指触摸 切换）
- [x] 歌曲黑名单管理
- [x] 查看各源搜索结果
- [x] 指定歌曲换源
- [x] 歌词缓存管理（增删）
- [x] 客户端选择与测试

---

## 二、新版后端能力

### 2.1 技术栈

| 项目 | 技术 |
|------|------|
| 语言 | Rust (edition 2024) |
| 异步运行时 | Tokio |
| 歌词处理 | lyrics_helper_rs 0.2.1 |
| 日志 | tracing |

### 2.2 已实现/待实现

- [x] 接入 tosu 获取当前歌曲
- [x] QQ / 网易云歌词搜索
- [x] 歌词缓存（增删、过期）
- [x] 持久化配置
- [x] 字体存储
- [x] 上传歌词
- [ ] **WebSocket 服务器**（核心待开发）
  - [x] 时间轴调整
  - [x] 歌曲更新推送
  - [x] 样式更新推送
  - [x] 拉黑/显示
  - [ ] 歌词换行指令
- [ ] HTTP API（配置存取等）

### 2.3 新版数据流（预期）

```
tosu ──► lyric-server（Rust） ──WebSocket──► 前端 LyricsBox（展示）
              │                                 │
              ├── QQ/网易云 API                  └── Controller（控制面板）
              ├── 歌词缓存（服务端）                │
              ├── 配置持久化（服务端）              └── WebSocket ──► lyric-server
              └── 字体存储（服务端）
```

---

## 三、可复用部分

以下旧版组件和代码**可以直接复用或少量修改后复用**：

### 3.1 界面（UI）— 几乎全部复用

| 类别 | 文件/组件 | 复用程度 | 说明 |
|------|-----------|----------|------|
| 歌词展示 | `pages/LyricsBox/index.tsx` | **高度复用** | 核心展示逻辑（滚动、对齐、字体）不变，仅数据来源从 tosuManager 变为 WebSocket |
| 控制面板框架 | `pages/Controller/index.tsx` | **高度复用** | 导航结构、暗夜模式切换、Mask 遮罩逻辑不变 |
| UI 组件库 | `components/ui/*` | **完全复用** | Button, Select, ToggleSwitch, DragPanel, Mask, DarkModeToggle, CustomColorSelector, Upload, ToggleList |
| 图标库 | `assets/Icons/*` | **完全复用** | 所有 SVG 图标 |
| 样式 | `index.css` + TailwindCSS | **完全复用** | 所有样式类和暗夜模式 |
| 客户端选择 | `ControlTools/Client/` | **高度复用** | 逻辑基本不变 |
| 黑名单管理 | `ControlTools/BlackList/` | **高度复用** | 表格、表单逻辑不变 |
| 歌词内容控制 | `ControlTools/Content/` | **中度复用** | 数据获取从 WS query 变为直接 WS 推送 |
| 文字样式 | `ControlTools/TextStyle/` | **高度复用** | 颜色选择器、对齐切换、开关组件不变 |
| 缓存管理 | `ControlTools/CacheManager/` | **中度复用** | 缓存操作从 IndexedDB 查询变为 WS 指令 |
| 路由结构 | `routes/index│高度复用** | Ctrl│Alt│T│三│指切换逻辑保留 |
| 
| Stores | `stores/settingsStore│` | **高度复用** | SolidJS signals│font, textColor, alignment 等 |

### ** 3│2 数据层 — │部分复用**

| 类别 | 文件 | 复用程度 │ 说明 |
|------|-----------|----------|------|
| 歌词数据模型 │ `services/managers│lyricManager│` | **高度复用** | Lyric 类（insert, jump│ nextTime）逻辑不变 |
| LRC │解析 | `utils│parseLyrics│` | **完全复用** │ │纯文本解析│无依赖 |
| 字体加载 | `utils│fonts│` | **完全复用** │ 本地字体文件加载 |
| 工具函数 | `utils│helpers│` | **完全复用** | generateRandomString, debounce, ms│str 等 |
| 类型定义 | `types/globalTypes│` │ │**高度复用** │ AlignType, Settings, BlacklistItem 等 |
│ 歌词类型 │ `types│lyricTypes│` | **高度复用** │ LyricRawLine, LyricLine, MusicInfo 等 |

### 3.3 需要重写的部分

| 类别 | 文件 | 原因 |
|------|------|------|
| tosu │连接 | `services/managers│tosuManager│` | 后端接管 tosu 连接 |
| 歌词适配器 | `adapters│*` | 后端接管歌词搜索 |
│ IndexedDB 缓存 │ `utils│cache│` | 后端接管缓存 |
│ WebSocket │服务 | `services│webSocketService│` | 协议重新设计，前端角色从│peer│变为 client│ 
│WebSocket │消息类型 | `types│wsTypes│` | 消息格式变更 |
│初始化入口 | `hooks/initializeApp│` | 注册│handler 需要调整 |
│ 配置存取 │ `services│configService│` │ 配置改由 WS 下发│不再直接 fetch |
│ 请求工具 │ `utils│request│` | 不再需要前端代理第三方 API 请求 |

---

## 四、WebSocket 协议

协议已在 `tosu-proxy/README.md` 中完整定义，前端也有对应的类型实现。

### 4.1 协议要点

- 两层连接模型：
  - `ws://host/api/ws` → 歌词接收端（OBS 展示页），接收歌词推送 + 设置广播
  - `ws://host/api/ws?setter=true` → 配置发送端（控制面板），不接收广播，发送设置后收响应
- 消息类型：
  - Lyric 事件（`type: "lyric"`）— 后端 → 前端单向推送
  - Setting 事件（`type: "setting"`）— 双向，含提交/广播/响应三种模式，通过 `key` 区分操作，`echo` 字段匹配请求-响应

### 4.2 已定义的事件列表

见 `tosu-proxy/README.md` 设置事件子列表（137 行附近），共 19 个 key，覆盖：

- 歌词推送（lyric + current + nextTime + sequence）
- 样式设置（字体、字号、颜色、对齐、翻译偏好、副歌词显示）
- 歌词换源（setLyricSource）
- 搜索结果查询（getLyricList）
- 完整歌词获取（getAllLyric）
- 黑名单（setBlock / setUnblock / getBlockList）
- 缓存（getCacheCount / setCacheClean）
- 歌词偏移（getLyricOffset / setLyricOffset）

### 4.3 前端已有实现

| 文件 | 角色 |
|------|------|
| `api/model.ts` | 协议类型定义（LyricLine, WebsocketLyric, WebsocketSetting, WebsocketSettingTypeMap 等），与 README 对齐 |
| `api/websocket.ts` | Setting 消息的 send/await 封装（setFont, getFont, setColor 等），基于 reconnecting-websocket |

这两个文件已实现了 19 个 setting key 的封装，但尚未与 LyricsBox / Controller 页面接通。

### 4.4 协议待补充项

| gap | 说明 | 影响页面 |
|-----|------|----------|
| 多客户端支持 | 缺少连接时 ID 分配、在线/离线通知、消息 target 路由 | ClientList（客户端选择） |
| 缓存列表分页 | 只有 getCacheCount + setCacheClean，缺少 getCacheList、removeCacheItem | CacheManager |
| 黑名单任意曲目 | setBlock/setUnblock 只能操作当前歌曲，缺少按 ID 增删任意项 | BlackList |
| 按 key 查歌词 | 缺少 getLyricByKey {provider, key}，搜索结果中预览歌词需要 | Content / SearchResult |
| 客户端闪烁测试 | 缺少 blink 指令 | ClientList "测试" 按钮 |
| 歌曲元信息 | Lyric 首次推送缺少 bid / title / artist | LyricsBox、Content |
| 配置持久化 | 无 getConfig / setConfig，也无 HTTP /api/config | initializeApp |

---

## 五、第二代前端任务列表

### Phase 1: 基础架构搭建

- [ ] **P1-1** 补充协议缺失项（与后端协商）
  - Lyric 事件增加 `bid`、`title`、`artist` 字段
  - 增加多客户端生命周期事件（online/offline）
  - Setting 消息增加 `target` 字段用于客户端路由
  - 补充缓存列表、黑名单增删、按 key 查歌词、blink 等 key

- [ ] **P1-2** 重写 `services/webSocketService.ts`
  - 从旧的 peer-to-peer 协议切换到 `api/model.ts` 的扁平 type 协议
  - 保留 registerHandler / registerQuery 模式，增加在线客户端管理
  - 歌词推送直接写入 store signals

- [ ] **P1-3** 重构 `initializeApp.ts`
  - 注册新版 WS handler（歌词 → LyricsBox，设置 → settingsStore，在线状态 → ClientList）
  - 接入配置获取（WS getConfig 或 HTTP /api/config）

- [ ] **P1-4** 清理旧数据层
  - 移除 tosuManager、adapters/*、IndexedDB 缓存（utils/cache.ts）
  - 保留 LyricManager（本地时间轴 jump/nextTime 计算）

### Phase 2: 核心功能对接

- [ ] **P2-1** 歌词展示对接
  - 后端 WebSocket 推送歌词 → 直接更新 LyricsBox 的 lyrics/cursor 信号
  - 移除去 tosuManager 的依赖
  - 保留滚动、对齐、字体、闪烁等展示效果

- [ ] **P2-2** 控制面板对接
  - 文字颜色 → WS push 到后端（或后端主动推送）
  - 对齐方式 → WS push
  - 翻译设置 → WS push
  - 副歌词显隐 → WS push

- [ ] **P2-3** 歌词内容控制对接
  - 当前歌词查看 → WS query
  - 搜索结果查看 → WS query（或后端主动推送搜索结果列表）
  - 换源操作 → WS push

- [ ] **P2-4** 黑名单对接
  - 列表查询 → WS query 或后端主动推送
  - 增删改 → WS push
  - 本地 store 同步

- [ ] **P2-5** 缓存管理对接
  - 缓存列表查询 → WS query
  - 删除缓存项 → WS push
  - 清空缓存 → WS push

### Phase 3: 体验完善

- [ ] **P3-1** 错误处理
  - WebSocket 断线重连提示
  - 后端不可用时降级 UI
  - 歌词获取失败的空状态

- [ ] **P3-2** 客户端选择优化
  - 首次连接自动选择
  - 客户端离线检测

- [ ] **P3-3** 未完成功能（来自旧版 TODO）
  - [ ] 歌词偏移微调
  - [ ] 歌词阴影修改
  - [ ] 主歌词/翻译歌词独立字体
  - [ ] 手动上传歌词
  - [ ] 黑名单 UI 更新修复

---

## 六、开发路线图

```
Phase 1（基础架构）
├── 补充协议缺失项（4.4 节列出的 gap）
├── 重写 webSocketService → 接入 api/model.ts 协议
├── 重构初始化流程
└── 清理旧数据层（tosuManager、adapters、cache）
      │
      ▼
Phase 2（核心功能对接）
├── 歌词展示 ← WebSocket 推送
├── 控制面板 ← WebSocket 双向通信
├── 歌词内容、换源
├── 黑名单
└── 缓存管理
      │
      ▼
Phase 3（体验完善）
├── 错误处理、断线重连
├── 空状态、加载态 UI
└── 旧版 TODO 遗留功能
```

### 建议开发顺序

1. **补充 WebSocket 协议缺失项**（前后端协商 4.4 节 gap）— 协议主体已定义，只需补漏
2. **重写 WebSocket 服务层**（切换到 `api/model.ts` 协议，接入歌词推送）
3. **对接歌词展示**（最核心功能，最早验证）
4. **对接控制面板**（逐个子面板接入）
5. **清理旧代码**（移除 tosuManager、adapters、cache 等）

---

## 七、技术决策建议

| 决策点 | 建议 | 理由 |
|--------|------|------|
| 框架 | 继续使用 SolidJS | 改动最小，团队熟悉 |
| 样式 | 继续使用 TailwindCSS 4 | 旧版样式直接复用 |
| 构建 | 继续使用 Vite | 无需变更 |
| 缓存位置 | 后端管理，前端不缓存 | 架构原则：后端处理数据 |
| 配置存取 | WebSocket 同步 + 后端持久化 | 减少 HTTP 端点，统一通信通道 |
| 歌词解析 | 后端解析好推送，还是前端解析？ | **后端解析**（推送结构化数据），前端只展示；Lyric 类的 jump/insert 可保留用于本地时间轴计算 |
| 多客户端支持 | 保留现有 client 选择机制 | 控制器需要知道控制哪个 OBS 端 |

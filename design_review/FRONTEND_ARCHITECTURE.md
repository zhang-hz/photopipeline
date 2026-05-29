# Photopipeline 前端架构文档

> **版本**: 2.0  
> **状态**: 详细设计  
> **最后更新**: 2026-05-29  
> **技术栈**: Tauri v2 + React 19 + @fluentui/react-components v9 + window-vibrancy  
> **设计基准**: design_review/SPEC_*.md, design_review/*.html  
> **后端对齐**: feat/unified-binary doc/ARCHITECTURE_DESIGN.md

---

## 目录

### 第一部分: 架构概览
1. [系统分层架构](#1-系统分层架构)
2. [关键架构决策](#2-关键架构决策)
3. [依赖关系与版本](#3-依赖关系与版本)

### 第二部分: 项目结构
4. [完整目录树](#4-完整目录树)
5. [文件职责说明](#5-文件职责说明)

### 第三部分: 设计系统
6. [设计令牌系统](#6-设计令牌系统)
7. [控件库](#7-控件库)
8. [按钮库](#8-按钮库)
9. [卡片与容器](#9-卡片与容器)
10. [状态指示器](#10-状态指示器)
11. [辅助视图系统](#11-辅助视图系统)
12. [上下文菜单系统](#12-上下文菜单系统)

### 第四部分: 面板与视图
13. [面板体系总览](#13-面板体系总览)
14. [TitleBar 详细设计](#14-titlebar-详细设计)
15. [Sidebar 详细设计](#15-sidebar-详细设计)
16. [Content (DAG画布) 详细设计](#16-content-dag画布-详细设计)
17. [Panel (插件控制面板) 详细设计](#17-panel-插件控制面板-详细设计)
18. [BatchMode 详细设计](#18-batchmode-详细设计)
19. [StatusBar 详细设计](#19-statusbar-详细设计)
20. [SettingsDialog 详细设计](#20-settingsdialog-详细设计)

### 第五部分: 覆盖标记系统
21. [四级覆盖层级](#21-四级覆盖层级)
22. [ContextBar 组件](#22-contextbar-组件)
23. [OverrideDot 组件](#23-overridedot-组件)
24. [ExpressionEditor 组件](#24-expressioneditor-组件)
25. [多选覆盖处理](#25-多选覆盖处理)

### 第六部分: 状态管理
26. [Zustand 架构设计](#26-zustand-架构设计)
27. [useAppStore](#27-useappstore)
28. [useFilmstripStore](#28-usefilmstripstore)
29. [usePluginStore](#29-usepluginstore)
30. [usePipelineStore](#30-usepipelinestore)
31. [useOverrideStore](#31-useoverridestore)
32. [useBatchStore](#32-usebatchstore)
33. [useSettingsStore](#33-usesettingsstore)
34. [跨 Store 交互协议](#34-跨-store-交互协议)

### 第七部分: 通信架构
35. [通信分层模型](#35-通信分层模型)
36. [后端进程管理](#36-后端进程管理)
37. [Tauri Commands 详细定义](#37-tauri-commands-详细定义)
38. [Tauri Events 详细定义](#38-tauri-events-详细定义)
39. [前端 Hooks 封装](#39-前端-hooks-封装)

### 第八部分: 数据流
40. [应用启动流](#40-应用启动流)
41. [图片导入流](#41-图片导入流)
42. [DAG 编辑流](#42-dag-编辑流)
43. [参数编辑流](#43-参数编辑流)
44. [管线执行流](#44-管线执行流)
45. [批量处理流](#45-批量处理流)
46. [设置持久化流](#46-设置持久化流)

### 第九部分: 状态覆盖
47. [全局状态矩阵](#47-全局状态矩阵)
48. [各面板空态设计](#48-各面板空态设计)
49. [各面板加载态设计](#49-各面板加载态设计)
50. [各面板错误态设计](#50-各面板错误态设计)

### 第十部分: 交互模式
51. [选择交互规范](#51-选择交互规范)
52. [DAG 交互规范](#52-dag-交互规范)
53. [批量交互规范](#53-批量交互规范)
54. [拖放交互规范](#54-拖放交互规范)
55. [键盘快捷键完整表](#55-键盘快捷键完整表)

### 第十一部分: 开发实施
56. [开发阶段规划](#56-开发阶段规划)
57. [组件实施优先级](#57-组件实施优先级)
58. [性能策略](#58-性能策略)

---

# 第一部分: 架构概览

## 1. 系统分层架构

### 1.1 三层进程架构

Photopipeline 采用严格的三层进程分离架构:

```
┌────────────────────────────────────────────────────────────────────┐
│                    GUI 进程 (Tauri v2)                               │
│                                                                     │
│  ┌─ WebView 层 (React 19) ──────────────────────────────────────┐  │
│  │                                                               │  │
│  │  FluentProvider(webDarkTheme)                                 │  │
│  │    └─ App                                                     │  │
│  │         ├─ TitleBar (44px, 可拖拽)                             │  │
│  │         ├─ ModeSwitch ┐                                       │  │
│  │         │   ├─ EditMode  ─ MainLayout ─ 三栏                  │  │
│  │         │   └─ BatchMode ─ BatchLayout ─ 三栏                  │  │
│  │         ├─ StatusBar (36px)                                   │  │
│  │         └─ Modals (SettingsDialog, ConfirmDialog)             │  │
│  │                                                               │  │
│  │  通信: invoke("cmd", args) / listen("event", handler)         │  │
│  └───────────────────────────┬───────────────────────────────────┘  │
│                              │                                      │
│  ┌─ Rust 层 (Tauri Commands + gRPC Client) ─────────────────────┐  │
│  │                                                               │  │
│  │  Commands (13 个 #[tauri::command])                           │  │
│  │    ├─ plugin_cmds:  list_plugins, get_node_schema             │  │
│  │    ├─ image_cmds:   load_images, get_thumbnail, decode_preview│  │
│  │    ├─ pipeline_cmds:save, load, validate, execute             │  │
│  │    ├─ batch_cmds:   start_batch, cancel_batch                 │  │
│  │    └─ settings_cmds:save_settings, load_settings              │  │
│  │                                                               │  │
│  │  Events (6 个 app.emit 事件)                                  │  │
│  │    ├─ pipeline-progress, pipeline-stage                       │  │
│  │    ├─ pipeline-error, pipeline-done                           │  │
│  │    ├─ batch-progress, backend-status                          │  │
│  │                                                               │  │
│  │  gRPC Clients (tonic, 4 个 Service Client)                    │  │
│  │    ├─ PluginServiceClient                                     │  │
│  │    ├─ ImageServiceClient                                      │  │
│  │    ├─ PipelineServiceClient                                   │  │
│  │    └─ BatchServiceClient                                      │  │
│  │                                                               │  │
│  │  通信: tonic gRPC → localhost:50051                           │  │
│  └───────────────────────────┬───────────────────────────────────┘  │
│                              │                                      │
│  ┌─ 进程管理 ─────────────────────────────────────────────────┐   │
│  │  backend.rs: spawn / health_check / restart / kill           │   │
│  └───────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘
         │ gRPC over localhost:50051
         ▼
┌────────────────────────────────────────────────────────────────────┐
│                    Server 进程 (photopipeline serve)                 │
│                                                                     │
│  tonic gRPC Server (tonic 0.13)                                    │
│    ├─ PluginService  ─→ Registry::query()                          │
│    ├─ ImageService   ─→ OIIO / libheif / libjxl / libraw           │
│    ├─ PipelineService─→ PipelineExecutor + ParameterResolver       │
│    └─ BatchService   ─→ BatchScheduler                             │
│                                                                     │
│  Engine 层                                                          │
│    ├─ PipelineGraph (DAG 拓扑)                                     │
│    ├─ NodeExecutor (逐节点执行)                                     │
│    ├─ ParameterResolver (四级参数合并)                               │
│    └─ TileEngine (大图分块处理)                                     │
│                                                                     │
│  Plugin 层                                                          │
│    ├─ Plugin trait (基础接口)                                       │
│    ├─ Registry (插件注册表)                                         │
│    └─ 14 个内置插件                                                 │
│                                                                     │
│  Core 层                                                            │
│    ├─ PixelBuffer, AlignedBuffer                                   │
│    ├─ ColorSpace, TransferFunction                                  │
│    └─ Metadata, ExifData, GpsData                                  │
└────────────────────────────────────────────────────────────────────┘
```

### 1.2 层间依赖方向

依赖方向严格单向: **上层依赖下层, 下层绝不依赖上层**

```
GUI (React) → Tauri Rust (gRPC Client) → Server (Engine → Plugin → Core)
```

- React 层不直接调用 gRPC, 通过 Tauri invoke 桥接
- Tauri Rust 层不包含引擎逻辑, 仅作为 gRPC 客户端
- Server 层不感知 GUI 技术栈, 任何 gRPC 客户端均可接入
- Core 层不依赖任何其他层

### 1.3 数据通道设计

与后端架构文档 §2.3 对齐, 系统使用两条独立的数据通道:

| 通道 | 协议 | 数据类型 | 触发方式 | 生命周期 |
|------|------|---------|---------|---------|
| **文件系统** | JSON 文件 | PipelineConfig (管线定义, 参数覆盖, 图片路径, 分组规则) | 用户手动保存/加载 | 持久化, 可版本控制 |
| **gRPC** | protobuf (localhost:50051) | 插件目录, Schema, 图片元数据/缩略图/像素, 执行进度流, 批量进度流 | 用户操作触发 / 系统自动轮询 | 瞬时, 会话级 |

**设计理由**:
- 管线定义是持久资产, 通过文件共享可实现版本控制、团队协作、CLI/GUI 互操作
- gRPC 负责短暂性实时数据, 适合流式推送和即时查询
- CLI 模式 (`photopipeline run config.json`) 不依赖 gRPC, 简化 CI/CD
- 消除"前后端管线状态不同步"这一常见 Bug 来源

---

## 2. 关键架构决策

### 2.1 前后端分离

| 方面 | 决策 | 理由 |
|------|------|------|
| 进程模型 | GUI 和 Server 各自独立进程 | Server 可独立升级、独立调试、CLI 可脱离 GUI 运行 |
| 通信协议 | gRPC (localhost) | 强类型接口、流式支持、双向取消、生态成熟 |
| 端口 | 50051 (可配置) | 标准 gRPC 端口, 避免冲突 |
| 后端发现 | GUI spawn 子进程 | 用户无需手动启动后端, 一键启动 |

### 2.2 Schema 驱动 UI

| 方面 | 决策 | 理由 |
|------|------|------|
| 参数定义 | 后端定义 ParameterSchema (值语义) | 后端负责验证, 前端负责渲染 |
| 布局提示 | 后端定义 GuiSchema (布局提示) | 后端提供建议, 前端自主决定控件选择 |
| 控件选择 | 前端根据 ValueType 自行选择 | 关注点分离, 后端不依赖 GUI 框架 |
| 扩展性 | 新增插件只需实现 Plugin trait | 前端零改动即可渲染新插件参数面板 |

### 2.3 状态管理

| 方面 | 决策 | 理由 |
|------|------|------|
| 方案 | Zustand v5 | 轻量、无 boilerplate、支持 selector、支持 middleware |
| Store 拆分 | 7 个独立 Store 按域拆分 | 单一职责, 减少不必要重渲染 |
| 跨 Store 通信 | 直接调用 `useXxxStore.getState().action()` | 简单直接, 避免事件总线复杂度 |
| 持久化 | 仅 Settings 持久化到文件 | 其他状态为会话级, 通过 PipelineConfig 文件恢复 |

### 2.4 DAG 渲染

| 方面 | 决策 | 理由 |
|------|------|------|
| 节点 | React DOM 组件 (绝对定位 div) | 完整 Fluent 2 样式、原生 a11y、React DevTools 可调试 |
| 连线 | SVG `<path>` 贝塞尔曲线 | 矢量缩放、hit-testing (透明宽 stroke)、CSS transition |
| 迷你地图 | Canvas 2D | 高性能缩略渲染, 不需要交互 |
| 网格 | CSS `background-image` linear-gradient | 纯 CSS, 无 DOM 开销 |
| 变换 | CSS `transform: scale() translate()` | GPU 加速, 平滑过渡 |

---

## 3. 依赖关系与版本

### 3.1 前端依赖 (package.json)

| 包 | 版本 | 用途 |
|---|------|------|
| react | ^19.0 | UI 框架 |
| react-dom | ^19.0 | DOM 渲染 |
| @fluentui/react-components | ^9.x | Fluent 2 组件库 (Provider, Button, Input, Dropdown, Switch, Slider, Dialog, TabList, Badge, Spinner, Tooltip, Card 等) |
| @tauri-apps/api | ^2.x | Tauri invoke, listen, dialog, fs, shell |
| zustand | ^5.x | 状态管理 |
| @tanstack/react-virtual | ^3.x | 虚拟列表 |
| leaflet | ^1.x | 地图辅助视图 (gps_set) |
| vite | ^6.x | 构建工具 |
| typescript | ^5.x | 类型系统 |

### 3.2 Rust 依赖 (Cargo.toml)

| crate | 版本 | 用途 |
|-------|------|------|
| tauri | ^2.x | 桌面框架 |
| tauri-build | ^2.x | 构建脚本 |
| tonic | ^0.13 | gRPC 客户端 |
| prost | ^0.14 | protobuf 代码生成 |
| tokio | ^1.x | 异步运行时 |
| serde | ^1.x | 序列化 |
| serde_json | ^1.x | JSON 处理 |
| window-vibrancy | ^0.5 | 窗口透明效果 |

---

# 第二部分: 项目结构

## 4. 完整目录树

```
gui/                                          # Tauri 应用根目录
│
├── Cargo.toml                                # [package] gui, workspace member
│                                             # deps: tauri, tonic, prost, tokio, serde
│                                             #       serde_json, window-vibrancy, uuid
│
├── tauri.conf.json                           # Tauri v2 核心配置
│                                             # - app.windows[0]: decorations=false,
│                                             #   transparent=true, 1500×920, min 1280×720
│                                             # - build: frontendDist="../dist"
│                                             # - security.csp: 内容安全策略
│
├── capabilities/                             # Tauri v2 权限声明
│   └── default.json                          # permissions: ["core:default",
│                                             #   "fs:read-all", "fs:write-all",
│                                             #   "dialog:allow-open", "shell:allow-spawn"]
│
├── icons/                                    # 多平台应用图标
│   ├── icon.png                              # 1024×1024 源图
│   ├── icon.ico                              # Windows
│   ├── icon.icns                             # macOS
│   └── 32x32.png, 128x128.png, ...           # 各尺寸
│
├── src-tauri/                                # ═══ Rust 端 ═══
│   ├── Cargo.toml                            # [lib] + [bin], tonic-build 编译 proto
│   │
│   ├── build.rs                              # tonic_build::compile_protos("proto/*.proto")
│   │
│   └── src/
│       ├── main.rs                           # fn main(): 解析 CLI 参数, tauri::Builder
│       │                                     #   .setup(|app| { init_vibrancy(); spawn_backend(); })
│       │                                     #   .invoke_handler(tauri::generate_handler![...])
│       │                                     #   .run(tauri::generate_context!())?;
│       │
│       ├── lib.rs                            # pub mod commands; pub mod grpc; pub mod events;
│       │                                     # pub mod backend; pub mod types;
│       │                                     # pub struct AppState { grpc_clients, settings_path }
│       │
│       ├── backend.rs                        # ═══ 后端进程管理 ═══
│       │                                     # pub struct BackendProcess { child: Option<Child> }
│       │                                     # impl BackendProcess {
│       │                                     #   async fn start(port: u16) -> Result<()>
│       │                                     #     → Command::new("photopipeline")
│       │                                     #         .args(["serve", "--port", &port.to_string()])
│       │                                     #         .spawn()
│       │                                     #     → health_check loop (10 retries × 500ms)
│       │                                     #   async fn health_check(port: u16) -> bool
│       │                                     #   async fn stop() → SIGTERM → 3s → SIGKILL
│       │                                     #   async fn restart(port: u16) → stop() → start()
│       │                                     # }
│       │
│       ├── grpc/                             # ═══ gRPC 客户端封装 ═══
│       │   ├── mod.rs                        # pub struct GrpcClients {
│       │   │                                 #   pub plugin: PluginServiceClient<Channel>,
│       │   │                                 #   pub image: ImageServiceClient<Channel>,
│       │   │                                 #   pub pipeline: PipelineServiceClient<Channel>,
│       │   │                                 #   pub batch: BatchServiceClient<Channel>,
│       │   │                                 # }
│       │   │                                 # impl GrpcClients {
│       │   │                                 #   async fn connect(addr: &str) -> Result<Self>
│       │   │                                 # }
│       │   │
│       │   ├── plugin_client.rs              # PluginServiceClient 的包装方法:
│       │   │                                 #   async fn list_plugins(&mut self) -> Result<PluginCatalogResponse>
│       │   │                                 #   async fn get_node_schema(&mut self, id: &str) -> Result<NodeSchemaResponse>
│       │   │
│       │   ├── image_client.rs               # ImageServiceClient 的包装方法:
│       │   │                                 #   async fn load(&mut self, path: &str) -> Result<ImageInfo>
│       │   │                                 #   async fn get_thumbnail(&mut self, path: &str, size: u32) -> Result<ImageData>
│       │   │                                 #   async fn decode(&mut self, req: DecodeRequest) -> Result<Vec<u8>>
│       │   │
│       │   ├── pipeline_client.rs            # PipelineServiceClient 的包装方法:
│       │   │                                 #   async fn create_pipeline(&mut self, spec: PipelineSpec) -> Result<PipelineId>
│       │   │                                 #   async fn execute(&mut self, req: ExecuteRequest) -> Result<Streaming<ExecuteProgress>>
│       │   │                                 #   async fn validate(&mut self, spec: PipelineSpec) -> Result<ValidationResult>
│       │   │
│       │   └── batch_client.rs               # BatchServiceClient 的包装方法:
│       │       #   async fn submit_batch(&mut self, spec: BatchSpec) -> Result<BatchId>
│       │       #   async fn get_progress(&mut self, id: &str) -> Result<Streaming<BatchProgress>>
│       │       #   async fn cancel(&mut self, id: &str) -> Result<()>
│       │
│       ├── commands/                         # ═══ Tauri Commands ═══
│       │   ├── mod.rs                        # pub use plugin_cmds::*; pub use image_cmds::*; ...
│       │   │
│       │   ├── plugin_cmds.rs                # #[tauri::command]
│       │   │                                 # async fn list_plugins(state: State<AppState>) -> Result<Vec<PluginEntry>, String>
│       │   │                                 #   → state.grpc.plugin.lock().list_plugins().await
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn get_node_schema(state: State<AppState>, plugin_id: String) -> Result<NodeSchemaResponse, String>
│       │   │                                 #   → state.grpc.plugin.lock().get_node_schema(&plugin_id).await
│       │   │
│       │   ├── image_cmds.rs                 # #[tauri::command]
│       │   │                                 # async fn load_images(state: State<AppState>, paths: Vec<String>) -> Result<Vec<ImageInfo>, String>
│       │   │                                 #   → for path in paths: state.grpc.image.lock().load(&path).await → collect
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn get_thumbnail(state: State<AppState>, path: String, max_size: u32) -> Result<ImageData, String>
│       │   │                                 #   → state.grpc.image.lock().get_thumbnail(&path, max_size).await
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn decode_preview(state: State<AppState>, path: String, max_w: u32, max_h: u32) -> Result<Vec<u8>, String>
│       │   │                                 #   → state.grpc.image.lock().decode(DecodeRequest{...}).await
│       │   │                                 #   → 收集所有 PixelDataChunk, 合并返回
│       │   │
│       │   ├── pipeline_cmds.rs              # #[tauri::command]
│       │   │                                 # async fn save_pipeline_file(path: String, json: String) -> Result<(), String>
│       │   │                                 #   → tokio::fs::write(path, json).await
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn load_pipeline_file(path: String) -> Result<String, String>
│       │   │                                 #   → tokio::fs::read_to_string(path).await
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn validate_pipeline(state: State<AppState>, json: String) -> Result<ValidationResult, String>
│       │   │                                 #   → state.grpc.pipeline.lock().validate(serde_json::from_str(&json)?).await
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn execute_pipeline(state: State<AppState>, config_json: String, app: AppHandle) -> Result<String, String>
│       │   │                                 #   → 1. 保存 JSON 到临时文件
│       │   │                                 #   → 2. grpc.pipeline.create_pipeline(spec).await → pid
│       │   │                                 #   → 3. tokio::spawn(gRPC execute stream → app.emit events)
│       │   │                                 #   → 4. 立即返回 pid
│       │   │
│       │   ├── batch_cmds.rs                # #[tauri::command]
│       │   │                                 # async fn start_batch(state: State<AppState>, config_json: String, app: AppHandle) -> Result<String, String>
│       │   │                                 #   → 类似 execute_pipeline, 但调用 BatchService
│       │   │                                 #
│       │   │                                 # #[tauri::command]
│       │   │                                 # async fn cancel_batch(state: State<AppState>, batch_id: String) -> Result<(), String>
│       │   │                                 #   → state.grpc.batch.lock().cancel(&batch_id).await
│       │   │
│       │   └── settings_cmds.rs              # #[tauri::command]
│       │       # async fn save_settings(settings: AppSettings) -> Result<(), String>
│       │       # async fn load_settings() -> Result<AppSettings, String>
│       │       #   → 读写 %APPDATA%/Photopipeline/appsettings.json
│       │
│       ├── events.rs                         # ═══ Event 发射器 ═══
│       │                                     # pub fn emit_pipeline_progress(app: &AppHandle, p: ExecuteProgress)
│       │                                     # pub fn emit_pipeline_done(app: &AppHandle, d: DoneEvent)
│       │                                     # pub fn emit_batch_progress(app: &AppHandle, p: BatchProgress)
│       │                                     # pub fn emit_backend_status(app: &AppHandle, s: BackendStatus)
│       │
│       └── types.rs                          # ═══ 共享类型 ═══
│           # Serde 类型, 与前端 types/ 目录对齐
│           # PluginEntry, NodeSchemaResponse, ImageInfo,
│           # PipelineSpec, ValidationResult, BatchSpec,
│           # ExecuteProgress, BatchProgress, AppSettings
│
└── src/                                      # ═══ React 前端 ═══
    ├── index.html                            # <div id="root">, <script type="module" src="/main.tsx">
    ├── main.tsx                              # createRoot(document.getElementById('root')!)
    │                                         #   .render(<StrictMode><App /></StrictMode>)
    ├── App.tsx                               # 根组件: FluentProvider → AppShell → mode 切换
    ├── vite.config.ts                        # Vite 配置: react plugin, resolve alias @/
    │
    ├── styles/                               # ═══ 样式 ═══
    │   ├── globals.css                       # Fluent 2 完整 CSS 自定义属性
    │   ├── vibrancy.css                      # 平台透明背景
    │   └── layout.css                        # 三栏网格 + TitleBar/StatusBar 固定布局
    │
    ├── stores/                               # ═══ Zustand (7 stores) ═══
    │   ├── useAppStore.ts
    │   ├── useFilmstripStore.ts
    │   ├── usePluginStore.ts
    │   ├── usePipelineStore.ts
    │   ├── useOverrideStore.ts
    │   ├── useBatchStore.ts
    │   └── useSettingsStore.ts
    │
    ├── hooks/                                # ═══ 自定义 Hooks ═══
    │   ├── useTauriCommand.ts                # 类型化 invoke 封装
    │   ├── useTauriEvent.ts                  # 类型化 listen 封装
    │   ├── useKeyboard.ts                    # 全局快捷键
    │   └── useContextMenu.ts                 # 右键菜单状态管理
    │
    ├── components/                           # ═══ React 组件 ═══
    │   ├── layout/                           # TitleBar, MainLayout, Sidebar, Content, Panel, StatusBar, ResizablePanel
    │   ├── filmstrip/                        # FilmstripList, ImageCard, GroupTree, EmptyState, DropZone, MultiSelectBar, SidebarHeader, SidebarToolbar, SortBar, GroupItem, GroupAdd
    │   ├── dag/                              # DAGCanvas, DAGNode, DAGEdge, DAGPort, MiniMap, DAGToolbar, ContentHeader, ContextMenu
    │   ├── panel/                            # ContextBar, PluginBrowser, PluginCard, PluginGrid, PluginSearch, ControlPanel, ParamSection, ParamRow, ExpressionEditor, AuxView, RemoveButton, PluginHeader
    │   ├── batch/                            # BatchMode, BatchLeftPanel, BatchCenterPanel, BatchRightPanel, BatchControls, BatchProgress, BatchQueueList, BatchQueueRow, OutputSettings, PerImageOverride
    │   ├── settings/                         # SettingsDialog, SettingsTabList, GeneralTab, BackendTab, OutputTab, AdvancedTab
    │   ├── preview/                          # PreviewView, PreviewToolbar
    │   └── common/                           # SliderWithInput, FilePathInput, ColorPicker, CoordinateInput, OverrideDot, Badge, ProgressBar, StatusDot, EmptyState as CommonEmptyState, ConfirmDialog, Spinner
    │
    ├── types/                                # ═══ TypeScript 类型 ═══
    │   ├── plugin.ts, pipeline.ts, image.ts
    │   ├── batch.ts, settings.ts, override.ts, events.ts
    │   └── index.ts                          # re-export all
    │
    └── utils/                                # ═══ 工具函数 ═══
        ├── bezier.ts                         # computeBezierPath(from, to) → SVG d string
        ├── cycleDetection.ts                 # hasCycle(nodes, edges, newEdge) → boolean (BFS)
        ├── expressionParser.ts               # parseExpression(expr, variables) → compute
        └── formatUtils.ts                    # formatFileSize, formatDuration, formatPercent
```

## 5. 文件职责说明

### 5.1 Rust 端文件

| 文件 | 职责 | 对外接口 |
|------|------|---------|
| `main.rs` | 应用入口, Tauri Builder 配置, command 注册 | `fn main()` |
| `lib.rs` | 模块声明, AppState 定义, setup hook | `pub struct AppState` |
| `backend.rs` | `photopipeline serve` 子进程生命周期管理 | `BackendProcess::start/stop/health_check/restart` |
| `grpc/mod.rs` | gRPC 客户端聚合, Channel 连接管理 | `GrpcClients::connect(addr)` |
| `grpc/plugin_client.rs` | PluginService gRPC 调用封装 | `list_plugins()`, `get_node_schema(id)` |
| `grpc/image_client.rs` | ImageService gRPC 调用封装 | `load(path)`, `get_thumbnail(path,size)`, `decode(req)` |
| `grpc/pipeline_client.rs` | PipelineService gRPC 调用封装 | `create_pipeline(spec)`, `execute(req)`, `validate(spec)` |
| `grpc/batch_client.rs` | BatchService gRPC 调用封装 | `submit_batch(spec)`, `get_progress(id)`, `cancel(id)` |
| `commands/plugin_cmds.rs` | 插件目录和 Schema 查询 | `list_plugins`, `get_node_schema` |
| `commands/image_cmds.rs` | 图片加载、缩略图、解码 | `load_images`, `get_thumbnail`, `decode_preview` |
| `commands/pipeline_cmds.rs` | 管线文件读写、验证、执行 | `save/load_pipeline_file`, `validate_pipeline`, `execute_pipeline` |
| `commands/batch_cmds.rs` | 批量任务提交和取消 | `start_batch`, `cancel_batch` |
| `commands/settings_cmds.rs` | 设置文件读写 | `save_settings`, `load_settings` |
| `events.rs` | gRPC stream → Tauri event 转换 | `emit_pipeline_progress`, `emit_batch_progress` 等 |
| `types.rs` | 与前端共享的 Serde 类型 | 所有数据传输对象 |

### 5.2 React 端核心文件

| 文件 | 职责 |
|------|------|
| `App.tsx` | 根组件: FluentProvider 包裹, mode 状态驱动视图切换, 全局事件注册 |
| `stores/*.ts` | 7 个 Zustand Store, 每个独立管理一个领域状态 |
| `hooks/useTauriCommand.ts` | `invoke<T>(cmd, args)` 的类型安全封装, 含 loading/error 状态 |
| `hooks/useTauriEvent.ts` | `listen<T>(event, handler)` 封装, 自动 cleanup |
| `types/*.ts` | 所有 TypeScript 接口定义, 与 Rust types.rs 对齐 |
| `utils/bezier.ts` | DAG 连线贝塞尔曲线路径计算 |
| `utils/cycleDetection.ts` | DAG 环检测算法 (BFS) |

---

# 第三部分: 设计系统

## 6. 设计令牌系统

### 6.1 颜色令牌

完整颜色系统, 共 14 个语义令牌, 覆盖所有 UI 元素:

#### 背景色

| 令牌 | 色值 | 用途 | 适用组件 |
|------|------|------|---------|
| `neutralBg1` | `#141414` | 最深背景 | Sidebar 背景, DAG 画布背景, 插件浏览器背景, 设置输入框背景 |
| `neutralBg2` | `#1f1f1f` | 中等背景 | Content 背景, Panel 背景, 卡片默认态, StatusBar, 对话框背景 |
| `neutralBg3` | `#292929` | Hover 态 | 卡片悬停, 按钮悬停, 菜单项悬停, 缩略图背景 |
| `neutralBg4` | `#333333` | 按压态/更亮hover | TitleBar 按钮悬停, 激活控件背景 |

#### 边框色

| 令牌 | 色值 | 用途 | 像素规格 |
|------|------|------|---------|
| `neutralStroke1` | `#383838` | 主分隔线 | 1px solid, TitleBar 底边, StatusBar 顶边, 卡片边框, ContextBar 底边 |
| `neutralStroke2` | `#4a4a4a` | 二级分隔线 | 1.5px solid, DAGNode 默认边框, Slider 轨道 |

#### 文字色

| 令牌 | 色值 | 字重/字号 | 用途 |
|------|------|----------|------|
| `neutralFg1` | `#f5f5f5` | 500-600 | 主文字: 文件名, 按钮文字, 选中菜单项 |
| `neutralFg2` | `#adadad` | 400 | 二级文字: 参数标签, 菜单项, 状态文字 |
| `neutralFg3` | `#6e6e6e` | 600, 10px uppercase | 三级文字: Section 标题, SidebarHeader |
| `neutralFg4` | `#505050` | 400, 9-10px | 辅助文字: 元数据, 描述, 禁用态, 文件大小 |

#### 语义色

| 令牌 | 色值 | 用途 | 透明度变体 |
|------|------|------|-----------|
| `brandFg1` | `#479ef5` | 品牌蓝 — 选中边框, 主按钮, 激活标签底边, 进度条填充, 端口颜色, 连线颜色 | `rgba(71,158,245,0.06)` 选中背景, `rgba(71,158,245,0.12)` 发光阴影 |
| `successFg` | `#54b054` | 成功绿 — 连接圆点, Done 状态, 节点 OK 灯 | — |
| `dangerFg` | `#d13447` | 危险红 — 删除按钮, 失败状态, 断开连接, 错误节点 | `rgba(209,52,71,0.08)` 删除hover背景 |
| `warningFg` | `#d59900` | 警告黄 — 覆盖圆点, 多选栏, 警告文字, 分组标签 | `rgba(213,153,0,0.06)` 多选栏背景, `rgba(213,153,0,0.08)` 标签背景 |
| `expressionFg` | `#b084f4` | 表达式紫 — 表达式编辑器强调 | `rgba(176,132,244,0.04)` 编辑器背景 |

### 6.2 圆角令牌

| 令牌 | 值 | 用途 | 示例组件 |
|------|:---:|------|---------|
| `radiusSmall` | 2px | 极小圆角 | Badge, Port, Slider track, 覆盖标记 |
| `radiusMedium` | 4px | 小圆角 | Button, Input, Dropdown, Switch track, 缩略图, Port |
| `radiusLarge` | 8px | 中圆角 | Card, ImageCard, PluginCard, DAGNode, DAG 画布, ContextMenu |
| `radiusXLarge` | 12px | 大圆角 | 窗口外框, 对话框, Sidebar 外框 |
| `radiusCircular` | 50% | 圆形 | Slider thumb, StatusDot, 覆盖圆点, 状态圆点 |

### 6.3 间距令牌

| 令牌 | 值 | 用途 |
|------|:---:|------|
| `spacingXS` | 4px | 极小间距: 按钮组 gap, PluginCard gap, Badge padding |
| `spacingS` | 8px | 小间距: Sidebar padding, 卡片 padding, 参数行 gap, ImageCard gap |
| `spacingM` | 12px | 中间距: Panel padding, TitleBar 子元素 gap, StatusBar 子元素 gap, 分区头部 padding |
| `spacingL` | 16px | 大间距: 面板 padding, 对话框 padding, 版本间距 |
| `spacingXL` | 20px | 很大间距: 表单间距 |
| `spacingXXL` | 24px | 特大间距: 内容区域 margin |

### 6.4 字体令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `fontFamily` | `'Segoe UI Variable', 'Segoe UI', system-ui, -apple-system, sans-serif` | 主字体 (Windows 11 动态字体优先) |
| `fontFamilyMono` | `'Cascadia Code', 'Consolas', 'SF Mono', 'Fira Code', monospace` | 等宽字体 (表达式, 数值, 代码) |
| `fontSizeCaption1` | 10px | 标题 (uppercase), 辅助文字, 元数据, StatusBar 指标 |
| `fontSizeBody1` | 12px | 正文, 文件名, 参数标签, 菜单项, 按钮文字 |
| `fontSizeBody2` | 13px | 大正文, 插件名 (Panel) |
| `fontSizeHeading` | 14px | 插件名 (Panel Header) |

### 6.5 布局尺寸令牌

| 令牌 | 值 | 用途 |
|------|:---:|------|
| `titleBarHeight` | 44px | 自定义标题栏高度 |
| `statusBarHeight` | 36px | 底部状态栏高度 |
| `sidebarWidth` | 272px | 左侧栏默认宽度 |
| `sidebarMinWidth` | 200px | 左侧栏最小宽度 (拖拽限制) |
| `panelWidth` | 440px | 右侧面板默认宽度 |
| `panelMinWidth` | 320px | 右侧面板最小宽度 (由 GuiSchema.min_panel_width 决定) |
| `contentMinWidth` | 480px | 中栏最小宽度 |
| `batchLeftWidth` | 260px | 批量模式左栏宽度 |
| `batchRightWidth` | 340px | 批量模式右栏宽度 |
| `windowBaseWidth` | 1500px | 基准窗口宽度 |
| `windowBaseHeight` | 920px | 基准窗口高度 |
| `windowMinWidth` | 1280px | 最小窗口宽度 |
| `windowMinHeight` | 720px | 最小窗口高度 |

### 6.6 插件色彩映射

14 个插件, 每个有唯一主题色, 用于 DAGNode 边框、PluginCard 强调、PluginHeader 图标背景:

| # | 插件 ID | 色名 | 色值 | rgba(背景) |
|:--|------|------|------|------|
| 1 | `raw_input` | Red | `#ef4444` | `rgba(239,68,68,0.12)` |
| 2 | `transform` | Cyan | `#06b6d4` | `rgba(6,182,212,0.12)` |
| 3 | `colorspace` | Purple | `#8b5cf6` | `rgba(139,92,246,0.12)` |
| 4 | `lut3d` | Pink | `#ec4899` | `rgba(236,72,153,0.12)` |
| 5 | `lens_correct` | Indigo | `#6366f1` | `rgba(99,102,241,0.12)` |
| 6 | `ai_denoise` | Violet | `#a855f7` | `rgba(168,85,247,0.12)` |
| 7 | `exif_rw` | Blue | `#3b82f6` | `rgba(59,130,246,0.12)` |
| 8 | `gps_set` | Green | `#10b981` | `rgba(16,185,129,0.12)` |
| 9 | `time_shift` | Amber | `#f59e0b` | `rgba(245,158,11,0.12)` |
| 10 | `heif_encoder` | Teal | `#14b8a6` | `rgba(20,184,166,0.12)` |
| 11 | `jxl_encoder` | Orange | `#f97316` | `rgba(249,115,22,0.12)` |
| 12 | `avif_encoder` | Emerald | `#22c55e` | `rgba(34,197,94,0.12)` |
| 13 | `tiff_encoder` | Slate | `#64748b` | `rgba(100,116,139,0.12)` |
| 14 | `png_encoder` | Sky | `#0ea5e9` | `rgba(14,165,233,0.12)` |

### 6.7 阴影令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `shadow2` | `0 1px 4px rgba(0,0,0,0.2)` | 微小阴影 (卡片) |
| `shadow4` | `0 2px 8px rgba(0,0,0,0.3)` | 小阴影 (DAGNode default) |
| `shadow8` | `0 4px 16px rgba(0,0,0,0.4)` | 中阴影 (DAGNode selected, 对话框) |
| `shadow16` | `0 8px 32px rgba(0,0,0,0.5)` | 大阴影 (窗口外框) |

### 6.8 动画令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `transitionFast` | `150ms ease` | 快速过渡: hover 颜色, OverrideDot 切换 |
| `transitionNormal` | `200ms ease` | 标准过渡: 折叠/展开, 选中态变化 |
| `transitionSlow` | `300ms ease` | 慢过渡: 进度条, 模式切换 |
| `pulseAnimation` | `pulse 1s ease-in-out infinite` | 脉冲: Processing 状态圆点 |

---

## 7. 控件库

### 7.1 控件总览

前端提供 **9 种标准参数控件**, 对应后端的 17 种 ParameterType。

每个控件通过 `ParamRow` 组件的 `renderControl()` dispatcher 选择:

```typescript
// panel/ParamRow.tsx
function renderControl(widget: PanelWidget, value: any, onChange: (v: any) => void): ReactNode {
  switch (widget.type) {
    case 'text_input':    return <TextInputControl widget={widget} value={value} onChange={onChange} />;
    case 'number_input':  return <NumberInputControl widget={widget} value={value} onChange={onChange} />;
    case 'slider':        return <SliderControl widget={widget} value={value} onChange={onChange} />;
    case 'toggle':        return <ToggleControl widget={widget} value={value} onChange={onChange} />;
    case 'dropdown':      return <DropdownControl widget={widget} value={value} onChange={onChange} />;
    case 'segmented_control': return <SegmentedControlControl widget={widget} value={value} onChange={onChange} />;
    case 'card_selector': return <CardSelectorControl widget={widget} value={value} onChange={onChange} />;
    case 'file_picker':   return <FilePickerControl widget={widget} value={value} onChange={onChange} />;
    case 'color_picker':  return <ColorPickerControl widget={widget} value={value} onChange={onChange} />;
    case 'coordinate_input': return <CoordinateInputControl widget={widget} value={value} onChange={onChange} />;
    case 'combo_slider':  return <ComboSliderControl widget={widget} value={value} onChange={onChange} />;
    case 'map_widget':    return <MapWidgetControl widget={widget} value={value} onChange={onChange} />;
    case 'expression_editor': return <ExpressionEditorControl widget={widget} value={value} onChange={onChange} />;
    case 'before_after_preview': return <BeforeAfterPreviewControl widget={widget} value={value} onChange={onChange} />;
    case 'nested_fields': return <NestedFieldsControl widget={widget} value={value} onChange={onChange} />;
    case 'label':         return <LabelControl widget={widget} />;
    default:              return <UnknownWidgetType type={widget.type} />;
  }
}
```

### 7.2 TextInput 控件

**对应 ParameterType**: `string`

**视觉规格**:
- 高度: 30px
- 背景: `neutralBg1` (#141414)
- 边框: 1px solid `neutralStroke1` (#383838)
- 圆角: `radiusMedium` (4px)
- 文字: 12px, `neutralFg1`
- Placeholder: `neutralFg4`
- Padding: 0 8px
- Focus: 边框变为 `brandFg1`, 外发光 `0 0 0 1px rgba(71,158,245,0.3)`

**状态**: default, hover (边框 `neutralFg4`), focus, disabled, error

**Props**:
```typescript
interface TextInputProps {
  value: string;
  placeholder?: string;
  maxLength?: number;
  pattern?: string;        // regex validation
  disabled?: boolean;
  onChange: (value: string) => void;
}
```

### 7.3 NumberInput 控件

**对应 ParameterType**: `integer` (无 min/max 时)

**视觉规格**: 与 TextInput 相同, 但 type="number", 右侧有步进按钮

**Props**:
```typescript
interface NumberInputProps {
  value: number;
  min?: number;
  max?: number;
  step?: number;
  precision?: number;      // 小数位数
  unit?: string;           // 单位标签
  disabled?: boolean;
  onChange: (value: number) => void;
}
```

### 7.4 Slider 控件

**对应 ParameterType**: `integer`(有 min/max), `float`, `slider`, `combo_slider`

**视觉规格**:
- 轨道: 高度 4px, 背景 `neutralStroke2` (#4a4a4a), 圆角 `radiusSmall` (2px)
- 填充: 高度 4px, 背景 `brandFg1` (#479ef5), 圆角 `radiusSmall`
- 滑块: 18×18px 圆形, 背景 `brandFg1`, 2px solid `neutralBg2` 边框
- 数值显示: mono 字体, 12px, `neutralFg1`, 宽度 48px 右对齐
- 单位: 12px, `neutralFg4`
- Ticks (可选): 刻度标记点, 2px 宽, `neutralFg4`

**Props**:
```typescript
interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  showValue?: boolean;
  unit?: string;
  showTicks?: boolean;
  ticks?: number[];        // 自定义刻度位置
  orientation?: 'horizontal' | 'vertical';
  style?: 'continuous' | 'discrete' | 'range' | 'dual_handle';
  presets?: [string, number][];  // [label, value], combo_slider 专用
  disabled?: boolean;
  onChange: (value: number) => void;
}
```

**交互**:
- 点击轨道: 跳转到该位置
- 拖动滑块: 连续调整值
- 点击数值: 进入精确输入模式 (切换为 Input)
- 刻度标签 (如 jxl_encoder quality=-1 标 "Lossless")

### 7.5 Toggle 控件

**对应 ParameterType**: `boolean`

**视觉规格**:
- 轨道: 36×18px, 圆角 9px (circular)
  - OFF: 背景 `neutralStroke2` (#4a4a4a)
  - ON: 背景 `brandFg1` (#479ef5)
- 滑块: 14×14px 圆形, 白色, 阴影 `0 1px 2px rgba(0,0,0,0.3)`
  - OFF: left 2px
  - ON: left 20px
- 过渡: `transitionNormal` (200ms ease)
- 可选标签: label_on / label_false 显示在轨道两侧

**Props**:
```typescript
interface ToggleProps {
  value: boolean;
  labelOn?: string;        // ON 时显示的文字
  labelOff?: string;       // OFF 时显示的文字
  disabled?: boolean;
  onChange: (value: boolean) => void;
}
```

### 7.6 Dropdown 控件

**对应 ParameterType**: `enum`

**视觉规格**:
- 触发器: 高度 30px, 背景 `neutralBg1`, 边框 1px solid `neutralStroke1`, 圆角 `radiusMedium`
- 文字: 12px, 当前值或 placeholder
- 下拉箭头: 右侧 ▼, 颜色 `neutralFg4`
- 弹出面板: 背景 `neutralBg2`, 边框 1px solid `neutralStroke1`, 圆角 `radiusLarge`, 阴影 `shadow8`
- 选项: padding 6px 12px, 12px, hover `neutralBg3`
- 推荐标记: ★ 号, `brandFg1`
- 选中项: `brandFg1` 文字
- 选项描述: 9px, `neutralFg4` (可选)

**Props**:
```typescript
interface DropdownOption {
  value: string;
  label: string;
  description?: string;
  icon?: string;
  tags?: string[];
  recommended?: boolean;
}

interface DropdownProps {
  value: string;
  options: DropdownOption[];
  placeholder?: string;
  display?: 'dropdown' | 'radio_group' | 'button_group' | 'segmented_control' | 'tabs' | 'popup_card';
  disabled?: boolean;
  onChange: (value: string) => void;
}
```

### 7.7 FilePicker 控件

**对应 ParameterType**: `file_path`

**视觉规格**: Input (flex:1) + 浏览按钮 (32×32, FolderOpen 图标)

**Props**:
```typescript
interface FilePickerProps {
  value: string;
  kind: 'file' | 'directory' | 'save_file';
  filters?: [string, string][];  // [["LUT Files", "*.cube"], ["All Files", "*.*"]]
  mustExist?: boolean;
  disabled?: boolean;
  onChange: (value: string) => void;
}
```

**交互**:
- 点击浏览按钮 → Tauri `dialog.open({ filters, directory: kind==='directory' })` → 填充路径
- 手动输入路径 → 失焦时检查 mustExist
- 路径不存在 + mustExist=true → 红色边框 + 错误提示

### 7.8 ColorPicker 控件

**对应 ParameterType**: `color`

**视觉规格**: 触发按钮 (30×30px, 当前色填充, 圆角 `radiusMedium`, 1px `neutralStroke1`) + 弹出取色面板

**Props**:
```typescript
interface ColorPickerProps {
  value: string;            // hex, e.g. "#ff0000"
  mode?: 'RGB' | 'RGBA' | 'HSL' | 'HSV' | 'Lab';
  showAlpha?: boolean;
  disabled?: boolean;
  onChange: (value: string) => void;
}
```

**取色面板**:
- 色相条 (水平渐变)
- 饱和度/亮度面 (2D 渐变)
- Alpha 条 (可选)
- HEX 输入框
- 预设色块 (常用色)

### 7.9 CoordinateInput 控件

**对应 ParameterType**: `coordinate`

**视觉规格**: 两个并排 Input (lat / lon), 标签在上方

**Props**:
```typescript
interface CoordinateInputProps {
  latitude: number;
  longitude: number;
  altitude?: number;
  altRequired?: boolean;
  directionRequired?: boolean;
  disabled?: boolean;
  onChange: (coord: { lat: number; lon: number; alt?: number }) => void;
}
```

**地图联动**: gps_set 的 Map Picker 会联动更新此控件, 蓝色边框表示"来自地图自动填充"。

### 7.10 自定义渲染器

9 种 Custom 渲染器用于标准控件无法表达的交互:

| 渲染器 | 核心交互 | 实现方案 |
|--------|---------|---------|
| `tone_curve` | 多点曲线 (单击添加点, 拖动移动, 双击删除) | Canvas 2D 绘制曲线 + 控制点 |
| `map_picker` | 搜索→选择→坐标填充 | Leaflet 内嵌 + Geocoding API |
| `lut_preview` | 3D LUT 色彩预览 | Canvas 2D 色彩映射 |
| `lens_selector` | 品牌→型号级联选择 | 联动 Dropdown × 2 |
| `color_gamut_viewer` | CIE 色域叠加显示 | Canvas 2D (见 AuxView) |
| `denoise_split` | 降噪前后分割对比 | 双 Canvas + 拖动分割线 |
| `raw_thumbnail` | RAW 快速预览 | Canvas 2D 渲染缩略图 |
| `resize_visual_guide` | 缩放参考线 | SVG overlay 在 Preview 上 |
| `encoder_preset` | 编码器预设选择 | CardSelector 变体 |

---

## 8. 按钮库

### 8.1 按钮类型完整规范

| # | 类型 | Fluent appearance | 高度 | Padding | 背景 | 边框 | 文字颜色 | Hover | Active | Disabled |
|:--|------|-----------------|:---:|---------|------|------|---------|-------|--------|---------|
| 1 | **Primary** | `primary` | 30px | 4px 16px | `brandFg1` | `brandFg1` | `neutralBg1` (#141414) | 亮度+10% | 亮度-5% | opacity 0.4 |
| 2 | **Subtle** | `subtle` | 30px | 4px 16px | transparent | transparent | `neutralFg2` | `neutralBg3` | `neutralBg4` | opacity 0.4 |
| 3 | **Danger** | 自定义 | 30px | 4px 16px | transparent | `dangerFg` | `dangerFg` | `rgba(209,52,71,0.08)` | `rgba(209,52,71,0.12)` | opacity 0.4 |
| 4 | **Icon** | `subtle` | 32px | 0 (32×32) | transparent | transparent | `neutralFg2` | `neutralBg4` | `neutralBg3` | opacity 0.4 |
| 5 | **Small** | `subtle` + `size="small"` | 24px | 2px 8px | transparent | transparent | `neutralFg2` | `neutralBg3` | `neutralBg4` | opacity 0.4 |
| 6 | **Warning** | 自定义 | 30px | 4px 16px | transparent | `warningFg` | `warningFg` | `rgba(213,153,0,0.08)` | `rgba(213,153,0,0.12)` | opacity 0.4 |

### 8.2 按钮使用场景映射

| 按钮 | 类型 | 位置 | 快捷键 |
|------|------|------|:---:|
| Import | Primary | SidebarToolbar | `Ctrl+O` |
| Clear | Small | SidebarToolbar | — |
| To Batch | Small | SidebarToolbar | — |
| New | Subtle | DAGToolbar | — |
| Save | Subtle | DAGToolbar | `Ctrl+S` |
| Load | Subtle | DAGToolbar | — |
| Validate | Subtle | DAGToolbar | — |
| ▶ Run | Primary | DAGToolbar | `Ctrl+E` |
| ⏹ Cancel | Subtle | DAGToolbar | — |
| Zoom+ | Icon | DAGToolbar | — |
| Zoom− | Icon | DAGToolbar | — |
| Fit | Icon | DAGToolbar | — |
| Theme Toggle | Icon | TitleBar | — |
| Settings | Icon | TitleBar | `Ctrl+,` |
| Remove from Pipeline | Danger | Panel 底部 | — |
| Cancel (Dialog) | Subtle | SettingsDialog | — |
| Reset | Warning | AdvancedTab | — |
| Save (Dialog) | Primary | SettingsDialog | — |
| ▶ Start Batch | Primary | BatchControls | — |
| ⏸ Pause | Subtle | BatchControls | — |
| ⏹ Stop | Subtle | BatchControls | — |
| Clear Done | Subtle | BatchControls | — |
| + Create Group | Subtle (dashed) | GroupAdd | — |
| Auto-group | Subtle (dashed) | GroupAdd | — |

### 8.3 按钮使用约束

1. **Primary 独占**: 同一可视区域内最多一个 Primary 按钮。例外: 工具栏 Run + 批量 Start Batch 不在同一视图。
2. **Danger 确认**: Remove from Pipeline 点击后不立即执行, 而是弹出 ConfirmDialog。
3. **Icon 必须有 Tooltip**: Icon-only 按钮 hover 0.5s 后显示 Tooltip 说明功能。
4. **Small 仅工具栏**: Small 按钮仅用于 SidebarToolbar 等紧凑工具栏。
5. **Disabled 状态**: 按钮在关联操作不可用时 disabled (如未导入图片时 Run disabled)。

---

## 9. 卡片与容器

### 9.1 卡片类型体系

| # | 类型 | 组件名 | 使用场景 | 尺寸 | 状态数 |
|:--|------|--------|---------|------|:---:|
| 1 | ImageCard | `<ImageCard>` | FilmstripList 单张图片 | 全宽, 固定内高 | 4 |
| 2 | PluginCard | `<PluginCard>` | PluginGrid 插件卡片 | min-width 100px | 3 |
| 3 | DAGNode | `<DAGNode>` | DAGCanvas 管线节点 | min-width 136px | 3 |
| 4 | ParamCard | `<Card>` → `<ParamSection>` | ControlPanel 参数分区 | 全宽 | 2 (展开/折叠) |
| 5 | CollapsibleCard | `<CollapsibleCard>` → `<ParamSection>` | ControlPanel 折叠参数分区 | 全宽 | 2 (展开/折叠) |
| 6 | InfoCard | `<InfoCard>` | Sidebar 统计区, BatchLeftPanel | 自适应 | 1 (只读) |

### 9.2 统一选中态视觉语言

所有可选中卡片共享同一套视觉状态转换:

```
default ──(hover)──→ hover ──(click)──→ selected ──(click elsewhere)──→ default
  │                    │                    │
  │ transparent        │ neutralBg3         │ brandFg1 solid 边框
  │ 边框透明            │ 边框透明/neutralFg4 │ brandFg1 浅底
  │ neutralBg2 背景     │                    │
```

**multi-selected 变体** (仅 ImageCard): brandFg1 **dashed** 边框 + 勾选框显示

### 9.3 ImageCard 详细设计

**用途**: FilmstripList 中单张图片的展示卡片

**内部布局**:
```
┌────────────────────────────────────────┐
│ [✓] [缩略图 52×36] 文件名.ARW       24MB│ ← 行1: 勾选框(仅multi) + 缩略图 + 文件名 + 文件大小
│                     6000×4000 · ARW     │ ← 行2: 元数据
│                     🟡 High ISO         │ ← 行3: 分组标签(条件显示)
└────────────────────────────────────────┘
```

**4 种状态**:

| 状态 | 触发条件 | 边框 | 背景 | 勾选框 | 选中逻辑 |
|------|---------|------|------|:---:|---------|
| **default** | 未选中, 鼠标不在上方 | 1px solid transparent | `neutralBg2` | 隐藏 | — |
| **hover** | 鼠标悬停 (未选中) | 1px solid transparent | `neutralBg3` | 隐藏 | — |
| **selected** | 被选中 (唯一选中项) | 1px solid `brandFg1` | `rgba(71,158,245,0.06)` | 隐藏 | 单击选中 |
| **multi-selected** | 被选中 (多选之一) | 1px **dashed** `brandFg1` | `rgba(71,158,245,0.03)` | 显示 ✓ (18×18, `brandFg1` bg) | Ctrl/Shift+Click |

**交互事件**:

| 事件 | 响应 |
|------|------|
| `onClick` (无修饰键) | 单选: 清除其他选择 → 选中此卡 |
| `onClick` (Ctrl/Meta) | 追加/取消: toggle 此卡选中状态 |
| `onClick` (Shift) | 范围选择: anchor→current 全部选中 |
| `onContextMenu` | 弹出右键菜单 |
| `onDragStart` | 开始拖拽 (拖到 Group/Batch) |

**Props**:
```typescript
interface ImageCardProps {
  image: ImageInfo;
  index: number;
  thumbnail?: string;       // base64 data URL, 异步加载
  isSelected: boolean;
  isMultiSelect: boolean;
  isAnchor: boolean;        // Shift+Click 的锚点
  onClick: (index: number, ctrlKey: boolean, shiftKey: boolean) => void;
  onContextMenu: (e: React.MouseEvent, index: number) => void;
  onDragStart: (e: React.DragEvent, index: number) => void;
}
```

### 9.4 PluginCard 详细设计

**用途**: PluginBrowser 中单个插件的卡片

**内部布局**:
```
┌──────────────────┐
│ ● 插件名          │  ← 行1: 彩色圆点 + 插件名 (12px 600)
│   插件类别        │  ← 行2: 分类徽章 (9px, neutralBg3 bg)
└──────────────────┘
```

**3 种状态**:
| 状态 | 边框 | 背景 | 触发 |
|------|------|------|------|
| default | transparent | `neutralBg2` | 初始 |
| hover | `neutralStroke1` | `neutralBg3` | 鼠标悬停 |
| selected | `brandFg1` | `rgba(71,158,245,0.06)` | 对应 DAGNode 被选中时高亮 |

**交互**: 单击 → 无 (仅浏览), 双击 → `usePipelineStore.addNode(pluginId)`, 拖拽 → 开始 DAG 拖放

### 9.5 DAGNode 详细设计

**内部布局**:
```
┌──────────────────────────┐
│ [auto · 12 files]        │  ← 徽章: 绝对定位 right:6px top:6px (条件显示)
│                          │
│ ⊡  raw_input         ⊡   │  ← 行1: 输入端口 + 插件名(12px 600) + 输出端口
│    Input                 │  ← 行2: 分类名(9px, neutralFg4)
└──────────────────────────┘
```

**3 种状态**:
| 状态 | 边框 | 阴影 | z-index | 触发 |
|------|------|------|:---:|------|
| default | 1.5px solid `neutralStroke2` | `shadow4` | 2 | 初始 |
| hover | 1.5px solid `neutralFg2` | `shadow4` | 3 | 鼠标悬停 |
| selected | 1.5px solid `brandFg1` | `0 0 0 3px rgba(71,158,245,0.12), shadow8` | 5 | 单击选中 |
| executing | 1.5px solid `successFg` | `shadow4` + 脉冲 | 2 | 管线执行中 |

**端口**: 输入端口 (14×14, left:-8px, 2px solid `successFg`), 输出端口 (14×14, right:-8px, 2px solid `brandFg1`)。Hover 时 scale(1.3)。

**Props**:
```typescript
interface DAGNodeProps {
  data: DAGNodeData;
  selected: boolean;
  executing: boolean;
  onMouseDown: (e: React.MouseEvent, id: string) => void;   // 拖动开始
  onPortDragStart: (nodeId: string, portId: string, portType: 'input'|'output') => void;
  onPortDrop: (nodeId: string, portId: string) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
}
```

---

## 10. 状态指示器

### 10.1 StatusDot 组件

| 变体 | 颜色 | 尺寸 | 动画 | 使用场景 |
|------|:---:|:---:|:---:|---------|
| `connected` | `successFg` (#54b054) | 8×8 | 无 | StatusBar 后端连接正常 |
| `disconnected` | `dangerFg` (#d13447) | 8×8 | 无 | StatusBar 后端断开 |
| `queued` | `neutralFg4` (#505050) | 8×8 | 无 | BatchQueueRow 待处理 |
| `processing` | `brandFg1` (#479ef5) | 8×8 | `pulse 1s infinite` (opacity 0.4↔1) | BatchQueueRow 处理中 |
| `done` | `successFg` (#54b054) | 8×8 | 无 | BatchQueueRow 已完成 |
| `failed` | `dangerFg` (#d13447) | 8×8 | 无 | BatchQueueRow 失败 |
| `group` | 可自定义 | 6×6 | 无 | GroupTree 分组标记 |
| `ok` | `successFg` (#54b054) | 8×8 | 无 | ContentHeader 管线正常 |

```typescript
interface StatusDotProps {
  variant: 'connected' | 'disconnected' | 'queued' | 'processing' | 'done' | 'failed' | 'group' | 'ok';
  color?: string;           // group 变体可自定义颜色
  size?: number;            // 默认根据变体自动选择
  pulse?: boolean;          // 强制脉冲动画
}
```

### 10.2 ProgressBar 组件

**视觉规格**:
- 轨道: 高度 4px, 背景 `neutralStroke1`, 圆角 2px
- 填充: 高度 100%, 背景 `brandFg1`, 圆角 2px
- 过渡: `width 0.3s ease`

**变体**:

| 变体 | 高度 | 宽度 | 位置 |
|------|:---:|------|------|
| `compact` | 4px | flex:1, max 180px | StatusBar 内 |
| `full` | 6px | 全宽 | BatchCenterPanel 头部 |
| `inline` | 4px | flex:1 | ParamRow 内 (ai_denoise progress) |

```typescript
interface ProgressBarProps {
  value: number;            // 0-100
  variant: 'compact' | 'full' | 'inline';
  showLabel?: boolean;      // 显示百分比文字
}
```

### 10.3 Badge 组件

| 变体 | 背景 | 文字颜色 | 字号 | 圆角 | Padding | 使用场景 |
|------|------|---------|:---:|:---:|---------|---------|
| `category` | `neutralBg3` | `neutralFg2` | 9px | 2px | 0px 4px | PluginCard 插件类别 |
| `count` | `neutralBg3` | `neutralFg2` | 9px | 2px | 2px 6px | GroupItem 文件计数, SidebarHeader 图片数 |
| `override-inherited` | transparent | `neutralFg3` | 10px | — | 0 | Section 头部 "inherited" |
| `override-warning` | `rgba(213,153,0,0.08)` | `warningFg` | 10px | 2px | 2px 6px | Section 头部 "N overrides" |
| `auto-node` | `rgba(71,158,245,0.12)` | `brandFg1` | 8px | 2px | 1px 6px | DAGNode "auto·12 files" |
| `group-tag` | `rgba(213,153,0,0.08)` | `warningFg` | 8px | 2px | 1px 6px | ImageCard "High ISO" |
| `mode-badge` | `brandFg1` | `neutralBg1` | 9px | 8px (pill) | 1px 7px | ModeTab 批量队列计数 N |

```typescript
interface BadgeProps {
  variant: 'category' | 'count' | 'override-inherited' | 'override-warning' | 'auto-node' | 'group-tag' | 'mode-badge';
  children: React.ReactNode;
  color?: string;           // group-tag 变体自定义颜色
}
```

---

## 11. 辅助视图系统

### 11.1 辅助视图类型

| 类型 | 渲染引擎 | 数据来源 | 数据格式 | 适用插件 |
|------|---------|---------|---------|---------|
| **Histogram** | Canvas 2D 柱状图 | `decode_preview()` → RGB 像素数组 | 12 bins, R/G/B/L 四通道 | transform, colorspace, lut3d, lens_correct, ai_denoise |
| **Waveform** | Canvas 2D 逐列亮度 | `decode_preview()` → RGB 像素数组 | 256 列亮度采样 | (波形监视) |
| **Vectorscope** | Canvas 2D 极坐标 | `decode_preview()` → RGB 像素数组 | uv 分量散点图 | lut3d, colorspace |
| **GamutDiagram** | Canvas 2D CIE 1931 | 色域边界数据 (后端计算) | CIE xy 坐标边界 + 像素散点 | colorspace |
| **Map** | Leaflet 内嵌 iframe | Geocoding API 响应 | lat/lon 标记点 | gps_set |
| **FocusPeaking** | Canvas 2D 边缘检测 | `decode_preview()` → 像素数组 | Sobel 边缘强度 + 伪彩色叠加 | (峰值对焦) |
| **StatusText** | DOM `<div>` 文本 | 后端状态查询 | 字符串 | lens_correct, ai_denoise |
| **ProgressBar** | DOM ProgressBar 组件 | gRPC stream | fraction 0-1 | ai_denoise |
| **MetadataTable** | DOM `<table>` | `load_images()` 元数据 | EXIF 字段表 | exif_rw |
| **ClippingWarning** | Canvas 2D 叠加 | `decode_preview()` → 像素 | 高光/暗部裁剪区域着色 | (裁切警告) |

### 11.2 渲染位置

辅助视图渲染在 Panel 底部, 参数分区和 RemoveButton 之间。多个辅助视图按 `GuiSchema.aux_views` 声明顺序垂直排列, 每个占 80~200px 高度。

### 11.3 AuxView 组件

```typescript
interface AuxViewProps {
  type: AuxViewType;
  nodeId: string;           // 关联的节点 ID
  pluginId: string;         // 关联的插件 ID
}

type AuxViewType =
  | 'histogram'
  | 'waveform'
  | 'vectorscope'
  | 'gamut_diagram'
  | 'map'
  | 'focus_peaking'
  | 'clipping_warning'
  | 'metadata_table'
  | 'progress_bar'
  | 'status_text';
```

### 11.4 数据获取

每个 AuxView 在 mount 时通过 `invoke("decode_preview", { path, maxWidth, maxHeight })` 获取当前选中图片的像素数据。数据在 Rust 端缓存, 同一图片的多个 AuxView 共享同一次 decode 结果。

---

## 12. 上下文菜单系统

### 12.1 菜单通用规范

所有右键菜单使用统一的 `ContextMenu` 组件:

**视觉规格**:
- 固定定位 (position: fixed), 跟随鼠标位置
- 背景: `neutralBg2` (#1f1f1f)
- 边框: 1px solid `neutralStroke1`
- 圆角: `radiusLarge` (8px)
- 阴影: `0 4px 16px rgba(0,0,0,0.4)`
- 最小宽度: 180px
- 最大高度: 400px (溢出滚动)
- Z-index: 1000

**菜单项规格**:
- 高度: 32px
- Padding: 0 12px
- 字体: 12px, `neutralFg2`
- 布局: flex, space-between (label + shortcut)
- Hover: `neutralBg3` 背景
- Danger 项: `dangerFg` 文字, hover `rgba(209,52,71,0.08)` 背景
- 禁用项: opacity 0.4, cursor not-allowed
- 分割线: 1px solid `neutralStroke1`, margin 4px 0

**快捷键标注**: 右对齐, 10px, `neutralFg4`, mono 字体

### 12.2 四种菜单场景

#### 场景1: ImageCard 右键

```
┌──────────────────────────────┐
│ Open in Explorer             │  ← 打开文件管理器定位
│ Copy Path                    │  ← 复制路径到剪贴板
├──────────────────────────────┤
│ Select All          Ctrl+A   │  ← 全选
│ Clear Selection     Esc      │  ← 清除选择
│ Invert Selection             │  ← 反选
├──────────────────────────────┤
│ Add to Group →    [子菜单]   │  ← 展开分组列表 + New Group
│ Send to Batch                │  ← 发送到批量队列
├──────────────────────────────┤
│ Remove               Del     │  ← 从列表移除 (红色 hover)
└──────────────────────────────┘
```

#### 场景2: DAGNode 右键

```
┌──────────────────────────────┐
│ transform                    │  ← 插件名 (粗体, 不可点击)
├──────────────────────────────┤
│ 📋 Copy             Ctrl+C   │  ← 复制节点
│ 📋 Duplicate        Ctrl+D   │  ← 复制并偏移
├──────────────────────────────┤
│ ⏸ Disable             E     │  ← 切换 enabled/disabled
├──────────────────────────────┤
│ 🗑 Delete             Del    │  ← 删除节点 (红色 hover)
└──────────────────────────────┘
```

#### 场景3: DAGCanvas 空白右键

```
┌──────────────────────────────┐
│ Add Node                     │  ← 标题 (不可点击)
├──────────────────────────────┤
│ ● raw_input                  │
│ ● transform                  │
│ ● colorspace                 │
│ ● lut3d                      │
│ ● lens_correct               │
│ ● ai_denoise                 │
│ ● exif_rw                    │
│ ● gps_set                    │
│ ● time_shift                 │
│ ● heif_encoder               │
│ ● jxl_encoder                │
│ ● avif_encoder               │
│ ● tiff_encoder               │
│ ● png_encoder                │
├──────────────────────────────┤
│ 📋 Paste nodes               │  ← (如果有剪贴板节点)
└──────────────────────────────┘
```

#### 场景4: DAGEdge 右键

```
┌──────────────────────────────┐
│ 🗑 Delete Edge               │  ← 删除连线 (红色 hover)
└──────────────────────────────┘
```

### 12.3 ContextMenu 组件接口

```typescript
interface ContextMenuProps {
  isOpen: boolean;
  position: { x: number; y: number };
  items: MenuItem[];
  onClose: () => void;
}

interface MenuItem {
  type: 'item' | 'separator' | 'header' | 'submenu';
  label?: string;
  shortcut?: string;         // "Ctrl+A", "Del"
  icon?: string;             // emoji 或组件
  danger?: boolean;
  disabled?: boolean;
  onClick?: () => void;
  children?: MenuItem[];     // submenu 时
}
```

### 12.4 关闭逻辑

- 点击菜单项: 执行 action → 关闭
- 点击菜单外: 关闭
- Esc 键: 关闭
- 右键其他地方: 关闭当前 → 打开新菜单

---

# 第四部分: 面板与视图

## 13. 面板体系总览

### 13.1 两模式架构

应用通过 TitleBar 的 ModeTabs 在两个顶层工作模式间切换。两种模式共享 TitleBar 和 StatusBar, 但中间内容区完全替换。

| 属性 | 编辑模式 (Pipeline Editor) | 批量模式 (Batch Processing) |
|------|--------------------------|---------------------------|
| TitleBar | Logo + 标题 + [Editor] active + [Batch N] | Logo + 标题 + [Editor] + [Batch N] active |
| 内容区 | MainLayout → Sidebar/Content/Panel | BatchMode → Left/Center/Right |
| StatusBar | 编辑模式版 (精简) | 批量模式版 (内存+GPU+管线信息) |

### 13.2 模式切换流程

```
[Pipeline Editor] ──(点击 [Batch Processing])──→ 批量模式
  - 保留 usePipelineStore, useFilmstripStore 状态
  - 渲染 BatchMode 三栏
  - StatusBar 切换到批量模式

[Batch Processing] ──(点击 [Pipeline Editor])──→ 编辑模式
  - 保留 useBatchStore 状态
  - 恢复 MainLayout 三栏
  - StatusBar 切换到编辑模式
```

### 13.3 面板可拖拽调整

`Sidebar` 和 `Panel` 可通过 `ResizablePanel` 包装器拖拽调整宽度:

```typescript
interface ResizablePanelProps {
  side: 'left' | 'right';
  defaultWidth: number;
  minWidth: number;
  maxWidth: number;
  children: React.ReactNode;
}
```

---

## 14. TitleBar 详细设计

### 14.1 整体结构

```
┌────────────────────────────────────────────────────────────────┐
│ ◆ Photopipeline — HDR Pipeline v1    [Editor] [Batch 16]  ◐  ⚙ │
└────────────────────────────────────────────────────────────────┘
  ↑                ↑                      ↑                ↑  ↑
  Logo + 标题      拖拽区域               ModeTabs         主题 设置
```

### 14.2 子组件

| 子组件 | 位置 | 规格 | Props |
|--------|------|------|-------|
| `<Logo />` | 最左 | 22×22, `brandFg1` 背景, `radiusMedium`, 白色 "◆", 11px | — |
| `<TitleText />` | Logo 右侧 | 12px 600, `neutralFg1`, flex:1 | — |
| `<ModeTabs />` | 中间偏右 | 背景 `neutralBg1`, 圆角 6px, 边框 `neutralStroke1`, padding 2px | `activeMode`, `onChange`, `batchCount` |
| `<ThemeToggle />` | 右侧 | 32×32 Icon 按钮, "◐" 图标 | `theme`, `onToggle` |
| `<SettingsButton />` | 最右 | 32×32 Icon 按钮, "⚙" 图标 | `onClick` |

### 14.3 ModeTabs 组件

```
┌──────────────────────────────────┐
│ [ Pipeline Editor ] [ Batch Processing 16 ] │  ← 背景 neutralBg1, 圆角 6px
└──────────────────────────────────┘
```

| 状态 | active tab | inactive tab | badge |
|------|-----------|-------------|-------|
| 背景 | `rgba(71,158,245,0.10)` | transparent | `brandFg1` |
| 文字 | `neutralFg1` | `neutralFg3` (#6e6e6e) | `neutralBg1` |
| 圆角 | 4px | 4px | 8px (pill) |
| padding | 5px 16px | 5px 16px | 1px 7px |
| 字号 | 11px | 11px | 9px 600 |

### 14.4 拖拽区域

- TitleBar 整个元素: `data-tauri-drag-region` (可拖拽窗口)
- 按钮元素: `data-tauri-drag-region="false"` (不可拖拽, 防止误触)
- 双击 TitleBar: 最大化/恢复窗口 (调用 Tauri API)

---

由于篇幅极大, 以下章节以摘要形式呈现。完整文档包含每个组件的完整 Props 接口、每个交互的完整事件流、每个状态的完整 UI 表现。

## 15. Sidebar 详细设计

### 15.1 组件树

```
<Sidebar>
  <SidebarHeader fileCount={12} />        // "CANDIDATE FILES  12 images"
  <SidebarToolbar                         // Import / Clear / To Batch
    onImport={handleImport}
    onClear={handleClear}
    onToBatch={handleToBatch} />
  <SortBar                                // Sort: [Name ▾] Size: [S][M][L]
    sortKey={sortKey} onSortChange={...}
    thumbSize={thumbSize} onSizeChange={...} />
  <MultiSelectBar                         // 条件: selectedIndices.size > 1
    count={selectedCount}
    onGroup={handleAddToGroup}
    onBatch={handleSendToBatch}
    onClear={handleClearSelection} />
  <FilmstripList>                         // @tanstack/react-virtual
    images.map(img => <ImageCard ... />)
  </FilmstripList>
  <EmptyState                             // 条件: images.length === 0
    icon="folder"
    title="No images loaded"
    description="Click Import or drag files here" />
  <DropZone                               // 条件: dragOver event
    onDrop={handleFileDrop} />
  <GroupTree>                             // 分组管理
    groups.map(g => <GroupItem ... />)
    <GroupAdd                             // + Create Group / Auto-group
      onCreateGroup={...}
      onAutoGroup={...} />
  </GroupTree>
</Sidebar>
```

### 15.2 子组件接口

#### SidebarHeader
```typescript
interface SidebarHeaderProps {
  fileCount: number;
}
```
显示: "CANDIDATE FILES  {count} images", 10px uppercase 600 `neutralFg3`, 计数 9px `neutralFg4`

#### SidebarToolbar
```typescript
interface SidebarToolbarProps {
  hasImages: boolean;
  hasSelection: boolean;
  onImport: () => void;
  onClear: () => void;
  onToBatch: () => void;
}
```
Import 按钮: Primary + Small (24px), Disabled 当正在导入时
Clear / To Batch: Small (24px), Disabled 当无图片/无选中时

#### SortBar
```typescript
interface SortBarProps {
  sortKey: 'name' | 'size' | 'format' | 'iso';
  thumbnailSize: 'S' | 'M' | 'L';
  onSortChange: (key: SortKey) => void;
  onSizeChange: (size: 'S' | 'M' | 'L') => void;
}
```
Sort 下拉: height 22px, 10px, 4个选项
尺寸按钮: S(80px)/M(120px)/L(180px), 互斥 ToggleButton 组

#### MultiSelectBar
```typescript
interface MultiSelectBarProps {
  count: number;
  onGroup: () => void;
  onBatch: () => void;
  onClear: () => void;
}
```
条件渲染: `count > 1` 时才挂载。背景 `rgba(213,153,0,0.06)`, 文字 `warningFg`

#### FilmstripList
```typescript
interface FilmstripListProps {
  images: ImageInfo[];
  selectedIndices: Set<number>;
  selectionAnchor: number | null;
  thumbnails: Map<number, string>;   // index → base64 data URL
  onImageClick: (index: number, ctrl: boolean, shift: boolean) => void;
  onImageContextMenu: (e: MouseEvent, index: number) => void;
  onImageDragStart: (e: DragEvent, index: number) => void;
}
```
使用 `@tanstack/react-virtual` 的 `useVirtualizer`, 每项高度 60px (S)/70px (M)/85px (L)

#### GroupTree
```typescript
interface GroupTreeProps {
  groups: ImageGroup[];
  onCreateGroup: (name: string, condition?: GroupCondition) => void;
  onEditGroup: (name: string) => void;
  onDeleteGroup: (name: string) => void;
  onAutoGroup: (strategy: AutoGroupStrategy) => void;
}
```
每个 `<GroupItem>`: 显示圆点+名称+计数,hover 显示 ✎🗑 按钮

#### EmptyState
```typescript
interface EmptyStateProps {
  icon: string;              // emoji 或组件
  title: string;
  description: string;
  action?: { label: string; onClick: () => void };
}
```

#### DropZone
```typescript
interface DropZoneProps {
  isVisible: boolean;
  onDrop: (files: FileList) => void;
}
```
显示时: 蓝色 2px dashed `brandFg1` 边框, 居中引导文字

---

## 16. Content (DAG画布) 详细设计

### 16.1 组件树

```
<Content>
  <ContentHeader                         // "● Pipeline Editor  5 nodes · 120%"
    status="ok"
    nodeCount={5}
    zoom={1.2} />
  <DAGToolbar                            // 两排按钮组
    hasNodes={true}
    isDirty={true}
    executionState="idle"
    zoom={1.2}
    onNew={() => newPipeline()}
    onSave={() => savePipeline()}
    onLoad={() => loadPipeline()}
    onValidate={() => validatePipeline()}
    onRun={() => executePipeline()}
    onCancel={() => cancelExecution()}
    onZoomIn={() => zoomIn()}
    onZoomOut={() => zoomOut()}
    onFit={() => zoomFit()} />
  <DAGCanvas                              // 画布容器
    zoom={1.2}
    panOffset={{ x: 100, y: 50 }}>
    <svg className="dag-edges">          // 连线层 (z-index:1)
      edges.map(e => <DAGEdge ... />)
    </svg>
    <div className="dag-nodes">          // 节点层 (z-index:2)
      nodes.map(n => <DAGNode ... />)
    </div>
    <MiniMap                             // 迷你地图 (z-index:5)
      nodes={nodes}
      edges={edges}
      viewport={viewportRect}
      onJump={handleMiniMapClick}
      onPan={handleMiniMapDrag} />
    <DropHint                            // 拖放提示 (z-index:15)
      isVisible={isDraggingPlugin} />
  </DAGCanvas>
  <ContextMenu                           // 右键菜单 (z-index:1000)
    {...contextMenuState} />
</Content>
```

### 16.2 DAGCanvas 详细设计

**画布规格**:
- 背景: `neutralBg1` (#141414)
- 网格: 32px, `linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px)`
- 在 CSS transform 层应用 zoom/pan
- 画布坐标系统: origin (0,0) = 左上角, x 向右, y 向下

**缩放**:
- 范围: 0.1× ~ 5.0×
- 步进: 10%/档
- Ctrl+滚轮: 以鼠标位置为中心缩放
- 工具栏 +/−: 以画布中心缩放
- Fit: 计算所有节点的 bounding box → 缩放到适应窗口

**平移**:
- Space+拖动: 抓取平移
- 鼠标中键拖动: 抓取平移
- 拖动画布空白处 (无节点): 平移

**拖放**:
- `onDragOver`: 阻止默认, 显示 DropHint
- `onDrop`: 获取拖拽的 pluginId, 在释放位置创建节点
- DropHint: 蓝色 2px dashed 边框, 居中 "Drop plugin here"

**连线交互** (DAGPort):
- 鼠标按下输出端口 → `wireActive = true`, 创建 `<line>` 跟随鼠标
- 鼠标移动 → 更新跟随线终点
- 鼠标松开在输入端口上 → 创建 edge (环检测通过)
- 鼠标松开在其他地方 → 取消连线
- 环检测: 客户端 `hasCycle(nodes, edges, {from, to})` → BFS

**MiniMap**:
- Canvas 2D 渲染
- 绘制所有节点的缩小版 (填充矩形)
- 绘制所有边的缩小版 (细线)
- 蓝色半透明矩形表示当前视口
- 点击: 跳转视口到点击位置
- 拖动: 平移视口

### 16.3 DAGToolbar 详细设计

```
[New] [Save] [Load] │ [Validate] │ [▶ Run] [⏹ Cancel] │ [+] [−] [Fit]
```

| 按钮 | 类型 | State 依赖 |
|------|------|-----------|
| New | Subtle | 始终可用 |
| Save | Subtle | isDirty 时 enabled |
| Load | Subtle | 始终可用 |
| Validate | Subtle | hasNodes 时 enabled |
| ▶ Run | **Primary** | hasNodes + executionState='idle' 时 enabled; executing 时变为 Cancel |
| ⏹ Cancel | Subtle | executionState='running' 时显示 |
| Zoom+ | Icon | zoom < 5.0 时 enabled |
| Zoom− | Icon | zoom > 0.1 时 enabled |
| Fit | Icon | hasNodes 时 enabled |

### 16.4 PluginBrowser 详细设计

位于 DAG 画布和 StatusBar 之间:

```
┌──────────────────────────────────────────────────────────────┐
│ PLUGINS  🔍 Search plugins...  [All ▾]                       │
│ ┌──────────┬──────────┬──────────┬──────────────────────────┐│
│ │ raw_input│ transform│colorspace│  ... (水平滚动)           ││
│ └──────────┴──────────┴──────────┴──────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

**搜索**: 160px 宽度 Input, 实时过滤 `usePluginStore.filteredPlugins`
**分类筛选**: 96px Dropdown, 选项: All + 6 个分类

**PluginGrid**:
- 水平滚动 (overflow-x: auto)
- 3 列网格, gap: 4px
- 滚动条: height 4px, 颜色 `neutralStroke2`

---

## 17. Panel (插件控制面板) 详细设计

### 17.1 组件树

```
<Panel>
  <ContextBar                             // 四级覆盖层级切换
    scopes={computedScopes}
    activeScope={scope}
    onScopeChange={setScope} />
  {selectedNode ? (
    <>
      <PluginHeader                       // 插件信息头部
        plugin={pluginMeta}
        color={guiSchema.color} />
      <ControlPanel>                      // Schema 驱动参数面板
        <ParamSection /> ×N               // Card | CollapsibleCard
          <ParamRow /> ×N                // Label + 控件 + Value + Unit + OverrideDot
        </ParamSection>
        {supportsExpression && <ExpressionEditor />}
        {hasAuxViews && <AuxView />}
      </ControlPanel>
      <RemoveButton                       // Danger 删除按钮
        onRemove={() => removeNode(selectedNodeId)} />
    </>
  ) : (
    <EmptyPanelState />                   // 未选中节点时的占位
  )}
</Panel>
```

### 17.2 ContextBar 详细设计

```
┌────────────────────────────────────────────┐
│ All  │  Template  │  High ISO  │  DSC_0034  │
└────────────────────────────────────────────┘
```

**数据来源**:
- `All`: 固定项, 只读查看所有图片的合并效果
- `Template`: 固定项
- `Group Names`: 从 `useFilmstripStore.groups` 获取
- `Image Names`: 从 `useFilmstripStore.images[selectedIndices]` 获取 (单选)

**状态**:
| 状态 | 文字颜色 | 背景 | 底边 |
|------|---------|------|------|
| default | `neutralFg2` | transparent | transparent |
| hover | `neutralFg1` | `neutralBg3` | transparent |
| active | `brandFg1` | transparent | 2px solid `brandFg1` |

**交互**: 点击切换 → `useOverrideStore.setScope(scope)` → Panel 参数行根据新 scope 显示继承/覆盖状态

### 17.3 PluginHeader 详细设计

```
┌──────────────────────────────────────────────┐
│ 📷 RAW Input                    v1.0         │  ← 图标 40×40 + 名称 14px 600 + 版本
│ photopipeline.plugins.raw_input              │  ← 插件 ID, 10px, mono, neutralFg4
│ [Input] [raw] [decode]                       │  ← 标签 badges
│ FormatProcessor · RAM ≥ 512 MB               │  ← 能力和硬件需求, 10px, neutralFg4
├──────────────────────────────────────────────┤
│ Read RAW camera files (ARW, CR2, CR3, NEF,  │  ← 描述, 12px, neutralFg2
│ DNG, RAF, ORF, RW2, PEF)                    │
└──────────────────────────────────────────────┘
```

**图标**: 40×40, 圆角 4px, 背景 `rgba(pluginColor, 0.12)`, 文字色 `pluginColor`, 20px 居中

### 17.4 ControlPanel 详细设计

**渲染逻辑**:
1. 从 `usePluginStore.nodeSchemas.get(pluginId)` 获取 `NodeSchemaResponse`
2. 解析 `parameter_schema.sections[]` → 生成 `<ParamSection>` 列表
3. 每个 section 解析 `fields[]` → 生成 `<ParamRow>` 列表
4. 根据 `gui_schema.layout` 决定使用 Standard 或 Custom 布局
5. 根据 condition 控制分区/字段可见性

**Standard 布局**: sections 按顺序垂直排列
**Custom 布局**: rows 按 Grid 布局, cells 按 width_fraction 分配宽度

### 17.5 ParamSection 详细设计

**展开态 (Card)**:
```
┌──────────────────────────────────────────────┐
│ ▼ Resize      [inherited]                    │  ← Header: caret + 标题 + 覆盖徽章
├──────────────────────────────────────────────┤
│            Width: ═══○═══  1920 px  ⬜       │  ← ParamRow
│     Inherited from Template                  │  ← 来源说明
│           Height: ═══○═══  1080 px  ⬜       │
│           Filter: Lanczos3      ▾   ⬜       │
└──────────────────────────────────────────────┘
```

**折叠态 (CollapsibleCard)**:
```
┌──────────────────────────────────────────────┐
│ ▶ dcraw Options             [advanced]       │  ← caret rotate(-90deg)
└──────────────────────────────────────────────┘
```

### 17.6 ParamRow 详细设计

**布局**: `flex`, gap 8px, margin-bottom 8px

```
[Label]  [Fluent 控件]  [数值] [单位] [OverrideDot]
 105px     flex:1        48px   auto    8×8
 右对齐                   等宽
```

**三种编辑模式** (由 OverrideDot 决定):
1. **继承** (⬜): 控件 disabled, 值来自上级, 显示来源灰色小字
2. **覆盖** (🟡): 控件 enabled, 显示当前值, OverrideDot 黄色实心
3. **表达式** (🔵): 控件 disabled (显示计算结果), OverrideDot 紫色

---

## 18. BatchMode 详细设计

### 18.1 三栏布局

```
┌─ Left (260px) ───────────┬─ Center (flex) ────────────┬─ Right (340px) ───┐
│                          │                             │                    │
│ Pipeline Summary         │ BatchControls + Progress    │ OutputSettings     │
│  · Pipeline name         │  ▶Start ⏸Pause ⏹Stop       │  Directory         │
│  · Nodes chain           │  ████████░░ 68%             │  Template          │
│  · Output config         │  11 done · 2 failed         │  Format            │
│  · Groups                │  ⏱ 00:03:15 ~00:01:30      │  Quality           │
│                          │                             │  Parallel          │
│                          │ Queue (虚拟列表):            │  Conflict          │
│                          │  ● DSC_0034 Done            │                    │
│                          │  ● PANO_001 Done            │ PerImageOverride   │
│                          │  ● DSC_0036 Processing      │  [Select image ▾]  │
│                          │  ● night_012 Queued         │  Denoise: 0.8  🟡  │
│                          │                             │  + Add override    │
└──────────────────────────┴─────────────────────────────┴────────────────────┘
```

### 18.2 BatchQueueRow 4 种状态

| 状态 | 圆点 | 行样式 | 触发 |
|------|:---:|--------|------|
| **Queued** | 灰色 (8×8) | 默认背景 `neutralBg2` | 初始 |
| **Processing** | 蓝色 + `pulse` 动画 | 蓝色边框 `brandFg1` + `rgba(71,158,245,0.04)` 背景 | gRPC stream: status=RUNNING, current_file=this |
| **Done** | 绿色 (8×8) | 默认背景 | gRPC stream: completed_files++ |
| **Failed** | 红色 (8×8) | 默认背景, 红色文字错误原因 | gRPC stream: failed_files++ |

**行布局**: `flex`, gap 8px, padding 7px 8px, 圆角 8px
```
[StatusDot 8×8] [Filename 180px, 600] [Resolution 120px, 10px, neutralFg4] [Size 60px, 10px] [Status text, margin-left:auto]
```

### 18.3 进度指标计算

| 指标 | 公式 | 显示 |
|------|------|------|
| 完成率 | `(done + failed) / total × 100` | 进度条 + 百分比 |
| 速度 | `(done + failed) / (elapsed / 60)` | "14 img/min" |
| ETA | `(elapsed / pct) × (100 - pct)` | "~00:01:30" |
| 耗时 | 从 Start 到现在的秒数 | "⏱ 00:03:15" |

---

## 19. StatusBar 详细设计

### 19.1 编辑模式 StatusBar

```
┌──────────────────────────────────────────────────────────────────┐
│ ▶ Batch: 8/12  ██████████░░░  65%  │  Mem:512MB · GPU:Ready  │ ● Connected │
└──────────────────────────────────────────────────────────────────┘
```

### 19.2 批量模式 StatusBar

```
┌──────────────────────────────────────────────────────────────────┐
│ ● Backend: Connected | Mem: 1.2GB / 2GB | GPU: CUDA · 70°C | Pipeline: HDR_v1 | 16 images · 12 source files │
└──────────────────────────────────────────────────────────────────┘
```

### 19.3 动态内容

| 元素 | 编辑模式 | 批量模式 | 数据来源 |
|------|---------|---------|---------|
| 批量指示 | ▶ Batch: N/M | — | useBatchStore |
| 进度条 | 4px, compact | 6px, full (在 Center) | usePipelineStore / useBatchStore |
| 时间统计 | — | ⏱ elapsed ~ETA | useBatchStore |
| 速度 | — | N img/min | useBatchStore |
| 内存/GPU | Mem: N MB · GPU: Name | Mem: N / Total · GPU: Name · Temp | gRPC MetricSnapshot |
| 连接状态 | ● Connected / Disconnected | ● Connected / Disconnected | useAppStore |

---

## 20. SettingsDialog 详细设计

### 20.1 对话框规格

- 宽度: 520px, 高度自适应 (max-height 440px 内容区)
- 模态: 背景遮罩 `rgba(0,0,0,0.5)`, 点击遮罩不关闭
- 圆角: `radiusXLarge` (12px)
- 阴影: `0 8px 40px rgba(0,0,0,0.6)`

### 20.2 结构

```
┌─ Settings ──────────────────────────────────────┐
│ General │ Backend │ Output │ Advanced            │  ← TabList
├─────────────────────────────────────────────────┤
│                                                  │
│  (tab content, overflow-y: auto, max-height 400) │
│                                                  │
├─────────────────────────────────────────────────┤
│                          [Cancel] [Reset] [Save] │  ← Footer
└──────────────────────────────────────────────────┘
```

### 20.3 设置行布局

```
[Label 120px 右对齐, 11px, neutralFg2] [控件 flex:1, height 28px]
[描述 9px, neutralFg4, margin-left: 130px]        ← 可选
```

### 20.4 保存逻辑

1. 打开对话框 → `useSettingsStore.load()` → 保存当前设置到 `snapshot`
2. 修改任意设置 → `isDirty = true`
3. Save: `invoke("save_settings", settings)` → 写入 appsettings.json → 应用主题
4. Cancel: `settings = snapshot`, `isDirty = false`
5. Reset: 确认对话框 → 恢复出厂默认 → `isDirty = true`

---

# 第五部分: 覆盖标记系统

## 21. 四级覆盖层级

参数值按照以下优先级解析 (来自后端 ParameterResolver):

```
┌────────────────────────────┐
│ Image Override (优先级 3)   │ ← 最高优先级, 单张图片独立覆盖
├────────────────────────────┤
│ Group Override (优先级 2)   │ ← 分组条件覆盖, 最后匹配的组胜出
├────────────────────────────┤
│ Template Default (优先级 1) │ ← 管线模板定义层
├────────────────────────────┤
│ Plugin Builtin (优先级 0)   │ ← 最低优先级, 插件代码默认值
└────────────────────────────┘
```

## 22. ContextBar 组件

```typescript
interface ContextBarProps {
  scopes: ScopeItem[];        // [{ id: 'all', label: 'All' }, { id: 'template', label: 'Template' }, ...]
  activeScope: OverrideScope;
  onScopeChange: (scope: OverrideScope, groupName?: string, imageIndex?: number) => void;
}

interface ScopeItem {
  id: string;                 // 'all' | 'template' | 'group:{name}' | 'image:{index}'
  label: string;
  type: 'all' | 'template' | 'group' | 'image';
}
```

## 23. OverrideDot 组件

```typescript
interface OverrideDotProps {
  state: 'inherited' | 'override' | 'expression';
  canEdit: boolean;          // All scope 时不可编辑
  onActivate: () => void;    // 继承 → 覆盖
  onRestore: () => void;     // 覆盖 → 继承
  onExpressionEdit: () => void;  // 覆盖 → 表达式
}
```

**状态转换**:
```
INHERITED(⬜) ──onActivate()──→ OVERRIDE(🟡) ──onExpressionEdit()──→ EXPRESSION(🔵)
     ↑                              │                                      │
     └────── onRestore() ───────────┘                                      │
     └────────────────────────── onRestore() ──────────────────────────────┘
```

## 24. ExpressionEditor 组件

```typescript
interface ExpressionEditorProps {
  nodeId: string;
  paramId: string;
  variables: VariableDef[];   // 可用变量列表
  expression: string;          // 当前表达式
  previews: { image: string; result: number }[];  // 实时预览
  onChange: (expression: string) => void;
  onClose: () => void;
}
```

## 25. 多选覆盖处理

当 ContextBar 切换到 Image 且选中多张图片时:

```typescript
interface MultiSelectOverrideState {
  varyingParams: Set<string>;           // 值不一致的参数
  uniformParams: Map<string, any>;      // 值一致的参数
  actions: {
    unify: (paramId: string, value: any) => void;   // 统一设置
    clearAll: (paramId: string) => void;             // 清除全部覆盖
  };
}
```

---

# 第六部分: 状态管理

## 26-34. Zustand 架构与 7 个 Store

(每个 Store 包含完整的 State interface、Actions interface、初始状态、中间件配置、测试策略。由于篇幅, 此处列出接口签名, 完整实现参见 `gui/src/stores/`。)

### useAppStore
```typescript
interface AppState { mode, theme, isBackendConnected, statusMessage }
interface AppActions { setMode, setTheme, setBackendStatus, setStatusMessage }
```

### useFilmstripStore
```typescript
interface FilmstripState { images, selectedIndices(Set), selectionAnchor, groups, sortKey, thumbnailSize, thumbnails(Map), isLoading }
interface FilmstripActions { importImages, removeImages, toggleSelect, selectAll, clearSelection, addToBatch, createGroup, deleteGroup, autoGroup, updateThumbnail, setSortKey, setThumbnailSize }
```

### usePluginStore
```typescript
interface PluginState { plugins, categories, nodeSchemas(Map), searchQuery, categoryFilter }
interface PluginActions { fetchPlugins, fetchNodeSchema }
// getter: filteredPlugins
```

### usePipelineStore
```typescript
interface PipelineState { nodes, edges, selectedNodeId, zoom, panOffset, isDirty, executionState, nodeExecutionStates(Map<id, status>), undoStack, redoStack, currentFilePath }
interface PipelineActions { addNode, removeNode, moveNode, connectEdge, removeEdge, selectNode, newPipeline, savePipeline, loadPipeline, validatePipeline, executePipeline, cancelExecution, undo, redo, setZoom, setPan }
```

### useOverrideStore
```typescript
interface OverrideState { scope, activeGroupName, activeImageIndex, overrides(Map), expressions(Map), dirtyParams(Set) }
interface OverrideActions { setScope, setOverride, clearOverride, setExpression, getEffectiveValue, getSectionOverrideInfo, hasVaryingValues }
```

### useBatchStore
```typescript
interface BatchState { queue, batchState, progress, outputSettings, perImageOverrides(Map) }
interface BatchActions { addToQueue, removeFromQueue, startBatch, pauseBatch, resumeBatch, stopBatch, clearDone, setOutputSetting, setPerImageOverride }
```

### useSettingsStore
```typescript
interface SettingsState { settings, snapshot, isDirty }
interface SettingsActions { load, save, reset, cancel, update }
```

### 跨 Store 交互协议

```
FilmstripStore.addToBatch() → BatchStore.addToQueue()
PipelineStore.selectNode()  → PluginStore.fetchNodeSchema() + OverrideStore.initForNode()
OverrideStore.setOverride() → PipelineStore.markDirty()
BatchStore.startBatch()     → invoke("start_batch") → event → BatchStore.updateProgress()
SettingsStore.save()        → invoke("save_settings") → AppStore.setTheme()
```

---

# 第七至十一部分

## 35-58. 通信架构、数据流、状态覆盖、交互模式、开发实施

(完整章节包含: 通信分层模型详细说明、13个 Command 的完整 Rust 实现伪代码、6个 Event 的完整 payload 定义、`useTauriCommand`/`useTauriEvent` 的完整实现代码、6个完整数据流的时序图、7×4 状态覆盖矩阵的详尽表格、所有交互模式的完整事件处理代码、55个键盘快捷键的完整定义表、6个开发阶段的详细任务分解、组件的实施优先级排序、性能优化策略——虚拟列表配置、Canvas 离屏渲染、React.memo 策略、gRPC 连接复用)

---

## 附录

### A. 与后端文档对齐清单
- 架构三层分离: ✓ (doc/ARCHITECTURE_DESIGN.md §3.1)
- 两条数据通道: ✓ (doc/ARCHITECTURE_DESIGN.md §2.3)
- Schema 驱动 UI: ✓ (doc/ARCHITECTURE_DESIGN.md §2.4)
- 四级参数覆盖: ✓ (doc/ARCHITECTURE_DESIGN.md §13)
- Configuration-driven: ✓ (doc/ARCHITECTURE_DESIGN.md §5)

### B. 术语表
| 术语 | 英文 | 定义 |
|------|------|------|
| 胶片条 | Filmstrip | 左侧图片候选列表 |
| DAG 画布 | DAG Canvas | 中间管线节点编辑器 |
| 覆盖 | Override | 在特定层级修改参数默认值 |
| 辅助视图 | Auxiliary View | 直方图/波形图等分析工具 |
| 批量模式 | Batch Mode | 多图片批量处理视图 |

### C. 参考文档
- `design_review/SPEC_MainWindow.md` — 主窗口布局规格
- `design_review/SPEC_Filmstrip.md` — 胶片条详细设计
- `design_review/SPEC_DAG.md` — DAG 画布详细设计
- `design_review/SPEC_Batch.md` — 批量模式详细设计
- `design_review/SPEC_Settings.md` — 设置对话框设计
- `design_review/SPEC_Plugins.md` — 14 插件 UI 规格
- `design_review/SPEC_Plugin_01~14` — 各插件详细规格
- `doc/ARCHITECTURE_DESIGN.md` — 后端架构设计
- `doc/INTERFACE_DESIGN.md` — 后端接口设计
- `doc/BACKEND_USER_GUIDE.md` — 后端用户指南

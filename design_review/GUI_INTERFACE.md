# Photopipeline GUI 接口文档

> **版本**: 2.0
> **状态**: 详细设计
> **最后更新**: 2026-05-29
> **对齐后端**: doc/INTERFACE_DESIGN.md v1.0 (feat/unified-binary, 2026-05-28)
> **技术栈**: Tauri v2 + React 19 + @fluentui/react-components v9

---

## 目录

1. [架构与通信概述](#1-架构与通信概述)
2. [外部接口: 后端 gRPC API](#2-外部接口-后端-grpc-api)
   - 2.1 [服务总览与 Proto 完整定义](#21-服务总览与-proto-完整定义)
   - 2.2 [PluginService](#22-pluginservice)
   - 2.3 [ImageService](#23-imageservice)
   - 2.4 [PipelineService](#24-pipelineservice)
   - 2.5 [BatchService](#25-batchservice)
   - 2.6 [ExecutionService v2](#26-executionservice-v2)
3. [外部接口: PipelineConfig 文件格式](#3-外部接口-pipelineconfig-文件格式)
4. [外部接口: ParameterSchema 格式 (21 种 ValueType)](#4-外部接口-parameterschema-格式)
5. [外部接口: GuiSchema 格式](#5-外部接口-guischema-格式)
6. [内部接口: Tauri Commands (13 个)](#6-内部接口-tauri-commands)
7. [内部接口: Tauri Events (6 个)](#7-内部接口-tauri-events)
8. [内部接口: Zustand Stores (7 个)](#8-内部接口-zustand-stores)
9. [内部接口: React 组件 Props](#9-内部接口-react-组件-props)
10. [内部接口: TypeScript 类型定义](#10-内部接口-typescript-类型定义)
11. [错误代码体系](#11-错误代码体系)
12. [接口交互流程](#12-接口交互流程)
附录A: [前后端接口对齐检查清单](#附录a-前后端接口对齐检查清单)
附录B: [类型别名对照表](#附录b-类型别名对照表)

---

## 1. 架构与通信概述

### 1.1 系统整体架构

Photopipeline 采用三层分离架构，GUI 前端通过 Tauri Rust 桥接层与后端 gRPC 服务通信。

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Photopipeline GUI                              │
│                        (Tauri v2 Desktop App)                         │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    React 19 Frontend                         │    │
│  │                                                              │    │
│  │  ┌──────────┐ ┌───────────┐ ┌────────────┐ ┌─────────────┐ │    │
│  │  │PluginGrid│ │ DagEditor │ │ControlPanel│ │ Filmstrip    │ │    │
│  │  │          │ │ (React    │ │(Parameter  │ │ Viewer       │ │    │
│  │  │ 插件浏览器│ │  Flow)    │ │ Panel)     │ │ 胶片浏览器   │ │    │
│  │  └────┬─────┘ └─────┬─────┘ └──────┬─────┘ └──────┬──────┘ │    │
│  │       │             │              │              │         │    │
│  │  ┌────┴─────────────┴──────────────┴──────────────┴──────┐ │    │
│  │  │              Zustand Stores (7 个)                     │ │    │
│  │  │  useAppStore / useFilmstripStore / usePluginStore     │ │    │
│  │  │  usePipelineStore / useOverrideStore / useBatchStore  │ │    │
│  │  │  useSettingsStore                                      │ │    │
│  │  └────────────────────────┬───────────────────────────────┘ │    │
│  │                           │ invoke / listen                  │    │
│  └───────────────────────────┼──────────────────────────────────┘    │
│                              │                                       │
│  ┌───────────────────────────┼──────────────────────────────────┐    │
│  │                    Tauri Rust 桥接层                          │    │
│  │                                                              │    │
│  │  ┌────────────────┐  ┌──────────────┐  ┌──────────────────┐ │    │
│  │  │ Commands (13)  │  │ Events (6)   │  │ gRPC Clients (4) │ │    │
│  │  │ #[tauri::       │  │ app.emit()   │  │ tonic::          │ │    │
│  │  │   command]      │  │              │  │  PluginService   │ │    │
│  │  │                │  │              │  │  ImageService    │ │    │
│  │  │                │  │              │  │  PipelineService │ │    │
│  │  │                │  │              │  │  BatchService    │ │    │
│  │  └────────────────┘  └──────────────┘  └──────────────────┘ │    │
│  └───────────────────────────┼──────────────────────────────────┘    │
└──────────────────────────────┼───────────────────────────────────────┘
                               │ localhost:50051 (gRPC over HTTP/2)
                               │
┌──────────────────────────────┼───────────────────────────────────────┐
│                    Backend (photopipeline serve)                      │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                   gRPC Server (tonic)                         │   │
│  │  PluginService │ ImageService │ PipelineService │ BatchService│   │
│  └────────────────────────┬─────────────────────────────────────┘   │
│                           │                                          │
│  ┌────────────────────────┴─────────────────────────────────────┐   │
│  │                     Engine Layer                              │   │
│  │  PipelineGraph / PipelineTemplate / NodeExecutor             │   │
│  │  ParameterResolver / TileEngine                               │   │
│  └────────────────────────┬─────────────────────────────────────┘   │
│                           │                                          │
│  ┌────────────────────────┴─────────────────────────────────────┐   │
│  │                   Plugin System                               │   │
│  │  Registry / Plugin Trait / ParameterSchema / ProgressSink    │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.2 两条通信通道

与后端架构文档 §2.3 完全对齐，GUI 通过两条通道与后端交互：

| 通道 | 内容 | 传输格式 | 协议/方式 | 触发方式 | 前端处理层 |
|------|------|---------|----------|---------|-----------|
| **文件系统** | PipelineConfig JSON | 持久化 JSON/TOML 文件 | `std::fs` 读写 | GUI 用户保存/加载操作 | Tauri Rust `fs` 模块 |
| **gRPC** (localhost:50051) | 插件发现, 图片加载/解码/缩略图, 管线创建/执行/验证, 批量提交/进度 | Protocol Buffers (HTTP/2) | tonic streaming | 用户操作触发 | Tauri Rust `tonic` 客户端 -> `invoke`/`event` |

### 1.3 接口调用链详解

```
                      [前端 TypeScript]                     [Tauri Rust]                   [Backend gRPC]

  1. 用户操作
     │
     ▼
  2. React Component
     │  e.g., RunButton.onClick()
     ▼
  3. Store Action
     │  usePipelineStore.executePipeline()
     ▼
  4. invoke("execute_pipeline", args)
     │  @tauri-apps/api/core
     ▼
  5. #[tauri::command]               ──────────────────►
     │  fn execute_pipeline(...)                        6. gRPC Client Call
     │                                                   tonic::Request::new(req)
     │                                                   client.create_pipeline(req)
     │                                                       │
     │                                                       ▼
     │                                                  7. Backend gRPC Server
     │                                                     handle CreatePipeline
     │                                                     return PipelineId
     │                                                       │
     │                                                  8. spawn: execute stream loop
     │                                                     client.execute(req)
     │                                                     for each msg in stream:
     ▼                                                        │
  9. Store Update                                              │
     │  usePipelineStore.                 ◄── emit ───────────┘
     │    updateExecutionState(msg)          app.emit("pipeline-progress", msg)
     ▼
 10. React Re-render
     │  DAGNode status light updates
     │  StatusBar progress bar updates
     ▼
 11. UI 呈现给用户
```

### 1.4 技术栈版本清单

| 层级 | 组件 | 版本/技术 | 职责 |
|------|------|----------|------|
| 桌面壳 | Tauri | v2 | 窗口管理, 系统托盘, 原生对话框, Rust 桥接 |
| 前端框架 | React | 19 | 组件化 UI, Hooks, 状态管理 |
| UI 组件库 | @fluentui/react-components | v9 | 微软 Fluent Design System 控件 |
| 状态管理 | Zustand | v5 | 轻量级全局状态, 7 个独立 Store |
| DAG 编辑器 | @xyflow/react (React Flow) | v12 | 节点图形编辑器, 缩放/平移/连线 |
| 数据请求 | @tauri-apps/api/core | v2 | `invoke()` / `listen()` 封装 |
| Rust 桥接 | tauri | v2 | `#[tauri::command]` / `AppHandle::emit()` |
| gRPC 客户端 | tonic | 0.12 | Rust 端 HTTP/2 gRPC 调用 |
| 后端服务 | photopipeline serve | Rust | gRPC Server (localhost:50051) |

### 1.5 前端目录结构与后端对应

```
gui/                                  # Tauri 前端项目根
├── src-tauri/                        # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs                   # Tauri 入口, 启动后端进程
│   │   ├── commands/                 # #[tauri::command] 定义
│   │   │   ├── mod.rs
│   │   │   ├── plugin_cmds.rs        # list_plugins, get_node_schema
│   │   │   ├── image_cmds.rs         # load_images, get_thumbnail, decode_preview
│   │   │   ├── pipeline_cmds.rs      # save/load_pipeline_file, validate/execute_pipeline
│   │   │   ├── batch_cmds.rs         # start_batch, cancel_batch
│   │   │   └── settings_cmds.rs      # save_settings, load_settings
│   │   ├── grpc/                     # tonic gRPC 客户端封装
│   │   │   ├── mod.rs
│   │   │   ├── plugin_client.rs      # PluginServiceClient
│   │   │   ├── image_client.rs       # ImageServiceClient
│   │   │   ├── pipeline_client.rs    # PipelineServiceClient
│   │   │   └── batch_client.rs       # BatchServiceClient
│   │   └── backend.rs               # 后端进程管理 (spawn, health check, kill)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                              # React 前端
│   ├── main.tsx                      # React 入口
│   ├── App.tsx                       # 根组件, 路由/布局
│   ├── components/
│   │   ├── layout/                   # 布局组件 (TitleBar, StatusBar, Panel, etc.)
│   │   ├── dag/                      # DAG 编辑器 (DAGNode, DAGEdge, DagCanvas)
│   │   ├── plugin/                   # 插件相关 (PluginGrid, PluginCard, PluginHeader)
│   │   ├── params/                   # 参数面板 (ControlPanel, ParamSection, ParamRow)
│   │   ├── image/                    # 图片相关 (Filmstrip, ImageCard, PreviewView)
│   │   ├── batch/                    # 批量处理 (BatchPanel, BatchQueueRow)
│   │   ├── settings/                 # 设置 (SettingsDialog, SettingsTab)
│   │   └── common/                   # 通用组件 (Toast, Dialog, ContextMenu)
│   ├── stores/                       # Zustand Stores
│   │   ├── useAppStore.ts
│   │   ├── useFilmstripStore.ts
│   │   ├── usePluginStore.ts
│   │   ├── usePipelineStore.ts
│   │   ├── useOverrideStore.ts
│   │   ├── useBatchStore.ts
│   │   └── useSettingsStore.ts
│   ├── hooks/                        # 自定义 Hooks
│   │   ├── useTauriCommand.ts        # invoke 封装 (loading/error 状态)
│   │   ├── useTauriEvent.ts          # listen 封装 (自动清理)
│   │   └── useThumbnail.ts           # 缩略图加载 + 缓存
│   ├── types/                        # TypeScript 类型定义
│   │   ├── plugin.ts                 # PluginEntry, NodeSchemaResponse
│   │   ├── pipeline.ts               # DAGNodeData, DAGEdgeData, PipelineConfig
│   │   ├── image.ts                  # ImageInfo, MetadataInfo, ImageData
│   │   ├── params.ts                 # ParameterSchema, ParameterField, GuiSchema
│   │   ├── override.ts               # OverrideScope, OverrideEntry, ValueWithSource
│   │   ├── batch.ts                  # BatchItem, BatchProgress, BatchOutputSettings
│   │   ├── settings.ts               # AppSettings
│   │   ├── events.ts                 # PipelineProgressPayload, BatchProgressPayload
│   │   └── errors.ts                 # ErrorCode, gRpcStatus, PluginError
│   └── utils/
│       ├── pipeline-config.ts        # PipelineConfig 序列化/反序列化
│       ├── param-resolver.ts         # 前端参数解析逻辑 (OverrideSource 优先级)
│       ├── expression-engine.ts      # 表达式求值 (前端条件/表达式)
│       └── dag-validator.ts          # DAG BFS 环检测 + 连通性验证
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 2. 外部接口: 后端 gRPC API

> 本节与后端 `doc/INTERFACE_DESIGN.md` §2 完全对齐。所有 Proto 定义、字段编号、类型、约束均与后端一致。

### 2.1 服务总览与 Proto 完整定义

后端通过 4 个 gRPC Service 暴露 13 个 RPC 方法，统一监听 `localhost:50051`。前端 Tauri Rust 层通过 4 个 `tonic` 客户端分别连接。

#### 2.1.1 服务总览表

| 服务 | Proto 包名 | proto 文件 | RPC 数量 | 返回类型 | 前端客户端 |
|------|-----------|-----------|---------|---------|-----------|
| `PluginService` | `photopipeline.plugin` | `proto/plugin.proto` | 2 | 单次 | `PluginServiceClient` |
| `ImageService` | `photopipeline.image` | `proto/image.proto` | 4 | 2单次+2流 | `ImageServiceClient` |
| `PipelineService` | `photopipeline.pipeline` | `proto/pipeline.proto` | 4 | 3单次+1流 | `PipelineServiceClient` |
| `BatchService` | `photopipeline.batch` | `proto/batch.proto` | 3 | 2单次+1流 | `BatchServiceClient` |

#### 2.1.2 全部 RPC 一览

| # | 服务 | RPC 方法 | 请求消息 | 响应消息 | 返回方式 | 前端调用时机 |
|---|------|---------|---------|---------|---------|-------------|
| 1 | PluginService | `ListPlugins` | `google.protobuf.Empty` | `PluginCatalogResponse` | 单次 | App 启动 / 点击 Refresh |
| 2 | PluginService | `GetNodeSchema` | `PluginIdRequest` | `NodeSchemaResponse` | 单次 | 选中 DAGNode / 浏览插件详情 |
| 3 | ImageService | `Load` | `ImagePath` | `ImageInfo` | 单次 | 导入图片后逐张加载元信息 |
| 4 | ImageService | `Decode` | `DecodeRequest` | `stream PixelDataChunk` | 流 | 预览视图 / 辅助视图数据 |
| 5 | ImageService | `GetThumbnail` | `ThumbnailRequest` | `ImageData` | 单次 | ImageCard 缩略图加载 |
| 6 | ImageService | `Encode` | `EncodeRequest` | `stream EncodeProgress` | 流 | 输出编码 (暂不直接调用) |
| 7 | PipelineService | `CreatePipeline` | `PipelineSpec` | `PipelineId` | 单次 | 管线执行前创建 |
| 8 | PipelineService | `Execute` | `ExecuteRequest` | `stream ExecuteProgress` | 流 | 用户点击 Run |
| 9 | PipelineService | `Validate` | `PipelineSpec` | `ValidationResult` | 单次 | 用户点击 Validate / 执行前自动 |
| 10 | PipelineService | `GetNodeSchema` | `PluginId` | `NodeSchema` | 单次 | 备选 schema 查询通道 |
| 11 | BatchService | `SubmitBatch` | `BatchSpec` | `BatchId` | 单次 | 用户点击 Start Batch |
| 12 | BatchService | `GetProgress` | `BatchId` | `stream BatchProgress` | 流 | SubmitBatch 后自动订阅 |
| 13 | BatchService | `Cancel` | `BatchId` | `google.protobuf.Empty` | 单次 | 用户点击 Stop / Cancel |

### 2.2 PluginService

完整 Proto 定义:

```protobuf
syntax = "proto3";
package photopipeline.plugin;

import "google/protobuf/empty.proto";
import "google/protobuf/struct.proto";

service PluginService {
  rpc ListPlugins(google.protobuf.Empty) returns (PluginCatalogResponse);
  rpc GetNodeSchema(PluginIdRequest) returns (NodeSchemaResponse);
}
```

#### 2.2.1 ListPlugins

**用途**: 获取所有已注册插件的完整目录，前端用此接口构建插件选择器 (PluginGrid/PluginCard)，是所有插件相关 UI 的数据源。

**请求**: `google.protobuf.Empty` -- 无参数

**响应**: `PluginCatalogResponse`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `plugins` | `repeated PluginEntry` | 1 | 是 | 已注册插件完整列表 | 按 `category` 分组后按 `name` 字母序排列 |
| `categories` | `repeated string` | 2 | 是 | 所有存在的类别名称 | 去重，字母序排列 |

**PluginEntry**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 | 前端使用 |
|------|------|------|------|------|------|---------|
| `id` | `string` | 1 | 是 | 插件全局唯一标识 | 点分隔命名, 如 `"photopipeline.plugins.ai_denoise"` | `GetNodeSchema` 参数, DAGNode 关联 |
| `name` | `string` | 2 | 是 | 面向用户的显示名称 | | PluginCard 标题, PluginGrid 显示 |
| `version` | `string` | 3 | 是 | 语义化版本 | 格式 `"MAJOR.MINOR.PATCH"` | PluginHeader 版本显示 |
| `category` | `string` | 4 | 是 | 插件类别 | 见 PluginCategory 枚举 | PluginGrid 分组依据, 分类筛选 |
| `description` | `string` | 5 | 是 | 插件功能描述 (Markdown) | | PluginHeader 描述 + Tooltip |
| `tags` | `repeated string` | 6 | 否 | 搜索标签列表 | | 搜索匹配 (大小写不敏感) |
| `requires_pixel_access` | `bool` | 7 | 是 | 是否需要访问像素数据 | | 确定节点端口类型 (输入/输出端口) |
| `requires_network` | `bool` | 8 | 是 | 是否需要网络访问 | | 离线模式警告图标 |
| `requires_filesystem` | `bool` | 9 | 是 | 是否需要文件系统访问 | | 沙箱权限提示图标 |
| `min_ram_mb` | `uint64` | 10 | 是 | 最低内存要求 (MB) | | PluginHeader 硬件需求显示 |

**前端使用流程**:
```
App 启动 → usePluginStore.fetchPlugins()
  → invoke("list_plugins")
  → gRPC PluginServiceClient.list_plugins(Empty)
  → PluginCatalogResponse
  → 缓存到 usePluginStore.plugins + usePluginStore.categories
  → PluginGrid 渲染分类 + 卡片
```

**完整 JSON 示例 (响应)**:
```json
{
  "plugins": [
    {
      "id": "photopipeline.plugins.raw_decoder",
      "name": "RAW Decoder",
      "version": "2.3.0",
      "category": "input",
      "description": "Decode RAW files from major camera manufacturers using LibRaw. Supports all common Bayer and X-Trans sensor layouts.",
      "tags": ["raw", "decode", "demosaic", "libraw"],
      "requires_pixel_access": false,
      "requires_network": false,
      "requires_filesystem": true,
      "min_ram_mb": 256
    },
    {
      "id": "photopipeline.plugins.bilateral_denoise",
      "name": "Bilateral Denoise",
      "version": "1.2.0",
      "category": "enhance",
      "description": "Edge-preserving bilateral noise reduction with spatial and tonal sigma controls.",
      "tags": ["denoise", "bilateral", "edge-preserving"],
      "requires_pixel_access": true,
      "requires_network": false,
      "requires_filesystem": false,
      "min_ram_mb": 512
    },
    {
      "id": "photopipeline.plugins.unsharp_mask",
      "name": "Unsharp Mask",
      "version": "1.0.1",
      "category": "enhance",
      "description": "Classic unsharp mask sharpening with amount, radius, and threshold controls.",
      "tags": ["sharpen", "unsharp-mask", "detail"],
      "requires_pixel_access": true,
      "requires_network": false,
      "requires_filesystem": false,
      "min_ram_mb": 256
    },
    {
      "id": "photopipeline.plugins.colorspace_convert",
      "name": "Colorspace Converter",
      "version": "1.5.0",
      "category": "color",
      "description": "Convert between color spaces with configurable rendering intent and gamut mapping.",
      "tags": ["color", "colorspace", "icc", "ocio"],
      "requires_pixel_access": true,
      "requires_network": false,
      "requires_filesystem": false,
      "min_ram_mb": 128
    },
    {
      "id": "photopipeline.plugins.heif_encoder",
      "name": "HEIF Encoder",
      "version": "3.1.0",
      "category": "format",
      "description": "High Efficiency Image Format encoder with 8/10/12-bit support, HDR metadata, and chroma subsampling options.",
      "tags": ["heif", "heic", "encode", "hdr", "av1"],
      "requires_pixel_access": false,
      "requires_network": false,
      "requires_filesystem": true,
      "min_ram_mb": 512
    }
  ],
  "categories": ["color", "enhance", "format", "input", "metadata", "transform"]
}
```

#### 2.2.2 GetNodeSchema

**用途**: 获取特定插件的参数模式 (ParameterSchema) 和 GUI 布局模式 (GuiSchema)，前端据此动态渲染参数面板 (ControlPanel)。这是 GUI 动态参数系统的核心接口。

**请求**: `PluginIdRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `id` | `string` | 1 | 是 | 插件唯一标识 | 必须与 `ListPlugins` 返回的 `id` 完全一致 |

**响应**: `NodeSchemaResponse`

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端处理 |
|------|------|------|------|------|---------|
| `plugin_id` | `string` | 1 | 是 | 插件 ID | 关联 DAGNode.pluginId |
| `name` | `string` | 2 | 是 | 显示名称 | PluginHeader 标题 |
| `version` | `string` | 3 | 是 | 版本号 | PluginHeader 副标题 |
| `category` | `string` | 4 | 是 | 类别 | PluginHeader 分类标签 |
| `description` | `string` | 5 | 是 | 描述文本 (Markdown) | PluginHeader 描述区域 |
| `parameter_schema` | `google.protobuf.Struct` | 6 | 是 | JSON 序列化的 ParameterSchema | `JSON.parse` -> ControlPanel 动态渲染 |
| `gui_schema` | `google.protobuf.Struct` | 7 | 是 | JSON 序列化的 GuiSchema | `JSON.parse` -> ControlPanel 布局 + PreviewView 配置 |

**错误条件**:
| gRPC Status | 条件 | 前端处理 |
|-------------|------|---------|
| `NOT_FOUND` | 插件 ID 不存在于 Registry 中 | Toast: "Plugin not found: {id}", ControlPanel 显示空状态 |

**前端调用时机**:
1. 用户选中 DAGNode -> `usePipelineStore.selectNode(id)` -> `usePluginStore.fetchNodeSchema(pluginId)`
2. 用户在 PluginGrid 中浏览插件详情 (双击/右键 Inspect)

**完整 JSON 示例 (响应)**:
```json
{
  "plugin_id": "photopipeline.plugins.bilateral_denoise",
  "name": "Bilateral Denoise",
  "version": "1.2.0",
  "category": "enhance",
  "description": "Edge-preserving bilateral noise reduction with spatial and tonal sigma controls. Suitable for high-ISO RAW images.",
  "parameter_schema": {
    "version": 1,
    "sections": [
      {
        "id": "basic",
        "label": "Basic Parameters",
        "description": "Core noise reduction settings",
        "icon": "settings",
        "collapsible": false,
        "default_collapsed": false,
        "fields": [
          {
            "id": "sigma_color",
            "label": "Color Sigma",
            "description": "Tonal range for pixel averaging. Larger values blur more across color boundaries.",
            "help_url": "https://docs.photopipeline.dev/plugins/denoise#sigma-color",
            "type": "float",
            "min": 0.0,
            "max": 100.0,
            "step": 0.1,
            "precision": 1,
            "unit": "px",
            "logarithmic": false,
            "style": "slider",
            "default": 10.0,
            "required": true,
            "advanced": false,
            "allow_override": true,
            "supports_expression": true
          },
          {
            "id": "sigma_spatial",
            "label": "Spatial Sigma",
            "description": "Spatial range for pixel averaging. Larger values smooth larger areas.",
            "help_url": "https://docs.photopipeline.dev/plugins/denoise#sigma-spatial",
            "type": "float",
            "min": 0.0,
            "max": 50.0,
            "step": 0.1,
            "precision": 1,
            "unit": "px",
            "logarithmic": false,
            "style": "slider",
            "default": 3.0,
            "required": true,
            "advanced": false,
            "allow_override": true,
            "supports_expression": true
          }
        ]
      },
      {
        "id": "advanced",
        "label": "Advanced",
        "description": "Fine-tuning parameters",
        "icon": "tune",
        "collapsible": true,
        "default_collapsed": true,
        "fields": [
          {
            "id": "kernel_size",
            "label": "Kernel Size",
            "description": "Size of the bilateral filter kernel. Larger kernels are slower but more accurate.",
            "type": "integer",
            "min": 3,
            "max": 31,
            "step": 2,
            "unit": "px",
            "style": "combo",
            "default": 9,
            "required": false,
            "advanced": true,
            "allow_override": false,
            "supports_expression": false
          },
          {
            "id": "border_mode",
            "label": "Border Handling",
            "description": "How to handle pixels near image borders",
            "type": "enum",
            "options": [
              {"value": "reflect", "label": "Reflect", "description": "Mirror edge pixels", "tags": ["quality"], "recommended": true},
              {"value": "clamp", "label": "Clamp", "description": "Repeat edge pixels", "tags": ["speed"], "recommended": false},
              {"value": "zero", "label": "Zero", "description": "Fill with black", "tags": [], "recommended": false}
            ],
            "display": "dropdown",
            "default": "reflect",
            "required": false,
            "advanced": true,
            "allow_override": false,
            "supports_expression": false
          }
        ]
      }
    ]
  },
  "gui_schema": {
    "layout": {"Standard": {"sections": []}},
    "icon": "denoise",
    "color": "#4A90D9",
    "preview": "live",
    "aux_views": ["histogram", "waveform"],
    "min_panel_width": 340
  }
}
```

### 2.3 ImageService

完整 Proto 定义:

```protobuf
syntax = "proto3";
package photopipeline.image;

service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);
  rpc Decode(DecodeRequest) returns (stream PixelDataChunk);
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);
  rpc GetThumbnail(ThumbnailRequest) returns (ImageData);
}
```

#### 2.3.1 Load

**用途**: 加载图像文件的元信息和基本属性，不解码像素数据。这是导入图像的第一步，每次导入一张图片调用一次。

**请求**: `ImagePath`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `path` | `string` | 1 | 是 | 图像文件绝对路径 | 必须存在且可读, Windows/Mac/Linux 均支持 |

**响应**: `ImageInfo`

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端使用 |
|------|------|------|------|------|---------|
| `id` | `string` | 1 | 是 | 会话级唯一 UUID | 图片标识, 覆盖操作 |
| `path` | `string` | 2 | 是 | 原始绝对路径 | 后续 Load/Decode 请求复用 |
| `filename` | `string` | 3 | 是 | 文件名 (含扩展名) | Filmstrip/ImageCard 显示 |
| `format` | `string` | 4 | 是 | 图像格式标识字符串 | ImageCard 格式标签 |
| `width` | `uint32` | 5 | 是 | 像素宽度 | Metadata 表格, 分组条件 |
| `height` | `uint32` | 6 | 是 | 像素高度 | Metadata 表格, 分组条件 |
| `file_size_bytes` | `uint64` | 7 | 是 | 文件大小 (字节) | ImageCard 信息, 排序 |
| `pixel_format` | `string` | 8 | 是 | 原始像素格式 | ImageCard 技术信息 |
| `color_space` | `string` | 9 | 是 | 颜色空间描述 | ImageCard 技术信息 |
| `metadata` | `MetadataInfo` | 10 | 否 | 摘要 EXIF/GPS 元数据 | ImageCard 详情面板, 条件分组 |

**MetadataInfo**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端使用 |
|------|------|------|------|------|---------|
| `make` | `optional string` | 1 | 否 | 相机制造商 | 条件表达式 `exif.make` |
| `model` | `optional string` | 2 | 否 | 相机型号 | 元数据表格 |
| `lens_model` | `optional string` | 3 | 否 | 镜头型号 | 元数据表格 |
| `date_time_original` | `optional string` | 4 | 否 | 拍摄时间 (ISO 8601) | 时间线排序 |
| `exposure_time` | `optional string` | 5 | 否 | 曝光时间 (如 "1/125") | 元数据表格 |
| `f_number` | `optional string` | 6 | 否 | 光圈值 (如 "5.6") | 元数据表格 |
| `iso` | `optional uint32` | 7 | 否 | ISO 感光度 | 条件表达式 `exif.iso`, 排序 |
| `focal_length` | `optional string` | 8 | 否 | 焦距 (如 "50mm") | 条件表达式 `exif.focal_length` |
| `latitude` | `optional double` | 9 | 否 | GPS 纬度 | 条件表达式 `gps.near()`, 地图显示 |
| `longitude` | `optional double` | 10 | 否 | GPS 经度 | 条件表达式 `gps.near()`, 地图显示 |
| `altitude` | `optional double` | 11 | 否 | GPS 海拔 (米) | 元数据表格 |

**错误条件**:
| gRPC Status | 条件 | 前端处理 |
|-------------|------|---------|
| `NOT_FOUND` | 文件路径不存在 | Toast error: "File not found: {path}", 从 Filmstrip 移除 |
| `INTERNAL` | 无法读取或解析文件头 | Toast error: "Failed to load image: {reason}", 跳过该文件 |

**前端使用流程**:
```
用户 Import -> dialog.open({multiple, filters:[image]}) -> 获取 paths[]
  -> for each path:
    invoke("load_images", { paths: [path] })
    -> gRPC ImageServiceClient.load(ImagePath { path })
    -> ImageInfo -> useFilmstripStore.images.push(info)
```

**完整 JSON 示例**:
```json
{
  "请求": {"path": "/home/user/photos/DSC001.ARW"},
  "响应": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "path": "/home/user/photos/DSC001.ARW",
    "filename": "DSC001.ARW",
    "format": "raw",
    "width": 6000,
    "height": 4000,
    "file_size_bytes": 49807360,
    "pixel_format": "u16",
    "color_space": "srgb",
    "metadata": {
      "make": "Sony",
      "model": "ILCE-7RM5",
      "lens_model": "FE 24-70mm F2.8 GM II",
      "date_time_original": "2025-03-15T14:22:08Z",
      "exposure_time": "1/500",
      "f_number": "5.6",
      "iso": 400,
      "focal_length": "24mm",
      "latitude": 34.0522,
      "longitude": -118.2437,
      "altitude": 87.5
    }
  }
}
```

#### 2.3.2 Decode

**用途**: 将图像文件解码为原始像素数据，通过流分块返回。前端使用此接口获取预览视图和辅助视图 (直方图/波形/矢量示波器) 所需的像素数据。

**请求**: `DecodeRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 | 前端默认值 |
|------|------|------|------|------|------|-----------|
| `path` | `string` | 1 | 是 | 图像文件路径 | 必须存在 | -- |
| `pixel_format` | `optional string` | 2 | 否 | 目标像素格式 | `"u8"` / `"u16"` / `"f16"` / `"f32"` | `"u8"` (预览) |
| `max_width` | `optional uint32` | 3 | 否 | 最大宽度 | 超过时等比缩放 | 预览面板宽度 |
| `max_height` | `optional uint32` | 4 | 否 | 最大高度 | 超过时等比缩放 | 预览面板高度 |
| `read_metadata` | `bool` | 5 | 是 | 是否同时提取元数据 | | `false` (预览时已有 Load 数据) |
| `apply_transfer` | `bool` | 6 | 是 | 是否应用传输函数 (线性化) | | `true` |

**响应流**: `PixelDataChunk`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|------|------|------|------|------|
| `offset` | `uint32` | 1 | 是 | 数据在完整缓冲区中的字节偏移 |
| `data` | `bytes` | 2 | 是 | 分块像素数据 |
| `total_size` | `uint32` | 3 | 是 | 解压后完整像素缓冲区总大小 (字节) |
| `is_last` | `bool` | 4 | 是 | 是否为最后一块数据 |

**分块策略**: 每块大小 = `min(256KB, total_size)`, 流按需提供背压。

**前端处理流程**:
```
invoke("decode_preview", {path, maxWidth, maxHeight})
  -> gRPC ImageServiceClient.decode(DecodeRequest) -> stream
  -> 收集所有 PixelDataChunk:
     chunks: Uint8Array[] = []
     for each chunk in stream:
       chunks.push(chunk.data)
       if chunk.is_last: break
  -> 合并: new Uint8Array(total_size), 按 offset 写入各 chunk
  -> 返回 { pixels: number[], width, height, layout }
  -> 渲染到 Canvas / ImageBitmap
```

**完整 JSON 示例**:
```json
{
  "请求": {
    "path": "/home/user/photos/DSC001.ARW",
    "pixel_format": "u8",
    "max_width": 1920,
    "max_height": 1280,
    "read_metadata": false,
    "apply_transfer": true
  },
  "响应流片段": [
    {"offset": 0, "data": "<base64: 256KB>", "total_size": 9830400, "is_last": false},
    {"offset": 262144, "data": "<base64: 256KB>", "total_size": 9830400, "is_last": false},
    {"offset": 9568256, "data": "<base64: 256KB>", "total_size": 9830400, "is_last": true}
  ]
}
```

#### 2.3.3 GetThumbnail

**用途**: 生成图像文件的快速 JPEG 缩略图。前端用于 ImageCard 和 Filmstrip 列表展示，通过 base64 内嵌显示。

**请求**: `ThumbnailRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 | 前端默认值 |
|------|------|------|------|------|------|-----------|
| `path` | `string` | 1 | 是 | 图像文件路径 | | -- |
| `max_size` | `uint32` | 2 | 是 | 缩略图最大边长 (像素) | 最小 1 | 256 (S), 384 (M), 512 (L) |

**响应**: `ImageData`

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端处理 |
|------|------|------|------|------|---------|
| `data` | `bytes` | 1 | 是 | JPEG 编码的缩略图数据 | `URL.createObjectURL(new Blob([data], {type:'image/jpeg'}))` |
| `width` | `uint32` | 2 | 是 | 缩略图实际宽度 | `<img>` 尺寸属性 |
| `height` | `uint32` | 3 | 是 | 缩略图实际高度 | `<img>` 尺寸属性 |
| `format` | `string` | 4 | 是 | 格式标识，固定为 `"jpeg"` | |

**前端调用时机**:
- `useFilmstripStore.importImages()` 完成后，对每张图像异步调用
- 用户调整缩略图大小时 (S/M/L)，使用新的 `max_size` 重新请求
- 使用 `useThumbnail` hook 管理缓存 (Map<path, base64_data_url>)

**代码示例 (TypeScript)**:
```typescript
// hooks/useThumbnail.ts
export function useThumbnail(path: string, maxSize: number): string | null {
  const [dataUrl, setDataUrl] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    invoke<{ data: number[]; width: number; height: number }>(
      "get_thumbnail", { path, maxSize }
    ).then(({ data }) => {
      if (cancelled) return;
      const blob = new Blob([new Uint8Array(data)], { type: "image/jpeg" });
      setDataUrl(URL.createObjectURL(blob));
    });
    return () => { cancelled = true; };
  }, [path, maxSize]);
  return dataUrl;
}
```

#### 2.3.4 Encode

**用途**: 将像素缓冲区编码为指定图像格式并写入文件。GUI 中通常不直接调用此接口，而是通过管线执行的 ENCODING 阶段自动调用。

**请求**: `EncodeRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 | 默认值 |
|------|------|------|------|------|------|--------|
| `pixel_data` | `bytes` | 1 | 是 | 原始像素数据 | | |
| `width` | `uint32` | 2 | 是 | 图像宽度 | | |
| `height` | `uint32` | 3 | 是 | 图像高度 | | |
| `layout` | `string` | 4 | 是 | 通道布局 | `"gray"` / `"gray_alpha"` / `"rgb"` / `"rgba"` | |
| `pixel_format` | `string` | 5 | 是 | 像素格式 | `"u8"` / `"u16"` / `"u32"` / `"f16"` / `"f32"` | |
| `output_path` | `string` | 6 | 是 | 输出文件路径 | | |
| `format` | `string` | 7 | 是 | 目标容器格式 | `"heif"` / `"avif"` / `"jxl"` / `"png"` / `"tiff"` / `"jpeg"` / `"webp"` | |
| `quality` | `optional float` | 8 | 否 | 编码质量 | `0.0 - 100.0` | `95.0` |
| `lossless` | `bool` | 9 | 是 | 是否无损编码 | | `false` |
| `bit_depth` | `uint32` | 10 | 是 | 输出位深度 | | `10` |
| `chroma_subsampling` | `optional string` | 11 | 否 | 色度子采样 | `"yuv444"` / `"yuv422"` / `"yuv420"` | |
| `encoder` | `optional string` | 12 | 否 | 编码器选择 | 如 `"rav1e"` / `"x265"` | |
| `effort` | `optional uint32` | 13 | 否 | 编码努力级别 | `0-10` | |
| `metadata` | `MetadataInfo` | 14 | 是 | 要嵌入输出文件的元数据 | | |

**响应流**: `EncodeProgress`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|------|------|------|------|------|
| `fraction` | `float` | 1 | 是 | 进度分数 `[0.0, 1.0]` |
| `message` | `string` | 2 | 是 | 进度描述 |
| `bytes_written` | `uint64` | 3 | 是 | 已写入字节数 |
| `done` | `bool` | 4 | 是 | 是否完成 |

### 2.4 PipelineService

完整 Proto 定义:

```protobuf
syntax = "proto3";
package photopipeline.pipeline;

import "google/protobuf/struct.proto";

service PipelineService {
  rpc CreatePipeline(PipelineSpec) returns (PipelineId);
  rpc Execute(ExecuteRequest) returns (stream ExecuteProgress);
  rpc Validate(PipelineSpec) returns (ValidationResult);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}
```

#### 2.4.1 CreatePipeline

**用途**: 在服务端创建并存储一条管线定义，返回管线 ID 供后续 Execute 使用。GUI 中通过 `execute_pipeline` command 内部调用。

**请求**: `PipelineSpec`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `name` | `string` | 1 | 是 | 管线名称 | 非空 |
| `nodes` | `repeated PipelineNode` | 2 | 是 | 管线节点列表 | 至少 1 个 |
| `edges` | `repeated PipelineEdge` | 3 | 否 | 节点连接边 | DAG 拓扑, 不允许环 |
| `params` | `map<string, google.protobuf.Struct>` | 4 | 否 | 全局参数覆盖 (按节点 ID 索引) | |
| `batch` | `BatchConfig` | 5 | 否 | 批量处理配置 | |

**PipelineNode**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 | 默认值 |
|------|------|------|------|------|------|--------|
| `id` | `string` | 1 | 是 | 节点唯一标识 | 管线内唯一 | |
| `plugin_id` | `string` | 2 | 是 | 对应的插件 ID | 必须存在于 Registry | |
| `label` | `string` | 3 | 否 | 节点显示标签 | 为空时使用 `id` | `""` |
| `enabled` | `bool` | 4 | 是 | 是否启用 | | `true` |
| `params` | `google.protobuf.Struct` | 5 | 否 | 节点级参数覆盖 (JSON) | | `null` |

**PipelineEdge**:

| 字段 | 类型 | 编号 | 必填 | 描述 |
|------|------|------|------|------|
| `from` | `string` | 1 | 是 | 源节点 ID |
| `to` | `string` | 2 | 是 | 目标节点 ID |

**BatchConfig**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 默认值 |
|------|------|------|------|------|--------|
| `parallel` | `int32` | 1 | 是 | 并行度 (同时处理文件数) | `1` |
| `output_pattern` | `string` | 2 | 否 | 输出文件名模式 | `"{name}_out"` |
| `on_conflict` | `string` | 3 | 否 | 文件冲突处理策略 | `"skip"` |
| `resume` | `bool` | 4 | 是 | 是否支持断点续传 | `false` |

**响应**: `PipelineId`

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `id` | `string` | 1 | UUID v4 格式的管线 ID |

**错误条件**:
| gRPC Status | 条件 | 前端处理 |
|-------------|------|---------|
| `INVALID_ARGUMENT` | 管线验证失败 (空节点、无效边、插件不存在) | 先调用 Validate 获取具体问题列表 |

**前端使用流程**:
```
execute_pipeline command:
  1. 从 usePipelineStore.nodes/edges 构建 PipelineSpec
  2. gRPC CreatePipeline(PipelineSpec) -> PipelineId { id }
  3. 使用 pipeline_id 调用 Execute
```

#### 2.4.2 Execute

**用途**: 执行已创建的管线，通过 gRPC server streaming 返回实时进度。这是管线执行的核心接口。

**请求**: `ExecuteRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `pipeline_id` | `string` | 1 | 是 | CreatePipeline 返回的管线 ID | |
| `image_path` | `string` | 2 | 是 | 输入图像绝对路径 | 文件必须存在 |
| `output_path` | `string` | 3 | 是 | 输出文件路径 (可为空字符串) | |

**响应流**: `ExecuteProgress`

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端使用 |
|------|------|------|------|------|---------|
| `stage` | `Stage` (enum) | 1 | 是 | 当前执行阶段 | StatusBar 阶段文字, 阶段图标 |
| `node_id` | `string` | 2 | 否 | 当前节点 ID (PROCESSING 阶段) | DAGNode 闪烁动画 |
| `node_label` | `string` | 3 | 否 | 当前节点显示标签 | StatusBar 进度文字 |
| `fraction` | `float` | 4 | 是 | 进度分数 `[0.0, 1.0]` | StatusBar 进度条 |
| `message` | `string` | 5 | 是 | 进度描述消息 | StatusBar 消息 |
| `elapsed_ms` | `int64` | 6 | 是 | 已用毫秒 | StatusBar 计时器 |

**Stage 枚举**:

| 值 | 名称 | 描述 | 前端图标 | 前端颜色 |
|---|------|------|---------|---------|
| 0 | `LOADING` | 正在加载输入图像 | FolderOpen | 蓝色 |
| 1 | `DECODING` | 正在解码图像数据 | ImageSearch | 蓝色 |
| 2 | `PROCESSING` | 正在执行处理节点 | PlayCircle | 绿色脉冲 |
| 3 | `ENCODING` | 正在编码输出文件 | Save | 蓝色 |
| 4 | `DONE` | 处理完成 | CheckmarkCircle | 绿色常亮 |
| 5 | `ERROR` | 处理出错 | ErrorCircle | 红色 |

**前端事件流**:
```
invoke("execute_pipeline", {configJson})
  -> [Rust] gRPC Execute stream -> for each msg:
     -> app.emit("pipeline-progress", {node_id, node_label, fraction, message, elapsed_ms})
     -> app.emit("pipeline-stage", {from_stage, to_stage})
     -> if stage == ERROR: app.emit("pipeline-error", {node_id, code, message})
     -> if stage == DONE: app.emit("pipeline-done", {output_paths, total_bytes, total_seconds})
```

#### 2.4.3 Validate

**用途**: 验证管线定义的有效性，不执行任何处理。GUI 工具栏有独立的 Validate 按钮，也可在执行前自动调用。

**请求**: `PipelineSpec` (结构同 2.4.1)

**响应**: `ValidationResult`

| 字段 | 类型 | 编号 | 描述 | 前端处理 |
|------|------|------|------|---------|
| `valid` | `bool` | 1 | 是否存在 Error 级别的验证问题 | 决定 Run 按钮是否 disabled |
| `issues` | `repeated ValidationIssue` | 2 | 所有验证问题列表 | 弹窗逐条展示 |

**ValidationIssue**:

| 字段 | 类型 | 编号 | 描述 | 前端处理 |
|------|------|------|------|---------|
| `severity` | `Severity` (enum) | 1 | 严重程度 | 决定图标和颜色 |
| `param` | `string` | 2 | 关联的参数路径 (如 `"denoise_1.sigma_color"`) | 错误参数高亮 |
| `message` | `string` | 3 | 人类可读的问题描述 | 弹窗文本 |

**Severity 枚举**:

| 值 | 名称 | 描述 | 前端图标 | 前端行为 |
|---|------|------|---------|---------|
| 0 | `INFO` | 信息性提示 | Info | 蓝色信息条 |
| 1 | `WARNING` | 警告 (不阻止执行) | Warning | 黄色警告条, Run 仍然可用 |
| 2 | `ERROR` | 错误 (阻止执行) | Error | 红色错误条, Run disabled |

**前端 Validate 弹窗布局**:
```
┌── Validation Results ──────────────────────────┐
│  Status: ⚠ 2 Warnings, 1 Error                 │
│                                                 │
│  🔴 [ERROR] denoise_1.sigma_color              │
│     Value 150.0 exceeds maximum 100.0           │
│                                                 │
│  🟡 [WARNING] sharpen_1.radius                  │
│     Large radius (>3.0) may cause halos         │
│                                                 │
│  🟡 [WARNING] encoder_1.bit_depth               │
│     8-bit output loses precision from 16-bit    │
│     pipeline                                     │
│                                                 │
│  [Close]  [Continue Anyway (if no errors)]      │
└─────────────────────────────────────────────────┘
```

#### 2.4.4 GetNodeSchema (PipelineService 内)

与 PluginService.GetNodeSchema 功能相同，作为备选 schema 查询通道。请求 `PluginId { id }`, 响应 `NodeSchema`。

### 2.5 BatchService

完整 Proto 定义:

```protobuf
syntax = "proto3";
package photopipeline.batch;

import "google/protobuf/empty.proto";

service BatchService {
  rpc SubmitBatch(BatchSpec) returns (BatchId);
  rpc GetProgress(BatchId) returns (stream BatchProgress);
  rpc Cancel(BatchId) returns (google.protobuf.Empty);
}
```

#### 2.5.1 SubmitBatch

**用途**: 提交批量处理任务。使用管线配置文件和 glob 文件模式匹配多张输入图像。

**请求**: `BatchSpec`

| 字段 | 类型 | 编号 | 必填 | 描述 | 默认值 | 前端控件 |
|------|------|------|------|------|--------|---------|
| `pipeline_config_path` | `string` | 1 | 是 | 管线配置 JSON/TOML 文件路径 | | (自动生成) |
| `file_pattern` | `string` | 2 | 是 | 输入文件 glob 匹配模式 | `"*.*"` | TextInput |
| `output_dir` | `string` | 3 | 是 | 输出目录路径 | `"."` | DirectoryPicker |
| `parallel` | `int32` | 4 | 是 | 并行度 (同时处理文件数) | `1` | SpinButton |
| `resume` | `bool` | 5 | 是 | 是否启用断点续传 | `false` | Switch |

**响应**: `BatchId`

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `id` | `string` | 1 | 批次 UUID v4 |

**前端使用流程**:
```
invoke("start_batch", {configJson})
  -> [Rust] 保存 PipelineConfig JSON 到临时文件
  -> gRPC BatchServiceClient.submit_batch(BatchSpec)
  -> BatchId { id }
  -> 立即返回 batch_id
  -> spawn: gRPC GetProgress stream loop -> emit("batch-progress", ...)
```

#### 2.5.2 GetProgress

**用途**: 订阅批处理进度流。提交批次后自动启动订阅。

**请求**: `BatchId`

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `id` | `string` | 1 | 批次 UUID |

**响应流**: `BatchProgress`

| 字段 | 类型 | 编号 | 必填 | 描述 | 前端使用 |
|------|------|------|------|------|---------|
| `status` | `Status` (enum) | 1 | 是 | 批次整体状态 | StatusBar 状态灯 |
| `total_files` | `int32` | 2 | 是 | 总文件数 | 进度条分母 |
| `completed_files` | `int32` | 3 | 是 | 已完成文件数 | 进度条分子 |
| `failed_files` | `int32` | 4 | 是 | 失败文件数 | 错误计数 (红色) |
| `current_file` | `string` | 5 | 否 | 当前处理的文件名 | StatusBar 文字 |
| `fraction` | `float` | 6 | 是 | 进度分数 `[0.0, 1.0]` | 进度条 |
| `progress_details` | `string` | 7 | 否 | 进度详情文本 | Tooltip |

**Status 枚举**:

| 值 | 名称 | 描述 | 前端图标 | 前端颜色 |
|---|------|------|---------|---------|
| 0 | `PENDING` | 待处理 (已提交, 排队中) | Clock | 灰色 |
| 1 | `RUNNING` | 运行中 | PlayCircle | 蓝色脉冲 |
| 2 | `DONE` | 全部完成 | CheckmarkCircle | 绿色 |
| 3 | `CANCELED` | 用户取消 | Cancel | 灰色 |
| 4 | `ERROR` | 批次出错 | ErrorCircle | 红色 |

#### 2.5.3 Cancel

**用途**: 取消正在运行的批次。

**请求**: `BatchId { id }`

**响应**: `google.protobuf.Empty`

**错误条件**: `NOT_FOUND` -- 批次 ID 不存在

**前端使用**:
```typescript
// useBatchStore.stopBatch()
await invoke("cancel_batch", { batchId: currentBatchId });
// -> gRPC BatchServiceClient.cancel(BatchId { id })
```

### 2.6 ExecutionService v2

以下为规划中的 v2 统一执行服务，合并 PipelineService.Execute 和 BatchService，提供更丰富的流事件类型。

```protobuf
syntax = "proto3";
package photopipeline.execution;

import "google/protobuf/struct.proto";

service ExecutionService {
  rpc Run(RunRequest) returns (stream RunEvent);
  rpc Cancel(CancelRequest) returns (google.protobuf.Empty);
}
```

#### 2.6.1 Run

**RunRequest**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|------|------|------|------|------|------|
| `config_path` | `string` | 1 | 是 | 管线配置 JSON 文件路径 | 必须存在且有效 |
| `image_paths` | `repeated string` | 2 | 是 | 输入图像路径列表 (单张或批量) | 至少 1 个 |
| `output_dir` | `string` | 3 | 是 | 输出目录 | |
| `output_pattern` | `string` | 4 | 否 | 输出文件名模式 | 默认 `"{name}_processed.{ext}"` |
| `filter` | `string` | 5 | 否 | 节点执行过滤器 (正则) | 只执行匹配名称的节点 |
| `metrics` | `bool` | 6 | 否 | 是否上报资源指标 | 默认 `false` |

**RunEvent (oneof)**:

| 编号 | 变体 | 描述 | 前端处理 |
|------|------|------|---------|
| 1 | `ProgressUpdate` | 节点级进度更新 | usePipelineStore.updateExecutionState() |
| 2 | `MetricSnapshot` | CPU/GPU/内存指标 | 性能监控面板 (仅在 metrics=true 时) |
| 3 | `StageTransition` | 阶段转换事件 | StatusBar 阶段文字切换 |
| 4 | `ErrorEvent` | 错误事件 (可恢复/致命) | DAGNode 红色状态灯, 错误参数高亮 |
| 5 | `DoneEvent` | 完成事件 (含统计) | 全部节点绿色常亮, StatusBar 统计 |
| 6 | `Heartbeat` | 心跳 (每 5 秒) | 超时检测 (30 秒无心跳则断连) |

**ProgressUpdate**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `node_id` | `string` | 1 | 当前节点 ID |
| `node_label` | `string` | 2 | 当前节点标签 |
| `fraction` | `float` | 3 | 进度分数 `[0.0, 1.0]` |
| `message` | `string` | 4 | 进度描述 |
| `elapsed_ms` | `int64` | 5 | 阶段已用毫秒 |

**MetricSnapshot**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `cpu_percent` | `float` | 1 | CPU 使用率 (%) |
| `memory_used_mb` | `uint64` | 2 | 已用内存 (MB) |
| `memory_total_mb` | `uint64` | 3 | 系统总内存 (MB) |
| `gpu_metrics` | `GpuMetrics` | 4 | GPU 指标 (可选) |
| `elapsed_seconds` | `float` | 5 | 总用时 (秒) |
| `bytes_processed` | `uint64` | 6 | 已处理字节数 |
| `throughput_mbps` | `float` | 7 | 吞吐量 (MB/s) |

**GpuMetrics**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `gpu_utilization` | `float` | 1 | GPU 利用率 (%) |
| `gpu_memory_used_mb` | `uint64` | 2 | GPU 已用内存 (MB) |
| `gpu_memory_total_mb` | `uint64` | 3 | GPU 总内存 (MB) |
| `temperature_celsius` | `float` | 4 | GPU 温度 |

**StageTransition**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `stage` | `Stage` (enum) | 1 | 目标阶段 |
| `previous_stage` | `Stage` (enum) | 2 | 前一阶段 |
| `timestamp_ms` | `int64` | 3 | 时间戳 (epoch ms) |

**ErrorEvent**:

| 字段 | 类型 | 编号 | 描述 | 前端处理 |
|------|------|------|------|---------|
| `code` | `string` | 1 | 错误码 (如 `"NODE_EXECUTION_FAILED"`) | 错误码表查找翻译 |
| `message` | `string` | 2 | 错误消息 | StatusBar / Toast |
| `node_id` | `optional string` | 3 | 关联的节点 ID | DAGNode 红色高亮 |
| `details` | `google.protobuf.Struct` | 4 | 附加错误上下文 (JSON) | 错误详情面板 |

**DoneEvent**:

| 字段 | 类型 | 编号 | 描述 | 前端使用 |
|------|------|------|------|---------|
| `output_paths` | `repeated string` | 1 | 所有输出文件路径 | 输出面板列表 |
| `total_bytes` | `uint64` | 2 | 总输出字节数 | 统计显示 |
| `total_seconds` | `float` | 3 | 总用时 (秒) | 统计显示 |
| `node_stats` | `repeated NodeResult` | 4 | 各节点统计数据 | 节点详情 Tooltip |

**NodeResult**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `node_id` | `string` | 1 | 节点 ID |
| `node_label` | `string` | 2 | 节点标签 |
| `status` | `string` | 3 | `"completed"` / `"failed"` / `"skipped"` |
| `elapsed_ms` | `uint64` | 4 | 执行用时 (ms) |
| `cpu_time_ms` | `uint64` | 5 | CPU 时间 (ms) |
| `gpu_time_ms` | `optional uint64` | 6 | GPU 时间 (ms) |
| `peak_memory_mb` | `uint64` | 7 | 峰值内存 (MB) |
| `input_pixels` | `uint64` | 8 | 输入像素总数 |
| `output_pixels` | `uint64` | 9 | 输出像素总数 |

#### 2.6.2 Cancel

**CancelRequest**:

| 字段 | 类型 | 编号 | 描述 |
|------|------|------|------|
| `pipeline_id` | `string` | 1 | 要取消的管线 ID |

**响应**: `google.protobuf.Empty`

---

## 3. 外部接口: PipelineConfig 文件格式

> 与后端 `doc/INTERFACE_DESIGN.md` §4 完全对齐。PipelineConfig 是 GUI 和 CLI 共享的 JSON / TOML 持久化格式，通过**文件系统**传递。

### 3.1 顶层结构 (PipelineConfig)

```json
{
  "name": "string (必填)",
  "version": "string (可选)",
  "description": "string (可选)",
  "pipelines": [ PipelineTemplate, ... ],
  "images": [ ImageEntry, ... ],
  "output": { ... },
  "groups": [ ParamGroup, ... ],
  "execution": { ... }
}
```

### 3.2 字段详解

#### 3.2.1 PipelineConfig 顶层字段

| 字段 | 类型 | 必填 | 描述 | 约束 | 示例值 |
|------|------|------|------|------|--------|
| `name` | `string` | **是** | 配置方案名称 | 非空, 最长 128 字符 | `"Raw to HEIF Pipeline"` |
| `version` | `string` | 否 | 配置文件版本号 | 语义化版本 | `"1.0"` |
| `description` | `string` | 否 | 用于概要说明的文本 | | `"Convert RAW files to 10-bit HEIF with denoising"` |
| `pipelines` | `PipelineTemplate[]` | **是** | 管线模板定义数组 | 至少 1 个元素 | |
| `images` | `ImageEntry[]` | 否 | 输入图像列表 | | |
| `output` | `OutputConfig` | 否 | 通用输出配置 | | |
| `groups` | `ParamGroup[]` | 否 | 全局条件参数组 | | |
| `execution` | `ExecutionConfig` | 否 | 执行环境配置 | | |

#### 3.2.2 PipelineTemplate

对应后端 `crates/engine/src/graph.rs :: PipelineTemplate`。

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端来源 |
|------|------|------|------|--------|---------|
| `metadata` | `TemplateMetadata` | 否 | 模板元信息 | `{}` | usePipelineStore 元数据 |
| `nodes` | `TemplateNode[]` | **是** | 节点定义列表 | | usePipelineStore.nodes |
| `edges` | `TemplateEdge[]` | 否 | 节点连接边 | `[]` | usePipelineStore.edges |
| `overrides` | `ImageOverride[]` | 否 | 图像级参数覆盖 | `[]` | useOverrideStore.overrides |
| `groups` | `ParamGroup[]` | 否 | 条件参数组 | `[]` | useFilmstripStore.groups |
| `batch` | `BatchConfig` | 否 | 批量处理配置 | `null` | useBatchStore.outputSettings |

**TemplateMetadata**:

| 字段 | 类型 | 必填 | 描述 | 示例值 |
|------|------|------|------|--------|
| `name` | `string` | 否 | 管线可读名称 | `"RAW to HEIF"` |
| `version` | `string` | 否 | 管线版本号 | `"2.1"` |
| `description` | `string` | 否 | 管线功能描述 | `"Standard raw development pipeline"` |

#### 3.2.3 TemplateNode

对应前端类型 `DAGNodeData` (序列化时字段名映射)。

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端来源 |
|------|------|------|------|--------|---------|
| `id` | `string` | **是** | 节点唯一标识 | | `DAGNodeData.id` |
| `plugin` | `string` | **是** | 对应的插件 ID | | `DAGNodeData.pluginId` |
| `label` | `string` | 否 | 用户可读的显示标签 | 使用 `id` | `DAGNodeData.label` |
| `enabled` | `bool` | 否 | 是否启用此节点 | `true` | `DAGNodeData.enabled` |
| `params` | `object` | 否 | 节点级参数值覆盖 | `{}` | `DAGNodeData.params` |

**JSON 示例**:
```json
{
  "id": "raw_decoder_1",
  "plugin": "photopipeline.plugins.raw_decoder",
  "label": "RAW Decode",
  "enabled": true,
  "params": {
    "demosaic": "AMaZE",
    "white_balance": "camera"
  }
}
```

#### 3.2.4 TemplateEdge

对应前端类型 `DAGEdgeData`。

| 字段 | 类型 | 必填 | 描述 | 约束 |
|------|------|------|------|------|
| `from` | `string` | **是** | 数据来源节点 ID | 必须存在于 `nodes` 中 |
| `to` | `string` | **是** | 数据目标节点 ID | 必须存在于 `nodes` 中 |

**JSON 示例**:
```json
{ "from": "raw_decoder_1", "to": "denoise_1" }
```

#### 3.2.5 ParamGroup (条件参数组)

对应后端 `crates/engine/src/graph.rs :: ParamGroup`。

| 字段 | 类型 | 必填 | 描述 | 约束 |
|------|------|------|------|------|
| `name` | `string` | **是** | 组名称 (UI 显示) | 唯一, 非空 |
| `condition` | `string` | **是** | 条件表达式字符串 | 必须是合法表达式 |
| `params` | `map<string, object>` | 否 | 按节点 ID 索引的参数覆盖 | 键为 `TemplateNode.id` |

#### 3.2.6 条件表达式语法 (condition 字段)

| 类别 | 语法 | 示例 | 求值为 true 的条件 |
|------|------|------|-------------------|
| **等值比较** | `exif.<field> == "value"` | `exif.make == "Canon"` | 相机制造商为 Canon |
| **数值大于等于** | `exif.<field> >= <number>` | `exif.iso >= 800` | ISO >= 800 |
| **数值小于等于** | `exif.<field> <= <number>` | `exif.focal_length <= 200` | 焦距 <= 200mm |
| **GPS 范围** | `gps.near(lat: <lat>, lon: <lon>, radius_km: <r>)` | `gps.near(lat: 34.05, lon: -118.24, radius_km: 10)` | 在坐标 10km 半径内 |
| **逻辑与** | `(expr1) and (expr2)` | `(exif.iso >= 800) and (exif.make == "Sony")` | 两个条件均满足 |
| **逻辑或** | `(expr1) or (expr2)` | `(exif.iso >= 800) or (exif.focal_length <= 24)` | 至少一个满足 |
| **三元表达式** | `condition ? "val1" : "val2"` | `exif.iso >= 800 ? "high" : "low"` | |
| **图像属性** | `image.<field> > <number>` | `image.width > 4000` | 图像宽度 > 4000 |

**可用 EXIF 变量**: `make`, `model`, `lens`, `iso`, `focal_length`, `aperture`, `shutter`, `datetime`
**可用图像变量**: `filename`, `width`, `height`, `filesize`
**可用 GPS 变量**: `latitude`, `longitude`, `altitude`

**JSON 示例 (完整 ParamGroup)**:
```json
{
  "name": "High ISO",
  "condition": "exif.iso >= 800",
  "params": {
    "denoise_1": {
      "sigma_color": 25.0,
      "sigma_spatial": 5.0
    }
  }
}
```

#### 3.2.7 ImageOverride (逐图参数覆盖)

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `image` | `string` | **是** | 目标图像的标识 (文件名或路径) |
| `params` | `map<string, object>` | 否 | 按节点 ID 索引的参数覆盖 |

**JSON 示例**:
```json
{
  "image": "DSC0034.ARW",
  "params": {
    "denoise_1": { "sigma_color": 30.0, "sigma_spatial": 6.0 }
  }
}
```

#### 3.2.8 BatchConfig (批量配置)

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端控件 |
|------|------|------|------|--------|---------|
| `parallel` | `int` | 否 | 并行文件数 | `1` | SpinButton (1-16) |
| `output_pattern` | `string` | 否 | 输出文件名模板 | `"{name}_out"` | TextInput |
| `on_conflict` | `string` | 否 | 冲突策略: `"skip"` / `"overwrite"` / `"rename"` | `"skip"` | Dropdown |
| `resume` | `bool` | 否 | 断点续传 | `false` | Switch |

**变量可用**: `{name}` (无扩展名文件名), `{ext}` (扩展名), `{index}` (序号), `{date}` (日期)

#### 3.2.9 ExecutionConfig (执行环境配置)

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|------|------|------|------|--------|
| `tile_threshold_pixels` | `int` | 否 | 超过此像素数启用瓦片处理 | `8847360` (~4K) |
| `max_ram_mb` | `int` | 否 | 最大可使用的内存 (MB) | `16384` |
| `timeout_seconds` | `int` | 否 | 单张图像处理超时 | `3600` |
| `gpu_backend` | `string` | 否 | GPU 后端: `"auto"` / `"cuda"` / `"metal"` / `"vulkan"` | `"auto"` |

### 3.3 完整 PipelineConfig JSON 示例

```json
{
  "name": "Raw to HEIF Pipeline",
  "version": "1.0",
  "description": "Convert Sony RAW files to 10-bit HEIF with AI denoising and color management",
  "pipelines": [
    {
      "metadata": {
        "name": "RAW to HEIF (Display P3)",
        "version": "2.1",
        "description": "Standard raw development pipeline targeting Display P3 colorspace"
      },
      "nodes": [
        {
          "id": "raw_decoder_1",
          "plugin": "photopipeline.plugins.raw_decoder",
          "label": "RAW Decode",
          "enabled": true,
          "params": {
            "demosaic": "AMaZE",
            "white_balance": "camera",
            "highlight_recovery": 3
          }
        },
        {
          "id": "denoise_1",
          "plugin": "photopipeline.plugins.bilateral_denoise",
          "label": "Bilateral Denoise",
          "enabled": true,
          "params": {
            "sigma_color": 10.0,
            "sigma_spatial": 3.0,
            "border_mode": "reflect"
          }
        },
        {
          "id": "sharpen_1",
          "plugin": "photopipeline.plugins.unsharp_mask",
          "label": "Unsharp Mask",
          "enabled": true,
          "params": {
            "amount": 1.2,
            "radius": 1.0,
            "threshold": 0
          }
        },
        {
          "id": "colorspace_1",
          "plugin": "photopipeline.plugins.colorspace_convert",
          "label": "Convert to Display P3",
          "enabled": true,
          "params": {
            "target": "display_p3",
            "intent": "perceptual",
            "gamut_mapping": "compress"
          }
        },
        {
          "id": "encoder_1",
          "plugin": "photopipeline.plugins.heif_encoder",
          "label": "HEIF Encode (10-bit)",
          "enabled": true,
          "params": {
            "quality": 95.0,
            "bit_depth": 10,
            "chroma_subsampling": "yuv444",
            "lossless": false
          }
        }
      ],
      "edges": [
        {"from": "raw_decoder_1", "to": "denoise_1"},
        {"from": "denoise_1", "to": "sharpen_1"},
        {"from": "sharpen_1", "to": "colorspace_1"},
        {"from": "colorspace_1", "to": "encoder_1"}
      ],
      "groups": [
        {
          "name": "High ISO (>=800)",
          "condition": "exif.iso >= 800",
          "params": {
            "denoise_1": { "sigma_color": 25.0, "sigma_spatial": 5.0 }
          }
        },
        {
          "name": "Low ISO (<800)",
          "condition": "exif.iso < 800",
          "params": {
            "denoise_1": { "sigma_color": 5.0, "sigma_spatial": 1.0 }
          }
        },
        {
          "name": "Canon-specific",
          "condition": "exif.make == \"Canon\"",
          "params": {
            "sharpen_1": { "amount": 1.5, "radius": 0.8 }
          }
        }
      ],
      "overrides": [
        {
          "image": "DSC0034.ARW",
          "params": {
            "denoise_1": { "sigma_color": 35.0, "sigma_spatial": 8.0 }
          }
        }
      ],
      "batch": {
        "parallel": 4,
        "output_pattern": "{name}_p3.heic",
        "on_conflict": "skip",
        "resume": true
      }
    }
  ],
  "images": [
    {"path": "/photos/DSC0001.ARW"},
    {"path": "/photos/DSC0002.ARW"},
    {"path": "/photos/DSC0034.ARW"}
  ],
  "output": {
    "directory": "/output/processed/",
    "format": "heif",
    "quality": 95.0
  },
  "execution": {
    "tile_threshold_pixels": 8847360,
    "max_ram_mb": 16384,
    "timeout_seconds": 3600,
    "gpu_backend": "auto"
  }
}
```

### 3.4 前端序列化/反序列化

**TypeScript 转换函数 (`utils/pipeline-config.ts`)**:
```typescript
import type { DAGNodeData, DAGEdgeData } from '@/types/pipeline';
import type { PipelineConfig, PipelineTemplate, TemplateNode, TemplateEdge } from '@/types/config';

export function buildPipelineConfig(
  name: string,
  nodes: DAGNodeData[],
  edges: DAGEdgeData[],
  groups: ParamGroup[],
  overrides: ImageOverride[],
  batchConfig?: BatchConfig
): PipelineConfig {
  return {
    name,
    version: "1.0",
    pipelines: [{
      metadata: { name, version: "1.0" },
      nodes: nodes.map(n => ({
        id: n.id,
        plugin: n.pluginId,
        label: n.label,
        enabled: n.enabled,
        params: n.params
      })),
      edges: edges.map(e => ({ from: e.fromNode, to: e.toNode })),
      groups,
      overrides,
      batch: batchConfig
    }]
  };
}

export function parsePipelineConfig(json: string): {
  nodes: DAGNodeData[];
  edges: DAGEdgeData[];
  groups: ParamGroup[];
  overrides: ImageOverride[];
} {
  const config: PipelineConfig = JSON.parse(json);
  const tmpl = config.pipelines[0];
  return {
    nodes: tmpl.nodes.map(n => ({
      id: n.id,
      pluginId: n.plugin,
      label: n.label ?? n.id,
      enabled: n.enabled ?? true,
      position: { x: 0, y: 0 },
      params: n.params ?? {},
      inputs: [],
      outputs: []
    })),
    edges: tmpl.edges.map(e => ({
      id: `${e.from}->${e.to}`,
      fromNode: e.from,
      toNode: e.to
    })),
    groups: tmpl.groups ?? [],
    overrides: tmpl.overrides ?? []
  };
}
```

---

## 4. 外部接口: ParameterSchema 格式

> 与后端 `doc/INTERFACE_DESIGN.md` §5 完全对齐。`ParameterSchema` 定义插件的用户可配置参数，通过 gRPC `GetNodeSchema` 以 `google.protobuf.Struct` (JSON) 返回，前端 `JSON.parse` 后动态渲染参数面板 (ControlPanel)。

### 4.1 顶层结构

| 字段 | 类型 | 必填 | 描述 | 约束 |
|------|------|------|------|------|
| `version` | `u32` | 是 | Schema 格式版本标识 | 当前为 `1` |
| `sections` | `ParameterSection[]` | 是 | 参数分组列表 | 至少 1 个 Section |

### 4.2 ParameterSection (参数节)

```json
{
  "id": "basic",
  "label": "Basic Parameters",
  "description": "Core image adjustment parameters",
  "icon": "settings",
  "collapsible": true,
  "default_collapsed": false,
  "fields": [ ParameterField, ... ]
}
```

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端渲染 |
|------|------|------|------|--------|---------|
| `id` | `string` | **是** | 节唯一标识 | | React key, 条件引用 |
| `label` | `string` | **是** | 节的显示标题 | | CollapsibleCard header |
| `description` | `string` | 否 | 节的描述/帮助文本 | `null` | Tooltip icon |
| `icon` | `string` | 否 | 节图标名称 | `null` | 24x24 icon |
| `collapsible` | `bool` | **是** | 是否可折叠 | | CollapsibleCard |
| `default_collapsed` | `bool` | **是** | 默认折叠状态 | | 初始 open 状态 |
| `fields` | `ParameterField[]` | **是** | 节内字段列表 | | 垂直排列的 ParamRow |

### 4.3 ParameterField (参数字段 -- 公共字段)

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端渲染 |
|------|------|------|------|--------|---------|
| `id` | `string` | **是** | 字段唯一标识 | | React key, param path |
| `label` | `string` | **是** | 字段显示标签 | | Label 组件 |
| `description` | `string` | 否 | 字段描述文本 | `null` | Tooltip/HelpText |
| `help_url` | `string` | 否 | 在线帮助文档链接 | `null` | (?) 按钮, 外部浏览器打开 |
| `type` | `string` | **是** | 值类型标签 (见 4.4) | | 决定渲染什么控件 |
| `default` | `any` | **是** | 默认值 | | 参数解析第 5 层回退值 |
| `required` | `bool` | **是** | 是否必填 | | 标记红色星号, Validate 检查 |
| `advanced` | `bool` | **是** | 是否为高级参数 | | 放入 CollapsibleCard "Advanced" |
| `allow_override` | `bool` | **是** | 是否允许外部覆盖 | `true` | 控制 OverrideScope 访问 |
| `supports_expression` | `bool` | **是** | 是否支持表达式输入 | `false` | 显示 fx 按钮 |
| `condition` | `Condition?` | 否 | 显示条件 | `null` | 控制可见性 |

### 4.4 ParameterType 类型全集 (21 种 ValueType)

#### 4.4.1 string

| 额外字段 | 类型 | 必填 | 描述 | 默认值 | 前端控件属性 |
|---------|------|------|------|--------|-------------|
| `max_length` | `usize` | **是** | 最大字符数 | | Input maxLength |
| `pattern` | `string` | 否 | 正则验证模式 | `null` | Input pattern, 实时校验 |
| `placeholder` | `string` | 否 | 占位提示文本 | `null` | Input placeholder |

**前端控件**: Fluent UI `<Input>`

**JSON 示例**:
```json
{
  "id": "output_filename",
  "label": "Output Filename",
  "description": "Base filename for the output file (without extension)",
  "type": "string",
  "max_length": 256,
  "pattern": "[a-zA-Z0-9_\\-]+",
  "placeholder": "output",
  "default": "output",
  "required": true,
  "advanced": false,
  "allow_override": true,
  "supports_expression": true
}
```

#### 4.4.2 integer

| 额外字段 | 类型 | 必填 | 描述 | 默认值 | 前端控件属性 |
|---------|------|------|------|--------|-------------|
| `min` | `i64` | **是** | 最小值 | | Slider min / SpinButton min |
| `max` | `i64` | **是** | 最大值 | | Slider max / SpinButton max |
| `step` | `i64` | **是** | 步进值 | | Slider step / SpinButton step |
| `unit` | `string` | 否 | 单位标签 | `null` | Label suffix |
| `style` | `IntegerWidget` | 否 | 控件样式 | `"spin_box"` | 决定前端控件类型 |

**IntegerWidget 枚举**: `"spin_box"` (SpinButton), `"slider"` (Slider), `"combo"` (Dropdown)

**JSON 示例**:
```json
{
  "id": "kernel_size",
  "label": "Kernel Size",
  "description": "Filter kernel diameter in pixels",
  "type": "integer",
  "min": 3,
  "max": 31,
  "step": 2,
  "unit": "px",
  "style": "combo",
  "default": 9,
  "required": false,
  "advanced": true,
  "allow_override": false,
  "supports_expression": false
}
```

#### 4.4.3 float

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `min` | `f64` | **是** | 最小值 | |
| `max` | `f64` | **是** | 最大值 | |
| `step` | `f64` | **是** | 步进增量 | |
| `precision` | `u8` | **是** | 显示小数位数 | |
| `unit` | `string` | 否 | 单位标签 | `null` |
| `logarithmic` | `bool` | 否 | 是否使用对数刻度 | `false` |
| `style` | `FloatWidget` | 否 | 控件样式 | `"spin_box"` |

**FloatWidget 枚举 (4 种)**: `"spin_box"` (SpinButton), `"slider"` (Slider), `"combo_slider"` (ComboSlider), `"drag_input"` (DragInput)

**JSON 示例**:
```json
{
  "id": "sigma_color",
  "label": "Color Sigma",
  "description": "Tonal smoothing radius",
  "type": "float",
  "min": 0.0,
  "max": 100.0,
  "step": 0.1,
  "precision": 1,
  "unit": "px",
  "logarithmic": false,
  "style": "slider",
  "default": 10.0,
  "required": true,
  "advanced": false,
  "allow_override": true,
  "supports_expression": true
}
```

#### 4.4.4 boolean

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `label_true` | `string` | 否 | 真值时的标签文字 | `null` |
| `label_false` | `string` | 否 | 假值时的标签文字 | `null` |

**前端控件**: Fluent UI `<Switch>`

**JSON 示例**:
```json
{
  "id": "enable_hdr",
  "label": "HDR Output",
  "description": "Enable High Dynamic Range output with PQ transfer function",
  "type": "boolean",
  "label_true": "On",
  "label_false": "Off",
  "default": false,
  "required": false,
  "advanced": true,
  "allow_override": true,
  "supports_expression": false
}
```

#### 4.4.5 enum

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `options` | `EnumOption[]` | **是** | 可选项列表 | |
| `display` | `EnumDisplay` | 否 | 显示样式 | `"dropdown"` |

**EnumDisplay 枚举 (6 种)**:

| 值 | 前端控件 | 适用场景 |
|---|---------|---------|
| `"dropdown"` | `<Dropdown>` | 5+ 选项, 空间受限 |
| `"radio_group"` | `<RadioGroup>` | 3-5 选项, 需全部可见 |
| `"button_group"` | `<ButtonGroup>` (ToggleButton 集) | 2-5 选项, 强调切换 |
| `"segmented_control"` | `<SegmentedControl>` | iOS 风格 2-5 互斥选项 |
| `"tabs"` | `<TabList>` | 关联不同配置卡片 |
| `"popup_card"` | `<PopupCard>` | 每选项有长描述或预览图 |

**EnumOption**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|------|------|------|------|--------|
| `value` | `string` | **是** | 选项值 (发送给后端) | |
| `label` | `string` | **是** | 选项显示标签 | |
| `description` | `string` | 否 | 选项详细描述 | `null` |
| `icon` | `string` | 否 | 选项图标 | `null` |
| `tags` | `string[]` | 是 | 选项标签 | `[]` |
| `recommended` | `bool` | 否 | 是否推荐此选项 | `false` |

**JSON 示例**:
```json
{
  "id": "demosaic_algorithm",
  "label": "Demosaic Algorithm",
  "description": "Algorithm for interpolating Bayer pattern to full RGB",
  "type": "enum",
  "options": [
    {"value": "amaze", "label": "AMaZE", "description": "Best overall quality", "tags": ["quality"], "recommended": true},
    {"value": "lmmse", "label": "LMMSE", "description": "Good for low-noise", "tags": ["quality"], "recommended": false},
    {"value": "vng4", "label": "VNG4", "description": "Fastest", "tags": ["speed"], "recommended": false},
    {"value": "ahd", "label": "AHD", "description": "Good balance", "tags": ["balanced"], "recommended": false}
  ],
  "display": "radio_group",
  "default": "amaze",
  "required": true,
  "advanced": false,
  "allow_override": false,
  "supports_expression": false
}
```

#### 4.4.6 color

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `mode` | `ColorMode` | 否 | 颜色模型 | `"RGB"` |
| `show_alpha` | `bool` | 否 | 是否显示 Alpha 通道 | `false` |

**ColorMode 枚举 (5 种)**: `"RGB"`, `"RGBA"`, `"HSL"`, `"HSV"`, `"Lab"`

**前端控件**: `<ColorPicker>` (根据 mode 切换色板类型)

**JSON 示例**:
```json
{
  "id": "background_color",
  "label": "Background Color",
  "type": "color",
  "mode": "RGBA",
  "show_alpha": true,
  "default": "#000000FF",
  "required": false,
  "advanced": false,
  "allow_override": true,
  "supports_expression": false
}
```

#### 4.4.7 file_path

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `kind` | `FilePathKind` | 否 | 路径类型 | `"file"` |
| `filters` | `[string, string][]` | 否 | 文件类型过滤器 | `[]` |
| `must_exist` | `bool` | 否 | 路径是否必须存在 | `false` |

**FilePathKind 枚举 (3 种)**: `"file"` (打开文件), `"directory"` (选择文件夹), `"save_file"` (保存对话框)

**前端控件**: Tauri dialog API 调用

**JSON 示例**:
```json
{
  "id": "lut_file",
  "label": "3D LUT File",
  "description": "Path to a .cube or .3dl LUT file for color grading",
  "type": "file_path",
  "kind": "file",
  "filters": [["LUT Files", "*.cube;*.3dl;*.look"], ["All Files", "*.*"]],
  "must_exist": true,
  "default": "",
  "required": false,
  "advanced": true,
  "allow_override": false,
  "supports_expression": false
}
```

#### 4.4.8 coordinate

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `alt_required` | `bool` | 否 | 高度是否必填 | `false` |
| `direction_required` | `bool` | 否 | 方向是否必填 | `false` |

**前端控件**: 自定义 `<CoordinateInput>` (纬度/经度/海拔/方向四字段)

**JSON 示例**:
```json
{
  "id": "target_location",
  "label": "Target GPS Location",
  "type": "coordinate",
  "alt_required": true,
  "direction_required": false,
  "default": {"lat": 0, "lon": 0, "alt": 0},
  "required": false,
  "advanced": true,
  "allow_override": true,
  "supports_expression": false
}
```

#### 4.4.9 slider

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `min` | `f64` | **是** | 最小值 | |
| `max` | `f64` | **是** | 最大值 | |
| `step` | `f64` | 否 | 步进值 | `1.0` |
| `show_ticks` | `bool` | 否 | 是否显示刻度标记 | `false` |
| `ticks` | `f64[]` | 否 | 自定义刻度位置 | `null` |
| `show_value` | `bool` | 否 | 是否显示当前数值标签 | `true` |
| `orientation` | `SliderOrientation` | 否 | 方向 | `"horizontal"` |
| `style` | `SliderStyle` | 否 | 滑块样式 | `"continuous"` |

**SliderOrientation**: `"horizontal"`, `"vertical"`
**SliderStyle (4 种)**: `"continuous"`, `"discrete"`, `"range"`, `"dual_handle"`

**JSON 示例**:
```json
{
  "id": "brightness",
  "label": "Brightness",
  "type": "slider",
  "min": -100.0,
  "max": 100.0,
  "step": 0.1,
  "show_ticks": true,
  "ticks": [-100, -50, 0, 50, 100],
  "show_value": true,
  "orientation": "horizontal",
  "style": "continuous",
  "default": 0.0,
  "required": false,
  "advanced": false,
  "allow_override": true,
  "supports_expression": true
}
```

#### 4.4.10 combo_slider

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `min` | `f64` | **是** | 最小值 | |
| `max` | `f64` | **是** | 最大值 | |
| `step` | `f64` | 否 | 步进值 | `1.0` |
| `presets` | `[[string, f64]]` | **是** | 预设值列表 (label, value) | |
| `unit` | `string` | 否 | 单位 | `null` |

**前端控件**: 自定义 `<ComboSlider>` (Dropdown + Slider 组合)

**JSON 示例**:
```json
{
  "id": "noise_reduction_strength",
  "label": "NR Strength",
  "type": "combo_slider",
  "min": 0.0, "max": 10.0, "step": 0.5,
  "presets": [["None", 0.0], ["Low", 2.0], ["Medium", 5.0], ["High", 7.5], ["Maximum", 10.0]],
  "unit": "dB",
  "default": 2.0,
  "required": false, "advanced": false, "allow_override": true, "supports_expression": false
}
```

#### 4.4.11 expression

| 额外字段 | 类型 | 必填 | 描述 |
|---------|------|------|------|
| `variables` | `VariableDef[]` | **是** | 可用变量列表 |

**VariableDef**:

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `name` | `string` | **是** | 变量名称 |
| `description` | `string` | **是** | 变量描述 |
| `var_type` | `string` | **是** | 变量数据类型: `"number"`, `"string"`, `"boolean"` |
| `example` | `string` | 否 | 示例值 |

**前端控件**: 自定义 `<ExpressionEditor>` (代码编辑器 + 变量自动补全 + 语法高亮)

**JSON 示例**:
```json
{
  "id": "custom_formula",
  "label": "Custom Formula",
  "type": "expression",
  "variables": [
    {"name": "iso", "description": "Sensor ISO sensitivity", "var_type": "number", "example": "400"},
    {"name": "focal_length", "description": "Lens focal length in mm", "var_type": "number", "example": "50"}
  ],
  "default": "iso * 0.01",
  "required": false, "advanced": true, "allow_override": true, "supports_expression": true
}
```

#### 4.4.12 preset

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `preset_schema_ref` | `string` | **是** | 预设模式引用 ID | |
| `builtin_presets` | `NamedPreset[]` | **是** | 内置预设列表 | |
| `allow_custom` | `bool` | 否 | 是否允许用户创建自定义预设 | `false` |
| `allow_import` | `bool` | 否 | 是否允许导入外部预设文件 | `false` |

**NamedPreset**:

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `name` | `string` | **是** | 预设名称 |
| `description` | `string` | 否 | 预设描述 |
| `params` | `map<string, any>` | **是** | 预设参数字典 |

**前端控件**: 自定义 `<PresetSelector>` (Grid/Card 预设浏览器 + Import 按钮)

**JSON 示例**:
```json
{
  "id": "color_profile",
  "label": "Color Profile",
  "type": "preset",
  "preset_schema_ref": "color_grading_v1",
  "builtin_presets": [
    {"name": "Cinematic Warm", "description": "Warm tones with lifted blacks", "params": {"temperature": 5800, "tint": 5, "contrast": 1.15}},
    {"name": "Clean Portrait", "description": "Bright, natural skin tones", "params": {"temperature": 5200, "tint": 0, "contrast": 1.0}},
    {"name": "Landscape Vibrant", "description": "Enhanced greens and blues", "params": {"temperature": 5500, "tint": -3, "contrast": 1.25}}
  ],
  "allow_custom": true, "allow_import": true,
  "default": "Clean Portrait",
  "required": false, "advanced": false, "allow_override": true, "supports_expression": false
}
```

#### 4.4.13 array

| 额外字段 | 类型 | 必填 | 描述 |
|---------|------|------|------|
| `element` | `ParameterField` | **是** | 数组元素的字段定义 (可嵌套任意类型) |
| `min_items` | `usize` | **是** | 最小元素数量 |
| `max_items` | `usize` | 否 | 最大元素数量 |

**前端控件**: 自定义 `<ArrayEditor>` (可添加/删除/重排的动态列表)

**JSON 示例**:
```json
{
  "id": "lut_chain",
  "label": "LUT Chain",
  "type": "array",
  "element": {
    "id": "lut_file", "label": "LUT File", "type": "file_path",
    "kind": "file", "filters": [["LUT Files", "*.cube;*.3dl"]], "must_exist": true,
    "default": "", "required": true, "advanced": false, "allow_override": false, "supports_expression": false
  },
  "min_items": 1, "max_items": 5,
  "default": [],
  "required": false, "advanced": true, "allow_override": false, "supports_expression": false
}
```

#### 4.4.14 map_widget

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `show_track` | `bool` | 否 | 是否显示 GPS 轨迹线 | `false` |
| `show_photos` | `bool` | 否 | 是否在地图上标记照片位置 | `false` |
| `allow_manual_pin` | `bool` | 否 | 是否允许手动放置标记 | `false` |

**前端控件**: Leaflet 嵌入式地图组件

**JSON 示例**:
```json
{
  "id": "gps_selector",
  "label": "Photo Location",
  "type": "map_widget",
  "show_track": true,
  "show_photos": true,
  "allow_manual_pin": true,
  "default": {"lat": 0, "lon": 0, "zoom": 2},
  "required": false, "advanced": true, "allow_override": true, "supports_expression": false
}
```

#### 4.4.15 before_after

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `zoom_levels` | `f64[]` | **是** | 可用的缩放级别 | |
| `show_histogram` | `bool` | 否 | 是否在对比视图中显示直方图 | `false` |

**前端控件**: 自定义 `<BeforeAfterPreview>` (左右/上下分割, 可拖动滑块)

**JSON 示例**:
```json
{
  "id": "before_after_view",
  "label": "Before/After Comparison",
  "type": "before_after",
  "zoom_levels": [0.125, 0.25, 0.5, 1.0, 2.0],
  "show_histogram": true,
  "default": null,
  "required": false, "advanced": false, "allow_override": false, "supports_expression": false
}
```

#### 4.4.16 separator

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---------|------|------|------|--------|
| `label` | `string` | 否 | 分隔线标签文字 | `null` |

**前端控件**: Fluent UI `<Divider>` (带可选标签)

**JSON 示例**:
```json
{
  "id": "sep_advanced",
  "label": "Advanced Settings",
  "type": "separator",
  "default": null,
  "required": false, "advanced": false, "allow_override": false, "supports_expression": false
}
```

#### 4.4.17 section

| 额外字段 | 类型 | 必填 | 描述 |
|---------|------|------|------|
| `fields` | `ParameterField[]` | **是** | 嵌套子字段列表 (支持任意深度嵌套) |

**前端控件**: 嵌套 `<ParamSection>` (可递归)

**JSON 示例**:
```json
{
  "id": "shadow_highlight_section",
  "label": "Shadows & Highlights",
  "type": "section",
  "fields": [
    {"id": "shadow_recovery", "label": "Shadow Recovery", "type": "slider", "min": 0.0, "max": 100.0, "step": 1.0, "style": "continuous", "default": 0.0, "required": false, "advanced": false, "allow_override": true, "supports_expression": false},
    {"id": "highlight_preservation", "label": "Highlight Preservation", "type": "slider", "min": 0.0, "max": 100.0, "step": 1.0, "style": "continuous", "default": 50.0, "required": false, "advanced": false, "allow_override": true, "supports_expression": false}
  ],
  "default": {},
  "required": false, "advanced": false, "allow_override": true, "supports_expression": false
}
```

### 4.5 条件显示系统 (Condition)

ParameterField 可附带 `condition` 字段，对齐后端 `crates/plugin/src/schema.rs :: Condition`。

```json
{
  "condition": { "field": "mode", "op": "equals", "value": "advanced" }
}
```

**条件操作符 (8 种)**:

| 操作符 | 描述 | JSON 示例 |
|--------|------|-----------|
| `equals` | 值相等 | `{"field": "mode", "op": "equals", "value": "gpx"}` |
| `not_equals` | 值不等 | `{"field": "format", "op": "not_equals", "value": "jpeg"}` |
| `greater_than` | 数值大于 | `{"field": "iso", "op": "greater_than", "value": 800}` |
| `less_than` | 数值小于 | `{"field": "quality", "op": "less_than", "value": 50}` |
| `contains` | 数组/字符串包含 | `{"field": "tags", "op": "contains", "value": "hdr"}` |
| `matches` | 正则匹配 | `{"field": "name", "op": "matches", "value": "^DSC.*"}` |
| `all_of` | 所有子条件均满足 | `{"op": "all_of", "conditions": [...]}` |
| `any_of` | 任一子条件满足 | `{"op": "any_of", "conditions": [...]}` |

**前端条件求值 (`utils/condition-eval.ts`)**:
```typescript
export function evaluateCondition(
  condition: Condition,
  context: Record<string, any>
): boolean {
  switch (condition.op) {
    case 'equals': return context[condition.field] === condition.value;
    case 'not_equals': return context[condition.field] !== condition.value;
    case 'greater_than': return Number(context[condition.field]) > Number(condition.value);
    case 'less_than': return Number(context[condition.field]) < Number(condition.value);
    case 'contains':
      const val = context[condition.field];
      return Array.isArray(val) ? val.includes(condition.value)
        : String(val).includes(String(condition.value));
    case 'matches':
      return new RegExp(condition.value as string).test(String(context[condition.field]));
    case 'all_of':
      return (condition.conditions as Condition[]).every(c => evaluateCondition(c, context));
    case 'any_of':
      return (condition.conditions as Condition[]).some(c => evaluateCondition(c, context));
    default: return true;
  }
}
```

### 4.6 前端控件映射总表

| # | type | 前端控件 | Fluent UI 组件 | 关键逻辑 |
|---|------|---------|---------------|---------|
| 1 | `string` | Input | `<Input>` | maxLength, pattern 校验 |
| 2 | `integer` | SpinButton / Slider / Combo | `<SpinButton>` / `<Slider>` / `<Dropdown>` | style 决定 |
| 3 | `float` | SpinButton / Slider / ComboSlider / DragInput | 同上 + 自定义 | 对数刻度, precision |
| 4 | `boolean` | Switch | `<Switch>` | label_true/false |
| 5 | `enum` | Dropdown / RadioGroup / ButtonGroup / SegmentedControl / Tabs / PopupCard | 6 种 | display 决定, recommended badge |
| 6 | `color` | ColorPicker | 自定义 | mode 决定色盘 (5 种) |
| 7 | `file_path` | FilePicker / DirectoryPicker | Tauri dialog | kind (3 种) |
| 8 | `coordinate` | CoordinateInput | 自定义 4 字段 | alt/direction required |
| 9 | `slider` | Slider / RangeSlider | `<Slider>` / 自定义 | style (4 种), 方向 |
| 10 | `combo_slider` | ComboSlider | 自定义 Dropdown+Slider | presets |
| 11 | `expression` | ExpressionEditor | 自定义 CodeMirror/Monaco | 变量补全 |
| 12 | `preset` | PresetSelector | 自定义 Grid/Card | builtin_presets |
| 13 | `array` | ArrayEditor | 自定义动态列表 | element 递归 |
| 14 | `map_widget` | MapWidget | Leaflet | show_track/photos |
| 15 | `before_after` | BeforeAfterPreview | 自定义双图分割 | zoom_levels, 同步缩放 |
| 16 | `separator` | Divider | `<Divider>` | label |
| 17 | `section` | ParamSection 嵌套 | `<CollapsibleCard>` 嵌套 | fields 递归 |

---

## 5. 外部接口: GuiSchema 格式

> 与后端 `crates/plugin/src/gui_schema.rs` 完全对齐。

### 5.1 顶层结构

| 字段 | 类型 | 必填 | 描述 | 默认值 | 前端使用 |
|------|------|------|------|--------|---------|
| `layout` | `GuiLayout` | **是** | 面板布局定义 | | ControlPanel 分区布局 |
| `icon` | `string?` | 否 | 面板图标名称 | `null` | PluginHeader icon |
| `color` | `string?` | 否 | 主题色 (hex) | `null` | PluginHeader 背景, DAGNode 边框 |
| `preview` | `PreviewMode` | **是** | 预览模式 | `"none"` | PreviewView 渲染方式 |
| `aux_views` | `AuxView[]` | **是** | 辅助视图列表 | `[]` | Panel 底部 AuxView 区域 |
| `min_panel_width` | `u32` | **是** | 最小面板宽度 (CSS px) | `320` | Panel 最小尺寸约束 |

### 5.2 GuiLayout (2 种布局模式)

#### Standard (标准布局)

```json
{
  "layout": {
    "Standard": {
      "sections": [
        {"param_section_id": "basic", "title_visible": true, "style": "card"},
        {"param_section_id": "advanced", "title_visible": true, "style": "collapsible_card"},
        {"param_section_id": "debug", "title_visible": false, "style": "plain"}
      ]
    }
  }
}
```

**SectionLayout**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|------|------|------|------|--------|
| `param_section_id` | `string` | **是** | 引用 `ParameterSchema.sections[].id` | |
| `title_visible` | `bool` | **是** | 是否显示节标题 | `true` |
| `style` | `SectionStyle` | 否 | 视觉样式 | `"card"` |

**SectionStyle**: `"card"` (带边框), `"collapsible_card"` (可折叠), `"plain"` (无边框)

#### Custom (自定义行布局)

```json
{
  "layout": {
    "Custom": {
      "rows": [
        {
          "height": "auto",
          "cells": [
            {"param_field_id": "temperature", "width_fraction": 0.5, "label_position": "top"},
            {"param_field_id": "tint", "width_fraction": 0.5, "label_position": "top"}
          ]
        },
        {
          "height": "normal",
          "cells": [
            {"param_field_id": "exposure", "width_fraction": 1.0, "label_position": "left"}
          ]
        }
      ]
    }
  }
}
```

**CellDef**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|------|------|------|------|--------|
| `param_field_id` | `string` | **是** | 引用 `ParameterField.id` | |
| `width_fraction` | `f32` | 否 | 宽度比例 (1.0 = 100%) | `1.0` |
| `label_position` | `string` | 否 | 标签位置: `"top"`, `"left"`, `"none"` | `"top"` |

### 5.3 PreviewMode (5 种预览模式)

| 值 | JSON 格式 | 描述 | 前端行为 |
|----|-----------|------|---------|
| `"none"` | `"preview": "none"` | 无预览 | 不渲染 PreviewView, 控制面板全宽 |
| `"live"` | `"preview": "live"` | 实时预览 | 每次参数变化立即重新 decode 并渲染 |
| `"manual_refresh"` | `"preview": "manual_refresh"` | 手动刷新 | 显示 "Refresh Preview" 按钮 |
| `BeforeAfter` | `"preview": {"BeforeAfter": {"default_split": 0.5, "orientation": "Horizontal", "lock_zoom": true}}` | 前后对比 | 双图分割+可拖动滑块+同步缩放 |
| `Tiled` | `"preview": {"Tiled": {"rows": 2, "cols": 2}}` | 平铺对比 | 网格多版本对比+同步缩放 |

### 5.4 AuxView (10 种辅助视图)

| # | 值 | 说明 | 渲染技术 | 数据来源 |
|---|-----|------|---------|---------|
| 1 | `"histogram"` | RGB/Luminance 直方图 | Canvas 2D | `decode_preview` 像素统计 |
| 2 | `"waveform"` | 逐列亮度波形 | Canvas 2D | `decode_preview` 像素 |
| 3 | `"vectorscope"` | 色彩矢量示波器 | Canvas 2D | `decode_preview` 像素 |
| 4 | `"gamut_diagram"` | CIE 1931 色域图 | Canvas 2D | 静态色域+像素聚集点 |
| 5 | `"map"` | GPS 地图 | Leaflet.js | `ImageInfo.metadata` GPS |
| 6 | `"focus_peaking"` | 边缘检测对焦峰值 | Canvas 2D | `decode_preview` 像素 |
| 7 | `"clipping_warning"` | 过曝/欠曝标记 | Canvas 2D 叠加 | `decode_preview` 像素 |
| 8 | `"metadata_table"` | EXIF/XMP/IPTC 表格 | Fluent `<Table>` | `ImageInfo.metadata` |
| 9 | `"progress_bar"` | 处理进度条 | Fluent `<ProgressBar>` | `pipeline-progress` event |
| 10 | `"status_text"` | 状态文字 | Fluent `<Text>` | store state |

### 5.5 前端 GuiSchema 渲染逻辑

```typescript
// components/params/ControlPanel.tsx
function ControlPanel({ nodeId }: { nodeId: string }) {
  const schema = usePluginStore(s => s.nodeSchemas.get(nodeId));
  if (!schema) return <Skeleton />;

  const { parameter_schema, gui_schema } = schema;

  return (
    <div style={{ minWidth: gui_schema.min_panel_width }}>
      <PluginHeader
        name={schema.name}
        version={schema.version}
        icon={gui_schema.icon}
        color={gui_schema.color}
      />
      {gui_schema.layout.Standard ? (
        <StandardLayout sections={gui_schema.layout.Standard.sections}
          paramSections={parameter_schema.sections} nodeId={nodeId} />
      ) : gui_schema.layout.Custom ? (
        <CustomLayout rows={gui_schema.layout.Custom.rows}
          allFields={parameter_schema.all_fields()} nodeId={nodeId} />
      ) : null}
      <PreviewView mode={gui_schema.preview} />
      {gui_schema.aux_views.map(view => (
        <AuxView key={view} type={view} />
      ))}
    </div>
  );
}
```

---

## 6. 内部接口: Tauri Commands

> Tauri Rust 端 `gui/src-tauri/src/commands/` 中定义 13 个 `#[tauri::command]`，前端通过 `invoke("cmd", args)` 调用。

### 6.1 插件类 Commands

#### 6.1.1 list_plugins

**Rust 签名**:
```rust
#[tauri::command]
async fn list_plugins(
    state: tauri::State<'_, AppState>,
) -> Result<ListPluginsResult, String>
```

**TypeScript invoke**:
```typescript
const result = await invoke<ListPluginsResult>("list_plugins");
// 无参数
// 返回: { plugins: PluginEntry[], categories: string[] }
```

**gRPC 调用链**: `PluginServiceClient.list_plugins(tonic::Request::new(()))`

**前端调用点**: `usePluginStore.fetchPlugins()`, App 启动时自动执行

#### 6.1.2 get_node_schema

**Rust 签名**:
```rust
#[tauri::command]
async fn get_node_schema(
    state: tauri::State<'_, AppState>,
    plugin_id: String,
) -> Result<NodeSchemaResponse, String>
```

**TypeScript invoke**:
```typescript
const schema = await invoke<NodeSchemaResponse>("get_node_schema", {
  pluginId: string
});
```

**gRPC 调用链**: `PluginServiceClient.get_node_schema(PluginIdRequest { id })`

**前端调用点**: `usePluginStore.fetchNodeSchema(pluginId)`, 缓存到 `nodeSchemas` Map

### 6.2 图片类 Commands

#### 6.2.1 load_images

**Rust 签名**:
```rust
#[tauri::command]
async fn load_images(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<ImageInfo>, String>
```

**TypeScript invoke**:
```typescript
const images = await invoke<ImageInfo[]>("load_images", {
  paths: string[]
});
```

**gRPC 调用链**: `ImageServiceClient.load(ImagePath { path })` x N (单张失败不阻塞)

**前端调用点**: `useFilmstripStore.importImages(paths)`

#### 6.2.2 get_thumbnail

**Rust 签名**:
```rust
#[tauri::command]
async fn get_thumbnail(
    state: tauri::State<'_, AppState>,
    path: String,
    max_size: u32,
) -> Result<ThumbnailResult, String>

#[derive(serde::Serialize)]
struct ThumbnailResult { data: Vec<u8>, width: u32, height: u32 }
```

**TypeScript invoke**:
```typescript
interface ThumbnailResult { data: number[]; width: number; height: number; }

const thumb = await invoke<ThumbnailResult>("get_thumbnail", {
  path: string,
  maxSize: number
});

// 前端转换为 Data URL
const blob = new Blob([new Uint8Array(thumb.data)], { type: 'image/jpeg' });
const url = URL.createObjectURL(blob);
```

**前端调用点**: `useThumbnail(path, maxSize)` hook

#### 6.2.3 decode_preview

**Rust 签名**:
```rust
#[tauri::command]
async fn decode_preview(
    state: tauri::State<'_, AppState>,
    path: String,
    max_width: u32,
    max_height: u32,
) -> Result<DecodePreviewResult, String>

#[derive(serde::Serialize)]
struct DecodePreviewResult {
    pixels: Vec<u8>, width: u32, height: u32, layout: String,
}
```

**TypeScript invoke**:
```typescript
interface DecodePreviewResult {
  pixels: number[]; width: number; height: number; layout: string;
}

const preview = await invoke<DecodePreviewResult>("decode_preview", {
  path: string, maxWidth: number, maxHeight: number
});

// 渲染到 Canvas
const ctx = canvas.getContext('2d')!;
const imageData = ctx.createImageData(preview.width, preview.height);
imageData.data.set(new Uint8ClampedArray(preview.pixels));
ctx.putImageData(imageData, 0, 0);
```

**gRPC 调用链**: `ImageServiceClient.decode(DecodeRequest) -> stream` (合并所有 chunk)

**前端调用点**: PreviewView, AuxView (histogram/waveform/vectorscope 等)

### 6.3 管线类 Commands

#### 6.3.1 save_pipeline_file

**Rust 签名**:
```rust
#[tauri::command]
async fn save_pipeline_file(path: String, json: String) -> Result<(), String>
```

**TypeScript invoke**:
```typescript
await invoke("save_pipeline_file", { path: string, json: string });
```

**gRPC 调用链**: 无 (纯文件系统操作 `std::fs::write`)

**前端调用点**: `usePipelineStore.savePipeline(path)` -> Ctrl+S

#### 6.3.2 load_pipeline_file

**Rust 签名**:
```rust
#[tauri::command]
async fn load_pipeline_file(path: String) -> Result<String, String>
```

**TypeScript invoke**:
```typescript
const json = await invoke<string>("load_pipeline_file", { path: string });
// JSON.parse(json) -> PipelineConfig -> parsePipelineConfig() -> DAGNodeData[] + DAGEdgeData[]
```

**前端调用点**: `usePipelineStore.loadPipeline(path)` -> 工具栏 Load

#### 6.3.3 validate_pipeline

**Rust 签名**:
```rust
#[tauri::command]
async fn validate_pipeline(
    state: tauri::State<'_, AppState>,
    json: String,
) -> Result<ValidationResult, String>
```

**TypeScript invoke**:
```typescript
const result = await invoke<ValidationResult>("validate_pipeline", {
  json: string   // PipelineSpec JSON
});
// 返回: { valid: boolean; issues: ValidationIssue[] }
```

**gRPC 调用链**: `PipelineServiceClient.validate(PipelineSpec)`

**前端调用点**: `usePipelineStore.validatePipeline()` -> 工具栏 Validate

#### 6.3.4 execute_pipeline

**Rust 签名**:
```rust
#[tauri::command]
async fn execute_pipeline(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config_json: String,
) -> Result<String, String>
```

**TypeScript invoke**:
```typescript
const pipelineId = await invoke<string>("execute_pipeline", {
  configJson: string   // 完整 PipelineConfig JSON
});
// pipelineId 立即返回, 进度通过 Tauri Events 异步推送
```

**内部流程**:
1. 保存 PipelineConfig JSON 到临时文件
2. `gRPC CreatePipeline(PipelineSpec)` -> `PipelineId`
3. `spawn tokio task`: `gRPC Execute(ExecuteRequest)` -> stream
4. 循环 stream -> `app.emit("pipeline-progress", msg)`

**前端调用点**: `usePipelineStore.executePipeline()` -> Run 按钮

#### 6.3.5 cancel_execution

**Rust 签名**:
```rust
#[tauri::command]
async fn cancel_execution(
    state: tauri::State<'_, AppState>,
    pipeline_id: String,
) -> Result<(), String>
```

**TypeScript invoke**:
```typescript
await invoke("cancel_execution", { pipelineId: string });
```

### 6.4 批量类 Commands

#### 6.4.1 start_batch

**Rust 签名**:
```rust
#[tauri::command]
async fn start_batch(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config_json: String,
) -> Result<String, String>
```

**TypeScript invoke**:
```typescript
const batchId = await invoke<string>("start_batch", { configJson: string });
```

**内部流程**:
1. 保存 PipelineConfig 到临时文件
2. `gRPC SubmitBatch(BatchSpec)` -> `BatchId`
3. `spawn tokio task`: `gRPC GetProgress(BatchId)` -> stream -> `app.emit("batch-progress", msg)`

**前端调用点**: `useBatchStore.startBatch()` -> Start Batch 按钮

#### 6.4.2 cancel_batch

**Rust 签名**:
```rust
#[tauri::command]
async fn cancel_batch(
    state: tauri::State<'_, AppState>,
    batch_id: String,
) -> Result<(), String>
```

**TypeScript invoke**:
```typescript
await invoke("cancel_batch", { batchId: string });
```

**前端调用点**: `useBatchStore.stopBatch()` -> Stop 按钮

### 6.5 设置类 Commands

#### 6.5.1 save_settings / load_settings

**Rust 签名**:
```rust
#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String>

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String>
```

**TypeScript invoke**:
```typescript
await invoke("save_settings", { settings: AppSettings });
const settings = await invoke<AppSettings>("load_settings");
```

**前端调用点**: `useSettingsStore.save()` / `useSettingsStore.load()`

### 6.6 Tauri Commands 总览表

| # | Command | 参数 | 返回 | gRPC 调用 | 流式 | 前端 Store |
|---|---------|------|------|----------|------|-----------|
| 1 | `list_plugins` | 无 | `ListPluginsResult` | `PluginService.ListPlugins` | 否 | usePluginStore |
| 2 | `get_node_schema` | `pluginId` | `NodeSchemaResponse` | `PluginService.GetNodeSchema` | 否 | usePluginStore |
| 3 | `load_images` | `paths[]` | `ImageInfo[]` | `ImageService.Load` xN | 否 | useFilmstripStore |
| 4 | `get_thumbnail` | `path, maxSize` | `ThumbnailResult` | `ImageService.GetThumbnail` | 否 | ImageCard |
| 5 | `decode_preview` | `path, maxW, maxH` | `DecodePreviewResult` | `ImageService.Decode` | 是(合并) | PreviewView |
| 6 | `save_pipeline_file` | `path, json` | `void` | 无 (fs) | 否 | usePipelineStore |
| 7 | `load_pipeline_file` | `path` | `string` | 无 (fs) | 否 | usePipelineStore |
| 8 | `validate_pipeline` | `json` | `ValidationResult` | `PipelineService.Validate` | 否 | usePipelineStore |
| 9 | `execute_pipeline` | `configJson` | `string` (id) | `CreatePipeline`+`Execute` | 是(event) | usePipelineStore |
| 10 | `cancel_execution` | `pipelineId` | `void` | 取消标志 | 否 | usePipelineStore |
| 11 | `start_batch` | `configJson` | `string` (id) | `SubmitBatch`+`GetProgress` | 是(event) | useBatchStore |
| 12 | `cancel_batch` | `batchId` | `void` | `BatchService.Cancel` | 否 | useBatchStore |
| 13 | `save/load_settings` | settings/无 | void/AppSettings | 无 (fs) | 否 | useSettingsStore |

---

## 7. 内部接口: Tauri Events

> Tauri Rust 端通过 `app.emit("event", payload)` 推送事件，前端通过 `useTauriEvent<T>(event, handler)` 监听。

### 7.1 Events 总览

| # | Event | Payload 类型 | 触发时机 | 前端 Store | UI 更新 |
|---|-------|-------------|---------|-----------|---------|
| 1 | `pipeline-progress` | `PipelineProgressPayload` | gRPC Execute stream 每条消息 | usePipelineStore | DAGNode 状态灯 + StatusBar |
| 2 | `pipeline-stage` | `StageTransitionPayload` | Execute stage 变化 | usePipelineStore | StatusBar 阶段文字 |
| 3 | `pipeline-error` | `PipelineErrorPayload` | 执行出错 | usePipelineStore | DAGNode 红色 + 参数高亮 |
| 4 | `pipeline-done` | `PipelineDonePayload` | 执行完成 | usePipelineStore | 全部绿色常亮 + StatusBar |
| 5 | `batch-progress` | `BatchProgressPayload` | gRPC GetProgress stream | useBatchStore | BatchQueueRow 状态 + ProgressBar |
| 6 | `backend-status` | `BackendStatusPayload` | 健康检查轮询 (每 5 秒) | useAppStore | StatusBar 连接灯 |

### 7.2 详细 Payload 定义

#### 7.2.1 pipeline-progress

```typescript
interface PipelineProgressPayload {
  node_id: string | null;
  node_label: string | null;
  fraction: number;       // [0.0, 1.0]
  message: string;
  elapsed_ms: number;
}
```

#### 7.2.2 pipeline-stage

```typescript
interface StageTransitionPayload {
  stage: 'LOADING' | 'DECODING' | 'PROCESSING' | 'ENCODING' | 'DONE' | 'ERROR';
  previous_stage: 'LOADING' | 'DECODING' | 'PROCESSING' | 'ENCODING' | 'DONE' | 'ERROR';
}
```

#### 7.2.3 pipeline-error

```typescript
interface PipelineErrorPayload {
  node_id: string | null;
  code: string;
  message: string;
}
```

#### 7.2.4 pipeline-done

```typescript
interface PipelineDonePayload {
  output_paths: string[];
  total_bytes: number;
  total_seconds: number;
}
```

#### 7.2.5 batch-progress

```typescript
interface BatchProgressPayload {
  batch_id: string;
  status: number;            // 0=PENDING, 1=RUNNING, 2=DONE, 3=CANCELED, 4=ERROR
  total_files: number;
  completed_files: number;
  failed_files: number;
  current_file: string | null;
  fraction: number;
  progress_details: string | null;
}
```

#### 7.2.6 backend-status

```typescript
interface BackendStatusPayload {
  connected: boolean;
  gpu_backend: string | null;
  memory_mb: number | null;
  version: string | null;
}
```

### 7.3 useTauriEvent Hook 实现

```typescript
// hooks/useTauriEvent.ts
import { useEffect, useRef } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export function useTauriEvent<T>(
  event: string,
  handler: (payload: T) => void
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<T>(event, (e) => handlerRef.current(e.payload)).then(fn => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [event]);
}
```

---

## 8. 内部接口: Zustand Stores

> 7 个独立 Zustand Store，各自暴露 state + actions + getters。

### 8.1 useAppStore

管理全局应用状态：模式切换、主题、后端连接状态。

```typescript
interface AppState {
  mode: 'edit' | 'batch';
  theme: 'dark' | 'light' | 'system';
  isBackendConnected: boolean;
  backendInfo: BackendInfo | null;
  statusMessage: string;
  recentFiles: string[];
}

interface BackendInfo {
  gpuBackend: string;
  memoryMb: number;
  version: string;
}

interface AppActions {
  setMode: (mode: 'edit' | 'batch') => void;
  setTheme: (theme: 'dark' | 'light' | 'system') => void;
  setBackendStatus: (connected: boolean, info?: BackendInfo | null) => void;
  setStatusMessage: (msg: string) => void;
  addRecentFile: (path: string) => void;
}
```

### 8.2 useFilmstripStore

管理导入的图像列表、选中状态、分组和缩略图。

```typescript
interface FilmstripState {
  images: ImageInfo[];
  selectedIndices: Set<number>;
  groups: ImageGroup[];
  sortKey: 'name' | 'size' | 'format' | 'iso' | 'date';
  thumbnailSize: 'S' | 'M' | 'L';
  thumbnailCache: Map<string, string>;  // path -> base64 data URL
  isImporting: boolean;
}

interface ImageGroup {
  name: string;
  condition: string;
  imageIds: string[];
}

interface FilmstripActions {
  importImages: (paths: string[]) => Promise<void>;
  removeImages: (indices: number[]) => void;
  toggleSelect: (index: number, ctrl: boolean, shift: boolean) => void;
  selectAll: () => void;
  clearSelection: () => void;
  setSortKey: (key: SortKey) => void;
  setThumbnailSize: (size: 'S' | 'M' | 'L') => void;
  sendToBatch: (indices: number[]) => void;
  createGroup: (name: string, condition: string) => void;
  deleteGroup: (name: string) => void;
  autoGroup: (strategy: 'iso' | 'make' | 'lens' | 'date') => void;
}

// Getters
interface FilmstripGetters {
  sortedImages: () => ImageInfo[];
  selectedImages: () => ImageInfo[];
  groupedImages: () => Map<string, ImageInfo[]>;
  selectionCount: () => number;
}
```

### 8.3 usePluginStore

管理插件目录、分类和节点 Schema 缓存。

```typescript
interface PluginState {
  plugins: PluginEntry[];
  categories: string[];
  nodeSchemas: Map<string, NodeSchemaResponse>;
  searchQuery: string;
  categoryFilter: string;
  isLoading: boolean;
  error: string | null;
}

interface PluginActions {
  fetchPlugins: () => Promise<void>;
  fetchNodeSchema: (pluginId: string) => Promise<NodeSchemaResponse>;
  setSearchQuery: (query: string) => void;
  setCategoryFilter: (category: string) => void;
  clearSchemaCache: () => void;
}

interface PluginGetters {
  filteredPlugins: () => PluginEntry[];
  pluginsByCategory: () => Map<string, PluginEntry[]>;
  getCachedSchema: (pluginId: string) => NodeSchemaResponse | undefined;
}
```

### 8.4 usePipelineStore

管理 DAG 图形编辑器状态 (节点、边、选择、撤销/重做、执行)。

```typescript
interface PipelineState {
  nodes: DAGNodeData[];
  edges: DAGEdgeData[];
  selectedNodeId: string | null;
  zoom: number;              // [0.1, 5.0]
  panOffset: { x: number; y: number };
  isDirty: boolean;
  executionState: 'idle' | 'running' | 'paused' | 'error';
  currentProgress: PipelineProgressPayload | null;
  undoStack: PipelineSnapshot[];
  redoStack: PipelineSnapshot[];
  pipelineName: string;
  currentFilePath: string | null;
}

interface PipelineSnapshot { nodes: DAGNodeData[]; edges: DAGEdgeData[]; }

interface PipelineActions {
  // 节点
  addNode: (pluginId: string, position?: {x:number;y:number}) => string;
  removeNode: (id: string) => void;
  moveNode: (id: string, pos: {x:number;y:number}) => void;
  toggleNodeEnabled: (id: string) => void;
  // 边
  connectEdge: (fromNode: string, toNode: string) => boolean;
  removeEdge: (id: string) => void;
  // 画布
  selectNode: (id: string | null) => void;
  setZoom: (zoom: number) => void;
  setPanOffset: (offset: {x:number;y:number}) => void;
  // 文件
  newPipeline: () => void;
  savePipeline: (path: string) => Promise<void>;
  loadPipeline: (path: string) => Promise<void>;
  // 控制
  validatePipeline: () => Promise<ValidationResult>;
  executePipeline: () => Promise<void>;
  cancelExecution: () => Promise<void>;
  // 撤销重做
  undo: () => void;
  redo: () => void;
  pushSnapshot: () => void;
}

interface PipelineGetters {
  selectedNode: () => DAGNodeData | null;
  selectedNodePluginId: () => string | null;
  canUndo: () => boolean;
  canRedo: () => boolean;
  hasCycle: () => boolean;
  topologicalOrder: () => string[] | null;
}
```

### 8.5 useOverrideStore

管理参数覆盖的五级优先级系统。

```typescript
type OverrideScope = 'all' | 'template' | 'group' | 'image';
type OverrideSource = 'plugin_default' | 'template' | 'group' | 'image' | 'expression';

interface OverrideState {
  scope: OverrideScope;
  activeGroupName: string | null;
  activeImageIndex: number | null;
  overrides: Map<string, OverrideEntry>;
  expressions: Map<string, string>;
  dirtyParams: Set<string>;
}

interface OverrideEntry {
  value: any;
  source: OverrideSource;
  sourceName?: string;
  timestamp: number;
}

interface ValueWithSource {
  value: any;
  source: OverrideSource;
  sourceName?: string;
  isOverridden: boolean;
  isExpression: boolean;
  isEditable: boolean;
}

interface OverrideActions {
  setScope: (scope: OverrideScope, groupName?: string, imageIndex?: number) => void;
  setOverride: (nodeId: string, paramId: string, value: any) => void;
  clearOverride: (nodeId: string, paramId: string) => void;
  setExpression: (nodeId: string, paramId: string, expr: string) => void;
  clearExpression: (nodeId: string, paramId: string) => void;
  getEffectiveValue: (nodeId: string, paramId: string, defaultValue: any) => ValueWithSource;
  getSectionOverrideInfo: (nodeId: string, sectionId: string) => { totalFields: number; overriddenFields: number; hasExpressions: boolean; };
  hasVaryingValues: (nodeId: string, paramId: string) => boolean;
  clearAllOverrides: () => void;
}
```

**参数解析优先级**: 表达式 > 图像 > 分组 > 模板 > 插件默认

### 8.6 useBatchStore

管理批量处理队列、进度和输出设置。

```typescript
interface BatchState {
  queue: BatchItem[];
  batchState: 'idle' | 'running' | 'paused' | 'done';
  currentBatchId: string | null;
  progress: BatchProgressInfo | null;
  outputSettings: BatchOutputSettings;
  perImageOverrides: Map<string, Record<string, Record<string, any>>>;
}

interface BatchItem {
  image: ImageInfo;
  status: 'queued' | 'processing' | 'done' | 'failed';
  errorMessage?: string;
  elapsedMs?: number;
}

interface BatchProgressInfo {
  done: number; failed: number; total: number;
  currentFile: string | null; fraction: number;
  elapsedSecs: number; etaSecs: number; speedPerMin: number;
}

interface BatchOutputSettings {
  directory: string; format: string; quality: number; bitDepth: number;
  chromaSubsampling: string; outputPattern: string; parallel: number; resume: boolean;
}

interface BatchActions {
  addToQueue: (images: ImageInfo[]) => void;
  removeFromQueue: (indices: number[]) => void;
  startBatch: () => Promise<void>;
  pauseBatch: () => void;
  resumeBatch: () => Promise<void>;
  stopBatch: () => Promise<void>;
  clearDone: () => void;
  updateProgress: (info: BatchProgressInfo) => void;
  setOutputSetting: <K extends keyof BatchOutputSettings>(key: K, value: BatchOutputSettings[K]) => void;
  setPerImageOverride: (imageId: string, nodeId: string, paramId: string, value: any) => void;
  clearPerImageOverrides: (imageId: string) => void;
}
```

### 8.7 useSettingsStore

管理应用设置 (持久化到 `%APPDATA%/Photopipeline/appsettings.json`)。

```typescript
interface SettingsState {
  settings: AppSettings;
  snapshot: AppSettings | null;
  isDirty: boolean;
  isLoading: boolean;
}

interface SettingsActions {
  load: () => Promise<void>;
  save: () => Promise<void>;
  reset: () => void;
  cancel: () => void;
  update: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
}
```

### 8.8 Store 间依赖关系

```
useAppStore
  ├── usePluginStore (启动时 fetchPlugins)
  ├── useSettingsStore (启动时 load)
  │
usePipelineStore
  ├── → usePluginStore (selectNode -> fetchNodeSchema)
  ├── → useOverrideStore (scope 切换)
  ├── → useBatchStore (sendToBatch)
  │
useFilmstripStore
  ├── → useBatchStore (sendToBatch)
  │
useOverrideStore
  ├── → usePipelineStore (监听 selectedNodeId)
  │
useBatchStore
  ├── → usePipelineStore (读取 nodes/edges)
  ├── → useFilmstripStore (读取 images)
```

---

## 9. 内部接口: React 组件 Props

### 9.1 组件层级树

```
App
├── TitleBar
├── MainLayout (三栏: 左-中-右)
│   ├── LeftPanel (插件浏览器)
│   │   ├── PluginSearch
│   │   └── PluginGrid → PluginCard[]
│   ├── CenterPanel (DAG + 胶片)
│   │   ├── DagToolbar
│   │   ├── DagCanvas → DAGNode[] + DAGEdge[]
│   │   └── Filmstrip → ImageCard[]
│   └── RightPanel (控制 + 预览)
│       ├── ControlPanel → PluginHeader + ContextBar + ParamSection[] → ParamRow[]
│       └── PreviewView
├── StatusBar
├── SettingsDialog
└── BatchPanel
    ├── BatchQueueGrid → BatchQueueRow[]
    └── BatchProgressFooter
```

### 9.2 核心组件 Props 定义

```typescript
// ImageCard
interface ImageCardProps {
  image: ImageInfo;
  index: number;
  state: 'default' | 'hover' | 'selected' | 'multi-selected';
  thumbnail: string | null;        // base64 Data URL, null=loading
  onSelect: (index: number, ctrl: boolean, shift: boolean) => void;
  onContextMenu: (e: React.MouseEvent, index: number) => void;
  onDoubleClick: (index: number) => void;
}

// DAGNode
interface DAGNodeProps {
  data: DAGNodeData;
  selected: boolean;
  isExecuting: boolean;
  executionStatus: 'idle' | 'running' | 'done' | 'error';
  onDragEnd: (id: string, pos: { x: number; y: number }) => void;
  onPortDragStart: (nodeId: string, portId: string, portType: 'input' | 'output') => void;
  onPortDrop: (nodeId: string, portId: string) => void;
}

// DAGEdge
interface DAGEdgeProps {
  from: { x: number; y: number };
  to: { x: number; y: number };
  selected: boolean;
  data: DAGEdgeData;
  onDelete: (id: string) => void;
}

// PluginCard
interface PluginCardProps {
  plugin: PluginEntry;
  highlighted: boolean;
  onDragStart: (pluginId: string) => void;
  onClick: (pluginId: string) => void;
}

// PluginHeader
interface PluginHeaderProps {
  name: string;
  version: string;
  category: string;
  description: string;
  icon?: string | null;
  color?: string | null;
  minRamMb?: number;
  capabilities: CapabilityTag[];
}
type CapabilityTag = 'pixel' | 'metadata' | 'format' | 'network' | 'filesystem' | 'gpu' | 'ai';

// ControlPanel
interface ControlPanelProps {
  nodeId: string | null;
}

// ParamSection
interface ParamSectionProps {
  section: ParameterSection;
  nodeId: string;
  collapsible: boolean;
  defaultCollapsed: boolean;
  style: 'card' | 'collapsible_card' | 'plain';
  titleVisible: boolean;
}

// ParamRow
interface ParamRowProps {
  field: ParameterField;
  nodeId: string;
  value: any;
  source: OverrideSource;
  sourceName?: string;
  isOverridden: boolean;
  isExpression: boolean;
  isEditable: boolean;
  onChange: (value: any) => void;
  onOverrideToggle: () => void;
  onExpressionEdit: () => void;
  onHelpClick: () => void;
}

// ContextBar
interface ContextBarProps {
  scopes: Array<{ id: OverrideScope; label: string }>;
  activeScope: OverrideScope;
  activeContextName?: string;
  onScopeChange: (scope: OverrideScope) => void;
}

// PreviewView
interface PreviewViewProps {
  mode: PreviewMode;
  imagePath: string | null;
  maxWidth: number;
  maxHeight: number;
  isLoading: boolean;
  error: string | null;
  onRefresh?: () => void;
}

// BatchQueueRow
interface BatchQueueRowProps {
  item: BatchItem;
  index: number;
  onClick: (index: number) => void;
  onEditOverrides: (imageId: string) => void;
  onRemove: (index: number) => void;
}

// SettingsTab
interface SettingsTabProps {
  settings: AppSettings;
  onChange: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
  readOnly?: boolean;
}
```

---

## 10. 内部接口: TypeScript 类型定义

### 10.1 types/plugin.ts

```typescript
interface PluginEntry {
  id: string; name: string; version: string; category: string;
  description: string; tags: string[];
  requires_pixel_access: boolean; requires_network: boolean;
  requires_filesystem: boolean; min_ram_mb: number;
}

interface NodeSchemaResponse {
  plugin_id: string; name: string; version: string; category: string;
  description: string;
  parameter_schema: ParameterSchema;   // JSON.parse 后
  gui_schema: GuiSchema;               // JSON.parse 后
}

interface ListPluginsResult {
  plugins: PluginEntry[];
  categories: string[];
}
```

### 10.2 types/pipeline.ts

```typescript
interface DAGNodeData {
  id: string; pluginId: string; label: string; enabled: boolean;
  position: { x: number; y: number };
  params: Record<string, any>;
  inputs: string[]; outputs: string[];
}

interface DAGEdgeData {
  id: string; fromNode: string; toNode: string;
}

interface PipelineSnapshot {
  nodes: DAGNodeData[]; edges: DAGEdgeData[];
}
```

### 10.3 types/image.ts

```typescript
interface ImageInfo {
  id: string; path: string; filename: string; format: string;
  width: number; height: number; file_size_bytes: number;
  pixel_format: string; color_space: string;
  metadata?: MetadataInfo;
}

interface MetadataInfo {
  make?: string; model?: string; lens_model?: string;
  date_time_original?: string; exposure_time?: string;
  f_number?: string; iso?: number; focal_length?: string;
  latitude?: number; longitude?: number; altitude?: number;
}
```

### 10.4 types/params.ts

```typescript
interface ParameterSchema {
  version: number;
  sections: ParameterSection[];
}

interface ParameterSection {
  id: string; label: string; description?: string | null;
  icon?: string | null;
  collapsible: boolean; default_collapsed: boolean;
  fields: ParameterField[];
}

interface ParameterField {
  // 公共
  id: string; label: string; description?: string | null;
  help_url?: string | null;
  type: ValueType;
  default: any; required: boolean; advanced: boolean;
  allow_override: boolean; supports_expression: boolean;
  condition?: Condition | null;
  // 类型特定 (按 type 有不同字段集)
  max_length?: number; pattern?: string | null; placeholder?: string | null;
  min?: number; max?: number; step?: number; precision?: number;
  unit?: string | null; logarithmic?: boolean;
  style?: string;
  label_true?: string | null; label_false?: string | null;
  options?: EnumOption[]; display?: EnumDisplay;
  mode?: ColorMode; show_alpha?: boolean;
  kind?: FilePathKind; filters?: [string, string][]; must_exist?: boolean;
  alt_required?: boolean; direction_required?: boolean;
  show_ticks?: boolean; ticks?: number[] | null; show_value?: boolean;
  orientation?: SliderOrientation;
  presets?: [string, number][];
  variables?: VariableDef[];
  preset_schema_ref?: string; builtin_presets?: NamedPreset[];
  allow_custom?: boolean; allow_import?: boolean;
  element?: ParameterField; min_items?: number; max_items?: number;
  show_track?: boolean; show_photos?: boolean; allow_manual_pin?: boolean;
  zoom_levels?: number[]; show_histogram?: boolean;
  fields?: ParameterField[];
}

// 枚举字面量
type ValueType = 'string' | 'integer' | 'float' | 'boolean' | 'enum' | 'color'
  | 'file_path' | 'coordinate' | 'slider' | 'combo_slider' | 'expression'
  | 'preset' | 'array' | 'map_widget' | 'before_after' | 'separator' | 'section';

type IntegerWidget = 'spin_box' | 'slider' | 'combo';
type FloatWidget = 'spin_box' | 'slider' | 'combo_slider' | 'drag_input';
type SliderOrientation = 'horizontal' | 'vertical';
type SliderStyle = 'continuous' | 'discrete' | 'range' | 'dual_handle';
type EnumDisplay = 'dropdown' | 'radio_group' | 'button_group'
  | 'segmented_control' | 'tabs' | 'popup_card';
type ColorMode = 'RGB' | 'RGBA' | 'HSL' | 'HSV' | 'Lab';
type FilePathKind = 'file' | 'directory' | 'save_file';

interface EnumOption {
  value: string; label: string; description?: string | null;
  icon?: string | null; tags: string[]; recommended?: boolean;
}

interface VariableDef {
  name: string; description: string; var_type: string; example?: string;
}

interface NamedPreset {
  name: string; description?: string; params: Record<string, any>;
}

interface Condition {
  field?: string; op: string; value?: any; conditions?: Condition[];
}

interface GuiSchema {
  layout: GuiLayout;
  icon?: string | null; color?: string | null;
  preview: PreviewMode;
  aux_views: AuxView[];
  min_panel_width: number;
}

interface GuiLayout {
  Standard?: { sections: SectionLayout[] };
  Custom?: { rows: CustomRow[] };
}

interface SectionLayout {
  param_section_id: string; title_visible: boolean;
  style: 'card' | 'collapsible_card' | 'plain';
}

interface CustomRow {
  height: string; cells: CellDef[];
}

interface CellDef {
  param_field_id: string; width_fraction: number;
  label_position: 'top' | 'left' | 'none';
}

type PreviewMode = 'none' | 'live' | 'manual_refresh'
  | { BeforeAfter: { default_split: number; orientation: string; lock_zoom: boolean } }
  | { Tiled: { rows: number; cols: number } };

type AuxView = 'histogram' | 'waveform' | 'vectorscope' | 'gamut_diagram' | 'map'
  | 'focus_peaking' | 'clipping_warning' | 'metadata_table'
  | 'progress_bar' | 'status_text';
```

### 10.5 types/override.ts

```typescript
type OverrideScope = 'all' | 'template' | 'group' | 'image';
type OverrideSource = 'plugin_default' | 'template' | 'group' | 'image' | 'expression';

interface OverrideEntry {
  value: any; source: OverrideSource; sourceName?: string; timestamp: number;
}

interface ValueWithSource {
  value: any; source: OverrideSource; sourceName?: string;
  isOverridden: boolean; isExpression: boolean; isEditable: boolean;
}
```

### 10.6 types/batch.ts

```typescript
interface BatchItem {
  image: ImageInfo;
  status: 'queued' | 'processing' | 'done' | 'failed';
  errorMessage?: string; elapsedMs?: number;
}

interface BatchProgressInfo {
  done: number; failed: number; total: number;
  currentFile: string | null; fraction: number;
  elapsedSecs: number; etaSecs: number; speedPerMin: number;
}

interface BatchOutputSettings {
  directory: string; format: string; quality: number; bitDepth: number;
  chromaSubsampling: string; outputPattern: string; parallel: number; resume: boolean;
}
```

### 10.7 types/settings.ts

```typescript
interface AppSettings {
  theme: 'dark' | 'light' | 'system'; language: string;
  maxRecentFiles: number; checkUpdates: boolean; telemetry: boolean;
  serverPath: string; port: number; autoStart: boolean;
  gpuBackend: string; logLevel: string;
  defaultFormat: string; defaultDirectory: string; jpegQuality: number;
  embedMetadata: boolean; thumbnailSize: number;
  tileSize: number; cacheDirectory: string; maxCacheSize: number;
  exifToolPath: string;
}
```

### 10.8 types/events.ts

```typescript
interface PipelineProgressPayload {
  node_id: string | null; node_label: string | null;
  fraction: number; message: string; elapsed_ms: number;
}

interface StageTransitionPayload {
  stage: 'LOADING' | 'DECODING' | 'PROCESSING' | 'ENCODING' | 'DONE' | 'ERROR';
  previous_stage: 'LOADING' | 'DECODING' | 'PROCESSING' | 'ENCODING' | 'DONE' | 'ERROR';
}

interface PipelineErrorPayload {
  node_id: string | null; code: string; message: string;
}

interface PipelineDonePayload {
  output_paths: string[]; total_bytes: number; total_seconds: number;
}

interface BatchProgressPayload {
  batch_id: string; status: number;
  total_files: number; completed_files: number; failed_files: number;
  current_file: string | null; fraction: number; progress_details: string | null;
}

interface BackendStatusPayload {
  connected: boolean; gpu_backend: string | null;
  memory_mb: number | null; version: string | null;
}
```

### 10.9 types/errors.ts

```typescript
type GrpcStatusCode = 'OK' | 'CANCELLED' | 'NOT_FOUND' | 'INVALID_ARGUMENT'
  | 'INTERNAL' | 'UNAVAILABLE' | 'DEADLINE_EXCEEDED' | 'RESOURCE_EXHAUSTED'
  | 'ALREADY_EXISTS' | 'UNIMPLEMENTED' | 'UNKNOWN';

type PluginErrorCode = 'PLUGIN_NOT_FOUND' | 'INVALID_PARAMETER' | 'GPU_NOT_AVAILABLE'
  | 'GPU_OUT_OF_MEMORY' | 'TIMEOUT' | 'INTERNAL_ERROR' | 'CANCELED' | 'IO_ERROR'
  | 'VALIDATION_FAILED' | 'NODE_EXECUTION_FAILED' | 'CIRCULAR_DEPENDENCY'
  | 'FILE_NOT_FOUND' | 'UNSUPPORTED_FORMAT' | 'ENCODING_FAILED' | 'DECODING_FAILED'
  | 'CONFIG_ERROR' | 'EXPRESSION_ERROR' | 'MISSING_TOOL' | 'OTHER'
  | 'PLUGIN_ALREADY_LOADED' | 'PLUGIN_LOAD_FAILED' | 'PLUGIN_VERSION_MISMATCH';

interface GrpcError { code: GrpcStatusCode; message: string; details?: string; }
interface PluginError { code: PluginErrorCode; message: string; pluginId?: string; nodeId?: string; paramId?: string; details?: Record<string, any>; }
```

---

## 11. 错误代码体系

### 11.1 gRPC 状态码映射

| gRPC Status | 数值 | 含义 | 前端处理 |
|------------|------|------|---------|
| `NOT_FOUND` | 5 | 资源不存在 | Toast: "{resource} not found" |
| `INVALID_ARGUMENT` | 3 | 参数校验失败 | ValidationResult 弹窗 |
| `INTERNAL` | 13 | 服务端内部错误 | StatusBar 红色, DAGNode 红色高亮 |
| `UNAVAILABLE` | 14 | 服务不可用 | StatusBar "Disconnected", 全部 disabled |
| `CANCELLED` | 1 | 操作被取消 | 静默处理 |
| `DEADLINE_EXCEEDED` | 4 | 操作超时 | Toast: "Timed out after {timeout}s" |
| `RESOURCE_EXHAUSTED` | 8 | 资源耗尽 | Toast: "GPU not available" |
| `ALREADY_EXISTS` | 6 | 已存在 | Toast: "{plugin} already loaded" |
| `UNIMPLEMENTED` | 12 | 不支持 | Toast: "Format not supported" |

### 11.2 PluginError 完整错误码表 (22 种)

| # | 错误码 | 描述 | 前端处理级别 |
|---|--------|------|------------|
| 1 | `PLUGIN_NOT_FOUND` | 插件未注册 | Toast Error |
| 2 | `PLUGIN_ALREADY_LOADED` | 插件已加载 | Toast Warning |
| 3 | `PLUGIN_LOAD_FAILED` | 插件加载失败 | Toast Error |
| 4 | `PLUGIN_VERSION_MISMATCH` | 版本不匹配 | Toast Warning |
| 5 | `INVALID_PARAMETER` | 参数无效 | ValidationResult issues |
| 6 | `MISSING_TOOL` | 外部工具缺失 | Toast Error + Settings link |
| 7 | `GPU_NOT_AVAILABLE` | GPU 不可用 | Toast Warning, 回退 CPU |
| 8 | `GPU_OUT_OF_MEMORY` | GPU 显存不足 | Toast Error |
| 9 | `EXPRESSION_ERROR` | 表达式错误 | 参数行红色高亮 |
| 10 | `TIMEOUT` | 处理超时 | Toast Error + 节点红色 |
| 11 | `INTERNAL_ERROR` | 内部错误 | Toast Error + bug report |
| 12 | `CANCELED` | 用户取消 | 静默处理 |
| 13 | `IO_ERROR` | IO 失败 | Toast Error |
| 14 | `VALIDATION_FAILED` | 管线验证失败 | ValidationResult 弹窗 |
| 15 | `NODE_EXECUTION_FAILED` | 节点执行失败 | DAGNode 红色 + StatusBar |
| 16 | `CIRCULAR_DEPENDENCY` | 循环依赖 | Toast Error, 连线拒绝 |
| 17 | `FILE_NOT_FOUND` | 文件未找到 | Toast Error, Filmstrip 移除 |
| 18 | `UNSUPPORTED_FORMAT` | 不支持的格式 | Toast Error |
| 19 | `ENCODING_FAILED` | 编码失败 | DAGNode 红色 |
| 20 | `DECODING_FAILED` | 解码失败 | Toast Error |
| 21 | `CONFIG_ERROR` | 配置错误 | Toast Error + 详情 |
| 22 | `OTHER` | 其他错误 | Toast Error |

### 11.3 前端错误处理策略矩阵

| 场景 | 视觉表现 | 用户通知 | 操作影响 | 恢复方式 |
|------|---------|---------|---------|---------|
| 后端未连接 | StatusBar 红点 "Disconnected" | Toast (首次) | 全部 disabled | 自动/手动启动后端 |
| 管线验证失败 | Validation 弹窗 | 弹窗 | Run disabled (ERROR) | 修正参数后重新 Validate |
| 管线执行错误 | DAGNode 红框+红色灯, 参数高亮 | Toast + StatusBar | Run 仍可用 | 修正参数重新 Run |
| 批量任务失败 | BatchQueueRow 红色圆点, 统计 "N failed" | BatchPanel 统计 | 其他文件继续 | 单独重试 |
| 文件不存在 | ImageCard 红色叉号 | Toast: "File not found" | 从 Filmstrip 移除 | 重新 Import |
| 缩略图加载失败 | ImageCard 占位图标 | 无 (静默) | 不影响其他操作 | 重新请求 |
| 表达式错误 | 参数行红色边框 + Tooltip | 内联提示 | 使用上次有效值 | 修正表达式 |

---

## 12. 接口交互流程

### 12.1 App 启动流程

```
App.main()
  ├─ [Tauri Setup Hook]
  │   ├─ 1. 检查 photopipeline 二进制
  │   ├─ 2. spawn("photopipeline serve --port 50051")
  │   ├─ 3. 健康检查轮询 (10 x 500ms)
  │   │     成功 → emit("backend-status", {connected:true, ...})
  │   │     失败 → emit("backend-status", {connected:false})
  │   └─ 4. 启动 5 秒轮询定时器
  │
  ├─ [React Mount]
  │   ├─ 5. usePluginStore.fetchPlugins()
  │   │      → invoke("list_plugins") → gRPC ListPlugins
  │   ├─ 6. useSettingsStore.load()
  │   │      → invoke("load_settings") → fs read
  │   └─ 7. useAppStore.setBackendStatus(connected)
  │
  └─ [UI Ready]
```

### 12.2 图片导入流程

```
User Click Import
  ├─ 1. Tauri dialog.open({multiple, filters:[image]}) → paths[]
  ├─ 2. useFilmstripStore.importImages(paths)
  │      ├─ invoke("load_images", {paths})
  │      │    → gRPC ImageService.Load x N → ImageInfo[]
  │      └─ 异步加载缩略图 (每张图片独立并行):
  │           invoke("get_thumbnail", {path, maxSize})
  │             → gRPC ImageService.GetThumbnail → JPEG bytes
  │             → new Blob + URL.createObjectURL → base64 data URL
  │             → useFilmstripStore.thumbnailCache.set(path, url)
  └─ 3. UI: Filmstrip 新增 ImageCard (占位图 → 缩略图渐入)
```

### 12.3 管线编辑 -- 保存 -- 加载流程

```
【编辑】
  User 拖放 PluginCard → addNode(pluginId)
  User 连线 → connectEdge(from, to) → 客户端 BFS 环检测
  User 编辑参数 → setOverride(nodeId, paramId, value)

【保存 (Ctrl+S)】
  ├─ 构建 PipelineConfig JSON
  ├─ invoke("save_pipeline_file", {path, json})
  │    → std::fs::write
  └─ isDirty = false, undoStack 压缩

【加载 (File > Open)】
  ├─ Tauri dialog.open → path
  ├─ invoke("load_pipeline_file", {path}) → JSON string
  ├─ parsePipelineConfig(json) → DAGNodeData[] + DAGEdgeData[]
  ├─ dagre 自动布局算法计算节点位置
  └─ fitView 自适应缩放渲染全部节点
```

### 12.4 管线执行完整时序图

```
User    React              Tauri Rust          Backend gRPC
───     ─────              ──────────          ────────────

▶ Run
 │
 ├─► executePipeline()
 │     │
 │     ├─ invoke("execute_pipeline") ──►
 │     │       {configJson}
 │     │                                 ├─ 保存 JSON 到临时文件
 │     │                                 ├─ gRPC CreatePipeline ─►
 │     │                                 │                          build_template()
 │     │                                 │                          validate()
 │     │                                 │                          into_graph()
 │     │                                 │   ← PipelineId ──────────┤
 │     │                                 │
 │     │                                 ├─ spawn: gRPC Execute ───►
 │     │                                 │   stream loop:
 │     │                                 │
 │     │   ← pipeline_id ────────────────┤ (立即返回)
 │     │
 │     │   emit("pipeline-progress") ◄────┤ msg.stage=LOADING
 │     │   emit("pipeline-stage")   ◄──────┤ msg.stage=DECODING
 │     │
 │     │   DAGNode 绿色闪烁,             │ msg.stage=PROCESSING
 │     │   StatusBar 进度+时间            │
 │     │
 │     │   emit("pipeline-done") ◄────────┤ msg.stage=DONE
 │     │
 │  executionState='idle'                 │
 │  DAGNode 全部绿色常亮                  │
 │  StatusBar 完成统计                    │
 │
 ▼

【错误场景】
  msg.stage=ERROR → emit("pipeline-error", {node_id, code, message})
  → DAGNode 红色边框+脉冲灯, 参数红色高亮, StatusBar 红色文字
```

### 12.5 批量处理完整流程

```
【准备阶段 (编辑模式)】
  User: Filmstrip Multi-Select → Right-click "Send to Batch"
  → useBatchStore.addToQueue(selectedImages)
  → TitleBar badge: "Batch [N]"

【切换批量模式】
  User: 点击 "Batch Processing [N]" Tab
  → useAppStore.setMode('batch')
  → CenterPanel → BatchPanel (BatchQueueGrid + OutputSettings)

【执行】
  User: ▶ "Start Batch"
  → invoke("start_batch", {configJson})
     ├─ [Rust] 保存配置到临时文件
     ├─ gRPC SubmitBatch → BatchId
     ├─ spawn: gRPC GetProgress stream → emit("batch-progress", msg)
     └─ 返回 batch_id

【进度更新 (event loop)】
  useTauriEvent("batch-progress", payload => {
    updateProgress({done, failed, total, currentFile, fraction});
  });
  UI: ProgressBar + 速度/ETA + BatchQueueRow 逐行更新
      Queued → Processing(蓝+pulse) → Done(绿) / Failed(红)

【完成】
  status=DONE → batchState='done'
  → "Done in {time}s" + [Open Output Folder] [Clear Done] 按钮
```

### 12.6 参数解析优先级数据流

```
getEffectiveValue(nodeId: "denoise_1", paramId: "sigma_color")
  │
  ├─ [第 1 层] 表达式: expressions.get("denoise_1.sigma_color")
  │   → 存在 → 求值 → { value: 14.0, source: "expression" }
  │
  ├─ [第 2 层] 图像覆盖: overrides(".", source="image")
  │   → { value: 35.0, source: "image", sourceName: "DSC0034.ARW" }
  │
  ├─ [第 3 层] 分组覆盖: overrides(".", source="group")
  │   → { value: 25.0, source: "group", sourceName: "High ISO" }
  │
  ├─ [第 4 层] 模板参数: overrides(".", source="template")
  │   → { value: 10.0, source: "template" }
  │
  └─ [第 5 层] 插件默认: ParameterField.default = 3.0
      → { value: 3.0, source: "plugin_default" }

  优先级: 表达式 > 图像 > 分组 > 模板 > 默认
  allow_override=false: 第 4 层后锁定, 第 2、3 层覆盖不生效
```

---

## 附录A: 前后端接口对齐检查清单

| # | 后端定义 | 前端对应 | 章节 | 状态 |
|---|---------|---------|------|:--:|
| 1 | `PluginService.ListPlugins` | `list_plugins` command | 2.2.1 / 6.1.1 | OK |
| 2 | `PluginService.GetNodeSchema` | `get_node_schema` command | 2.2.2 / 6.1.2 | OK |
| 3 | `ImageService.Load` | `load_images` command | 2.3.1 / 6.2.1 | OK |
| 4 | `ImageService.Decode` | `decode_preview` command | 2.3.2 / 6.2.3 | OK |
| 5 | `ImageService.GetThumbnail` | `get_thumbnail` command | 2.3.3 / 6.2.2 | OK |
| 6 | `ImageService.Encode` | 管线 ENCODING 阶段 | 2.3.4 | OK |
| 7 | `PipelineService.CreatePipeline` | `execute_pipeline` 内部 | 2.4.1 / 6.3.4 | OK |
| 8 | `PipelineService.Execute` stream | `pipeline-progress` event | 2.4.2 / 7.2.1 | OK |
| 9 | `PipelineService.Validate` | `validate_pipeline` command | 2.4.3 / 6.3.3 | OK |
| 10 | `BatchService.SubmitBatch` | `start_batch` 内部 | 2.5.1 / 6.4.1 | OK |
| 11 | `BatchService.GetProgress` stream | `batch-progress` event | 2.5.2 / 7.2.5 | OK |
| 12 | `BatchService.Cancel` | `cancel_batch` command | 2.5.3 / 6.4.2 | OK |
| 13 | `ExecutionService` v2 | 预留适配 | 2.6 | PLANNED |
| 14 | `PipelineConfig` JSON | TS type + 序列化 | 3 / 10.2 | OK |
| 15 | `ParameterSchema` (21 types) | TS type + widget 映射 | 4 / 10.4 | OK |
| 16 | `GuiSchema` (2+5+10) | TS type + ControlPanel | 5 / 10.4 | OK |
| 17 | `Condition` (8 操作符) | `evaluateCondition()` | 4.5 | OK |
| 18 | `Stage` 枚举 (6 stages) | `pipeline-stage` event | 2.4.2 | OK |
| 19 | `Status` 枚举 (5) | `BatchQueueRow` 5 态 | 2.5.2 | OK |
| 20 | `Severity` 枚举 (3) | `ValidationResult` 展示 | 2.4.3 / 11.4 | OK |
| 21 | `PluginError` (22 种) | 错误码表 + 处理矩阵 | 11.2 / 11.3 | OK |
| 22 | `ParameterResolver` 五级 | `getEffectiveValue()` | 8.5 / 12.6 | OK |
| 23 | `ProgressSink` | events + cancel | 6.3.5 / 7.2.3 | OK |

---

## 附录B: 类型别名对照表

### B.1 Rust -> TypeScript 类型映射

| Rust | TypeScript | 说明 |
|------|-----------|------|
| `PluginId = String` | `string` | 插件 ID |
| `NodeId = Uuid` | `string` | 节点 ID |
| `ImageId = Uuid` | `string` | 图像 ID |
| `BatchId = Uuid` | `string` | 批次 ID |
| `u32` / `u64` / `i64` | `number` | 整数 |
| `f32` / `f64` | `number` | 浮点数 |
| `bool` | `boolean` | 布尔值 |
| `Vec<T>` | `T[]` | 数组 |
| `HashMap<K,V>` | `Record<K,V>` / `Map<K,V>` | 键值对 |
| `Option<T>` | `T \| null` | 可选值 |
| `bytes` (proto) | `number[]` / `Uint8Array` | 二进制 |
| `google.protobuf.Struct` | `object` (JSON.parse) | 动态 JSON |
| `google.protobuf.Empty` | `void` | 空 |

### B.2 PluginCategory 枚举对照

| Rust | 值 | 前端显示 |
|------|-----|---------|
| `Input` | `"input"` | "Input" (蓝色) |
| `Metadata` | `"metadata"` | "Metadata" (灰色) |
| `Color` | `"color"` | "Color" (橙色) |
| `Transform` | `"transform"` | "Transform" (紫色) |
| `Enhance` | `"enhance"` | "Enhance" (绿色) |
| `Merge` | `"merge"` | "Merge" (青色) |
| `Format` | `"format"` | "Format" (黄色) |
| `External` | `"external"` | "External" (红色) |
| `Custom(String)` | `"custom:*"` | Custom |

---

> **文档结束**
>
> 本文档覆盖:
> - 13 个 gRPC RPC 方法的完整 Proto 定义、字段表、JSON 示例
> - 13 个 Tauri Commands 的 Rust + TypeScript 双语言签名
> - 6 个 Tauri Events 的完整 Payload 类型定义
> - PipelineConfig JSON 格式 (所有嵌套结构)
> - ParameterSchema 全部 21 种 ValueType + 前端控件映射
> - GuiSchema 完整格式 (2 Layout + 5 PreviewMode + 10 AuxView)
> - 7 个 Zustand Store 的完整 State/Actions/Getters 接口
> - 12 个核心 React 组件的 Props 接口
> - 22 种 PluginError 错误码 + gRPC 映射 + 前端处理矩阵
> - 6 个完整的接口交互流程/时序图
> - 23 项前后端接口对齐检查清单
>
> 与后端 `doc/INTERFACE_DESIGN.md` (2550 行) 完全对齐。

# Photopipeline 架构设计文档

**文档版本**: 2.0  
**撰写日期**: 2026-05-28  
**项目代码**: photopipeline (Ultra-High-Precision Cross-Platform Image Post-Processing Pipeline)  
**文档语言**: 简体中文  

---

## 目录

1. [项目概览与设计目标](#1-项目概览与设计目标)
2. [架构原则](#2-架构原则)
3. [系统架构总览](#3-系统架构总览)
4. [统一二进制设计](#4-统一二进制设计)
5. [配置驱动架构](#5-配置驱动架构)
6. [核心数据类型体系](#6-核心数据类型体系)
7. [插件架构](#7-插件架构)
8. [参数 Schema 设计](#8-参数-schema-设计)
9. [节点间数据类型契约](#9-节点间数据类型契约)
10. [gRPC 服务设计](#10-grpc-服务设计)
11. [执行引擎](#11-执行引擎)
12. [管线图模型](#12-管线图模型)
13. [参数解析器](#13-参数解析器)
14. [TileEngine 分块处理](#14-tileengine-分块处理)
15. [内置插件目录](#15-内置插件目录)
16. [数据流图](#16-数据流图)
17. [向后兼容策略](#17-向后兼容策略)
18. [安全设计](#18-安全设计)
19. [性能设计](#19-性能设计)
20. [错误处理策略](#20-错误处理策略)
21. [日志与遥测](#21-日志与遥测)
22. [项目结构](#22-项目结构)
23. [CI/CD 集成](#23-cicd-集成)
24. [开发阶段路线图](#24-开发阶段路线图)

---

## 1. 项目概览与设计目标

### 1.1 项目定位

Photopipeline 是一个超高精度跨平台图像后处理应用程序。与消费级图像工具不同，Photopipeline 面向专业摄影师和图像工作者，提供端到端的 16 位及以上精度管线处理能力。

核心特征：

| 维度 | 设计 |
|------|------|
| 语言 | Rust workspace（核心引擎 + CLI + Server），前端技术待定 |
| 计算管线 | Halide（CPU SIMD + GPU）/ 纯 Rust 后备 |
| 图像 I/O | OpenImageIO + 系统原生库（libheif / libjxl / lcms2） |
| 元数据 | ExifTool 子进程（标准）+ 内置 parser（轻量） |
| 色彩管理 | LittleCMS2 + OpenColorIO（VFX 级） |
| 前后端通信 | gRPC + protobuf（localhost） |
| 像素格式 | u8 / u16 / u32 / f16 / f32，主力 ≥16bit |
| 主力输出 | JXL 16bit（libjxl effort=7-9）+ HEIF 10bit（x265 veryslow 444 grain） |
| 插件加载 | Native .so/.dll + WASM + ExternalTool + Builtin + Remote（架构预留） |
| 配置格式 | JSON（PipelineConfig），CLI 与 GUI 共享同一文件 |
| 可执行体 | 统一二进制 `photopipeline`，子命令：serve / run / validate / schema / plugins |

### 1.2 核心设计目标

1. **精度优先**: 管线全程 16bit+，在任何节点不得降低位深度。默认工作格式为 u16 或 f32。
2. **配置驱动**: 管线定义是独立的 JSON 文件，不通过 gRPC 传递。CLI 直接读取 GUI 产出的同一配置文件执行。
3. **零拷贝传递**: 元数据通过 Arc 共享，像素数据仅在必要时触发 COW（写时复制）。
4. **实时通信**: gRPC 仅用于 GUI 实时交互（进度流、指标快照、阶段转换事件），管线定义通过文件传递。
5. **关注点分离**: 插件定义参数语义，后端不选择 GUI 控件；前端根据 ValueType 自行决定渲染方式。
6. **跨平台一致性**: 同一 PipelineConfig 文件在 Windows、macOS、Linux 上产生的输出完全一致。

### 1.3 技术栈

```
前端层 (待定)
  │
  ├── gRPC client
  │
──┼────────────────────── localhost:50051 ──────────────────────
  │
后端层 (Rust)
  ├── crates/core        — 基础类型（PixelFormat, ColorSpace, Metadata 等）
  ├── crates/plugin      — 插件框架（Plugin trait, Registry, Schema）
  ├── crates/plugins     — 14 个内置插件（每个插件一个文件）
  ├── crates/engine      — 执行引擎（DAG, NodeExecutor, ParameterResolver, TileEngine）
  ├── crates/server      — gRPC 传输 + CLI 命令（统一二进制）
  ├── crates/halide      — Halide FFI 绑定（feature-gated, CI 编译）
  ├── crates/oiio        — OpenImageIO FFI 绑定（feature-gated, CI 编译）
  └── crates/test-defs   — 测试公共定义
```

---

## 2. 架构原则

### 2.1 分层架构

系统严格遵循四层单向依赖：

```
┌──────────────────────────────────────────────┐
│ server/    gRPC 传输层                       │
│            读取 Registry，序列化到 JSON       │
├──────────────────────────────────────────────┤
│ engine/    执行引擎层                        │
│            消费 trait object，不依赖具体类型   │
├──────────────────────────────────────────────┤
│ plugins/   插件实现层                        │
│            每个插件一个文件，实现 Plugin trait │
├──────────────────────────────────────────────┤
│ plugin/    插件框架层                        │
│            Plugin trait, Registry, Schema    │
├──────────────────────────────────────────────┤
│ core/      基础类型层                        │
│            PixelFormat, ColorSpace, Metadata │
└──────────────────────────────────────────────┘
```

**依赖方向**: 上层依赖下层，下层绝不依赖上层。这意味着：
- `core/` 不依赖任何其他 crate
- `plugin/` 仅依赖 `core/`
- `plugins/` 依赖 `core/` + `plugin/`
- `engine/` 依赖 `core/` + `plugin/`（消费 trait object，不依赖 `plugins/`）
- `server/` 依赖所有层，但仅通过 `plugin/` 的 Registry 接口访问插件

### 2.2 依赖反转

引擎层（`engine/`）通过 trait object 操作插件，不依赖任何具体类型。这确保了：
- 新增插件无需修改引擎代码
- 插件可以独立测试
- 第三方插件与内置插件享有完全平等的地位

```rust
// 引擎只看到 trait object，不关心具体实现
let processor: Arc<dyn PixelProcessor> = registry.get_pixel_processor(&plugin_id)?;
let stats = processor.process_pixels(input, output, params, progress).await?;
```

### 2.3 配置与实时通信分离

这是 Photopipeline 最核心的架构决策之一：

| 通道 | 内容 | 格式 | 触发方式 |
|------|------|------|----------|
| **文件系统** | 管线配置、图片路径、参数覆盖 | PipelineConfig JSON | 用户显式保存/加载 |
| **gRPC** | 实时进度、指标快照、错误事件 | protobuf 流 | 执行过程中自动推送 |

**设计理由**:
- 管线定义是持久性资产，应存储为文件，支持版本控制、分享、备份
- gRPC 负责短暂性实时数据，不负责管线持久化
- CLI 模式无需启动 gRPC 服务器即可执行管线，大大简化了 CI/CD 集成和批量处理场景
- 前后端对管线的理解完全来自同一文件，消除了 "前后端管线状态不同步" 这一常见 Bug 来源

### 2.4 Schema 驱动 UI

后端定义参数的**值语义**（ValueType），前方端自行决定如何渲染。后端不假设 GUI 框架、控件类型或布局方式。ParameterSchema 告诉前端"这是一个 0-100 的整数，步长为 1"，前端可以选择 SpinBox、Slider 或 Combo 来展示。

### 2.5 零拷贝优先

- 元数据（Metadata）始终通过 `Arc<RwLock<Metadata>>` 共享，读操作零拷贝
- 像素数据（PixelBuffer）在单消费者场景下原地修改，不产生副本
- 多消费者场景下通过 Arc 共享只读访问
- GPU 数据保持在 VRAM 中，仅传递 GpuHandle
- COW（写时复制）仅在必要时触发

---

## 3. 系统架构总览

### 3.1 三层架构

```
┌──────────────────────────────────────────────────────────────┐
│ GUI 层（平台原生，gRPC 客户端）                               │
│  技术方案: 待定（原方案 C# WinUI3/SwiftUI/GTK4 已废弃）       │
│  职责: PipelineConfig 编辑、管线可视化、实时预览、批量管理      │
├──────────────────────────────────────────────────────────────┤
│ Server 层（Rust, localhost:50051）                            │
│  - PipelineExecutor: 执行管线                                 │
│  - PluginRegistry: 管理所有已注册插件                          │
│  - BatchScheduler: 批量处理调度                               │
│  - ParameterResolver: 四级参数合并                            │
│  - TileEngine: 大图像分块处理                                 │
├──────────────────────────────────────────────────────────────┤
│ Compute 层                                                    │
│  Halide kernels | OIIO | libheif | libjxl | lcms2            │
│  ExifTool subprocess | 商业 API stubs (v2.0)                 │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Crate 依赖关系图

```
server ──┬── engine ──┬── plugin ──┬── core
         │            │
         ├── plugins ─┤
         │            │
         └── core ────┘

halide ── core (独立，feature-gated)
oiio   ── core (独立，feature-gated)
```

### 3.3 运行时组件交互

```
启动:
  main() → Registry::new() → register_all() → Server::builder()
                                              → 注册 gRPC 服务
                                              → serve_with_shutdown()

执行请求:
  gRPC Execute(request) → build_template(spec) → NodeExecutor::execute(graph)
  → 拓扑排序 → 逐节点: resolve params → validate → process_*()
  → stream ExecuteProgress

CLI 执行:
  cli run config.json → 读取 PipelineConfig JSON
  → Registry::new() → register_all() → 同 NodeExecutor::execute(graph)
  → 控制台输出 [████████████] 进度条
```

---

## 4. 统一二进制设计

### 4.1 设计理念

整个 Photopipeline 项目编译为**单一二进制文件** `photopipeline`，通过子命令切换不同的运行模式。旧有的 `cli/` 独立目录已被合并到 `crates/server/src/commands/` 中。

### 4.2 子命令一览

```
photopipeline
├── serve        启动 gRPC 服务器（GUI 后台模式）
├── run          直接执行 PipelineConfig 文件（CLI 模式）
├── validate     验证 PipelineConfig 文件语法和语义
├── schema       导出所有插件的 ParameterSchema（用于前端代码生成）
└── plugins      列出所有已注册插件及其信息
```

### 4.3 CLI mode 与 gRPC mode 的代码共享

两种模式共享完全相同的 `execute_config()` 代码路径：

```
                  ┌─────────────────────────┐
                  │  execute_config(path)   │
                  │  (统一的入口函数)         │
                  └───────────┬─────────────┘
                              │
          ┌───────────────────┴───────────────────┐
          │                                       │
  ┌───────┴────────┐                    ┌─────────┴──────────┐
  │  CLI mode       │                    │  gRPC mode          │
  │  run 子命令     │                    │  serve 子命令       │
  │                 │                    │                     │
  │  Registry::new  │                    │  Registry::new      │
  │  register_all   │                    │  register_all       │
  │  read config    │                    │  read config        │
  │  NodeExecutor   │                    │  NodeExecutor       │
  │  ▸ execute()    │                    │  ▸ execute()        │
  │  输出到控制台    │                    │  stream to gRPC    │
  └─────────────────┘                    └─────────────────────┘
```

### 4.4 设计动机

1. **消除代码重复**: 旧架构中 CLI 和 Server 各自实现管线执行逻辑，维护成本双倍且容易出现行为分歧
2. **简化部署**: 单一二进制，无需区分 "服务器" 和 "客户端工具"
3. **CI/CD 友好**: `photopipeline run config.json` 一行命令即可在 CI 中执行完整管线
4. **调试便利**: 开发者可以在不启动 gRPC 服务器的情况下直接测试管线逻辑

---

## 5. 配置驱动架构

### 5.1 PipelineConfig 结构

管线配置是一个独立的 JSON 文件，既是 GUI 的持久化格式，也是 CLI 的输入格式。没有管线定义通过 gRPC 传递。

```json
{
  "name": "My HDR Pipeline",
  "version": "1.0.0",
  "description": "HDR processing for landscape photos",
  "pipelines": [
    {
      "id": "main",
      "label": "Main Pipeline",
      "nodes": [
        {
          "id": "source",
          "plugin": "raw_input",
          "label": "RAW Input",
          "enabled": true,
          "params": {
            "demosaic": "amaze",
            "white_balance": "camera"
          }
        },
        {
          "id": "denoise",
          "plugin": "ai_denoise",
          "label": "AI Denoise",
          "enabled": true,
          "params": {
            "strength": 0.5,
            "model": "default"
          }
        },
        {
          "id": "colorspace",
          "plugin": "colorspace",
          "label": "Color Space",
          "enabled": true,
          "params": {
            "from": "camera",
            "to": "rec2020",
            "intent": "relative_colorimetric"
          }
        },
        {
          "id": "output",
          "plugin": "heif_encoder",
          "label": "HEIF 10-bit",
          "enabled": true,
          "params": {
            "quality": 95,
            "bit_depth": 10,
            "chroma": "yuv444",
            "encoder": "x265"
          }
        }
      ],
      "edges": [
        { "from": "source", "to": "denoise" },
        { "from": "denoise", "to": "colorspace" },
        { "from": "colorspace", "to": "output" }
      ]
    }
  ],
  "images": [
    {
      "path": "photos/DSC0001.ARW",
      "pipeline": "main",
      "params": {
        "denoise": { "strength": 0.8 }
      }
    },
    {
      "path": "photos/DSC0002.ARW",
      "pipeline": "main"
    }
  ],
  "output": {
    "directory": "output/",
    "naming": "{filename}_{date}",
    "format": "heif",
    "on_conflict": "skip"
  },
  "groups": [
    {
      "name": "High ISO",
      "condition": "exif.iso >= 1600",
      "params": {
        "denoise": { "strength": 0.9 }
      }
    },
    {
      "name": "Sony Cameras",
      "condition": "exif.make == 'SONY'",
      "params": {
        "colorspace": { "to": "sgrb" }
      }
    }
  ],
  "execution": {
    "parallel": 4,
    "resume": true,
    "timeout_seconds": 3600
  }
}
```

### 5.2 字段说明

| 字段 | 类型 | 必需 | 描述 |
|------|------|:--:|------|
| `name` | String | 否 | 管线名称，用于显示 |
| `version` | String | 否 | 语义版本号 |
| `description` | String | 否 | 管线描述 |
| `pipelines` | Array | **是** | 管线定义数组，至少一个 |
| `pipelines[].id` | String | **是** | 管线唯一标识符 |
| `pipelines[].label` | String | 否 | 人类可读标签 |
| `pipelines[].nodes` | Array | **是** | 节点数组 |
| `pipelines[].nodes[].id` | String | **是** | 节点唯一 ID（在管线内唯一） |
| `pipelines[].nodes[].plugin` | String | **是** | 插件 ID，必须匹配 Registry 中注册的插件 |
| `pipelines[].nodes[].label` | String | 否 | 节点显示名称，默认使用 id |
| `pipelines[].nodes[].enabled` | Boolean | 否 | 节点是否启用，默认 true |
| `pipelines[].nodes[].params` | Object | 否 | 模板级参数覆盖 |
| `pipelines[].edges` | Array | 否 | 边数组，定义节点间数据流 |
| `pipelines[].edges[].from` | String | **是** | 源节点 id |
| `pipelines[].edges[].to` | String | **是** | 目标节点 id |
| `images` | Array | **是** | 待处理图片数组 |
| `images[].path` | String | **是** | 图片文件路径 |
| `images[].pipeline` | String | **是** | 使用的管线 id |
| `images[].params` | Object | 否 | 图片级参数覆盖（最高优先级） |
| `output` | Object | 否 | 输出配置 |
| `output.directory` | String | 否 | 输出目录，默认 "." |
| `output.naming` | String | 否 | 命名模式，支持 `{filename}`, `{date}`, `{pipeline}` |
| `output.format` | String | 否 | 输出格式，如 "heif", "jxl", "tiff" |
| `output.on_conflict` | String | 否 | 冲突策略: "skip", "overwrite", "rename" |
| `groups` | Array | 否 | 条件参数组 |
| `groups[].name` | String | **是** | 组名 |
| `groups[].condition` | String | **是** | 条件表达式（如 `exif.iso >= 800`） |
| `groups[].params` | Object | **是** | 条件满足时应用的参数覆盖 |
| `execution` | Object | 否 | 执行参数 |
| `execution.parallel` | Number | 否 | 并行处理的图片数，默认 1 |
| `execution.resume` | Boolean | 否 | 是否支持断点续传，默认 false |
| `execution.timeout_seconds` | Number | 否 | 单张图片超时时间 |

### 5.3 参数合并优先级

```
image override (优先级 3, 最高 — images[].params)
  └> group override (优先级 2, 最后匹配的组胜出 — groups[].params)
      └> template default (优先级 1 — pipelines[].nodes[].params)
          └> plugin builtin default (优先级 0, 最低 — ParameterSchema.defaults())
```

合并顺序（`ParameterResolver::resolve()`）:

1. 从插件 Schema 获取所有字段的默认值 → `result`
2. 合并模板级参数覆盖 → `result.merge(template_params)`
3. 对每个匹配的分组条件，按顺序合并分组参数 → `result.merge(group_params)`
4. 合并图片级覆盖 → `result.merge(image_params)`
5. 解析参数中的表达式 `${...}` → `resolve_expressions(result)`

**关键约束**: 标记为 `allow_override: false` 的参数，在任何层级的覆盖都会被忽略。这是通过 "模板快照" 机制实现的——在 group override 之前保存不允许覆盖的字段值，在所有合并完成后恢复。

### 5.4 PipelineConfig 的生命周期

```
创建/编辑:
  GUI 用户操作 → 生成 PipelineConfig JSON → 保存到文件

验证:
  photopipeline validate config.json
  → 检查 JSON 结构
  → 检查所有引用的 plugin ID 是否已注册
  → 检查节点间的数据类型兼容性
  → 返回验证结果

执行:
  photopipeline run config.json
  → 读取 JSON → 解析 PipelineConfig
  → 遍历 images[] → 对每个图片:
      → 构建 PipelineGraph (from pipeline nodes/edges)
      → 设置 ParameterResolver (模板/分组/图片覆盖)
      → NodeExecutor::execute(graph, image)
      → 输出到 output.directory
```

### 5.5 与旧架构的区别

| 方面 | 旧架构 | 新架构 |
|------|--------|--------|
| 配置格式 | TOML | JSON |
| 配置位置 | CLI 直接内联或 TOML 文件 | 统一 JSON 文件，GUI 和 CLI 共用 |
| 管线通过 gRPC | 是（PipelineSpec protobuf） | 否（仅通过文件） |
| 配置验证 | 分散在各处 | 统一的 validate 子命令 |
| 参数覆盖 | 三级（default/template/override） | 四级（default/template/group/image） |

---

## 6. 核心数据类型体系

所有核心类型定义在 `crates/core/src/` 中，按职责分为以下文件：

### 6.1 types.rs — 全局类型定义

```rust
// 标识符类型（全部使用类型别名提高可读性）
pub type PluginId = String;        // 插件唯一标识符，如 "colorspace"
pub type NodeId = Uuid;            // 管线节点 UUID
pub type ImageId = Uuid;           // 图片 UUID
pub type BatchId = Uuid;           // 批次 UUID
pub type PortId = Uuid;            // 端口 UUID

// 插件版本
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,       // 预发布标签，如 "alpha"
}

// 版本需求
pub struct VersionRequirement {
    pub min_version: PluginVersion,
    pub max_version: Option<PluginVersion>,  // None = 无上界
}

// 插件分类
pub enum PluginCategory {
    Input,       // 输入（raw_input）
    Metadata,    // 元数据（exif_rw, gps_set, time_shift）
    Color,       // 色彩（colorspace, lut3d）
    Transform,   // 变换（transform）
    Enhance,     // 增强（lens_correct, ai_denoise）
    Merge,       // 合成（预留）
    Format,      // 格式（heif/jxl/avif/png/tiff encoder）
    External,    // 外部工具
    Custom(String),  // 自定义
}

// GPU 后端
pub enum GpuBackend {
    None,
    CUDA,
    Metal,
    Vulkan,
    Auto,
}

// AI 后端
pub enum AiBackend {
    ONNX,
    TensorRT,
    CoreML,
    OpenVINO,
    Burn,
}

// 图像格式
pub enum ImageFormat {
    HEIF, HEIC, AVIF, JXL, PNG, TIFF, JPEG,
    WEBP, OpenEXR, RAW, DNG, PPM, PGM, BMP,
    Unknown(String),
}

// 硬件信息
pub struct HardwareInfo {
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_ram_mb: u64,
    pub gpus: Vec<GpuContext>,
}

pub struct GpuContext {
    pub backend: GpuBackend,
    pub device_name: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub compute_units: u32,
}

// 处理统计
pub struct ProcessingStats {
    pub elapsed_ms: u64,
    pub cpu_time_ms: u64,
    pub gpu_time_ms: Option<u64>,
    pub peak_memory_mb: u64,
    pub input_pixels: u64,
    pub output_pixels: u64,
}
```

### 6.2 image.rs — 图像缓冲区与数据

```rust
// 像素格式（全部 ≥8bit）
pub enum PixelFormat {
    U8,   // 1 字节/通道
    U16,  // 2 字节/通道
    U32,  // 4 字节/通道
    F16,  // 2 字节/通道 (half float)
    F32,  // 4 字节/通道 (full float)
}

// 通道布局
pub enum ChannelLayout {
    Gray,        // 1 通道
    GrayAlpha,   // 2 通道
    RGB,         // 3 通道
    RGBA,        // 4 通道
    Planar(u8),  // N 通道平面格式
    Custom(u8),  // N 通道自定义格式
}

// 页对齐缓冲区（支持 GPU 映射）
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,
}

// 像素缓冲区（核心数据结构）
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub layout: ChannelLayout,
    pub format: PixelFormat,
    pub color_space: ColorSpace,
    pub icc_profile: Option<Vec<u8>>,
    pub data: AlignedBuffer,
}

// GPU 缓冲区句柄
pub struct GpuBuffer {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: GpuBackend,
}

// AI 张量
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,
}

// 解码/编码选项
pub struct DecodeOptions {
    pub pixel_format: Option<PixelFormat>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub read_metadata: bool,
    pub apply_transfer: bool,
    pub icc_profile: Option<Vec<u8>>,
}

pub struct EncodeOptions {
    pub format: ImageFormat,
    pub quality: Option<f32>,
    pub lossless: bool,
    pub bit_depth: u8,
    pub chroma_subsampling: Option<ChromaSubsampling>,
    pub encoder: Option<String>,
    pub effort: Option<u8>,
    pub compression: Option<String>,
    pub embed_profile: Option<bool>,
}

// 分块布局
pub struct TileLayout {
    pub image_width: u32,
    pub image_height: u32,
    pub tile_size: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub overlap: u32,
    pub total_tiles: u32,
}
```

### 6.3 metadata.rs — 元数据模型

```rust
pub struct Metadata {
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub iptc: Option<IptcData>,
    pub gps: Option<GpsData>,
    pub custom: Vec<CustomTag>,
}

// EXIF 数据（65+ 字段）
pub struct ExifData {
    // 相机信息
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub serial_number: Option<String>,

    // 拍摄参数
    pub iso: Option<u32>,
    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub focal_length: Option<String>,
    pub aperture_value: Option<String>,
    pub shutter_speed_value: Option<String>,
    pub exposure_bias: Option<String>,

    // 时间
    pub date_time_original: Option<DateTime<Utc>>,
    pub date_time_digitized: Option<DateTime<Utc>>,
    pub offset_time_original: Option<String>,

    // 图像属性
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub color_space: Option<u16>,
    pub bits_per_sample: Option<Vec<u16>>,

    // 原始标签
    pub maker_note: Option<Vec<u8>>,
    pub raw_tags: Vec<RawExifTag>,
    // ... 等 65+ 字段
}

// GPS 数据
pub struct GpsData {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub img_direction: Option<f64>,
    pub speed: Option<f64>,
    pub track: Option<f64>,
    pub satellites: Option<String>,
    pub dop: Option<f64>,
    // ... 等 25+ 字段
}

// GPX 轨迹支持
pub struct GpxTrack {
    pub name: Option<String>,
    pub points: Vec<GpxPoint>,
    pub duration_seconds: Option<f64>,
    pub distance_meters: Option<f64>,
}
```

### 6.4 设计预留的数据类型（待实现）

以下类型在架构设计中已定义，将在后续版本中逐步添加到 `core/src/data_types.rs`：

```rust
// 地理坐标
pub struct GeoCoordinate {
    pub lat: f64,
    pub lng: f64,
    pub alt: Option<f64>,
}

// 带时区的时间戳
pub struct TimestampWithZone {
    pub datetime: DateTime<Utc>,
    pub tz: String,
}

// 裁剪区域
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

// 缩放规格
pub struct ResizeSpec {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: ResizeMode,   // Fit, Fill, Stretch, MaxDimension, MinDimension
}

// 镜头配置文件
pub struct LensProfile {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<f64>,
    pub aperture: Option<f64>,
}

// 编码规格
pub struct EncodingSpec {
    pub quality: Option<f32>,
    pub lossless: bool,
    pub bit_depth: u8,
    pub chroma_subsampling: Option<ChromaSubsampling>,
    pub compression: Option<String>,
}

// 节点间数据类型契约
pub struct DataTypeSpec {
    pub pixel_format: Vec<PixelFormat>,
    pub layout: Vec<ChannelLayout>,
    pub color_space: Vec<ColorSpace>,
    pub bit_depth: Option<u8>,
    pub metadata: Vec<MetadataScope>,
}
```

---

## 7. 插件架构

### 7.1 插件框架概览

插件架构是 Photopipeline 的核心扩展机制，定义了从基础元数据到高级像素处理的完整能力层次。

```
                    ┌──────────┐
                    │  Plugin   │  基础 trait（所有插件必须实现）
                    └─────┬────┘
       ┌──────────┐  ┌────┴────┐  ┌──────────────┐
       │Metadata  │  │  Pixel   │  │   Format     │
       │Processor │  │Processor │  │  Processor   │
       └──────────┘  └────┬────┘  └──────────────┘
                    ┌─────┴─────┐
                    │GpuProcessor│  ← GPU 加速扩展（v2.0）
                    └─────┬─────┘
                    ┌─────┴─────┐
                    │AiProcessor │  ← AI/ONNX 扩展
                    └────────────┘
       ┌───────────────┐
       │ExternalTool   │  ← 外部工具透传（v2.0）
       │Processor      │
       └───────────────┘
```

### 7.2 Plugin 基础 Trait

每个插件必须实现的基础 trait：

```rust
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    // ── 元数据 ──
    fn id(&self) -> &PluginId;              // 唯一标识符
    fn name(&self) -> &str;                 // 人类可读名称
    fn version(&self) -> PluginVersion;     // 语义版本
    fn category(&self) -> PluginCategory;   // 分类
    fn description(&self) -> &str;          // 描述文本
    fn tags(&self) -> &[String];            // 搜索标签

    // ── 硬件声明 ──
    fn requires_pixel_access(&self) -> bool; // 是否需要访问像素数据
    fn produces_pixel_output(&self) -> bool; // 是否产生像素输出
    fn supported_hardware(&self) -> HardwareRequirement;

    // ── Schema（零分配静态引用） ──
    fn parameter_schema(&self) -> &ParameterSchema;
    fn gui_schema(&self) -> &GuiSchema;

    // ── 生命周期 ──
    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self) -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet)
        -> PluginResult<Vec<ValidationIssue>>;
}
```

**设计要点**:

1. `parameter_schema()` 和 `gui_schema()` 返回 `&'static` 引用（实际通过 `&self` 返回，由插件内部存储的静态常量支持），实现零分配访问
2. `requires_pixel_access()` 声明了插件是否需要像素缓冲区。元数据处理器（如 exif_rw）不需要像素访问，引擎在拓扑排序后对这类节点跳过 PixelBuffer 分配
3. `produces_pixel_output()` 标记插件是否产生新的或修改过的像素输出。大多数 Format/Metadata 插件返回 false
4. `validate()` 允许插件在收到参数集后执行额外的语义验证

### 7.3 能力 Trait（Capability Traits）

插件通过额外实现一个或多个能力 trait 来声明自己支持的操作类型：

#### PixelProcessor — 像素处理

```rust
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self) -> Vec<ColorSpace>;
    fn required_gpu_backend(&self) -> Option<GpuBackend>;

    async fn process_pixels(
        &self,
        input: &PixelBuffer,
        output: &mut PixelBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}
```

实现此 trait 的插件: `colorspace`, `lut3d`, `transform`, `lens_correct`, `ai_denoise`

#### FormatProcessor — 编解码

```rust
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn format_id(&self) -> ImageFormat;
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn can_decode(&self, data: &FormatProbe) -> bool;
    fn can_encode(&self, format: &ImageFormat) -> bool;

    async fn decode(&self, data: &[u8], opts: &DecodeOptions)
        -> PluginResult<DecodedImage>;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata,
                    opts: &EncodeOptions) -> PluginResult<Vec<u8>>;
}
```

实现此 trait 的插件: `heif_encoder`, `jxl_encoder`, `avif_encoder`, `tiff_encoder`, `png_encoder`, `raw_input`

#### MetadataProcessor — 元数据处理

```rust
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;

    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet)
        -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget,
                            metadata: &Metadata, params: &ParameterSet)
        -> PluginResult<MetadataWriteReport>;
}
```

实现此 trait 的插件: `exif_rw`, `gps_set`, `time_shift`

#### GpuProcessor — GPU 加速（v2.0 预留）

```rust
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;

    async fn process_gpu(
        &self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}
```

#### AiProcessor — AI 推理

```rust
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;
    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet)
        -> PluginResult<Tensor>;
}
```

实现此 trait 的插件: `ai_denoise`

#### ExternalToolProcessor — 外部工具透传（v2.0 预留）

```rust
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;
    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(&self, input: &[PathBuf], output: &PathBuf,
                     params: &ParameterSet) -> PluginResult<()>;
}
```

### 7.4 Registry 注册表

Registry 是插件的中央注册和管理中心：

```rust
pub struct Registry {
    entries: DashMap<PluginId, RegistryEntry>,
    manifests: DashMap<PluginId, PluginManifest>,
    load_order: RwLock<Vec<PluginId>>,

    // 按能力 trait 索引的 DashMap
    metadata_processors: DashMap<PluginId, Arc<dyn MetadataProcessor>>,
    pixel_processors:     DashMap<PluginId, Arc<dyn PixelProcessor>>,
    format_processors:    DashMap<PluginId, Arc<dyn FormatProcessor>>,
    ai_processors:        DashMap<PluginId, Arc<dyn AiProcessor>>,
}
```

**核心设计**:

1. **Arc<dyn Plugin>**: 所有插件由 Arc 共享，支持多线程并发访问
2. **能力索引**: 每种能力 trait 有独立的 DashMap，`get_pixel_processor()` 直接在类型层面返回 `Arc<dyn PixelProcessor>`，避免了运行时动态转换
3. **两阶段注册**: 插件先注册为基础 Plugin，再注册其能力 trait
4. **并发安全**: DashMap（无锁并发 HashMap）+ RwLock 保护加载顺序

**注册流程**（以 heif_encoder 为例）:

```rust
let p: Arc<HeifEncoderPlugin> = Arc::new(HeifEncoderPlugin::new());
registry.register(p.clone() as Arc<dyn Plugin>)?;           // 步骤 1
registry.register_format_processor(p as Arc<dyn FormatProcessor>)?;  // 步骤 2
```

**查询 API**:

```rust
// 基础查询
registry.get("heif_encoder")       → Option<Arc<dyn Plugin>>
registry.get_pixel_processor("id") → Option<Arc<dyn PixelProcessor>>
registry.get_format_processor("id") → Option<Arc<dyn FormatProcessor>>

// 筛选查询
registry.query(&PluginQuery {
    category: Some(PluginCategory::Format),
    tags: vec!["lossless"],
    keyword: Some("heif"),
    requires_pixel: Some(true),
    enabled_only: true,
}) → Vec<Arc<dyn Plugin>>

// 按分类
registry.by_category(PluginCategory::Enhance) → Vec<Arc<dyn Plugin>>

// 列出全部
registry.all() → Vec<Arc<dyn Plugin>>
registry.manifests() → Vec<PluginManifest>
registry.categories() → Vec<PluginCategory>
```

### 7.5 插件加载器类型（v2.0 预留）

| Loader | 格式 | 热重载 | 用途 |
|--------|------|:--:|------|
| Builtin | 编译进 binary | 否 | 核心始终可用的插件 |
| Native | .so / .dll / .dylib | 否 | 高性能第三方插件 |
| WASM | .wasm | 是 | 安全沙箱第三方插件 |
| ExternalTool | 子进程调用 | 否 | ExifTool / ffmpeg 等外部工具 |
| Remote | URL 下载安装 | 否 | 插件市场分发 |

### 7.6 如何添加新插件

以添加一个名为 `watermark` 的新插件为例：

**步骤 1**: 在 `crates/plugins/src/` 下创建 `watermark.rs`

```rust
#[derive(Debug)]
pub struct WatermarkPlugin {
    schema: ParameterSchema,
    gui_schema: GuiSchema,
}

impl WatermarkPlugin {
    pub fn new() -> Self {
        Self {
            schema: Self::build_schema(),
            gui_schema: Self::build_gui_schema(),
        }
    }

    fn build_schema() -> ParameterSchema {
        // 定义参数: 文本内容、字体大小、透明度、位置等
    }
}

#[async_trait]
impl Plugin for WatermarkPlugin { /* ... */ }

#[async_trait]
impl PixelProcessor for WatermarkPlugin {
    // 覆写像素处理逻辑
}
```

**步骤 2**: 在 `crates/plugins/src/lib.rs` 中注册

```rust
pub mod watermark;
// 在 register_all() 中添加:
{
    let p: Arc<watermark::WatermarkPlugin> = Arc::new(watermark::WatermarkPlugin::new());
    registry.register(p.clone() as Arc<dyn Plugin>);
    registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
}
```

**步骤 3**: 无需修改 `engine/`、`server/`、`core/` 中的任何代码。新插件自动可通过 Registry 查询，自动出现在前端插件列表中。

---

## 8. 参数 Schema 设计

### 8.1 设计哲学

ParameterSchema 定义了插件参数的**值语义**，而不是 GUI 控件类型。后端告诉前端"这是一个 0-100 的整数，步长为 1"，前端自主决定使用 SpinBox、Slider 还是 Combo 来展示。

### 8.2 ParameterSchema 结构

```rust
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}

pub struct ParameterSection {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub fields: Vec<ParameterField>,
}

pub struct ParameterField {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub help_url: Option<String>,
    pub field_type: ParameterType,    // 值类型（核心）
    pub default: serde_json::Value,   // 默认值
    pub required: bool,
    pub advanced: bool,               // 是否高级选项
    pub allow_override: bool,         // 是否允许覆盖
    pub supports_expression: bool,    // 是否支持 ${...} 表达式
}
```

### 8.3 ParameterType — 10 种值类型

```rust
#[serde(tag = "type")]
pub enum ParameterType {
    // 1. 字符串输入
    String {
        max_length: usize,
        pattern: Option<String>,       // 正则验证
        placeholder: Option<String>,
    },

    // 2. 整数
    Integer {
        min: i64,
        max: i64,
        step: i64,
        unit: Option<String>,          // 单位: "px", "ms"
        style: IntegerWidget,          // SpinBox | Slider | Combo
    },

    // 3. 浮点数
    Float {
        min: f64,
        max: f64,
        step: f64,
        precision: u8,                 // 小数位数
        unit: Option<String>,
        logarithmic: bool,             // 对数尺度
        style: FloatWidget,            // SpinBox | Slider | ComboSlider | DragInput
    },

    // 4. 布尔开关
    Boolean {
        label_true: Option<String>,    // "On" / "启用"
        label_false: Option<String>,   // "Off" / "禁用"
    },

    // 5. 枚举选择
    Enum {
        options: Vec<EnumOption>,
        display: EnumDisplay,          // Dropdown | RadioGroup | SegmentedControl | Tabs
    },

    // 6. 颜色选择
    Color {
        mode: ColorMode,               // RGB | RGBA | HSL | HSV | Lab
        show_alpha: bool,
    },

    // 7. 文件路径
    FilePath {
        kind: FilePathKind,            // File | Directory | SaveFile
        filters: Vec<(String, String)>,  // [("Images", "*.jpg;*.png")]
        must_exist: bool,
    },

    // 8. 坐标输入
    Coordinate {
        alt_required: bool,            // 是否需要海拔
        direction_required: bool,      // 是否需要方向
    },

    // 9. 滑块
    Slider {
        min: f64,
        max: f64,
        step: f64,
        show_ticks: bool,
        ticks: Option<Vec<f64>>,       // 刻度位置
        show_value: bool,
        orientation: SliderOrientation,
        style: SliderStyle,            // Continuous | Discrete | Range | DualHandle
    },

    // 10. 组合滑块
    ComboSlider {
        min: f64,
        max: f64,
        step: f64,
        presets: Vec<(String, f64)>,   // [("Low", 1.0), ("High", 9.0)]
        unit: Option<String>,
    },

    // 11. 表达式
    Expression {
        variables: Vec<VariableDef>,   // 可用变量定义
    },

    // 12. 预设
    Preset {
        preset_schema_ref: String,     // 引用的 schema id
        builtin_presets: Vec<NamedPreset>,
        allow_custom: bool,
        allow_import: bool,
    },

    // 13. 动态数组
    Array {
        element: Box<ParameterField>,  // 元素 schema
        min_items: usize,
        max_items: Option<usize>,
    },

    // 14. 地图控件
    MapWidget {
        show_track: bool,
        show_photos: bool,
        allow_manual_pin: bool,
    },

    // 15. 前后对比预览
    BeforeAfter {
        zoom_levels: Vec<f64>,
        show_histogram: bool,
    },

    // 辅助类型
    Separator { label: Option<String> },
    Section { fields: Vec<ParameterField> },
}
```

### 8.4 预定义 Custom Renderer（9 种）

当 ParameterType 不足以描述交互需求时，通过 Custom 类型指定专业渲染器：

| Renderer Name | 用途 | 适用插件 |
|---------------|------|----------|
| `tone_curve` | 色调曲线编辑器（多点曲线） | colorspace |
| `map_picker` | 地图选点（交互式地图） | gps_set |
| `lut_preview` | LUT 预览（3D 色彩可视化） | lut3d |
| `lens_selector` | 镜头选择器（品牌→型号级联） | lens_correct |
| `color_gamut_viewer` | 色域可视化（CIE 图） | colorspace |
| `denoise_split` | 降噪前后分割预览 | ai_denoise |
| `raw_thumbnail` | RAW 缩略图快速预览 | raw_input |
| `resize_visual_guide` | 缩放可视化参考线 | transform |
| `encoder_preset` | 编码器预设选择器 | heif/jxl/avif encoder |

### 8.5 字段条件系统

每个 FieldDef 和 ParameterGroup 支持 `visible_when` 和 `enabled_when`：

```rust
enum Condition {
    Equals { field: String, value: serde_json::Value },
    NotEquals { field: String, value: serde_json::Value },
    GreaterThan { field: String, value: f64 },
    LessThan { field: String, value: f64 },
    Contains { field: String, value: String },
    Matches { field: String, pattern: String },
    AllOf(Vec<Condition>),
    AnyOf(Vec<Condition>),
}
```

**示例**: 编码器参数仅在 `encoder == "x265"` 时可见

```json
{
  "id": "crf",
  "field_type": { "type": "float", "min": 0, "max": 51, "step": 0.5 },
  "visible_when": { "equals": { "field": "encoder", "value": "x265" } }
}
```

### 8.6 GuiSchema

GuiSchema 提供前端布局提示，与 ParameterSchema 分离（关注点分离）：

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,             // Standard | Custom
    pub icon: Option<String>,          // 插件图标
    pub color: Option<String>,         // 主题色
    pub preview: PreviewMode,          // 预览模式
    pub aux_views: Vec<AuxView>,       // 辅助视图
    pub min_panel_width: u32,
}

pub enum PreviewMode {
    None,
    Live,
    ManualRefresh,
    BeforeAfter { default_split: f32, orientation: SplitOrientation, lock_zoom: bool },
    Tiled { rows: u32, cols: u32 },
}

pub enum AuxView {
    Histogram,
    Waveform,
    Vectorscope,
    GamutDiagram,
    Map,
    FocusPeaking,
    ClippingWarning,
    MetadataTable,
    ProgressBar,
    StatusText,
}
```

### 8.7 ParameterSet — 运行时参数集

```rust
pub struct ParameterSet {
    pub values: HashMap<String, serde_json::Value>,
}

impl ParameterSet {
    pub fn get_str(&self, key: &str) -> Option<&str>;
    pub fn get_i64(&self, key: &str) -> Option<i64>;
    pub fn get_f64(&self, key: &str) -> Option<f64>;
    pub fn get_bool(&self, key: &str) -> Option<bool>;
    pub fn merge(&mut self, other: &ParameterSet);  // 浅合并，覆盖同名键
}
```

使用 `serde_json::Value` 而非泛型，牺牲编译期类型检查换取灵活性和 JSON 序列化零成本。

---

## 9. 节点间数据类型契约

### 9.1 问题陈述

在管线执行中，节点的输出格式必须与后续节点的输入要求兼容。例如：
- colorspace 节点输出 `(u16, RGB, Rec2020)`，但下一个节点 transform 期望 `(f32, RGBA, *)`
- 不匹配会导致静默错误或崩溃

### 9.2 DataTypeSpec（待实现）

```rust
pub struct DataTypeSpec {
    pub pixel_format: Vec<PixelFormat>,
    pub layout: Vec<ChannelLayout>,
    pub color_space: Vec<ColorSpace>,
    pub bit_depth: Option<u8>,
    pub metadata: Vec<MetadataScope>,
}
```

每个 TemplateNode 声明可选的 `input_type` 和 `output_type`：

```json
{
  "id": "denoise",
  "plugin": "ai_denoise",
  "input_type": {
    "pixel_format": ["u16", "f32"],
    "layout": ["RGB", "RGBA"],
    "bit_depth": 16
  },
  "output_type": {
    "pixel_format": ["f32"],
    "layout": ["RGB"]
  }
}
```

### 9.3 兼容性检查

引擎在构建 PipelineGraph 时检查节点间的类型兼容性：

```
节点 A 的 output_type ∩ 节点 B 的 input_type = ∅ → 硬错误，管线无效

具体检查维度:
  pixel_format: A.output ∩ B.input  (至少一个交集)
  layout:        A.output ∩ B.input
  color_space:   A.output ∩ B.input
  bit_depth:     A.output >= B.input (接收方可以接受更高位深度)
  metadata:      A.output ⊇ B.input  (接收方需要的元数据必须在输出中)
```

### 9.4 向后兼容

- 不声明 `input_type` / `output_type` 的节点被视为 `Any`（接受/产生任何类型）
- 保证旧配置文件在新版本引擎中继续有效

---

## 10. gRPC 服务设计

### 10.1 设计原则

**gRPC 仅承载实时交互数据**。管线定义通过文件系统传递，gRPC 负责：
- 插件信息查询（前端浏览可用插件）
- 图片信息预览（加载缩略图、元数据）
- 执行控制和进度流（Run + 实时事件流 + Cancel）

### 10.2 服务定义

#### PluginService — 插件发现

```protobuf
service PluginService {
  rpc ListPlugins(PluginQuery) returns (PluginList);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}
```

- `ListPlugins`: 按分类、标签、关键词筛选插件列表
- `GetNodeSchema`: 获取单个插件的完整 ParameterSchema（JSON 字符串）和 GuiSchema

#### ImageService — 图片操作

```protobuf
service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);       // 获取图片基本信息
  rpc Decode(DecodeRequest) returns (stream PixelDataChunk);  // 解码像素数据（流）
  rpc GetThumbnail(ThumbnailRequest) returns (ImageData);     // 获取缩略图
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);  // 编码输出（流）
}
```

#### ExecutionService — 执行控制

```protobuf
service ExecutionService {
  rpc Run(RunRequest) returns (stream RunEvent);  // 执行管线，流式返回事件
  rpc Cancel(CancelRequest) returns (google.protobuf.Empty);
}
```

### 10.3 RunEvent 一合体设计

```
message RunEvent {
  oneof event {
    ProgressUpdate progress = 1;
    MetricSnapshot metrics = 2;
    StageTransition stage = 3;
    ErrorEvent error = 4;
    DoneEvent done = 5;
  }
}

message ProgressUpdate {
  string node_id = 1;
  string node_label = 2;
  float fraction = 3;          // 0.0 - 1.0
  string message = 4;
  int64 elapsed_ms = 5;
}

message MetricSnapshot {
  float cpu_percent = 1;
  uint64 memory_mb = 2;
  uint64 gpu_memory_mb = 3;
  float throughput_mbps = 4;
}

message StageTransition {
  Stage from_stage = 1;
  Stage to_stage = 2;
}

enum Stage {
  LOADING = 0;
  DECODING = 1;
  PROCESSING = 2;
  ENCODING = 3;
  SAVING = 4;
  DONE = 5;
  ERROR = 6;
}
```

### 10.4 JSON 响应使用 String 类型

所有 JSON 响应（Schema、配置等）使用 `string` 类型传输，不使用 `google.protobuf.Struct`。理由：
- 前端直接 `JSON.parse(response)` 即可
- 避免 protobuf 和 JSON 之间的类型映射问题（如 i64 → f64 精度损失）
- 后端 `serde_json::to_string()` 一行搞定

### 10.5 当前实现状态

当前服务器实现了三个 gRPC 服务（与上述设计有差异，正在向新设计迁移）：

**PipelineService** (当前):
- `CreatePipeline(PipelineSpec) → PipelineId` — 创建管线并存储到内存
- `Execute(ExecuteRequest) → stream ExecuteProgress` — 执行管线
- `Validate(PipelineSpec) → ValidationResult` — 验证管线定义
- `GetNodeSchema(PluginId) → NodeSchema` — 获取插件 Schema

**ImageService** (当前):
- `Load(ImagePath) → ImageInfo` — 加载图片信息
- `Decode(DecodeRequest) → stream PixelDataChunk` — 流式解码像素数据
- `Encode(EncodeRequest) → stream EncodeProgress` — 编码
- `GetThumbnail(ThumbnailRequest) → ImageData` — JPEG 缩略图

**BatchService** (当前):
- `SubmitBatch(BatchSpec) → BatchId` — 提交批量任务
- `GetProgress(BatchId) → stream BatchProgress` — 查询进度
- `Cancel(BatchId) → Empty` — 取消任务

### 10.6 gRPC 服务器的启动流程

```rust
#[tokio::main]
async fn main() {
    // 1. 初始化日志
    init_telemetry(TelemetryConfig { ... });

    // 2. 安装 panic hook
    install_panic_hook();

    // 3. 创建 Registry 并注册所有插件
    let registry = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&registry);
    // registry.all().len() == 14

    // 4. 创建 ParameterResolver
    let resolver = Arc::new(ParameterResolver::new());

    // 5. 构建共享状态
    let state = Arc::new(SharedState::new(registry, resolver));

    // 6. 启动 gRPC 服务器
    Server::builder()
        .add_service(PipelineServiceServer::new(PipelineServiceImpl::new(state.clone())))
        .add_service(ImageServiceServer::new(ImageServiceImpl::new(state.clone())))
        .add_service(BatchServiceServer::new(BatchServiceImpl::new(state.clone())))
        .serve_with_shutdown("0.0.0.0:50051".parse()?, shutdown_signal())
        .await?;
}
```

### 10.7 SharedState

```rust
pub struct SharedState {
    pub registry: Arc<Registry>,
    pub resolver: Arc<ParameterResolver>,
    pub graphs: RwLock<HashMap<Uuid, PipelineGraph>>,
    pub batch_jobs: RwLock<HashMap<Uuid, BatchJobState>>,
}
```

`SharedState` 是所有 gRPC 服务共享的中央状态。它包装在 `Arc` 中，在线程间安全共享。

---

## 11. 执行引擎

### 11.1 NodeExecutor — 核心执行器

```rust
pub struct NodeExecutor {
    pub registry: Arc<Registry>,
    pub resolver: Arc<ParameterResolver>,
}
```

`NodeExecutor` 是管线执行的中央协调器。它消费 `Arc<Registry>` 和 `Arc<ParameterResolver>`，通过 trait object 调用插件，不依赖任何具体插件类型。

### 11.2 执行流程（execute 方法详解）

```
execute(graph, image_info, buffer, metadata, progress)
  │
  ├─ 1. 拓扑排序
  │     graph.topological_order() → Vec<NodeId>
  │     失败 → 返回 CircularDependency 错误
  │
  ├─ 2. 初始化执行上下文
  │     ExecutionContext { image_info, buffer, metadata, node_states }
  │
  ├─ 3. 逐节点执行（按拓扑序）
  │     for node_id in topological_order:
  │       │
  │       ├─ 3a. 检查取消标志
  │       │       progress.is_canceled() → 返回 Canceled 错误
  │       │
  │       ├─ 3b. 检查节点启用状态
  │       │       node.enabled == false → 标记 Skipped, 继续下一个
  │       │
  │       ├─ 3c. 查找插件
  │       │       registry.get(&node.plugin_id) → Plugin trait object
  │       │       未找到 → 返回 NotFound 错误
  │       │
  │       ├─ 3d. 解析参数（四级合并）
  │       │       resolver.resolve(node_id, image_id, schema, metadata, image_info)
  │       │       → 再合并 node.parameter_overrides
  │       │
  │       ├─ 3e. 验证参数
  │       │       plugin.validate(&final_params)
  │       │       有 Error 级别问题 → 返回 ValidationFailed 错误
  │       │
  │       ├─ 3f. 调度到正确的处理器
  │       │       if plugin.requires_pixel_access():
  │       │         → process_pixel_node()  // pixel > 8.8M 触发 TileEngine
  │       │       elif 有 MetadataProcessor:
  │       │         → process_metadata_node()
  │       │       elif 有 FormatProcessor:
  │       │         → process_format_node()
  │       │       else:
  │       │         → 返回 NotFound 错误
  │       │
  │       └─ 3g. 更新节点状态
  │             ctx.node_states[node_id] = Completed(stats)
  │
  └─ 4. 返回执行结果
        ExecutionResult { buffer, encoded_output, metadata, node_states }
```

### 11.3 三种处理器逻辑

#### process_pixel_node（像素处理）

```
1. 检查 ctx.buffer 是否存在 → 不存在则错误
2. 创建 output buffer（同尺寸、同格式）
3. 如果 pixel_count > 8.8M 像素:
   → TileEngine.process_tiled()  分块处理
   否则:
   → processor.process_pixels()  直接处理
4. ctx.buffer = Some(output)
```

#### process_metadata_node（元数据处理）

```
1. 创建 MetadataTarget { path, format }
2. processor.read_metadata(&target, params)
   → 合并读取到的元数据到 ctx.metadata
3. processor.write_metadata(&mut target, &ctx.metadata, params)
   → 写入元数据到目标文件
```

#### process_format_node（格式处理）

```
1. 检查 ctx.buffer 是否存在
2. 如果是输入专用格式处理器（can_encode() == false）:
   → 直通像素缓冲区（输入已由 server 加载）
3. 否则:
   → processor.encode(input, metadata, options)
   → ctx.encoded_output = Some(encoded_bytes)
```

### 11.4 惰性像素分配

像素缓冲区（PixelBuffer）仅在第一个需要访问像素的插件节点执行时才被分配。元数据处理器（exif_rw, gps_set, time_shift）不分配像素内存。这意味着一个仅修改 GPS 坐标的管线可以处理数千张图片而不消耗像素内存。

### 11.5 自动分块决策

```
pixel_count > 8_847_360 (4096×2160)
  → 启动 TileEngine，默认 tile size 1024×1024，overlap 64px
  → 每个 tile 单独调用 process_pixels
  → 结果回拼到 output buffer
```

---

## 12. 管线图模型

### 12.1 PipelineGraph — 运行时图

```rust
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<(PortId, PortId)>,
}

pub struct PipelineNode {
    pub id: NodeId,
    pub label: String,
    pub plugin_id: PluginId,
    pub enabled: bool,
    pub position: (f64, f64),
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub parameter_overrides: Option<ParameterSet>,
}
```

### 12.2 PipelineTemplate — 配置模板

```rust
pub struct PipelineTemplate {
    pub metadata: TemplateMetadata,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub overrides: Vec<ImageOverride>,
    pub groups: Vec<ParamGroup>,
    pub batch: Option<BatchConfig>,
}

pub struct TemplateNode {
    pub id: String,
    pub plugin: String,
    pub label: Option<String>,
    pub enabled: bool,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

pub struct TemplateEdge {
    pub from: String,
    pub to: String,
}
```

### 12.3 图操作

```rust
impl PipelineGraph {
    pub fn add_node(&mut self, plugin_id: String, label: String) -> NodeId;
    pub fn remove_node(&mut self, node_id: NodeId) -> bool;
    pub fn connect(&mut self, from_port: PortId, to_port: PortId)
        -> Result<(), PluginError>;
    pub fn disconnect(&mut self, from_port: PortId, to_port: PortId) -> bool;
    pub fn topological_order(&self) -> Result<Vec<NodeId>, PluginError>;
    pub fn has_cycle(&self) -> bool;
    pub fn validate_graph(&self) -> Result<(), Vec<String>>;
    pub fn from_template(template: &PipelineTemplate) -> Self;
}
```

**关键验证规则**:

1. 不能连接端口到自身
2. 不能连接同一节点的两个端口
3. 不能创建重复边
4. 添加边后立即检测环，有环则拒绝添加并回滚
5. `validate_graph()` 检查: 重复节点 ID、未知端口引用、环检测

### 12.4 拓扑排序（Kahn 算法）

```rust
pub fn topological_order(&self) -> Result<Vec<NodeId>, PluginError> {
    // 1. 构建入度表和邻接表
    // 2. 收集所有入度为 0 的节点
    // 3. BFS: 移除节点，减少邻居入度，入度为 0 入队
    // 4. 结果长度 != 节点数 → 存在环
}
```

### 12.5 Template → Graph 转换

`from_template()` 方法完成从配置文件到运行时图的转换：
1. 遍历 TemplateNode，为每个创建 PipelineNode（分配 UUID、端口）
2. 建立 id_map（模板 id → NodeId）
3. 遍历 TemplateEdge，建立端口连接
4. 传递 parameter_overrides

---

## 13. 参数解析器

### 13.1 ParameterResolver

```rust
pub struct ParameterResolver {
    pub template_params: HashMap<NodeId, ParameterSet>,
    pub group_overrides: Vec<(GroupCondition, HashMap<NodeId, ParameterSet>)>,
    pub image_overrides: HashMap<(ImageId, NodeId), ParameterSet>,
    pub expr_engine: ExpressionEngine,
}
```

### 13.2 四级合并算法（resolve 方法详解）

```rust
pub fn resolve(
    &self,
    node_id: NodeId,
    image_id: ImageId,
    schema: &ParameterSchema,
    metadata: &Metadata,
    image_info: &ImageInfo,
) -> ParameterSet {

    // 第一级: 插件内置默认值
    let mut result = self.resolve_plugin_defaults(schema);

    // 第二级: 模板级覆盖 (priority 1)
    if let Some(template_params) = self.template_params.get(&node_id) {
        result.merge(template_params);
    }

    // 快照: 保存不允许覆盖的字段值
    let template_snapshot = schema.all_fields()
        .filter(|f| !f.allow_override)
        .filter_map(|f| result.values.get(&f.id).map(|v| (f.id.clone(), v.clone())))
        .collect();

    // 第三级: 分组覆盖 (priority 2)
    for (condition, node_params) in &self.group_overrides {
        if self.evaluate_condition(condition, metadata, image_info) {
            if let Some(group_params) = node_params.get(&node_id) {
                result.merge(group_params);
            }
        }
    }

    // 第四级: 图片覆盖 (priority 3,最高)
    if let Some(image_params) = self.image_overrides.get(&(image_id, node_id)) {
        result.merge(image_params);
    }

    // 恢复: 不允许覆盖的字段恢复为模板快照值
    for (key, value) in template_snapshot {
        result.values.insert(key, value);
    }

    // 表达式求值: 解析 ${...}
    self.resolve_expressions(&mut result, metadata, image_info);

    result
}
```

### 13.3 GroupCondition 条件系统

```rust
pub enum GroupCondition {
    ExifEq { tag: String, value: String },   // exif.make == "Canon"
    ExifGte { tag: String, value: f64 },      // exif.iso >= 1600
    ExifLte { tag: String, value: f64 },      // exif.iso <= 400
    GpsNear { lat: f64, lon: f64, radius_km: f64 },  // 距离 ≤ R
    Always,
    And(Vec<GroupCondition>),
    Or(Vec<GroupCondition>),
    Expression(String),                       // "${exif.iso > 400 && exif.iso < 3200}"
}
```

GPS 距离计算使用 Haversine 公式，精度约 0.5%。

### 13.4 ExpressionEngine

表达式引擎支持 `${...}` 语法：

```
支持的变量:
  exif.iso           → ExifData.iso
  exif.make          → ExifData.make
  exif.model         → ExifData.model
  exif.aperture      → ExifData.aperture_value || f_number
  exif.shutter       → ExifData.shutter_speed_value || exposure_time
  exif.focal_length  → ExifData.focal_length
  exif.lens          → ExifData.lens_model
  image.filename     → ImageInfo.filename
  image.width        → ImageInfo.width
  image.height       → ImageInfo.height
  image.filesize     → ImageInfo.file_size_bytes

支持的运算:
  ${exif.iso > 800}                    → 比较 (>, <, >=, <=, ==, !=)
  ${exif.make == "Canon"}             → 字符串比较
  ${exif.iso >= 400 ? 'high' : 'low'}  → 三元表达式
  ${exif.make} ${exif.model}          → 拼接

字面量:
  数字: ${3.14} 或 ${42}
  字符串: ${\"hello\"} 或 ${'world'}
```

表达式引擎通过递归下降解析：
1. 先检查是否为三元表达式（`cond ? true : false`）
2. 检查比较运算符
3. 解析变量引用
4. 将数值表达式自动按 `f64` 比较，字符串按精确匹配比较

---

## 14. TileEngine 分块处理

### 14.1 设计动机

一张 4096×2160 的 f32 RGBA 图像占用约 135MB 内存。直接在单次调用中处理可能导致：
- 峰值内存过高
- 缓存不友好（超出 L3 缓存容量）
- GPU 显存不足（对大模型）

分块处理将大图像拆分为小块（默认 1024×1024），每个块约 1-16MB（取决于位深和通道数）。

### 14.2 TileEngine 实现

```rust
pub struct TileEngine {
    pub default_tile_size: u32,   // 默认 1024
    pub overlap: u32,              // 默认 64
    pub max_parallel: usize,      // 默认 CPU 核心数
}
```

### 14.3 处理流程

```
process_tiled(processor, input, params, progress)
  │
  ├─ 1. 计算分块布局
  │     TileLayout::new(width, height, tile_size, overlap)
  │     例如: 4096×2160, 1024 tile, 64 overlap
  │     → tiles_x = 5, tiles_y = 3, total_tiles = 15
  │
  ├─ 2. 分配输出缓冲区
  │     output = PixelBuffer::new(width, height, layout, format)
  │
  ├─ 3. 逐块处理
  │     for each tile:
  │       ├─ 从输入复制 tile 到临时缓冲区 (tile_input)
  │       ├─ 调用 processor.process_pixels(tile_input, tile_output)
  │       └─ 将 tile_output 拼回 output 的对应位置
  │
  └─ 4. 返回完整输出
```

### 14.4 TileLayout 算法

```rust
let stride = tile_size - overlap;  // 有效步长 (1024 - 64 = 960)
let tiles_x = ceil(width / stride); // 列数
let tiles_y = ceil(height / stride); // 行数
let total_tiles = tiles_x * tiles_y;

// 边界块可能小于 tile_size（由 saturating_sub 确保不超出图像边界）
```

### 14.5 触发条件

在 `process_pixel_node()` 中：
```rust
if input.pixel_count() > 8_847_360 {  // > 4096×2160
    TileEngine::default().process_tiled(processor, input, params, progress).await
} else {
    processor.process_pixels(input, output, params, progress).await
}
```

### 14.6 当前限制与未来改进

- **当前**: 逐块串行处理（按顺序处理每个 tile），overlap 区域的像素结果被后写的 tile 覆盖
- **未来**: 并行 tile 处理（Rayon 线程池），重叠区域混合（blending）

---

## 15. 内置插件目录

### 15.1 插件清单（14 个已实现）

| # | Plugin ID | 分类 | 能力 Trait | 像素访问 | 后端依赖 |
|:--:|-----------|------|-----------|:--:|------|
| 1 | `exif_rw` | Metadata | MetadataProcessor | 否 | ExifTool + kamadak-exif |
| 2 | `gps_set` | Metadata | MetadataProcessor | 否 | ExifTool + geo crate |
| 3 | `time_shift` | Metadata | MetadataProcessor | 否 | chrono + ExifTool |
| 4 | `colorspace` | Color | PixelProcessor | 是 | Halide + lcms2 |
| 5 | `lut3d` | Color | PixelProcessor | 是 | Halide |
| 6 | `transform` | Transform | PixelProcessor | 是 | Halide |
| 7 | `lens_correct` | Enhance | PixelProcessor | 是 | LensFun + Halide |
| 8 | `ai_denoise` | Enhance | PixelProcessor + AiProcessor | 是 | ONNX Runtime |
| 9 | `raw_input` | Input | FormatProcessor | 是 | dcraw / LibRaw |
| 10 | `heif_encoder` | Format | FormatProcessor | 是 | libheif + x265 |
| 11 | `jxl_encoder` | Format | FormatProcessor | 是 | libjxl |
| 12 | `avif_encoder` | Format | FormatProcessor | 是 | libheif + aom |
| 13 | `tiff_encoder` | Format | FormatProcessor | 是 | OIIO |
| 14 | `png_encoder` | Format | FormatProcessor | 是 | lodepng |

### 15.2 编码器品质推荐

| 格式 | 编码器 | 推荐设置 | 品质评级 |
|------|------|------|:--:|
| HEIF 10-bit | x265 | preset=veryslow, crf=18, 444, tune=grain | 极高 |
| HEIF 10-bit (GPU) | NVENC | Turing+, b-frames, 10bit | 高 |
| HEIF 10-bit (Mac) | VideoToolbox | Apple Silicon HW | 高 |
| JXL 16-bit | libjxl | effort=7-9, distance=0.5（视觉无损） | 极高 |
| JXL 无损 | libjxl | effort=7-9, distance=0 | 完美 |
| AVIF 10-bit | aom | cpu-used=4, cq-level=18 | 高 |
| PNG 16-bit | lodepng | 无损 | 完美 |
| TIFF 16-bit | OIIO | LZW/ZIP 压缩 | 完美 |

### 15.3 插件分类统计

```
Input:       1 (raw_input)
Metadata:    3 (exif_rw, gps_set, time_shift)
Color:       2 (colorspace, lut3d)
Transform:   1 (transform)
Enhance:     2 (lens_correct, ai_denoise)
Format:      5 (heif, jxl, avif, tiff, png)
Total:      14
```

---

## 16. 数据流图

### 16.1 配置文件 → 执行流程

```
┌──────────────────────────────────────────────────────────────┐
│                   PipelineConfig (JSON 文件)                  │
│                                                              │
│  pipelines[] ──→ images[] ──→ groups[] ──→ output ──→ exec  │
└───────────────┬──────────────┬──────────────┬────────────────┘
                │              │              │
        ┌───────┘       ┌──────┘       ┌──────┘
        ▼               ▼              ▼
┌───────────────┐ ┌─────────────┐ ┌──────────────┐
│ PipelineGraph │ │ Image List   │ │  Resolver    │
│  nodes/edges  │ │  with params │ │  4-level     │
└───────┬───────┘ └──────┬──────┘ │  merge        │
        │                │        └──────┬───────┘
        └────────┬───────┘               │
                 ▼                       ▼
          ┌─────────────────────────────────────┐
          │         NodeExecutor.execute()       │
          │                                      │
          │  for image in images:                │
          │    graph.topological_order()          │
          │    for node in order:                │
          │      resolve params                  │
          │      validate                        │
          │      process_*()                     │
          │    save output                       │
          └─────────────────────────────────────┘
                           │
                           ▼
          ┌─────────────────────────────────────┐
          │  output/                             │
          │  ├── DSC0001_2026-05-28.heif         │
          │  ├── DSC0002_2026-05-28.heif         │
          │  └── ...                             │
          └─────────────────────────────────────┘
```

### 16.2 gRPC 请求/响应流

#### 执行流（Execute / Run）

```
GUI                          gRPC Server                    Engine
 │                               │                            │
 │── Run(config_path) ──────────►│                            │
 │                               │── read PipelineConfig     │
 │                               │── build PipelineGraph     │
 │                               │                            │
 │                               │── execute(graph, image) ─►│
 │                               │                            │── topo sort
 │   ◄── RunEvent{LOADING} ──────│                            │
 │                               │                            │── load image
 │   ◄── RunEvent{PROCESSING} ───│                            │── node 1
 │   ◄── RunEvent{PROCESSING} ───│                            │── node 2
 │   ◄── RunEvent{METRICS}  ─────│                            │── snapshot
 │   ◄── RunEvent{PROCESSING} ───│                            │── node 3
 │   ◄── RunEvent{DONE} ─────────│                            │── complete
 │                               │                            │
```

#### 浏览流（图像信息查询，与管理配置无关）

```
GUI                          gRPC Server
 │                               │
 │── Load(image_path) ──────────►│── 检测格式
 │   ◄── ImageInfo  ─────────────│── 提取尺寸、EXIF
 │                               │
 │── GetThumbnail(path, 256) ───►│── 加载图片
 │   ◄── ImageData (JPEG)  ──────│── 缩放到 256px
 │                               │── 编码为 JPEG
```

#### 插件发现流

```
GUI                          gRPC Server
 │                               │
 │── ListPlugins(query) ────────►│── Registry::query()
 │   ◄── PluginList ─────────────│── 序列化为 JSON
 │                               │
 │── GetNodeSchema(plugin_id) ──►│── Registry::get(id)
 │   ◄── NodeSchema ─────────────│── plugin.parameter_schema()
 │                               │── plugin.gui_schema()
 │                               │── 序列化为 JSON 字符串
```

### 16.3 内存数据流

```
输入文件
  │
  ▼
FormatProcessor.decode()
  │
  ├── PixelBuffer (像素数据) ──────────────────────┐
  │                                                 │
  └── Metadata (EXIF/XMP/GPS) ───────┐              │
                                      ▼              ▼
                              ┌─ ExecutionContext ────────┐
                              │                            │
  MetadataProcessor nodes:    │  metadata (Arc<Metadata>) │
    read_metadata() ──────────┤                            │
    write_metadata() ─────────┤                            │
                              │                            │
  PixelProcessor nodes:       │  buffer (PixelBuffer)     │
    process_pixels() ─────────┤                            │
                              │                            │
  FormatProcessor nodes:      │  encoded_output (Vec<u8>) │
    encode() ─────────────────┤                            │
                              └────────────────────────────┘
                                      │
                                      ▼
                              输出文件
```

---

## 17. 向后兼容策略

### 17.1 PipelineConfig 版本化

```
PipelineConfig.version 字段采用 SemVer:
  - PATCH 版本变化: 完全向后兼容（添加可选字段）
  - MINOR 版本变化: 向后兼容（添加新功能，废弃旧字段）
  - MAJOR 版本变化: 不兼容变更（需要迁移工具）
```

### 17.2 Schema 版本化

`ParameterSchema.version` 字段允许前端识别 schema 变更。前端应：
- 忽略未知字段（前向兼容）
- 为缺失字段使用默认值（后向兼容）

### 17.3 插件版本兼容

`PluginVersion` + `VersionRequirement` 机制：
- 每个插件声明其版本
- 管线可以声明 `min_version` / `max_version` 约束
- 不兼容的插件版本在管线验证时被检测

### 17.4 配置迁移路径

当 PipelineConfig 格式发生重大变更时，提供迁移工具：

```bash
photopipeline migrate --from v1 --to v2 config.v1.json > config.v2.json
```

### 17.5 节点数据类型的向后兼容

节点的 `input_type` 和 `output_type` 声明为可选。不声明的节点被视为 `Any`（接受任何输入、产生任何输出），确保旧配置文件在新引擎中继续工作。

---

## 18. 安全设计

### 18.1 localhost-only 绑定

gRPC 服务器默认绑定 `127.0.0.1:50051`（当前实现使用 `0.0.0.0`，需要修改）。生产环境中绝不对外暴露 gRPC 端口。

### 18.2 文件系统访问控制

- 图片路径和输出目录通过配置文件指定，服务器不做路径遍历
- 编码输出和输入文件由服务器端验证路径存在
- 外部工具插件（v2.0）需要 `trusted` 标志才能执行任意命令

### 18.3 PipelineConfig 验证

- `validate` 子命令检查: JSON 结构、插件 ID 有效性、边引用正确性、环形依赖
- `validate_graph()` 检查: 重复节点 ID、未知端口引用
- 插件 `validate()` 方法执行语义验证（如数值范围、编码器可用性）

### 18.4 表达式安全

ExpressionEngine 仅支持预定义的变量名（`exif.*` 和 `image.*`），不暴露任意文件系统访问。表达式求值失败返回错误而非 panic。

### 18.5 WASM 沙箱（v2.0 预留）

WASM 插件在独立沙箱中运行：
- 有限内存分配
- 无文件系统访问（除非显式授权）
- 计算时间限制（超时自动终止）

### 18.6 错误信息泄露

生产模式下，面向用户的错误信息不包含内部路径或堆栈跟踪。内部详情仅记录到日志。

---

## 19. 性能设计

### 19.1 零拷贝数据传递

```
元数据层:
  Arc<Metadata> 全程共享，读操作完全无锁（Arc::clone 仅增加引用计数）

像素层:
  - 单消费者: Arc 不加写时复制，原地修改
  - 多消费者: Arc 共享只读访问
  - COW 仅在真正需要写入且存在多个引用时触发

GPU 层:
  GpuBuffer.handle 在 VRAM 中传递，不经过系统内存
  直到编码阶段才回读
```

### 19.2 AlignedBuffer — 页对齐内存

```rust
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,  // 64 字节对齐，支持 AVX-512 和 GPU 映射
}
```

对齐内存的优势：
- SIMD 指令（SSE2, AVX2, AVX-512）可以直接操作
- 可以直接映射到 GPU（cudaHostRegister）
- 系统分配器通常已满足 16 字节对齐要求

### 19.3 分块处理

- 默认 tile size: 1024×1024
- 默认 overlap: 64px（避免边界伪影）
- 触发阈值: 8,847,360 像素（4096×2160）
- 未来: 并行 tile 处理（Rayon 线程池）

### 19.4 静态 Schema 避免堆分配

`parameter_schema()` 返回对静态常量的引用，零堆分配：

```rust
impl Plugin for ColorSpacePlugin {
    fn parameter_schema(&self) -> &ParameterSchema {
        &self.schema  // 在 new() 中构建一次，之后只返回引用
    }
}
```

### 19.5 编译期优化

Cargo.toml 中的 release profile：
```toml
[profile.release]
opt-level = 3       # 最大优化
lto = true          # 链接时优化
codegen-units = 1   # 单代码生成单元（更好的内联）
panic = "abort"     # panic 时直接终止（更小的二进制）
```

### 19.6 并行处理

- 批量处理: 多张图片并行处理（`execution.parallel`）
- 未来: 管线内并行（独立子 DAG 分支可以并行执行）
- 未来: 分块并行（多个 tile 同时处理）

### 19.7 性能计时

```rust
pub struct PerfTimer {
    label: String,
    start: Instant,
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        // 自动记录耗时到 tracing
        tracing::debug!(target: "perf", label = %self.label,
                        elapsed_ms = self.start.elapsed().as_millis(),
                        "{} took {}ms", self.label, elapsed_ms);
    }
}
```

使用方式：
```rust
let _timer = PerfTimer::with_target("pipeline_execute", "pipeline");
// ... 执行逻辑 ...
// 函数返回时自动记录耗时
```

---

## 20. 错误处理策略

### 20.1 错误类型体系

```rust
pub enum PluginError {
    // 插件相关
    NotFound(PluginId),                    // 插件未找到
    AlreadyLoaded { plugin: PluginId },    // 重复加载
    LoadFailed { plugin, reason },         // 加载失败
    VersionMismatch { plugin, actual, required },  // 版本不匹配

    // 参数相关
    InvalidParameter { plugin, field, message },    // 参数无效
    ExpressionError { plugin, field, error },        // 表达式错误

    // 资源相关
    GpuNotAvailable { plugin, backend },             // GPU 不可用
    GpuOutOfMemory { plugin, needed, available },    // GPU 内存不足
    MissingTool { plugin, tool, required },          // 缺少外部工具

    // 执行相关
    Timeout { plugin, elapsed, timeout },            // 超时
    Canceled { plugin },                             // 用户取消
    Internal { plugin, message },                    // 内部错误
    Io { plugin, error },                            // IO 错误

    // 管线相关
    ValidationFailed(String),                        // 验证失败
    NodeExecutionFailed { node, message },           // 节点执行失败
    CircularDependency,                              // 环形依赖

    // 文件相关
    FileNotFound(String),                            // 文件未找到
    UnsupportedFormat(String),                       // 不支持的格式
    EncodingFailed(String),                          // 编码失败
    DecodingFailed(String),                          // 解码失败

    // 通用
    Config(String),                                  // 配置错误
    Other(String),                                   // 其他
}
```

### 20.2 错误传播策略

- 使用 `thiserror` crate 自动派生 `Display` 和 `Error` trait
- `PluginResult<T>` = `Result<T, PluginError>`
- 所有可恢复错误通过 `PluginResult` 传播
- 不可恢复错误（OOM、数据结构不一致）使用 `panic!`

### 20.3 验证问题分级

```rust
pub enum ValidationIssue {
    Error { param: String, message: String },    // 阻止执行
    Warning { param: String, message: String },  // 建议修复
    Info { param: String, message: String },     // 信息提示
}
```

管线执行前，先调用 `plugin.validate(&params)`。存在 Error 级别问题时阻止执行。

---

## 21. 日志与遥测

### 21.1 TelemetryConfig

```rust
pub struct TelemetryConfig {
    pub output: LogOutput,           // Console | File | Journald | None
    pub default_filter: String,      // "info", "debug", "photopipeline=debug"
    pub file_dir: Option<String>,
    pub file_prefix: Option<String>,
    pub ansi_colors: bool,
}

pub enum LogOutput {
    Console,
    File(String),
    Journald,
    None,
}
```

### 21.2 日志级别映射

| 环境 | 默认过滤器 | 说明 |
|------|-----------|------|
| 开发 | `debug` | 详细日志，包含函数进入/退出 |
| 服务器 | `photopipeline_server=info` | 服务器信息级别 |
| 生产 | `info` | 仅关键信息 |
| 性能分析 | `perf=debug` | 性能计时日志 |

### 21.3 日志输出格式

**控制台模式** (ansi_colors: true):
```
2026-05-28T12:00:00.123Z  INFO photopipeline_server: Starting gRPC server on 0.0.0.0:50051
2026-05-28T12:00:01.456Z DEBUG photopipeline_engine::executor: Node 'AI Denoise' completed in 234ms
```

**文件模式** (JSON):
```json
{
  "timestamp": "2026-05-28T12:00:01.456Z",
  "level": "DEBUG",
  "target": "photopipeline_engine::executor",
  "fields": {
    "message": "Node 'AI Denoise' completed in 234ms",
    "node_label": "AI Denoise",
    "elapsed_ms": 234
  }
}
```

### 21.4 关键遥测点

| 组件 | 遥测内容 |
|------|----------|
| Registry | 插件注册/注销、查询命中/未命中 |
| NodeExecutor | 执行开始/完成、节点耗时、失败原因 |
| ParameterResolver | 参数解析路径、条件匹配 |
| PipelineGraph | 图构建、环检测、拓扑排序 |
| TileEngine | 分块布局、块处理进度 |
| gRPC Server | 请求/响应、流状态 |

### 21.5 动态日志级别

支持通过环境变量 `RUST_LOG` 动态调整日志级别，无需重启：

```bash
RUST_LOG="photopipeline_engine=debug" photopipeline run config.json
```

---

## 22. 项目结构

```
photopipeline/
├── Cargo.toml                    # workspace 根
├── Cargo.lock
├── justfile                       # 任务运行器
├── rust-toolchain.toml           # Rust 1.90+
│
├── crates/
│   ├── core/                     # 核心类型（零依赖最小化）
│   │   └── src/
│   │       ├── lib.rs            # 模块导出
│   │       ├── types.rs          # 全局类型 (PluginId, NodeId, PluginCategory...)
│   │       ├── image.rs          # 图像类型 (PixelFormat, PixelBuffer, TileLayout...)
│   │       ├── color.rs          # 色彩类型 (ColorSpace...)
│   │       ├── metadata.rs       # 元数据模型 (Metadata, ExifData, GpsData...)
│   │       ├── error.rs          # 错误类型 (PluginError, ValidationIssue)
│   │       ├── perf.rs           # 性能计时 (PerfTimer)
│   │       ├── telemetry.rs      # 日志遥测 (TelemetryConfig)
│   │       └── panic_hook.rs     # panic hook
│   │
│   ├── plugin/                   # 插件框架（仅依赖 core）
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── trait_def.rs      # Plugin / PixelProcessor / FormatProcessor / ...
│   │       ├── registry.rs       # Registry (DashMap 索引)
│   │       ├── schema.rs         # ParameterSchema / ParameterSet
│   │       └── gui_schema.rs     # GuiSchema / PanelWidget
│   │
│   ├── plugins/                  # 14 个内置插件（每个一个文件）
│   │   └── src/
│   │       ├── lib.rs            # register_all() 注册函数
│   │       ├── raw_input.rs      # RAW 输入
│   │       ├── exif_rw.rs         # EXIF 读写
│   │       ├── gps_set.rs         # GPS 设置
│   │       ├── time_shift.rs      # 时间偏移
│   │       ├── colorspace.rs      # 色彩空间
│   │       ├── lut3d.rs           # 3D LUT
│   │       ├── transform.rs       # 几何变换
│   │       ├── lens_correct.rs    # 镜头校正
│   │       ├── ai_denoise.rs      # AI 降噪
│   │       ├── heif_encoder.rs    # HEIF 编码
│   │       ├── jxl_encoder.rs     # JXL 编码
│   │       ├── avif_encoder.rs    # AVIF 编码
│   │       ├── tiff_encoder.rs    # TIFF 编码
│   │       └── png_encoder.rs     # PNG 编码
│   │
│   ├── engine/                   # 执行引擎（仅依赖 core + plugin）
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs          # PipelineGraph / PipelineTemplate
│   │       ├── executor.rs       # NodeExecutor / ExecutionContext
│   │       ├── params.rs         # ParameterResolver / ExpressionEngine
│   │       └── tile.rs           # TileEngine
│   │
│   ├── server/                   # gRPC 服务端 + CLI 命令（统一二进制）
│   │   └── src/
│   │       ├── lib.rs            # SharedState / prost ↔ JSON 转换
│   │       ├── main.rs           # gRPC 服务器入口
│   │       └── services/
│   │           ├── mod.rs
│   │           ├── pipeline.rs   # PipelineService 实现
│   │           ├── image.rs      # ImageService 实现
│   │           └── batch.rs      # BatchService 实现
│   │
│   ├── halide/                   # Halide FFI（feature-gated，CI 编译）
│   ├── oiio/                     # OIIO FFI（feature-gated，CI 编译）
│   └── test-defs/               # 测试公共定义
│
├── cli/                          # CLI 二进制（当前独立，将合并到 server）
├── proto/                        # Protobuf 定义
│   ├── pipeline.proto
│   ├── image.proto
│   └── batch.proto
│
├── halide_generators/            # Halide C++ 生成器（CI 编译）
├── examples/                     # 管线配置示例
├── tests/                        # 集成测试
├── benches/                      # 性能基准测试
├── scripts/                      # 构建脚本
├── vendor/                       # vendored 依赖（libjxl-sys patched）
│
└── doc/                          # 文档
    └── ARCHITECTURE_DESIGN.md    # 本文档
```

### 22.1 Crate 依赖关系总表

| Crate | 依赖 |
|-------|------|
| `core` | 无内部依赖 |
| `plugin` | `core` |
| `plugins` | `core`, `plugin` |
| `engine` | `core`, `plugin` |
| `server` | `core`, `plugin`, `engine`, `plugins` |
| `halide` | `core` (feature-gated) |
| `oiio` | `core` (feature-gated) |
| `test-defs` | `core`, `plugin`, `engine` |

---

## 23. CI/CD 集成

### 23.1 GitHub Actions 工作流矩阵

```
┌───────────────────────────────────────────────────┐
│ build-rust.yml                                    │
│  matrix: [ubuntu, windows, macos] × [x86_64]     │
│  steps: setup-rust → apt/choco/brew deps          │
│         → cargo build --workspace                 │
│         → cargo test --workspace                  │
├───────────────────────────────────────────────────┤
│ build-halide.yml                                  │
│  matrix: [ubuntu, windows, macos]                 │
│  output: 各平台 Halide 构建产物 (动态库)           │
│  memory: 8GB+ (需要更大实例)                       │
├───────────────────────────────────────────────────┤
│ build-oiio.yml                                    │
│  matrix: [ubuntu, windows, macos]                 │
│  output: 各平台 OIIO 构建产物                      │
│  memory: 8GB+                                     │
├───────────────────────────────────────────────────┤
│ test.yml                                          │
│  needs: [build-rust]                              │
│  steps: cargo test --workspace                    │
│         cargo clippy -- -D warnings               │
│         cargo fmt --check                         │
├───────────────────────────────────────────────────┤
│ release.yml                                       │
│  needs: [build-rust, build-halide, build-oiio, test]│
│  steps: pack → upload to GitHub Releases          │
└───────────────────────────────────────────────────┘
```

### 23.2 本地开发替代策略

由于开发环境（4C/8T, 3.3GB RAM）无法编译 Halide 和 OIIO（需要 8GB+），采用以下分工：

| 组件 | 编译位置 | 理由 |
|------|:--:|------|
| Rust workspace (core/plugin/plugins/engine/server) | 本地 cargo build | 4C/8T 足够 |
| Halide generators (C++) | GitHub Actions | 需要 8GB+ RAM |
| OIIO (C++) | GitHub Actions | 依赖繁多 |
| protobuf .proto → .rs | 本地 | 轻量级 |
| 系统开发库 (libheif-dev...) | 本地 apt install | 预编译秒装 |

### 23.3 CLI 在 CI 中的使用

统一二进制使得 CI 管线极其简洁：

```yaml
# GitHub Actions 中的典型使用
- name: Process test images
  run: |
    photopipeline validate test-pipeline.json
    photopipeline run test-pipeline.json

# 批量验证所有示例配置
- name: Validate examples
  run: |
    for f in examples/*.json; do
      photopipeline validate "$f" || exit 1
    done
```

### 23.4 依赖补丁

项目维护了部分上游 crate 的本地补丁（vendor/ 目录）：

```toml
[patch.crates-io]
libjxl-src = { path = "vendor/libjxl-src-patched" }
libjxl-sys = { path = "vendor/libjxl-sys-patched" }
libraw-sys = { path = "vendor/libraw-sys-patched" }
```

这些补丁用于修复上游 bug 或在官方修复发布前临时适配特定平台问题。

---

## 24. 开发阶段路线图

| Phase | 名称 | 目标 | 产出 |
|:---:|------|------|------|
| 0 | 设计文档 | 规划与架构设计 | ARCHITECTURE_DESIGN.md（本文档） |
| 1 | 环境搭建 | 配置 | Rust 工具链、开发库、Git 初始化、CI 脚手架 |
| 2 | Core Crate | 类型定义 | ImageBuffer, Metadata, ColorSpace, Error 类型 |
| 3 | Plugin System | 框架 | Plugin trait, Registry, Schema, GuiSchema |
| 4 | Pipeline Engine | 运行时 | PipelineGraph, NodeExecutor, ParameterResolver, TileEngine |
| 5 | Builtin Plugins | 功能实现 | 14 个内置插件 |
| 6 | 统一二进制 | 合并 CLI | 子命令: serve, run, validate, schema, plugins |
| 7 | gRPC 重构 | 实时通信 | PluginService, ImageService, ExecutionService |
| 8 | Halide/OIIO | 计算层 | Halide generators, OIIO FFI, CI 编译 |
| 9 | PipelineConfig | 配置驱动 | JSON PipelineConfig 格式、验证、迁移工具 |
| 10 | DataTypeSpec | 类型契约 | 节点间数据类型兼容性检查 |
| 11 | GUI 前端 | 桌面端 | 技术选型待定 |
| 12 | CI/CD | DevOps | 全平台 GitHub Actions 矩阵 |
| 13 | 性能优化 | 调优 | 并行 tile 处理、SIMD 优化、GPU 加速 |
| 14 | 安全审计 | 安全 | 输入验证、沙箱、权限模型 |
| 15 | 正式发布 | v1.0 | 文档、示例、安装包 |

---

## 附录 A: 术语表

| 术语 | 英文 | 说明 |
|------|------|------|
| 管线 | Pipeline | 由节点和边组成的 DAG，描述图像处理步骤 |
| 节点 | Node | 管线中的一个处理步骤，绑定到一个插件 |
| 插件 | Plugin | 实现特定图像处理功能的模块 |
| 模板 | Template | PipelineConfig 中定义的管线结构 |
| 参数集 | ParameterSet | 键值对形式的运行时参数 |
| 参数覆盖 | Parameter Override | 在更高优先级层级覆盖的默认参数 |
| 元数据 | Metadata | EXIF、XMP、IPTC、GPS 四类图像元数据 |
| 像素缓冲区 | PixelBuffer | 图像像素数据的运行时表示 |
| 对齐缓冲区 | AlignedBuffer | 页对齐的字节缓冲区，支持 GPU 映射 |
| 分块 | Tiling / Tile | 将大图像切分为小块独立处理 |
| 拓扑排序 | Topological Sort | 对 DAG 节点按依赖关系线性排序 |
| 能力 Trait | Capability Trait | 插件声明支持的操作类型（Pixel/Metadata/Format/GPU/AI） |
| 注册表 | Registry | 插件的中央注册和查找中心 |
| 执行上下文 | ExecutionContext | 单张图片处理期间的运行时状态 |
| 进度槽 | ProgressSink | 插件报告进度的回调接口 |
| 条件分组 | Conditional Group | 根据图片元数据自动应用的分组参数覆盖 |
| 表达式引擎 | ExpressionEngine | 解析 `${exif.iso > 800 ? 'high' : 'low'}` 的引擎 |
| 零拷贝 | Zero-Copy | 通过 Arc 共享数据，避免不必要的内存复制 |
| 写时复制 | COW (Copy-on-Write) | 仅在需要修改且存在多个引用时复制数据 |

## 附录 B: 关键文件索引

| 文件 | 内容 |
|------|------|
| `crates/core/src/types.rs` | 所有基础类型定义 (1020 行) |
| `crates/core/src/image.rs` | 像素格式、缓冲区、分块布局 (914 行) |
| `crates/core/src/metadata.rs` | 元数据模型 (1009 行) |
| `crates/core/src/error.rs` | 错误类型体系 (384 行) |
| `crates/plugin/src/trait_def.rs` | 所有 Plugin trait 和 Capability trait (212 行) |
| `crates/plugin/src/schema.rs` | ParameterSchema, ParameterType, ParameterSet (960 行) |
| `crates/plugin/src/registry.rs` | Registry 实现 (728 行) |
| `crates/engine/src/graph.rs` | PipelineGraph, PipelineTemplate (1060 行) |
| `crates/engine/src/executor.rs` | NodeExecutor 核心执行逻辑 (1010 行) |
| `crates/engine/src/params.rs` | ParameterResolver, ExpressionEngine (1376 行) |
| `crates/engine/src/tile.rs` | TileEngine 实现 (373 行) |
| `crates/plugins/src/lib.rs` | 所有 14 个插件的注册 (102 行) |
| `crates/server/src/main.rs` | gRPC 服务器启动入口 (68 行) |
| `crates/server/src/services/pipeline.rs` | PipelineService 实现 (498 行) |
| `crates/server/src/services/image.rs` | ImageService 实现 (353 行) |
| `crates/server/src/services/batch.rs` | BatchService 实现 (482 行) |

---

*文档结束*

*本文档基于代码库实际实现撰写（crates/core, crates/plugin, crates/plugins, crates/engine, crates/server），版本对应 `feat/backend-frontend-interaction-redesign` 分支。所有 Rust 代码片段直接来自源代码文件。*

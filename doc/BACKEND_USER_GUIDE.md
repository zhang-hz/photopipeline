# Photopipeline 后端用户指南

> 本文档面向需要使用、扩展或集成 Photopipeline 后端的开发者。涵盖从源码构建到插件开发、管线执行引擎、gRPC 服务器、CLI 命令参考以及高级主题的完整内容。

---

## 目录

### 第一部分：入门
1. [系统要求](#1-系统要求)
2. [从源码构建](#2-从源码构建)
3. [运行你的第一条管线](#3-运行你的第一条管线)
4. [项目结构详解](#4-项目结构详解)

### 第二部分：管线配置
5. [PipelineConfig JSON 格式完整参考](#5-pipelineconfig-格式完整参考)
6. [管线定义](#6-管线定义)
7. [输入输出规范](#7-输入输出规范)
8. [条件参数分组](#8-条件参数分组)
9. [参数合并规则](#9-参数合并规则)
10. [批量处理配置](#10-批量处理配置)
11. [完整配置示例](#11-完整配置示例)

### 第三部分：插件开发
12. [插件架构概述](#12-插件架构概述)
13. [编写你的第一个插件](#13-编写你的第一个插件)
14. [Plugin Trait 参考](#14-plugin-trait-参考)
15. [能力 Trait](#15-能力-trait)
16. [ParameterSchema 编写](#16-parameterschema-编写)
17. [17 种 ValueType 详解](#17-17-种-valuetype-详解)
18. [条件系统](#18-条件系统)
19. [自定义渲染器](#19-自定义渲染器)
20. [注册插件](#20-注册插件)
21. [插件验证](#21-插件验证)
22. [测试插件](#22-测试插件)

### 第四部分：执行引擎
23. [管线执行流程](#23-管线执行流程)
24. [DAG 拓扑与节点排序](#24-dag-拓扑与节点排序)
25. [参数解析](#25-参数解析)
26. [分块引擎](#26-分块引擎)
27. [进度报告](#27-进度报告)
28. [取消机制](#28-取消机制)
29. [性能考量](#29-性能考量)

### 第五部分：服务端与 gRPC
30. [启动服务端](#30-启动服务端)
31. [gRPC API 参考](#31-grpc-api-参考)
32. [通过 gRPC 运行管线](#32-通过-grpc-运行管线)
33. [CLI 模式 vs 服务端模式](#33-cli-模式-vs-服务端模式)
34. [后端服务生命周期](#34-后端服务生命周期)

### 第六部分：CLI 参考
35. [完整 CLI 命令参考](#35-完整-cli-命令参考)
36. [脚本与自动化示例](#36-脚本与自动化示例)
37. [与 Shell 脚本和 CI/CD 集成](#37-与-shell-脚本和-cicd-集成)

### 第七部分：高级主题
38. [自定义数据类型](#38-自定义数据类型)
39. [节点间数据类型契约](#39-节点间数据类型契约)
40. [FFI 集成](#40-ffi-集成)
41. [内存管理](#41-内存管理)
42. [色彩管理](#42-色彩管理)
43. [16 位以上精度保证](#43-16-位以上精度保证)

### 第八部分：参考
44. [14 个内置插件目录](#44-14-个内置插件目录)
45. [核心类型参考](#45-核心类型参考)
46. [错误码参考](#46-错误码参考)
47. [配置文件格式参考](#47-配置文件格式参考)
48. [术语表](#48-术语表)

---

## 第一部分：入门

### 1. 系统要求

Photopipeline 是一个 Rust workspace 项目，运行时需要以下依赖：

| 依赖 | 最低版本 | 用途 |
|---|---|---|
| Rust | 1.90+ | 编译核心 Workspace |
| CMake | 3.20+ | Halide / OIIO 构建（仅 CI 环境，本地非必须） |
| pkg-config | — | 系统库检测 |
| libheif-dev | 1.12+ | HEIF / AVIF 编解码支持 |
| libjxl-dev | 0.8+ | JPEG XL 编解码支持 |
| liblcms2-dev | 2.0+ | ICC 色彩管理 |
| exiftool | 12.00+ | EXIF / XMP / IPTC 元数据操作（可选） |

**各平台安装系统依赖：**

Ubuntu / Debian：
```bash
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl
```

macOS (Homebrew)：
```bash
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool
```

Windows (vcpkg)：
```powershell
vcpkg install libheif libjxl lcms2
```

开发环境最小配置推荐 4 核 CPU 和 8GB RAM。如需本地编译 Halide 和 OIIO 的 C++ 部分，建议 16GB 以上 RAM（通常这些组件在 CI 上编译即可，本地无需重新构建）。

---

### 2. 从源码构建

#### 2.1 克隆仓库

```bash
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
```

#### 2.2 使用 Cargo 构建

```bash
# Debug 构建
cargo build --workspace

# Release 构建（推荐用于生产）
cargo build --profile ci --workspace
```

构建产物位置：
- `target/debug/photopipeline` — CLI 主程序（debug 模式）
- `target/release/photopipeline` — CLI 主程序（release 模式）

注意：项目已重构为统一二进制。只有一个 `photopipeline` 二进制文件，服务端功能通过子命令 `photopipeline serve` 启动。

#### 2.3 使用 Justfile 构建

项目的 `justfile` 提供了预定义的任务命令：

| 命令 | 说明 |
|---|---|
| `just build` | Debug 构建整个 workspace |
| `just build-release` | 使用 `ci` profile 进行 Release 构建 |
| `just test` | 运行全部测试（`--no-fail-fast`） |
| `just lint` | 运行 Clippy 静态检查 |
| `just fmt` | 自动格式化代码 |
| `just fmt-check` | 检查代码格式 |
| `just ci-lint` | CI 预检（lint + fmt-check） |
| `just clean` | 清理构建产物 |

#### 2.4 验证安装

```bash
# 打印帮助信息
./target/release/photopipeline --help

# 列出所有已注册插件（应显示 14 个）
./target/release/photopipeline plugin list
```

---

### 3. 运行你的第一条管线

#### 3.1 管线工作原理

Photopipeline 的管线是一个有向无环图（DAG），由节点（Node）和有向边（Edge）组成。每个节点绑定一个插件（Plugin），数据沿边的方向从上游流向下游。管线引擎将按拓扑排序顺序依次执行各节点。

```
source(raw_input) ──→ exif(exif_rw) ──→ gps(gps_set) ──→ color(colorspace) ──→ output(jxl_encoder)
```

#### 3.2 Hello World 管线

创建 TOML 配置文件 `hello.toml`：

```toml
[metadata]
name = "Hello World"
version = "1.0"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"

[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = { gps_mode = "manual", latitude = 30.5728, longitude = 104.0668 }

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = { source_color_space = "srgb", target_color_space = "rec2020_pq" }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = { bit_depth = "16", effort = 7 }

[[edges]]
from = "source"
to = "exif"

[[edges]]
from = "exif"
to = "gps"

[[edges]]
from = "gps"
to = "color"

[[edges]]
from = "color"
to = "output"
```

运行：

```bash
photopipeline pipeline run -c hello.toml -i DSC0001.ARW -o result.jxl
```

在运行前可先验证管线：

```bash
photopipeline pipeline validate -c hello.toml
```

成功验证时的输出：
```
✓ Pipeline validation passed
  Nodes: 5
  Edges: 4
  No cycles detected
```

---

### 4. 项目结构详解

```
photopipeline/
├── crates/
│   ├── core/              # 共享基础类型
│   │   └── src/
│   │       ├── lib.rs         # crate 入口
│   │       ├── image.rs       # PixelFormat, PixelBuffer, AlignedBuffer, TileLayout
│   │       ├── color.rs       # ColorSpace, TransferFunction, ColorPrimaries, ICC 生成
│   │       ├── metadata.rs    # Metadata, ExifData, XmpData, GpsData, GpxTrack
│   │       ├── error.rs       # PluginError 枚举（21 种变体）, ValidationIssue
│   │       ├── types.rs       # PluginId, ImageFormat, GpuBackend, AiBackend, GuiSchema 等
│   │       ├── panic_hook.rs  # 全局 panic hook
│   │       ├── perf.rs        # PerfTimer 工具
│   │       └── telemetry.rs   # tracing 日志初始化
│   ├── plugin/             # 插件框架
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── trait_def.rs   # Plugin, PixelProcessor, FormatProcessor 等 Trait
│   │       ├── registry.rs    # Registry（线程安全插件注册表）
│   │       ├── schema.rs      # ParameterSchema, ParameterSet, 17 种 ParameterType
│   │       └── gui_schema.rs  # GuiSchema 重导出
│   ├── engine/             # 执行引擎
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs       # PipelineGraph, PipelineTemplate, DAG 拓扑排序
│   │       ├── executor.rs    # NodeExecutor, ExecutionContext, 节点三路分派
│   │       ├── params.rs      # ParameterResolver, ExpressionEngine, 四级优先级
│   │       └── tile.rs        # TileEngine 分块并行处理
│   ├── plugins/            # 14 个内置插件实现
│   │   └── src/
│   │       ├── lib.rs         # register_all() 全局注册函数
│   │       ├── exif_rw.rs     # EXIF 读写插件（MetadataProcessor）
│   │       ├── gps_set.rs     # GPS 坐标设置（MetadataProcessor）
│   │       ├── time_shift.rs  # 时间偏移（MetadataProcessor）
│   │       ├── colorspace.rs  # 色彩空间转换（PixelProcessor）
│   │       ├── lut3d.rs       # 3D LUT 应用（PixelProcessor）
│   │       ├── transform.rs   # 几何变换（PixelProcessor）
│   │       ├── lens_correct.rs # 镜头校正（PixelProcessor）
│   │       ├── ai_denoise.rs  # AI 降噪（PixelProcessor + AiProcessor）
│   │       ├── raw_input.rs   # RAW 解码输入（FormatProcessor）
│   │       ├── heif_encoder.rs # HEIF 编码器（FormatProcessor）
│   │       ├── jxl_encoder.rs  # JPEG XL 编码器（FormatProcessor）
│   │       ├── avif_encoder.rs # AVIF 编码器（FormatProcessor）
│   │       ├── tiff_encoder.rs # TIFF 编码器（FormatProcessor）
│   │       └── png_encoder.rs  # PNG 编码器（FormatProcessor）
│   ├── external/           # 外部工具封装（ExifTool, libvips, 商业 API stubs）
│   ├── server/             # gRPC 服务端实现
│   └── oiio/               # OIIO FFI 绑定（feature-gated）
├── cli/                    # CLI 二进制
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # 统一入口，通过 clap 定义所有子命令
│       ├── config.rs          # 配置加载
│       └── commands/
│           ├── pipeline.rs    # pipeline run / validate
│           ├── plugin.rs      # plugin list / info
│           └── batch.rs       # batch run / validate
├── proto/                  # Protobuf 服务定义
│   ├── pipeline.proto         # PipelineService（创建/执行/验证/查询）
│   ├── image.proto            # ImageService（加载/解码/编码/缩略图）
│   └── batch.proto            # BatchService（提交/进度/取消）
├── halide_generators/      # Halide C++ 生成器源文件（CI 上编译为计算库）
├── examples/               # 管线 TOML 配置示例
│   └── hdr_pipeline.toml
├── justfile                # Just 任务运行器
├── Cargo.toml              # Workspace 根
└── README.md
```

**各 Crate 职责总结：**

| Crate | 职责 | 关键输出类型 |
|---|---|---|
| `core` | 基础共享类型，无外部依赖逻辑 | `PixelBuffer`, `Metadata`, `ColorSpace`, `PluginError`, `ImageFormat` |
| `plugin` | 插件框架定义 | `Plugin` trait, `PixelProcessor` trait, `Registry`, `ParameterSchema` |
| `plugins` | 14 个内置插件的具体实现 | `ExifRwPlugin`, `AiDenoisePlugin`, `JxlEncoderPlugin` 等 |
| `engine` | 管线 DAG 构建、执行、参数解析、分块处理 | `PipelineGraph`, `NodeExecutor`, `ParameterResolver`, `TileEngine` |
| `server` | gRPC 服务端 | gRPC 服务实现 |
| `halide` | Halide 内核 FFI 绑定 | Halide 计算函数的 Rust 接口 |
| `oiio` | OpenImageIO FFI 绑定 | OIIO 读写操作的 Rust 接口 |

---

## 第二部分：管线配置

### 5. PipelineConfig 格式完整参考

Photopipeline 管线使用 TOML v1.0 格式定义。整个管线配置分为 6 个顶层段（Section），其中只有 `[[nodes]]` 是必填的。

#### 5.1 顶层配置段总览

| 配置段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `[metadata]` | 单表 | 否 | 管线元信息（名称、版本、描述） |
| `[[nodes]]` | 数组表 | **是** | 节点定义（至少一个） |
| `[[edges]]` | 数组表 | 否 | 节点间的有向边 |
| `[[overrides]]` | 数组表 | 否 | 逐图参数覆盖 |
| `[[groups]]` | 数组表 | 否 | 条件分组参数覆盖 |
| `[batch]` | 单表 | 否 | 批量处理配置 |

#### 5.2 TOML 格式规则

- 使用标准 [TOML v1.0](https://toml.io/) 格式
- 节点 ID 必须唯一，推荐使用 `snake_case`
- 插件 ID 格式为 `photopipeline.plugins.{name}`
- 参数值遵循 JSON 类型映射：字符串加引号，数值直接书写，布尔值为 `true` / `false`
- 表达式使用 `${ }` 包裹，支持嵌套在三元运算符中

---

### 6. 管线定义

#### 6.1 [metadata] — 管线元信息

```toml
[metadata]
name = "My Pipeline"
version = "1.0"
description = "从 RAW 到 16 位 JXL 的完整处理流程"
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `name` | string | 否 | 管线名称，CLI 和 GUI 中显示 |
| `version` | string | 否 | 管线语义化版本号 |
| `description` | string | 否 | 详细描述，支持多行 |

#### 6.2 [[nodes]] — 节点定义

每个 `[[nodes]]` 条目定义一个管线节点：

```toml
[[nodes]]
id = "node_id"              # 必填：节点唯一标识
plugin = "full.plugin.id"   # 必填：绑定的插件 ID
label = "显示名称"           # 可选：GUI 中显示名称
enabled = true              # 可选：是否启用（默认 true）
params = { key = "value" }  # 可选：参数覆盖表
```

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | string | 是 | — | 管线全局唯一标识符 |
| `plugin` | string | 是 | — | 插件完整 ID，如 `photopipeline.plugins.colorspace` |
| `label` | string | 否 | 等于 `id` | GUI 中显示的名称 |
| `enabled` | bool | 否 | `true` | `false` 时节点被跳过 |
| `params` | table | 否 | `{}` | 参数覆盖表，键为参数 ID，值为参数值 |

**PipelineGraph 内部表示：**

每个节点在构建时会被转换为内部的 `PipelineNode` 结构体：

```rust
pub struct PipelineNode {
    pub id: NodeId,           // UUID
    pub label: String,
    pub plugin_id: PluginId,
    pub enabled: bool,
    pub position: (f64, f64), // GUI 布局坐标
    pub inputs: Vec<PortId>,  // 输入端口列表
    pub outputs: Vec<PortId>, // 输出端口列表
    pub parameter_overrides: Option<ParameterSet>,
}
```

每个节点自动获得一个输入端口和一个输出端口（均为 UUID）。端口是边的连接点。

#### 6.3 [[edges]] — 边定义

```toml
[[edges]]
from = "source_node_id"
to = "target_node_id"
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `from` | string | 是 | 源节点 ID（数据输出端） |
| `to` | string | 是 | 目标节点 ID（数据输入端） |

**约束规则：**

1. 不允许自环（`from == to`）
2. 不允许重复边
3. 不允许形成环（创建时使用拓扑排序检测）
4. `from` 和 `to` 必须引用存在的节点 ID

图验证函数 `PipelineGraph::validate_graph()` 会在运行时检查以上所有约束。

---

### 7. 输入输出规范

管线的输入和输出不是通过特殊配置段定义，而是通过管线两端的节点隐式确定：

- **输入端**：第一个节点通常为输入类插件（如 `raw_input`），它从文件系统读取图像数据
- **输出端**：最后一个节点通常为格式类插件（如 `jxl_encoder`），它将处理结果编码为文件

管线的数据流严格按照 DAG 拓扑排序执行。`ExecutionResult` 包含最终的处理结果：

```rust
pub struct ExecutionResult {
    pub buffer: Option<PixelBuffer>,     // 最终像素缓冲区（无格式输出节点时）
    pub encoded_output: Option<Vec<u8>>, // 最终编码后的字节数据（有格式输出节点时）
    pub metadata: Metadata,              // 最终元数据
    pub node_states: HashMap<NodeId, NodeRunState>, // 每个节点的执行状态
}
```

---

### 8. 条件参数分组

`[[groups]]` 段允许基于图片属性（EXIF、GPS、图像尺寸）动态应用参数。分组按定义顺序评估，后匹配者覆盖先匹配者。

```toml
[[groups]]
name = "高 ISO — 强力降噪"
condition = "exif.iso >= 1600"
[groups.params.ai_denoise]
  denoise_strength = 90
  detail_preservation = 30

[[groups]]
name = "极暗 — 最高降噪"
condition = "${exif.iso >= 3200}"
[groups.params.ai_denoise]
  denoise_strength = 95
  denoise_model = "high_quality_v2"
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `name` | string | 是 | 分组名称 |
| `condition` | string | 是 | 条件表达式 |
| `params.<node_id>` | table | 否 | 匹配时应用的参数 |

**内部实现 — GroupCondition 枚举：**

```rust
pub enum GroupCondition {
    ExifEq { tag: String, value: String },
    ExifGte { tag: String, value: f64 },
    ExifLte { tag: String, value: f64 },
    GpsNear { lat: f64, lon: f64, radius_km: f64 },
    Always,
    And(Vec<GroupCondition>),
    Or(Vec<GroupCondition>),
    Expression(String),
}
```

支持的标签有：`iso`, `make`, `model`, `lens`, `focal_length`, `aperture`, `shutter`。GPS 附近检测使用 Haversine 公式计算距离。

---

### 9. 参数合并规则

参数解析采用**四级优先级**，高优先级覆盖低优先级：

```
级别 3: 图像覆盖 (Image Override)      ← 最高优先级
  └─ 级别 2: 分组覆盖 (Group Override)  ← 后匹配者胜出
      └─ 级别 1: 模板默认 (Template Default)
          └─ 级别 0: 插件内置 (Plugin Builtin) ← 最低优先级
```

**ParameterResolver::resolve() 内部流程：**

```
1. 从 ParameterSchema::defaults() 初始化结果集
2. 合并模板参数 (template_params[node_id])
3. 快照不可覆盖字段 (allow_override == false)
4. 遍历所有分组，匹配条件的合并其参数
5. 合并图像覆盖 (image_overrides[(image_id, node_id)])
6. 恢复不可覆盖字段的快照值
7. 求值结果集中所有 ${ } 表达式
```

**注意：** 标记 `allow_override = false` 的字段在任何覆盖层级都不会被修改。这保证了某些关键参数（如安全相关）始终由插件控制。

---

### 10. 批量处理配置

`[batch]` 段配置批量处理的参数：

```toml
[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `parallel` | int | `1` | 并行处理数量 |
| `output_pattern` | string | — | 输出路径模板 |
| `on_conflict` | string | `"skip"` | 冲突策略：`"skip"` 或 `"overwrite"` |
| `resume` | bool | `false` | 支持断点续传 |

**输出路径模板占位符：**

| 占位符 | 说明 | 示例值 |
|---|---|---|
| `{filename}` | 输入文件名（不含扩展名） | `DSC0001` |
| `{ext}` | 输入文件扩展名 | `ARW` |
| `{date}` | EXIF 日期 (YYYY-MM-DD) | `2024-05-15` |
| `{camera}` | 相机型号 | `EOS_R5` |
| `{iso}` | ISO 值 | `800` |

---

### 11. 完整配置示例

#### 11.1 简单管线（RAW 转 JXL）

```toml
[metadata]
name = "RAW 转 JXL"
version = "1.0"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = { bit_depth = "16", effort = 7, lossless = false }

[[edges]]
from = "raw"
to = "output"
```

#### 11.2 HDR 处理管线

```toml
[metadata]
name = "HDR 处理"
version = "1.0"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32", apply_white_balance = true }

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "srgb",
  target_color_space = "rec2020_pq",
  rendering_intent = "relative_colorimetric",
  black_point_compensation = true,
  gamut_mapping = "compress",
  embed_icc = true,
}

[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = { denoise_strength = 50, denoise_model = "standard_v2" }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.heif_encoder"
params = { bit_depth = "10", chroma_subsampling = "444", quality = 95 }

[[edges]]
from = "raw"
to = "color"

[[edges]]
from = "color"
to = "denoise"

[[edges]]
from = "denoise"
to = "output"
```

#### 11.3 自适应批量管线

```toml
[metadata]
name = "自适应批量管线"
version = "1.0"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"

[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = {
  gps_mode = "gpx_track",
  gpx_file = "track.gpx",
  max_interpolation_gap = 300,
}

[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = {
  denoise_strength = "${exif.iso > 1600 ? 80 : 30}",
  detail_preservation = "${exif.iso > 3200 ? 25 : 50}",
}

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "srgb",
  target_color_space = "rec2020_pq",
}

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = {
  bit_depth = "16",
  effort = "${exif.iso > 3200 ? 9 : 7}",
}

[[edges]]
from = "raw"
to = "exif"

[[edges]]
from = "exif"
to = "gps"

[[edges]]
from = "gps"
to = "denoise"

[[edges]]
from = "denoise"
to = "color"

[[edges]]
from = "color"
to = "output"

[[groups]]
name = "高 ISO 强力降噪"
condition = "exif.iso >= 3200"
[groups.params.ai_denoise]
  denoise_strength = 95

[[groups]]
name = "极暗最高降噪"
condition = "${exif.iso >= 6400}"
[groups.params.ai_denoise]
  denoise_strength = 95
  detail_preservation = 25

[batch]
parallel = 2
output_pattern = "{date}/{filename}.jxl"
on_conflict = "skip"
resume = true
```

---

## 第三部分：插件开发

### 12. 插件架构概述

Photopipeline 的插件系统基于 **Trait 分层 + Schema 驱动** 架构。核心设计原则：

1. **分层 Trait：** 每个插件必须实现基础 `Plugin` trait。根据功能可选择实现 1-6 种能力 trait。
2. **Schema 驱动 GUI：** 参数通过 `ParameterSchema` 声明式定义，GUI 面板自动生成。后端定义值语义，前端选择控件。
3. **多态加载：** 支持 5 种加载方式——`Builtin`（编译内置）、`Native`（动态库）、`WASM`（沙箱）、`ExternalTool`（子进程）、`Remote`（市场分发）。
4. **组合优于继承：** 单个插件可同时实现多个能力 trait。例如 `ai_denoise` 同时实现 `PixelProcessor` + `AiProcessor`。

**Trait 层次图：**

```
                        ┌──────────┐
                        │  Plugin   │  基础 Trait（所有插件必须实现）
                        └─────┬────┘
           ┌──────────┐  ┌────┴────┐  ┌──────────────┐
           │Metadata  │  │  Pixel   │  │   Format     │
           │Processor │  │Processor │  │  Processor   │
           └──────────┘  └────┬────┘  └──────────────┘
                        ┌─────┴─────┐
                        │GpuProcessor│  ← GPU 计算扩展 (v2.0)
                        └─────┬─────┘
                        ┌─────┴─────┐
                        │AiProcessor │  ← AI 推理扩展
                        └────────────┘
           ┌───────────────┐
           │ExternalTool   │  ← 外部工具透传 (v2.0)
           │Processor      │
           └───────────────┘
```

---

### 13. 编写你的第一个插件

以下是一个完整的自定义插件的逐步教程。我们将创建一个简单的亮度调整插件。

#### Step 1：确定插件类型

| 场景 | Category | 能力 Trait |
|---|---|---|
| 读取新图像格式 | Format | `Plugin` + `FormatProcessor` |
| 写入新图像格式 | Format | `Plugin` + `FormatProcessor` |
| 元数据处理 | Metadata | `Plugin` + `MetadataProcessor` |
| 像素滤镜/变换 | Enhance / Transform / Color | `Plugin` + `PixelProcessor` |
| AI 推理 | Enhance | `Plugin` + `PixelProcessor` + `AiProcessor` |
| GPU 计算 | 任意 | `Plugin` + `GpuProcessor` |
| 调用外部工具 | External | `Plugin` + `ExternalToolProcessor` |

我们的亮度插件属于 Enhance 类别，需要实现 `Plugin` + `PixelProcessor`。

#### Step 2：创建插件结构体

在 `crates/plugins/src/` 下创建 `brightness.rs`：

```rust
use photopipeline_core::*;
use photopipeline_plugin::*;

#[derive(Debug, Clone)]
pub struct BrightnessPlugin {
    id: String,
}

impl BrightnessPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.brightness".to_string(),
        }
    }
}
```

#### Step 3：定义静态 ParameterSchema

使用 `LazyLock` 确保单次初始化：

```rust
use std::sync::LazyLock;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "basic".into(),
                label: "基本设置".into(),
                description: Some("亮度调整核心参数".into()),
                icon: Some("brightness".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "amount".into(),
                        label: "亮度".into(),
                        description: Some("亮度调整量 (-1.0 到 1.0)".into()),
                        help_url: None,
                        field_type: ParameterType::Float {
                            min: -1.0,
                            max: 1.0,
                            step: 0.01,
                            precision: 2,
                            unit: None,
                            logarithmic: false,
                            style: FloatWidget::Slider,
                        },
                        default: serde_json::json!(0.0),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: true,
                    },
                ],
            },
        ],
    }
});
```

#### Step 4：定义静态 GuiSchema

```rust
static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection {
                    param_section_id: "basic".into(),
                    title_visible: true,
                    style: SectionStyle::Card,
                },
            ],
        },
        icon: Some("brightness".into()),
        color: Some("#f59e0b".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![AuxView::Histogram],
        min_panel_width: 320,
    }
});
```

#### Step 5：实现 Plugin Trait

```rust
#[async_trait]
impl Plugin for BrightnessPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "亮度调整" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Enhance }
    fn description(&self) -> &str { "提升或降低图像整体亮度" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { true }
    fn produces_pixel_output(&self) -> bool { true }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement { min_ram_mb: 256, ..Default::default() }
    }
    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(v) = params.get_f64("amount") {
            if v < -1.0 || v > 1.0 {
                issues.push(ValidationIssue::Error {
                    param: "amount".into(),
                    message: "亮度值必须在 -1.0 到 1.0 之间".into(),
                });
            }
        }
        Ok(issues)
    }
}
```

#### Step 6：实现 PixelProcessor

```rust
#[async_trait]
impl PixelProcessor for BrightnessPlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }
    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }
    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![ColorSpace::SRGB, ColorSpace::LINEAR_SRGB]
    }
    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        None // CPU only
    }

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "亮度处理中...");

        let amount = params.get_f64("amount").unwrap_or(0.0);

        // 复制元数据
        output.data.data.copy_from_slice(&input.data.data);
        output.width = input.width;
        output.height = input.height;
        output.layout = input.layout;
        output.format = input.format;
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        // 像素处理逻辑（按格式分派）
        match input.format {
            PixelFormat::U8 => { /* u8 亮度调整逻辑 */ }
            PixelFormat::U16 => { /* u16 亮度调整逻辑 */ }
            PixelFormat::F32 => { /* f32 亮度调整逻辑 */ }
            _ => {}
        }

        let pixels = input.pixel_count();
        progress.set_progress(1.0, "完成");

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: 0,
            input_pixels: pixels,
            output_pixels: pixels,
        })
    }
}
```

#### Step 7：在 `register_all()` 中注册

编辑 `crates/plugins/src/lib.rs`：

```rust
pub mod brightness;

pub fn register_all(registry: &Arc<Registry>) {
    // ... 已有插件注册 ...
    {
        let p: Arc<brightness::BrightnessPlugin> = Arc::new(brightness::BrightnessPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    }
}
```

#### Step 8：验证注册成功

```bash
cargo build
photopipeline plugin list | grep brightness
photopipeline plugin info photopipeline.plugins.brightness
```

---

### 14. Plugin Trait 参考

基础 `Plugin` trait 是所有插件的基石。每个方法都有明确的语义要求：

```rust
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    fn id(&self)                        -> &PluginId;
    fn name(&self)                      -> &str;
    fn version(&self)                   -> PluginVersion;
    fn category(&self)                  -> PluginCategory;
    fn description(&self)               -> &str;
    fn tags(&self)                      -> &[String];
    fn requires_pixel_access(&self)     -> bool;
    fn produces_pixel_output(&self)     -> bool;
    fn supported_hardware(&self)        -> HardwareRequirement;

    fn parameter_schema(&self)          -> &ParameterSchema;
    fn gui_schema(&self)                -> &GuiSchema;

    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self)                         -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet)      -> PluginResult<Vec<ValidationIssue>>;
}
```

**关键方法说明：**

| 方法 | 返回类型 | 说明 |
|---|---|---|
| `id()` | `&PluginId` | 全局唯一标识符，约定格式 `photopipeline.plugins.{snake_case}` |
| `name()` | `&str` | 用户显示名称，支持中文 |
| `version()` | `PluginVersion` | 语义化版本 `{ major, minor, patch, pre }` |
| `category()` | `PluginCategory` | 9 种类别之一：Input, Metadata, Color, Transform, Enhance, Merge, Format, External, Custom |
| `requires_pixel_access()` | `bool` | 返回 `false` 时管线引擎不分配像素内存（零拷贝路径） |
| `produces_pixel_output()` | `bool` | 返回 `true` 表示插件输出新的像素缓冲区 |
| `parameter_schema()` | `&ParameterSchema` | 返回静态 Schema 引用（通常使用 `LazyLock`） |
| `validate()` | `PluginResult<Vec<ValidationIssue>>` | 参数合法性验证，不通过则管线执行失败 |
| `initialize()` | `PluginResult<()>` | 异步初始化（加载模型、验证工具可用性等） |
| `shutdown()` | `PluginResult<()>` | 异步清理（释放 GPU 资源、卸载模型等） |

**requires_pixel_access 与 produces_pixel_output 的四种组合：**

| `requires_pixel_access` | `produces_pixel_output` | 典型插件 | 管线行为 |
|---|---|---|---|
| `false` | `false` | `exif_rw`, `gps_set`, `time_shift` | 零像素内存，仅操作元数据 |
| `true` | `true` | `colorspace`, `lut3d`, `transform`, `ai_denoise` | 读取+写入像素缓冲区 |
| `true` | `false` | `raw_input` | 输入解码产生像素，不消费上游像素 |
| `false` | `true` | （极少见） | 产生像素但不读取 |

---

### 15. 能力 Trait

#### 15.1 MetadataProcessor — 元数据处理

用于仅读写元数据、不触碰像素的插件：

```rust
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;
    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet) -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet) -> PluginResult<MetadataWriteReport>;
}
```

`MetadataScope` 枚举：`EXIF`, `XMP`, `IPTC`, `GPS`, `All`

执行引擎中 metadata 节点的处理流程：
1. 调用 `read_metadata()` 读取元数据
2. 合并读取结果到 `ExecutionContext.metadata`
3. 调用 `write_metadata()` 写回元数据
4. 统计写入/跳过的标签数（`MetadataWriteReport`）

#### 15.2 PixelProcessor — 像素处理

用于逐像素操作的插件：

```rust
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self)  -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self)   -> Vec<ColorSpace>;
    fn required_gpu_backend(&self)     -> Option<GpuBackend>;

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}
```

**执行引擎的行为：**

- 检查 `input.pixel_count()` 超过 8.8M 像素阈值（约 4096x2160）时，自动启用 `TileEngine` 分块处理
- 小于阈值时直接调用 `process_pixels()`，传入完整的 `input` 和 `output` buffer
- `output` buffer 由引擎预先创建并传入，尺寸和格式与 `input` 相同

#### 15.3 FormatProcessor — 格式编解码

用于文件格式的编码/解码：

```rust
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn format_id(&self) -> ImageFormat;
    fn can_decode(&self, data: &FormatProbe) -> bool;
    async fn decode(&self, data: &[u8], options: &DecodeOptions) -> PluginResult<DecodedImage>;
    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata, options: &EncodeOptions) -> PluginResult<Vec<u8>>;
}
```

**执行引擎的行为：**

- `can_encode()` 返回 `false` 的 FormatProcessor 被视为纯输入插件。引擎会将其作为透传节点处理（像素缓冲区不变）。
- `can_encode()` 返回 `true` 的 FormatProcessor 作为输出插件。引擎将执行 `encode()` 并将结果写入 `ExecutionContext.encoded_output`。

#### 15.4 GpuProcessor — GPU 计算

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

支持的 GPU 后端：`CUDA`, `Metal`, `Vulkan`, `Auto`（在 types.rs 中定义）。

#### 15.5 AiProcessor — AI 推理

```rust
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;
    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor>;
}
```

AI 后端：`ONNX`, `TensorRT`, `CoreML`, `OpenVINO`, `Burn`

`ModelInfo` 支持 4 种模型来源：`Bundled`（内嵌）、`ExternalFile`（本地路径）、`HuggingFace`（自动下载）、`Url`（HTTP 下载）。

#### 15.6 ExternalToolProcessor — 外部工具透传

```rust
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;
    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(&self, input_paths: &[PathBuf], output_path: &Path, params: &ParameterSet) -> PluginResult<()>;
}
```

#### 15.7 ProgressSink — 进度与取消

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

- `set_progress()` 应在长耗时操作中定期调用，fraction 范围为 `[0.0, 1.0]`
- `is_canceled()` 应在每个迭代周期检查，返回 `true` 时应尽快退出并返回 `PluginError::Canceled`

---

### 16. ParameterSchema 编写

`ParameterSchema` 是插件参数系统的核心。它声明式定义了插件的所有参数，GUI 面板据此自动生成控件。

```rust
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}
```

**ParameterSchema 方法：**

- `empty()` — 创建空 Schema
- `field(section_id, field_id)` — 按 ID 查找字段
- `defaults()` — 返回全部字段默认值的 `ParameterSet`
- `all_fields()` — 获取所有字段引用

**ParameterSection：**

```rust
pub struct ParameterSection {
    pub id: String,                   // 段落 ID
    pub label: String,                // 段落标题
    pub description: Option<String>,  // 段落描述
    pub icon: Option<String>,         // 图标名称
    pub collapsible: bool,            // 是否可折叠
    pub default_collapsed: bool,      // 默认折叠状态
    pub fields: Vec<ParameterField>,  // 参数字段列表
}
```

**ParameterField：**

```rust
pub struct ParameterField {
    pub id: String,                  // 字段 ID（全局唯一）
    pub label: String,               // 字段标签
    pub description: Option<String>,  // 字段描述/帮助文本
    pub help_url: Option<String>,     // 在线帮助 URL
    pub field_type: ParameterType,    // 参数类型
    pub default: serde_json::Value,   // 默认值
    pub required: bool,               // 是否必填
    pub advanced: bool,               // 是否高级参数（默认折叠）
    pub allow_override: bool,         // 是否允许外部覆盖
    pub supports_expression: bool,    // 是否支持 ${ } 表达式
}
```

---

### 17. 17 种 ValueType 详解

Photopipeline 的 `ParameterType` 枚举有 **17 种**变体：

#### 1. String — 文本输入

```rust
ParameterType::String {
    max_length: usize,
    pattern: Option<String>,     // 正则验证
    placeholder: Option<String>, // 占位文本
}
```

#### 2. Integer — 整数值

```rust
ParameterType::Integer {
    min: i64,
    max: i64,
    step: i64,
    unit: Option<String>,        // 如 "px"
    style: IntegerWidget,        // SpinBox | Slider | Combo
}
```

#### 3. Float — 浮点值

```rust
ParameterType::Float {
    min: f64,
    max: f64,
    step: f64,
    precision: u8,               // 小数位数
    unit: Option<String>,        // 如 "dB"
    logarithmic: bool,           // 对数滑块
    style: FloatWidget,          // SpinBox | Slider | ComboSlider | DragInput
}
```

#### 4. Boolean — 开关

```rust
ParameterType::Boolean {
    label_true: Option<String>,
    label_false: Option<String>,
}
```

#### 5. Enum — 枚举选择

```rust
ParameterType::Enum {
    options: Vec<EnumOption>,
    display: EnumDisplay,        // Dropdown | RadioGroup | ButtonGroup | SegmentedControl | Tabs | PopupCard
}
```

`EnumOption` 包含 `value`, `label`, `description`, `icon`, `tags`, `recommended`。

#### 6. Color — 颜色选择器

```rust
ParameterType::Color {
    mode: ColorMode,             // RGB | RGBA | HSL | HSV | Lab
    show_alpha: bool,
}
```

#### 7. FilePath — 文件路径选择器

```rust
ParameterType::FilePath {
    kind: FilePathKind,          // File | Directory | SaveFile
    filters: Vec<(String, String)>, // 如 ("LUT Files", "*.cube")
    must_exist: bool,
}
```

#### 8. Coordinate — 经纬度坐标

```rust
ParameterType::Coordinate {
    alt_required: bool,
    direction_required: bool,
}
```

#### 9. Slider — 滑块

```rust
ParameterType::Slider {
    min: f64,
    max: f64,
    step: f64,
    show_ticks: bool,
    ticks: Option<Vec<f64>>,      // 自定义刻度
    show_value: bool,
    orientation: SliderOrientation, // Horizontal | Vertical
    style: SliderStyle,           // Continuous | Discrete | Range | DualHandle
}
```

#### 10. ComboSlider — 预设组合滑块

```rust
ParameterType::ComboSlider {
    min: f64,
    max: f64,
    step: f64,
    presets: Vec<(String, f64)>,  // 预设标签-值对
    unit: Option<String>,
}
```

#### 11. Expression — 表达式编辑器

```rust
ParameterType::Expression {
    variables: Vec<VariableDef>,  // 可用变量
}
```

`VariableDef` 包含 `name`, `description`, `var_type` (`"number"` | `"string"` | `"boolean"`), `example`。

#### 12. Preset — 预设管理器

```rust
ParameterType::Preset {
    preset_schema_ref: String,          // 引用参数 Schema ID
    builtin_presets: Vec<NamedPreset>,  // 内置预设
    allow_custom: bool,                 // 允许用户自定义
    allow_import: bool,                 // 允许导入外部预设
}
```

#### 13. Array — 动态列表

```rust
ParameterType::Array {
    element: Box<ParameterField>,  // 元素 Schema
    min_items: usize,
    max_items: Option<usize>,
}
```

#### 14. MapWidget — 地图选取器

```rust
ParameterType::MapWidget {
    show_track: bool,
    show_photos: bool,
    allow_manual_pin: bool,
}
```

#### 15. BeforeAfter — 前后对比预览

```rust
ParameterType::BeforeAfter {
    zoom_levels: Vec<f64>,
    show_histogram: bool,
}
```

#### 16. Separator — 分隔线

```rust
ParameterType::Separator {
    label: Option<String>,
}
```

#### 17. Section — 嵌套段落

```rust
ParameterType::Section {
    fields: Vec<ParameterField>,
}
```

---

### 18. 条件系统

`GroupCondition` 提供 8 种条件类型用于分组参数匹配：

| 条件类型 | 参数 | 说明 |
|---|---|---|
| `ExifEq` | `tag`, `value` | EXIF 标签等于某值 |
| `ExifGte` | `tag`, `value` | EXIF 标签大于等于 |
| `ExifLte` | `tag`, `value` | EXIF 标签小于等于 |
| `GpsNear` | `lat`, `lon`, `radius_km` | GPS 坐标在半径范围内 |
| `Always` | — | 始终为 true |
| `And` | `Vec<GroupCondition>` | 全部为 true |
| `Or` | `Vec<GroupCondition>` | 任意为 true |
| `Expression` | `String` | 表达式求值 |

GpsNear 使用 **Haversine 公式**计算两点间的大圆距离（地球半径 6371km）。

---

### 19. 自定义渲染器

`GuiSchema` 定义了插件在 GUI 中的呈现方式：

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub preview: PreviewMode,
    pub aux_views: Vec<AuxView>,
    pub min_panel_width: u32,
}
```

**GuiLayout：**

- `Standard { sections }` — 标准段落布局，`sections` 引用 `ParameterSection::id`
- `Custom { rows }` — 自定义行列布局，通过 `GuiRow` / `GuiCell` 精确控制位置

**PreviewMode：**

- `None` — 无预览
- `Live` — 实时预览
- `ManualRefresh` — 手动刷新预览
- `BeforeAfter { default_split, orientation, lock_zoom }` — 前后对比模式
- `Tiled { rows, cols }` — 多宫格预览

**AuxView（辅助视图）：**

`Histogram`, `Waveform`, `Vectorscope`, `GamutDiagram`, `Map`, `FocusPeaking`, `ClippingWarning`, `MetadataTable`, `ProgressBar`, `StatusText`

---

### 20. 注册插件

#### Builtin 方式（内置）

在 `crates/plugins/src/lib.rs` 的 `register_all()` 函数中注册：

```rust
pub fn register_all(registry: &Arc<Registry>) {
    {
        let p: Arc<MyPlugin> = Arc::new(MyPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        // 注册能力 Trait
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
        // let _ = registry.register_format_processor(...)
        // let _ = registry.register_ai_processor(...)
    }
}
```

插件随主程序一同编译。优势是始终可用，无加载失败风险。

#### Native 动态库方式

创建独立 Rust `cdylib` crate，提供 `plugin_entry` 导出函数返回 `Box<dyn Plugin>`。放置 `.so`/`.dll`/`.dylib` 以及 `.toml` manifest 文件到插件搜索路径。

#### Registry API

| 方法 | 说明 |
|---|---|
| `register(Arc<dyn Plugin>)` | 注册基础插件 |
| `register_pixel_processor(Arc<dyn PixelProcessor>)` | 注册像素处理器 |
| `register_format_processor(Arc<dyn FormatProcessor>)` | 注册格式处理器 |
| `register_metadata_processor(Arc<dyn MetadataProcessor>)` | 注册元数据处理器 |
| `register_gpu_processor(Arc<dyn GpuProcessor>)` | 注册 GPU 处理器 |
| `register_ai_processor(Arc<dyn AiProcessor>)` | 注册 AI 处理器 |
| `register_external_tool_processor(Arc<dyn ExternalToolProcessor>)` | 注册外部工具处理器 |
| `get(id)` | 按 ID 获取插件 |
| `query(PluginQuery)` | 条件查询 |
| `all()` | 获取所有插件 |

**PluginQuery 查询条件：**

```rust
pub struct PluginQuery {
    pub category: Option<PluginCategory>,
    pub tags: Vec<String>,
    pub requires_pixel: Option<bool>,
    pub keyword: Option<String>,
    pub enabled_only: bool,
}
```

---

### 21. 插件验证

`Plugin::validate()` 方法在管线执行前被调用，返回 `Vec<ValidationIssue>`：

```rust
pub enum ValidationIssue {
    Error { param: String, message: String },
    Warning { param: String, message: String },
    Info { param: String, message: String },
}
```

- `Error` 级别的问题会导致管线执行失败
- `Warning` 和 `Info` 级别的问题会被记录但不会阻止执行

管线引擎在 `NodeExecutor::execute()` 中检查 Error 级别的 Issue 并返回 `PluginError::ValidationFailed`。

---

### 22. 测试插件

必须为每个插件编写单元测试和集成测试。

**单元测试（在插件模块底部）：**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_identity() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.id(), "photopipeline.plugins.my_plugin");
    }

    #[test]
    fn test_schema_defaults() {
        let plugin = MyPlugin::new();
        let defaults = plugin.parameter_schema().defaults();
        assert_eq!(defaults.get_f64("amount"), Some(0.0));
    }

    #[test]
    fn test_validation_accepts_valid() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("amount".into(), serde_json::json!(0.5));
        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validation_rejects_out_of_range() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("amount".into(), serde_json::json!(5.0));
        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(!issues.is_empty());
    }
}
```

**集成测试（在 `cli/tests/` 中）：**

```rust
#[test]
fn test_my_plugin_registered() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    assert!(registry.get(&"photopipeline.plugins.my_plugin".into()).is_some());
}
```

**运行测试的命令：**

```bash
# 全部测试
cargo test --workspace --no-fail-fast

# 指定模块
cargo test -p photopipeline-plugins -- my_plugin

# 集成测试
cargo test -p photopipeline-cli --test integration_test

# CI 质量检查
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## 第四部分：执行引擎

### 23. 管线执行流程

`NodeExecutor` 是管线执行的核心。它接收 `PipelineGraph`、`ImageInfo`、可选的 `PixelBuffer`、`Metadata` 以及 `ProgressSink`，协调整个执行过程。

**完整执行流程：**

```
1. 拓扑排序 → Vec<NodeId>
2. 创建 ExecutionContext { image_info, buffer, metadata, node_states }
3. 遍历节点（按拓扑顺序）：
   a. 检查 cancel → 返回 PluginError::Canceled
   b. 检查 enabled → false 则 Skip
   c. 查找插件 → 不存在则 NotFound
   d. ParameterResolver::resolve() → 四级参数合并 + 表达式求值
   e. Plugin::validate() → 检查参数合法性（Error 级别失败）
   f. 按插件能力分派执行：
      - requires_pixel_access() == true → process_pixel_node()
        - 像素数 > 8.8M → TileEngine::process_tiled()
        - 否则 → PixelProcessor::process_pixels()
      - MetadataProcessor → process_metadata_node()
      - FormatProcessor → process_format_node()
      - 无匹配 → NotFound 错误
   g. 记录 NodeRunState
4. 返回 ExecutionResult
```

**执行引擎的三路分派逻辑：**

```rust
if plugin.requires_pixel_access() {
    // → 像素处理路径
    self.process_pixel_node(&mut ctx, node, &final_params, progress).await
} else if registry.get_metadata_processor(&node.plugin_id).is_some() {
    // → 元数据处理路径
    self.process_metadata_node(&mut ctx, node, &final_params).await
} else if registry.get_format_processor(&node.plugin_id).is_some() {
    // → 格式处理路径
    self.process_format_node(&mut ctx, node, &final_params).await
} else {
    // → 未找到匹配的处理器
    Err(PluginError::NotFound)
}
```

---

### 24. DAG 拓扑与节点排序

`PipelineGraph::topological_order()` 使用 **Kahn 算法**（BFS + 入度表）从有向边中推导执行顺序：

```
1. 构建 port_to_node 映射（每个端口属于哪个节点）
2. 根据边构建 adjacency 邻接表和 in_degree 入度表
3. 入度为 0 的节点入队
4. 逐一出队：将该节点加入结果，所有邻居入度减 1
5. 入度变为 0 的邻居入队
6. 结果长度不等于节点总数 → 存在环 → Err(CircularDependency)
```

**关键特性：**

- 入度为 0 的节点最先执行（通常为输入/RAW 解码节点）
- 无依赖关系的节点在同一层级（例如 diamond 结构中 B 和 C 并发可达）
- 当前版本串行执行各节点，未来版本将支持不互相依赖节点的并行执行
- 在 `connect()` 时实时检测环，创建边后立即调用 `has_cycle()` 确认

---

### 25. 参数解析

`ParameterResolver` 实现了四级优先级参数合并系统和表达式引擎。

**四级优先级解析流程（`resolve()` 方法）：**

```
1. resolve_plugin_defaults(schema)       → 从 Schema 提取默认值
2. 合并 template_params[node_id]         → 管线模板中的参数覆盖
3. 快照 allow_override == false 字段       → 保存不可覆盖字段的当前值
4. 遍历所有分组条件，匹配则合并参数        → 后匹配者覆盖先匹配者
5. 合并 image_overrides[(image_id, node_id)] → 最高优先级覆盖
6. 恢复不可覆盖字段的快照值                → 保证安全参数不被修改
7. resolve_expressions()                 → 求值所有含 ${ } 的参数
```

**表达式引擎（ExpressionEngine）：**

`ExpressionEngine` 支持行内表达式语法 `${expression}`，在参数解析的最终阶段自动求值。

**变量命名空间：**

`exif.*` — ISO、光圈、快门、焦距、品牌、型号、镜头
`image.*` — 文件名、宽度、高度、文件大小

**运算符：** `>`, `<`, `>=`, `<=`, `==`, `!=`, `? :`

**表达式解析内部实现：**

- `find_ternary()` — 通过括号平衡匹配找到最外层的 `?` 位置
- `find_matching_colon()` — 找到匹配的 `:` 位置（考虑嵌套三元）
- `eval_comparison()` — 拆分运算符左侧和右侧，分辨类型并比较
- `resolve_variable()` — 按前缀 `exif.` / `image.` 分派到具体解析函数
- 浮点数比较使用 `f64::EPSILON` 容差
- 字符串比较精确匹配，区分大小写

---

### 26. 分块引擎

当处理的图像像素数超过阈值（约 4096x2160 = 8.8M 像素）时，`NodeExecutor` 自动启用 `TileEngine` 进行分块处理。

**TileEngine 参数：**

- `default_tile_size`：默认 1024px
- `overlap`：块与块之间的重叠区域，默认 64px（防止边界伪影）
- `max_parallel`：最大并行块数，默认使用 `available_parallelism()`

**TileLayout 计算：**

```rust
pub fn new(image_width: u32, image_height: u32, tile_size: u32, overlap: u32) -> Self {
    let stride = tile_size.saturating_sub(overlap).max(1);
    let tiles_x = image_width.div_ceil(stride);
    let tiles_y = image_height.div_ceil(stride);
    // ...
}
```

**分块处理流程：**

1. 创建 `TileLayout` 计算分块布局
2. 遍历所有块（`iter_tiles()`）
3. 对每个块：从源 `PixelBuffer` 复制块数据到 `tile_input`
4. 对每个块：调用 `processor.process_pixels(tile_input, tile_output, ...)`
5. 收集所有 `TileResult`
6. 将处理后的块拼接回完整的输出 `PixelBuffer`

**分块优势：**
- 降低峰值 VRAM 占用（4096x2160 的 f32 RGBA 约 135MB，分成 16 个 256px 块后每块约 1MB）
- CPU 缓存友好
- 允许外部平台之间的取消（每个 tile 之间检查 `is_canceled()`）
- 边界重叠（overlap）确保无可见接缝

---

### 27. 进度报告

`ProgressSink` trait 是进度报告和取消的抽象接口：

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

**引擎层的进度调用点：**

- 节点执行开始前：`set_progress(i/node_count, "processing node {label}")`
- TileEngine 每块处理前：`set_progress(i/tile_count, "processing tile {i}/{total}")`
- 节点执行全部完成：`set_progress(1.0, "complete")`

---

### 28. 取消机制

取消通过 `ProgressSink::is_canceled()` 实现。引擎和插件中的多个检查点确保可及时响应取消请求：

1. **节点间检查：** 每个节点开始执行前检查
2. **分块处理检查：** 每个 tile 开始处理前检查
3. **参数验证后：** 验证失败的节点标记为 Failed

被取消的执行返回 `PluginError::Canceled { plugin: PluginId }`。

通过 gRPC 的 `BatchService::Cancel()` RPC 可以取消批量执行。CLI 模式通过 Ctrl+C 发送取消信号。

---

### 29. 性能考量

**内存模型：**

管线引擎采用惰性求值 + 写时复制的内存策略：

- 纯元数据插件（`requires_pixel_access() == false`）：完全不分配像素内存
- 首个像素处理插件：引擎在调用前创建输入和输出 `PixelBuffer`
- 大图像自动分块：像素数超过 8.8M 时自动启用 TileEngine

**内存估算公式：**

```
所需内存 ≈ 像素数 × 通道数 × 字节/通道 × (分块数并行 + 2)
```

例如 4096x2160 f32 RGBA ≈ 135MB，分 16 块处理，每块约 1MB，并行 4 块 ≈ 4MB + 输出 135MB ≈ 140MB。

**并行度推荐：**

| 场景 | 推荐 parallel | 原因 |
|---|---|---|
| 纯元数据管线 | CPU 核心数 x 2 | I/O 密集型 |
| CPU 像素处理 | CPU 核心数 | 计算密集型 |
| GPU 像素处理 | 2-4 | GPU 并发瓶颈 |
| 内存受限 (< 8GB) | 1-2 | 避免 OOM |

**Tile 大小推荐：**

| 图像尺寸 | 推荐 tile_size | 原因 |
|---|---|---|
| < 2000px | 256 | 减少开销 |
| 2000-6000px | 512 | 默认平衡 |
| > 6000px | 1024 | 减少分块数 |

---

## 第五部分：服务端与 gRPC

### 30. 启动服务端

Photopipeline 使用统一的 `photopipeline` 二进制文件，服务端通过 `serve` 子命令启动：

```bash
# 默认配置启动（localhost:50051）
photopipeline serve

# 自定义监听地址
photopipeline serve --addr 0.0.0.0:50051

# 指定日志级别
photopipeline serve --log-level debug
```

服务端启动时会：
1. 初始化 `tracing` 日志系统
2. 安装全局 `panic_hook`
3. 创建 `Registry` 并加载所有内置插件（通过 `photopipeline_plugins::register_all()`）
4. 基于 `tokio` 异步运行时启动 gRPC 服务器
5. 注册三个 gRPC 服务：`PipelineService`、`ImageService`、`BatchService`

---

### 31. gRPC API 参考

Photopipeline 定义了三套 gRPC 服务，通过 Protobuf v3 定义。所有服务均运行在 `localhost:50051`（默认）。

#### 31.1 PipelineService — 管线服务

```protobuf
service PipelineService {
  rpc CreatePipeline(PipelineSpec) returns (PipelineId);
  rpc Execute(ExecuteRequest) returns (stream ExecuteProgress);
  rpc Validate(PipelineSpec) returns (ValidationResult);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}
```

| RPC | 输入 | 输出 | 说明 |
|---|---|---|---|
| `CreatePipeline` | `PipelineSpec` | `PipelineId` | 创建管线实例 |
| `Execute` | `ExecuteRequest` | stream `ExecuteProgress` | 执行管线，流式返回进度 |
| `Validate` | `PipelineSpec` | `ValidationResult` | 验证管线结构 |
| `GetNodeSchema` | `PluginId` | `NodeSchema` | 获取插件的 Schema 定义 |

**ExecuteProgress 消息（流式）：**

```protobuf
message ExecuteProgress {
  enum Stage {
    LOADING = 0; DECODING = 1; PROCESSING = 2; ENCODING = 3; DONE = 4; ERROR = 5;
  }
  Stage stage = 1;
  string node_id = 2;
  string node_label = 3;
  float fraction = 4;
  string message = 5;
  int64 elapsed_ms = 6;
}
```

#### 31.2 ImageService — 图像服务

```protobuf
service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);
  rpc Decode(DecodeRequest) returns (stream PixelDataChunk);
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);
  rpc GetThumbnail(ThumbnailRequest) returns (ImageData);
}
```

| RPC | 输入 | 输出 | 说明 |
|---|---|---|---|
| `Load` | `ImagePath` | `ImageInfo` | 加载图像元信息 |
| `Decode` | `DecodeRequest` | stream `PixelDataChunk` | 解码图像为像素数据块 |
| `Encode` | `EncodeRequest` | stream `EncodeProgress` | 编码像素数据为文件 |
| `GetThumbnail` | `ThumbnailRequest` | `ImageData` | 获取缩略图 |

#### 31.3 BatchService — 批量服务

```protobuf
service BatchService {
  rpc SubmitBatch(BatchSpec) returns (BatchId);
  rpc GetProgress(BatchId) returns (stream BatchProgress);
  rpc Cancel(BatchId) returns (google.protobuf.Empty);
}
```

| RPC | 输入 | 输出 | 说明 |
|---|---|---|---|
| `SubmitBatch` | `BatchSpec` | `BatchId` | 提交批量任务 |
| `GetProgress` | `BatchId` | stream `BatchProgress` | 获取批量处理进度 |
| `Cancel` | `BatchId` | `Empty` | 取消批量处理 |

---

### 32. 通过 gRPC 运行管线

**架构设计理念：**

管线定义（TOML 文件）由前端管理并作为文件路径传递给后端，**不在 gRPC 负载中传输管线 JSON**。gRPC 仅用于实时交互（执行进度、图像编解码、进度流）。

**典型交互流程：**

```
1. GUI 创建管线 → 保存为 TOML 文件
2. GUI 调用 CreatePipeline(PipelineSpec{name, config_path}) → 获取 PipelineId
3. GUI 调用 Execute(ExecuteRequest{pipeline_id, image_path, output_path}) → 建立 stream
4. 服务端返回 ExecuteProgress 流 → GUI 更新进度条和消息
5. 服务端返回 stage = DONE → 完成
```

---

### 33. CLI 模式 vs 服务端模式

| 特性 | CLI 模式 | 服务端模式 (gRPC) |
|---|---|---|
| 使用场景 | 脚本、自动化、CI/CD | GUI 交互、实时预览 |
| 配置方式 | TOML 文件 + 命令行参数 | TOML 文件路径通过 RPC 传递 |
| 进度反馈 | 终端输出 | 流式 `ExecuteProgress` |
| 批量处理 | `batch run` 子命令 | `BatchService.SubmitBatch` RPC |
| 并行度 | `[batch].parallel` 配置 | `BatchSpec.parallel` 参数 |
| 启动开销 | 每次执行启动新进程 | 服务端常驻，首次执行无冷启动开销 |

---

### 34. 后端服务生命周期

```
启动
  ├─ 注册所有内置插件（14 个）
  ├─ 启动 gRPC 服务器
  ├─ 监听请求循环
  │   ├─ PipelineService::CreatePipeline
  │   │   └─ 解析 TOML → 构建 PipelineGraph → 在内存中缓存
  │   ├─ PipelineService::Execute
  │   │   └─ 创建 NodeExecutor → 按拓扑排序执行 → 流式返回进度
  │   ├─ ImageService::Decode
  │   │   └─ 通过 FormatProcessor 流式解码 → 返回 PixelDataChunk
  │   ├─ BatchService::SubmitBatch
  │   │   └─ 创建 BatchScheduler → 并行执行多张图片
  │   └─ BatchService::Cancel
  │       └─ 设置取消标志 → 执行中的任务检查并退出
  └─ SIGTERM/SIGINT → 优雅关闭
      ├─ 等待正在执行的任务完成
      ├─ 释放 GPU 资源
      ├─ 关闭 gRPC 服务器
      └─ 退出
```

---

## 第六部分：CLI 参考

### 35. 完整 CLI 命令参考

Photopipeline 使用 `clap` 构建 CLI，项目为统一二进制文件，通过子命令区分功能。

**全局结构：**

```
photopipeline [OPTIONS] <COMMAND>

COMMANDS:
  pipeline    管线管理
    run       运行管线
    validate  验证管线配置
  plugin      插件管理
    list      列出所有插件
    info      查看插件详情
  batch       批量处理
    run       批量运行
    validate  批量验证
  serve       启动 gRPC 服务端
```

#### 35.1 pipeline run — 执行管线

```bash
photopipeline pipeline run \
  -c, --config <CONFIG>    # TOML 管线配置文件
  -i, --input <INPUT>      # 输入图像文件
  -o, --output <OUTPUT>    # 输出文件
```

#### 35.2 pipeline validate — 验证管线

```bash
photopipeline pipeline validate \
  -c, --config <CONFIG>    # 管线配置文件
```

验证内容包括：管线结构是否合法、边是否引用存在的节点、是否存在环。

#### 35.3 plugin list — 列出插件

```bash
photopipeline plugin list
```

按分类（Input, Metadata, Color, Transform, Enhance, Format）分组显示所有已注册插件。

#### 35.4 plugin info — 插件详情

```bash
photopipeline plugin info photopipeline.plugins.colorspace
```

输出插件的完整元信息、参数 Schema 及 GUI Schema。

#### 35.5 batch run — 批量处理

```bash
photopipeline batch run \
  -c, --config <CONFIG>       # 管线配置文件
  -p, --pattern <PATTERN>     # 文件 glob 模式 [默认: *.ARW]
  -o, --output <OUTPUT_DIR>   # 输出目录 [默认: ./output/]
```

#### 35.6 batch validate — 批量验证

```bash
photopipeline batch validate \
  -c, --config <CONFIG>       # 管线配置文件
  -p, --pattern <PATTERN>     # 文件 glob 模式
```

#### 35.7 serve — 启动服务端

```bash
photopipeline serve \
  --addr 0.0.0.0:50051        # 监听地址 [默认: localhost:50051]
  --log-level debug            # 日志级别 [默认: info]
```

---

### 36. 脚本与自动化示例

**处理单张图片（简单）：**

```bash
#!/bin/bash
photopipeline pipeline run \
  -c pipeline.toml \
  -i "$1" \
  -o "${1%.*}.jxl"
```

**批量处理目录中所有 ARW 文件：**

```bash
#!/bin/bash
for f in ~/photos/*.ARW; do
  echo "Processing $f..."
  photopipeline pipeline run -c pipeline.toml -i "$f" -o "output/$(basename "${f%.*}").jxl"
done
```

**先验证后执行：**

```bash
#!/bin/bash
if photopipeline pipeline validate -c pipeline.toml; then
  photopipeline pipeline run -c pipeline.toml -i "$1" -o "${1%.*}.jxl"
else
  echo "管线验证失败!" >&2
  exit 1
fi
```

---

### 37. 与 Shell 脚本和 CI/CD 集成

**GitHub Actions 集成示例：**

```yaml
name: Process Images
on:
  push:
    paths:
      - 'photos/**'

jobs:
  process:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Photopipeline
        run: cargo install photopipeline
      - name: Validate Pipeline
        run: photopipeline pipeline validate -c pipeline.toml
      - name: Process Images
        run: photopipeline batch run -c pipeline.toml -p "photos/**/*.ARW" -o "output/"
```

**Docker 集成：**

```dockerfile
FROM rust:1.90-slim
RUN apt-get update && apt-get install -y libheif-dev libjxl-dev liblcms2-dev
COPY . /app
WORKDIR /app
RUN cargo build --profile ci --workspace
ENTRYPOINT ["./target/release/photopipeline"]
```

---

## 第七部分：高级主题

### 38. 自定义数据类型

#### 38.1 GeoCoordinate

`GpsData` 结构体完整定义了 GPS 地理坐标数据类型，包含纬度、经度、海拔、方向、速度、轨道等 20+ 个字段。它还提供了：

- `has_coordinates()` — 检查是否具有有效坐标
- `coordinate_tuple()` — 获取 (纬度, 经度) 元组

#### 38.2 TimestampWithZone

时间通过 `chrono::DateTime<Utc>` 统一表示 UTC 时间戳。`time_shift` 插件通过 `shift_hours`、`shift_minutes`、`shift_seconds` 以及 `source_timezone`、`target_timezone` 参数实现时区偏移。

#### 38.3 AlignedBuffer

`AlignedBuffer` 是页对齐的内存缓冲区，用于像素数据的零拷贝传输。支持 64 字节对齐以兼容 GPU 直接映射：

```rust
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,
}
```

提供安全的类型转换方法：
- `as_u16_slice()` — 通过 `bytemuck::cast_slice` 将 u8 缓冲区转换为 u16 切片
- `as_f32_slice()` — 转换为 f32 切片

当系统分配器返回的内存不满足对齐要求时，会自动进行手动对齐分配（overallocate + offset）。

#### 38.4 Tensor

```rust
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,  // F32 | F16 | I8 | U8
}
```

`Tensor` 用于 AI 推理插件（如 `ai_denoise`），支持 4 种数据类型。

---

### 39. 节点间数据类型契约

管线中节点间的数据传递不是通过显式类型系统，而是通过 `ExecutionContext` 的共享状态实现：

```
ExecutionContext {
    image_info: ImageInfo,        // 输入图像元信息（全程不变）
    buffer: Option<PixelBuffer>,  // 像素缓冲区（首个像素插件分配，后续原地修改）
    encoded_output: Option<Vec<u8>>, // 最终编码输出（最后一个格式插件写入）
    metadata: Metadata,            // 元数据（全程共享，各元数据插件增量修改）
    node_states: HashMap<NodeId, NodeRunState>, // 各节点执行状态
}
```

**数据传递路径：**

- `PixelBuffer`：由首个 `requires_pixel_access() == true` 的节点触发分配。后续像素处理节点在同一个 buffer 上原地修改。
- `Metadata`：由元数据插件增量修改（`read_metadata` 读取后合并，`write_metadata` 写回）。
- `encoded_output`：由格式插件（`can_encode == true`）写入。如果管线无输出节点，返回值保持为 `None`，由 CLI 上层处理。

---

### 40. FFI 集成

#### 40.1 Halide FFI 绑定

Halide 是用于 CPU SIMD 和 GPU 计算的核心计算引擎。Halide C++ 生成器源文件位于 `halide_generators/` 目录：

```
halide_generators/
├── CMakeLists.txt
├── colorspace_generator.cpp   → 色彩空间转换内核
├── resize_generator.cpp       → 图像缩放内核
└── tonemap_generator.cpp      → 色调映射内核
```

这些 C++ 文件在 CI 上编译，生成平台特定的 Halide 计算库。Rust 端通过 FFI 绑定调用这些预编译的内核。

插件 `colorspace`、`lut3d`、`transform` 使用 Halide 进行像素处理。如果 Halide 内核不可用，这些插件使用纯 Rust 后备实现。

#### 40.2 OIIO FFI 绑定

OpenImageIO 的 FFI 绑定封装在 crate `oiio/` 中，支持额外的格式读写（如 OpenEXR）。OIIO 绑定通过 feature gate 控制启用。

#### 40.3 System Library FFI

三个编码器插件直接绑定系统 C 库：

- `heif_encoder` → libheif (通过 x265)
- `jxl_encoder` → libjxl
- `avif_encoder` → libheif (通过 aom)

这些插件同时支持内置编码器和外部命令行工具（`heif-enc`、`cjxl`、`avifenc`）。

---

### 41. 内存管理

#### 41.1 Arc 共享与写时复制 (COW)

Photopipeline 使用 `Arc` 智能指针实现零拷贝内存共享：

```
Metadata 插件           → Arc<Metadata> 共享，零拷贝
Metadata → Metadata     → 始终共享同一 Arc
Metadata → Pixel 插件   → 仅单消费者写入时触发写时复制
Pixel → Pixel（单消费者）→ Arc 不重复，原地修改
Pixel → Pixel（多消费者）→ Arc 共享，只读
GPU → GPU              → GpuHandle 传递，数据留在 VRAM 直到编码
```

#### 41.2 惰性分配

像素缓冲区仅在管线中包含至少一个 `requires_pixel_access() == true` 的节点时分配。纯元数据管线（如 GPS 标注）完全不分配像素内存，峰值内存仅约 8MB。

#### 41.3 内存对齐

`AlignedBuffer` 请求 64 字节对齐，兼容 AVX-512 向量指令和 GPU 直接映射。当标准分配器无法满足对齐要求时，使用 fallback 手动对齐策略。

---

### 42. 色彩管理

Photopipeline 内置了完整的色彩科学实现，不依赖外部库即可进行色彩空间转换。

#### 42.1 ColorSpace 结构

```rust
pub struct ColorSpace {
    pub primaries: ColorPrimaries,   // 色域原色
    pub transfer: TransferFunction,  // 传递函数
    pub white_point: WhitePoint,     // 白点
    pub hdr_nits: Option<f32>,       // HDR 亮度（大于 203 nit 则为 HDR）
}
```

#### 42.2 支持的色域原色 (ColorPrimaries)

`SRGB`, `BT709`, `DisplayP3`, `AdobeRGB`, `ProPhoto`, `BT2020`, `Rec2100`, `ACES`, `ACEScg`, `DCIP3`, `CIEXYZ`

每种原色都定义了 (x,y) 色度坐标。

#### 42.3 支持的传递函数 (TransferFunction)

`Linear`, `SRGB`, `Gamma22`, `Gamma24`, `Gamma26`, `Gamma28`, `PQ` (ST.2084), `HLG`, `SLog3`, `LogC`, `Custom(f64)`

每种传递函数都实现了 `decode_to_linear()` 和 `encode_from_linear()` 方法。

#### 42.4 色彩空间转换矩阵

`ColorSpace::conversion_matrix_to()` 生成从一个色彩空间到另一个色彩空间的 3x3 转换矩阵。核心算法：

1. 获取源和目标色域原色的 XYZ 矩阵
2. 通过 Bradford 色适应模型进行白点转换
3. 计算 3x3 转换矩阵

#### 42.5 ICC Profile 生成

`ColorSpace::generate_icc_profile()` 可以从色彩空间定义生成二进制 ICC v2 profile。这允许在最终输出中嵌入准确的 ICC profile，确保下游应用可以正确解释色彩。

#### 42.6 外部色彩管理

插件 `colorspace` 同时支持：
- **内置方式：** 使用 Photopipeline 内置矩阵和传递函数
- **LittleCMS2：** 通过 `lcms2` 进行 ICC profile 级转换
- **OpenColorIO：** 通过 `ocio_config` 参数支持 VFX 级色彩管理

#### 42.7 渲染意图 (RenderingIntent)

`Perceptual` | `RelativeColorimetric` | `Saturation` | `AbsoluteColorimetric`

#### 42.8 色域映射 (GamutMapping)

`Clip` | `Compress` | `LuminancePreserve`

---

### 43. 16 位以上精度保证

Photopipeline 的核心设计原则是**端到端 16 位以上精度**。

#### 43.1 PixelFormat 体系

```rust
pub enum PixelFormat {
    U8,   // 8 位无符号整数（不推荐）
    U16,  // 16 位无符号整数
    U32,  // 32 位无符号整数
    F16,  // 16 位浮点（半精度）
    F32,  // 32 位浮点（单精度）
}
```

- `is_high_precision()` 返回除 `U8` 以外的所有格式（≥ 16 位）
- `is_float()` 用于区分整数和浮点格式
- `bytes_per_channel()` 返回每通道字节数

#### 43.2 精度保证机制

1. **禁止隐式降级：** 不存在任何隐蔽的位深度转换。如果相机记录 14 位 RAW 数据，Photopipeline 将完整保留所有比特。
2. **格式匹配：** PixelProcessor 的 `supported_input_formats()` 和 `supported_output_formats()` 确保插件仅处理其支持的格式。
3. **编码器控制：** 各编码器支持精确的位深度设置（HEIF 8/10 bit，JXL 8/10/12/16 bit，PNG 8/16 bit，TIFF 8/16/32f bit）。
4. **累积误差防护：** 通过维持整个管线的浮点格式（f32），避免多次色调映射造成的累积舍入误差。在最终编码阶段才转换到目标位深度。

#### 43.3 精度最佳实践

- RAW 解码始终输出 f32（`output_format = "f32"`）
- 色彩空间转换在 f32 中进行
- 仅在最终编码阶段降级到位深度（如 10-bit HEIF 或 16-bit JXL）

---

## 第八部分：参考

### 44. 14 个内置插件目录

#### 44.1 元数据类插件（3 个）

**exif_rw — EXIF 读写**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.exif_rw` |
| 类别 | Metadata |
| 能力 Trait | Plugin + MetadataProcessor |
| 后端 | ExifTool + kamadak-exif |
| 像素访问 | 否 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `read_exif` | bool | `true` | 读取 EXIF |
| `read_xmp` | bool | `true` | 读取 XMP |
| `read_iptc` | bool | `true` | 读取 IPTC |
| `read_gps` | bool | `true` | 读取 GPS |
| `overwrite_original` | bool | `false` | 覆盖原文件 |
| `preserve_makernote` | bool | `true` | 保留 MakerNote |
| `write_exif` | enum | `"all"` | 写入策略：`all` / `selected` / `none` |
| `exiftool_path` | string | `"exiftool"` | exiftool 路径 |
| `exiftool_args` | string | `""` | 附加参数 |

**gps_set — GPS 坐标设置**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.gps_set` |
| 类别 | Metadata |
| 能力 Trait | Plugin + MetadataProcessor |
| 后端 | ExifTool + geo crate |
| 像素访问 | 否 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `gps_mode` | enum | `"manual"` | GPS 模式：`manual` / `gpx_track` / `clear` |
| `latitude` | float | — | 纬度 (-90~90) |
| `longitude` | float | — | 经度 (-180~180) |
| `altitude` | float | — | 海拔 (m) |
| `gpx_file` | file_path | — | GPX 轨迹文件 |
| `time_offset_seconds` | int | `0` | 相机-GPS 时间偏移 |
| `max_interpolation_gap` | int | `300` | 最大插值间隔（秒） |

**time_shift — 时间偏移**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.time_shift` |
| 类别 | Metadata |
| 能力 Trait | Plugin + MetadataProcessor |
| 后端 | chrono + ExifTool |
| 像素访问 | 否 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `shift_hours` | int | `0` | 偏移小时 (-23~23) |
| `shift_minutes` | int | `0` | 偏移分钟 (-59~59) |
| `shift_seconds` | int | `0` | 偏移秒 (-59~59) |
| `source_timezone` | string | `"UTC"` | 源时区 |
| `target_timezone` | string | `"local"` | 目标时区 |
| `increment_per_image` | float | `0.0` | 每张递增秒数 |
| `batch_image_index` | int | `0` | 批次图片索引 |

#### 44.2 色彩类插件（2 个）

**colorspace — 色彩空间转换**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.colorspace` |
| 类别 | Color |
| 能力 Trait | Plugin + PixelProcessor |
| 后端 | Halide + lcms2 |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `source_color_space` | enum | `"auto"` | 源色彩空间 |
| `target_color_space` | enum | — | 目标色彩空间 |
| `rendering_intent` | enum | `"relative_colorimetric"` | 渲染意图 |
| `black_point_compensation` | bool | `true` | 黑点补偿 |
| `gamut_mapping` | enum | `"compress"` | 色域映射 |
| `embed_icc` | bool | `true` | 嵌入 ICC Profile |
| `icc_profile_path` | file_path | `""` | 自定义 ICC 文件路径 |

**lut3d — 3D LUT 应用**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.lut3d` |
| 类别 | Color |
| 能力 Trait | Plugin + PixelProcessor |
| 后端 | Halide |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `lut_path` | file_path | — | LUT 文件路径 |
| `lut_format` | enum | `"cube"` | 格式：cube / 3dl / look / csp |
| `intensity` | float | `100.0` | 混合强度 (0~100) |
| `input_color_space` | enum | `"srgb"` | LUT 期望输入色彩空间 |
| `clamp_output` | bool | `true` | 钳制输出 |
| `interpolation_method` | enum | `"tetrahedral"` | 四面体 / 三线性 |

#### 44.3 变换类插件（1 个）

**transform — 几何变换**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.transform` |
| 类别 | Transform |
| 能力 Trait | Plugin + PixelProcessor |
| 后端 | Halide |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `resize_mode` | enum | `"none"` | 缩放模式：none / absolute / percentage / long_edge / short_edge / megapixels |
| `target_width` | int | `1920` | 目标宽度 (px) |
| `target_height` | int | `1080` | 目标高度 (px) |
| `scale_percent` | float | `100.0` | 缩放百分比 |
| `long_edge_px` | int | `2048` | 长边目标像素 |
| `angle` | float | `0.0` | 旋转角度 (-360~360) |
| `flip_horizontal` | bool | `false` | 水平翻转 |
| `flip_vertical` | bool | `false` | 垂直翻转 |
| `crop_enabled` | bool | `false` | 启用裁剪 |
| `crop_x`, `crop_y` | int | `0` | 裁剪起点 |
| `crop_width`, `crop_height` | int | `1920`/`1080` | 裁剪尺寸 |
| `filter_type` | enum | `"lanczos3"` | 滤波：bilinear / lanczos3 / nearest |

#### 44.4 增强类插件（2 个）

**lens_correct — 镜头校正**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.lens_correct` |
| 类别 | Enhance |
| 能力 Trait | Plugin + PixelProcessor |
| 后端 | LensFun + Halide |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `correction_mode` | enum | `"auto"` | auto / manual / off |
| `correct_distortion` | bool | `true` | 校正畸变 |
| `correct_tca` | bool | `true` | 校正横向色差 |
| `correct_vignetting` | bool | `true` | 校正暗角 |
| `correct_geometry` | bool | `false` | 独立几何校正 |
| `lensfun_db_path` | file_path | — | LensFun 数据库路径 |
| `camera_make` | string | `""` | 相机品牌（manual） |
| `camera_model` | string | `""` | 相机型号（manual） |
| `lens_model` | string | `""` | 镜头型号（manual） |

**ai_denoise — AI 降噪**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.ai_denoise` |
| 类别 | Enhance |
| 能力 Trait | Plugin + PixelProcessor + AiProcessor |
| 后端 | ONNX / TensorRT / CoreML |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `denoise_model` | enum | `"standard_v2"` | lightweight_v1 / standard_v2 / high_quality_v2 / raw_denoise_v1 |
| `denoise_strength` | float | `50.0` | 降噪强度 (0~100) |
| `detail_preservation` | float | `50.0` | 细节保留 (0~100) |
| `color_noise_reduction` | float | `75.0` | 色彩噪声降低 (0~100) |
| `ai_backend` | enum | `"onnx_cpu"` | onnx_cpu / onnx_cuda / tensorrt / coreml / openvino |
| `tile_size` | int | `0` | 分块大小 (0=自动) |
| `use_fp16` | bool | `true` | 半精度推理 |

#### 44.5 输入插件（1 个）

**raw_input — RAW 输入解码**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.raw_input` |
| 类别 | Input |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | dcraw / LibRaw |
| 像素访问 | 是 |

参数表：
| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `raw_mode` | enum | `"auto"` | auto / dcraw / libraw / rawtherapee |
| `output_format` | enum | `"f32"` | u16 / f32 |
| `half_size` | bool | `false` | 半分辨率解码 |
| `apply_white_balance` | bool | `true` | 应用相机白平衡 |
| `dcraw_path` | string | `"dcraw"` | dcraw 二进制路径 |
| `dcraw_extra_args` | string | `""` | 附加参数 |

**支持的 RAW 格式：** Sony (.arw)、Canon (.cr2/.cr3)、Nikon (.nef)、Adobe DNG (.dng)、Fujifilm (.raf)、Olympus (.orf)、Panasonic (.rw2)、Pentax (.pef)、Hasselblad (.3fr)、Mamiya (.mef)、Leaf (.mos)、Epson (.erf)、Phase One (.iiq)、Samsung (.srw)、Sigma (.x3f)。

#### 44.6 格式编码插件（5 个）

**heif_encoder — HEIF 编码器**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.heif_encoder` |
| 类别 | Format |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | libheif + x265 |
| 像素访问 | 是 |

参数：`quality` (float, 0-100), `lossless` (bool), `bit_depth` (enum, "8"/"10"), `chroma_subsampling` (enum), `encoder_effort` (int, 0-10), `heif_enc_path` (string)

**jxl_encoder — JPEG XL 编码器**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.jxl_encoder` |
| 类别 | Format |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | libjxl |
| 像素访问 | 是 |

参数：`quality` (float, 0-100, -1=lossless), `lossless` (bool), `bit_depth` (enum, "8"/"10"/"12"/"16"), `effort` (int, 1-9), `modular` (bool), `cjxl_path` (string)

**avif_encoder — AVIF 编码器**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.avif_encoder` |
| 类别 | Format |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | libheif + aom |
| 像素访问 | 是 |

参数：`quality` (float, 0-100), `speed` (int, 0-10), `bit_depth` (enum), `chroma_subsampling` (enum), `lossless` (bool), `avifenc_path` (string)

**tiff_encoder — TIFF 编码器**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.tiff_encoder` |
| 类别 | Format |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | OIIO |
| 像素访问 | 是 |

参数：`compression` (enum, "none"/"lzw"/"deflate"/"packbits"), `bigtiff` (bool), `embed_icc` (bool), `pixel_format` (enum, "u8"/"u16"/"f32")

**png_encoder — PNG 编码器**

| 属性 | 值 |
|---|---|
| ID | `photopipeline.plugins.png_encoder` |
| 类别 | Format |
| 能力 Trait | Plugin + FormatProcessor |
| 后端 | lodepng |
| 像素访问 | 是 |

参数：`compression_level` (int, 0-9), `bit_depth` (enum, "8"/"16"), `embed_icc` (bool), `include_exif` (bool), `color_type` (enum, "rgb"/"rgba"/"gray"/"graya")

---

### 45. 核心类型参考

#### 45.1 PixelFormat（像素格式）

| 枚举值 | 每通道字节 | 是否浮点 | 高精度 |
|---|---|---|---|
| `U8` | 1 | 否 | 否 |
| `U16` | 2 | 否 | 是 |
| `U32` | 4 | 否 | 是 |
| `F16` | 2 | 是 | 是 |
| `F32` | 4 | 是 | 是 |

#### 45.2 ChannelLayout（通道布局）

| 枚举值 | 通道数 | 交叉存储 |
|---|---|---|
| `Gray` | 1 | 是 |
| `GrayAlpha` | 2 | 是 |
| `RGB` | 3 | 是 |
| `RGBA` | 4 | 是 |
| `Planar(n)` | n | 否 |
| `Custom(n)` | n | 否 |

#### 45.3 ImageFormat（图像格式）

支持的格式：`HEIF`, `HEIC`, `AVIF`, `JXL`, `PNG`, `TIFF`, `JPEG`, `WEBP`, `OpenEXR`, `RAW`, `DNG`, `PPM`, `PGM`, `BMP`, `Unknown(String)`

#### 45.4 GpuBackend（GPU 后端）

`None`, `CUDA`, `Metal`, `Vulkan`, `Auto`

#### 45.5 AiBackend（AI 后端）

`ONNX`, `TensorRT`, `CoreML`, `OpenVINO`, `Burn`

#### 45.6 PluginCategory（插件类别）

`Input`, `Metadata`, `Color`, `Transform`, `Enhance`, `Merge`, `Format`, `External`, `Custom(String)`

#### 45.7 PluginVersion（语义化版本）

```rust
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>, // 如 "alpha", "rc1"
}
```

#### 45.8 ProcessingStats（处理统计）

```rust
pub struct ProcessingStats {
    pub elapsed_ms: u64,
    pub cpu_time_ms: u64,
    pub gpu_time_ms: Option<u64>,
    pub peak_memory_mb: u64,
    pub input_pixels: u64,
    pub output_pixels: u64,
}
```

#### 45.9 DecodeOptions（解码选项）

```rust
pub struct DecodeOptions {
    pub pixel_format: Option<PixelFormat>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub read_metadata: bool,
    pub apply_transfer: bool,
    pub icc_profile: Option<Vec<u8>>,
}
```

#### 45.10 EncodeOptions（编码选项）

```rust
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
```

---

### 46. 错误码参考

`PluginError` 定义了 21 种错误变体，每种都包含丰富的上下文信息：

| 错误变体 | 触发条件 | 携带字段 |
|---|---|---|
| `NotFound` | 插件未注册 | `plugin: PluginId` |
| `AlreadyLoaded` | 插件重复注册 | `plugin: PluginId` |
| `LoadFailed` | 动态库加载失败 | `plugin, reason` |
| `VersionMismatch` | 版本不满足要求 | `plugin, actual, required` |
| `InvalidParameter` | 参数校验失败 | `plugin, field, message` |
| `MissingTool` | 外部工具未找到 | `plugin, tool, required` |
| `GpuNotAvailable` | GPU 后端不可用 | `plugin, backend` |
| `GpuOutOfMemory` | GPU 内存不足 | `plugin, needed, available` |
| `ExpressionError` | 表达式求值失败 | `plugin, field, error` |
| `Timeout` | 处理超时 | `plugin, elapsed, timeout` |
| `Internal` | 内部错误 | `plugin, message` |
| `Canceled` | 用户取消 | `plugin` |
| `Io` | I/O 错误 | `plugin, error` (source: std::io::Error) |
| `ValidationFailed` | 管线验证失败 | `String` |
| `NodeExecutionFailed` | 节点执行失败 | `node, message` |
| `CircularDependency` | 管线存在环 | — |
| `FileNotFound` | 文件不存在 | `String` |
| `UnsupportedFormat` | 不支持的格式 | `String` |
| `EncodingFailed` | 编码失败 | `String` |
| `DecodingFailed` | 解码失败 | `String` |
| `Config` | 配置错误 | `String` |
| `Other` | 通用错误 | `String` |

**ValidationIssue：**

```rust
pub enum ValidationIssue {
    Error { param: String, message: String },    // 阻止执行
    Warning { param: String, message: String },  // 记录但不阻止
    Info { param: String, message: String },     // 信息性
}
```

---

### 47. 配置文件格式参考

#### 47.1 完整的 TOML 配置模板

```toml
# ── Pipeline Metadata ──
[metadata]
name = "管线名称"
version = "1.0"
description = "管线描述"

# ── Nodes (必填，至少一个) ──
[[nodes]]
id = "node_id"              # 节点唯一 ID (snake_case)
plugin = "photopipeline.plugins.colorspace"  # 插件完整 ID
label = "显示名称"           # 可选，GUI 中显示
enabled = true              # 可选，默认 true
params = { key = "value" }  # 可选，参数覆盖表
# ... 更多节点

# ── Edges (可选) ──
[[edges]]
from = "source_node"
to = "target_node"
# ... 更多边

# ── Overrides (可选) ──
[[overrides]]
image = "DSC0003.ARW"
[overrides.params.node_id]
  param_key = "override_value"
# ... 更多覆盖

# ── Groups (可选) ──
[[groups]]
name = "组名"
condition = "exif.iso >= 1600"
[groups.params.node_id]
  param_key = "group_value"
# ... 更多分组

# ── Batch (可选) ──
[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

#### 47.2 参数值类型映射

| TOML 值 | JSON 等价类型 | ParameterSet 方法 |
|---|---|---|
| `"hello"` | `Value::String` | `get_str()` |
| `42` | `Value::Number(i64)` | `get_i64()` |
| `3.14` | `Value::Number(f64)` | `get_f64()` |
| `true` / `false` | `Value::Bool` | `get_bool()` |

---

### 48. 术语表

| 术语 | 英文 | 说明 |
|---|---|---|
| 管线 | Pipeline | 由节点和有向边组成的有向无环图（DAG），定义图像处理流程 |
| 节点 | Node | 管线中的最小执行单元，绑定一个插件 |
| 插件 | Plugin | 实现 Plugin trait 的可执行单元 |
| 有向无环图 | DAG | Directed Acyclic Graph，管线的基础数据结构 |
| 拓扑排序 | Topological Sort | 根据边依赖关系推导节点执行顺序 |
| 能力 Trait | Capability Trait | PixelProcessor、MetadataProcessor 等功能接口 |
| Schema 驱动 | Schema-Driven | 参数控件由插件 Schema 声明式定义，前端自动生成 |
| 元数据 | Metadata | EXIF、XMP、IPTC、GPS 等图像附属信息 |
| 像素缓冲区 | PixelBuffer | 图像像素数据的运行时表示 |
| 对齐缓冲区 | AlignedBuffer | 页对齐的内存缓冲区，支持 GPU 映射和 SIMD |
| 分块引擎 | TileEngine | 大图像的分块并行处理引擎 |
| 参数解析器 | ParameterResolver | 实现四级优先级参数合并的组件 |
| 表达式引擎 | ExpressionEngine | 实现 `${ }` 行内表达式求值的组件 |
| 注册表 | Registry | 线程安全的全局插件注册表 |
| 执行上下文 | ExecutionContext | 管线执行期间的共享状态容器 |
| 色彩空间 | ColorSpace | 由色域原色、传递函数和白点定义的颜色模型 |
| 传递函数 | TransferFunction | 线性光和编码值之间的映射函数 |
| 渲染意图 | RenderingIntent | 色域映射策略（perceptual / relative / saturation / absolute） |
| 色域映射 | GamutMapping | 将一个色彩空间映射到另一个时的边界处理策略 |
| 写时复制 | COW (Copy-on-Write) | 仅在写入时复制数据的惰性内存策略 |
| 零拷贝 | Zero-Copy | 通过 Arc 共享指针避免数据拷贝 |
| 格式探测 | FormatProbe | 通过扩展名、MIME 类型和魔数判断文件格式 |
| 断点续传 | Resume | 中断后的批处理从中断点继续 |
| Haversine 公式 | Haversine Formula | 计算球面上两点大圆距离的公式 |
| 白点 | White Point | 色彩空间中性灰的色度坐标 |
| ICC Profile | ICC Profile | 描述设备色彩特性的二进制文件 |
| Halide | Halide | 高性能图像计算语言和编译器 |
| OIIO | OpenImageIO | 专业图像 I/O 库 |

---

*文档结束。版本 1.0，基于 Photopipeline 源码（分支 feat/backend-frontend-interaction-redesign），2026 年 5 月。*

# Photopipeline 接口设计文档

> **版本**: 1.0  
> **状态**: 设计阶段  
> **最后更新**: 2026-05-28  
> **语言**: 简体中文

---

## 目录

1. [概述](#1-概述)
2. [外部 gRPC API（Proto 定义）](#2-外部-grpc-api)
   - 2.1 [PluginService](#21-pluginservice)
   - 2.2 [ImageService](#22-imageservice)
   - 2.3 [PipelineService](#23-pipelineservice)
   - 2.4 [BatchService](#24-batchservice)
   - 2.5 [ExecutionService（v2 设计）](#25-executionservicev2-设计)
3. [CLI 接口](#3-cli-接口)
4. [PipelineConfig 文件格式](#4-pipelineconfig-文件格式)
5. [ParameterSchema 格式](#5-parameterschema-格式)
6. [内部 Rust API](#6-内部-rust-api)
   - 6.1 [Plugin Trait 及能力 Trait](#61-plugin-trait-及能力-trait)
   - 6.2 [Registry API](#62-registry-api)
   - 6.3 [ParameterSchema API](#63-parameterschema-api)
   - 6.4 [ParameterSet API](#64-parameterset-api)
   - 6.5 [Engine API](#65-engine-api)
   - 6.6 [Core Types](#66-core-types)
7. [ProgressSink / ProgressReporter Trait](#7-progresssink--progressreporter-trait)
8. [Config File Loading](#8-config-file-loading)
9. [错误代码体系](#9-错误代码体系)
10. [接口交互流程](#10-接口交互流程)
    - 10.1 [序列图](#101-序列图)
    - 10.2 [数据流](#102-数据流)

---

## 1. 概述

Photopipeline 是一个超高精度跨平台图像后处理应用。系统由以下层次组成：

```
┌──────────────────────────────────────────────────────────┐
│  Frontend (任意语言/gRPC客户端)                           │
│  - CLI (Rust)  - GUI (C# WPF/Avalonia)                   │
├──────────────────────────────────────────────────────────┤
│  gRPC Server (Rust/tonic)                                │
│  ├── PluginService    (插件发现/模式查询)                  │
│  ├── ImageService     (图像加载/解码/缩略图/编码)          │
│  ├── PipelineService  (管线创建/执行/验证/节点模式)          │
│  └── BatchService     (批量处理)                          │
├──────────────────────────────────────────────────────────┤
│  Engine (Rust)                                            │
│  ├── PipelineGraph/PipelineTemplate                       │
│  ├── NodeExecutor                                         │
│  ├── ParameterResolver                                    │
│  └── TileEngine                                           │
├──────────────────────────────────────────────────────────┤
│  Plugin System (Rust)                                     │
│  ├── Plugin trait + 能力 traits                           │
│  ├── Registry                                             │
│  ├── ParameterSchema / ParameterSet                       │
│  └── ProgressSink                                         │
├──────────────────────────────────────────────────────────┤
│  Core Types (Rust/photopipeline-core)                     │
│  ├── PixelBuffer / AlignedBuffer                          │
│  ├── ColorSpace / ColorPrimaries / TransferFunction       │
│  ├── Metadata / ExifData / GpsData                        │
│  ├── ImageFormat / PluginCategory / PluginError           │
│  └── HardwareInfo / ProcessingStats                       │
└──────────────────────────────────────────────────────────┘
```

通信方式：
- **后端 ↔ 任意前端**: gRPC (Protocol Buffers)
- **管线配置**: JSON 文件（由前端生成/编辑，后端读取并执行）
- **插件参数模式**: JSON（通过 gRPC 传输，嵌入 `google.protobuf.Struct`）
- **CLI**: 直接调用 Rust 函数，不使用 gRPC

---

## 2. 外部 gRPC API

### 服务总览

| 服务 | RPC | 返回 |
|---|---|---|
| `PluginService` | `ListPlugins(Empty) → PluginCatalogResponse` | 单次 |
| `PluginService` | `GetNodeSchema(PluginIdRequest) → NodeSchemaResponse` | 单次 |
| `ImageService` | `Load(ImagePathRequest) → ImageInfoResponse` | 单次 |
| `ImageService` | `Decode(DecodeRequest) → stream PixelDataChunk` | 流 |
| `ImageService` | `GetThumbnail(ThumbnailRequest) → ImageDataResponse` | 单次 |
| `ImageService` | `Encode(EncodeRequest) → stream EncodeProgress` | 流 |
| `PipelineService` | `CreatePipeline(PipelineSpec) → PipelineId` | 单次 |
| `PipelineService` | `Execute(ExecuteRequest) → stream ExecuteProgress` | 流 |
| `PipelineService` | `Validate(PipelineSpec) → ValidationResult` | 单次 |
| `PipelineService` | `GetNodeSchema(PluginId) → NodeSchema` | 单次 |
| `BatchService` | `SubmitBatch(BatchSpec) → BatchId` | 单次 |
| `BatchService` | `GetProgress(BatchId) → stream BatchProgress` | 流 |
| `BatchService` | `Cancel(BatchId) → Empty` | 单次 |

### 2.1 PluginService

服务定义位于 `proto/` 目录，Package: `photopipeline.plugin`。

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

#### 2.1.1 ListPlugins

**用途**: 获取所有已注册插件的目录，前端用此接口构建插件选择器。

**请求**: `google.protobuf.Empty`（无参数）

**响应**: `PluginCatalogResponse`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `plugins` | `repeated PluginEntry` | 1 | 是 | 已注册插件列表 | 按 category 分组排序 |
| `categories` | `repeated string` | 2 | 是 | 所有存在的类别名称 | 去重，字母序 |

**PluginEntry**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `id` | `string` | 1 | 是 | 插件唯一标识 | 点分隔命名，如 `"bilateral.denoise"` |
| `name` | `string` | 2 | 是 | 插件显示名称 | |
| `version` | `string` | 3 | 是 | 语义化版本 | 格式 `"MAJOR.MINOR.PATCH"` |
| `category` | `string` | 4 | 是 | 插件类别 | 见 PluginCategory 枚举 |
| `description` | `string` | 5 | 是 | 插件描述 | |
| `tags` | `repeated string` | 6 | 否 | 标签列表 | 用于搜索/过滤 |
| `requires_pixel_access` | `bool` | 7 | 是 | 是否需要像素数据 | |
| `requires_network` | `bool` | 8 | 是 | 是否需要网络 | |
| `requires_filesystem` | `bool` | 9 | 是 | 是否需要文件系统 | |
| `min_ram_mb` | `uint64` | 10 | 是 | 最低内存要求 (MB) | |

**示例 JSON**:
```json
{
  "plugins": [
    {
      "id": "bilateral.denoise",
      "name": "Bilateral Denoise",
      "version": "1.2.0",
      "category": "enhance",
      "description": "Edge-preserving bilateral noise reduction",
      "tags": ["denoise", "bilateral"],
      "requires_pixel_access": true,
      "requires_network": false,
      "requires_filesystem": false,
      "min_ram_mb": 512
    }
  ],
  "categories": ["color", "enhance", "format", "input", "metadata", "transform"]
}
```

#### 2.1.2 GetNodeSchema

**用途**: 获取特定插件的参数模式，前端用此接口动态渲染参数面板。

**请求**: `PluginIdRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `id` | `string` | 1 | 是 | 插件唯一标识 |

**响应**: `NodeSchemaResponse`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `plugin_id` | `string` | 1 | 是 | 插件 ID |
| `name` | `string` | 2 | 是 | 显示名称 |
| `version` | `string` | 3 | 是 | 版本号 |
| `category` | `string` | 4 | 是 | 类别 |
| `description` | `string` | 5 | 是 | 描述文本 |
| `parameter_schema` | `google.protobuf.Struct` | 6 | 是 | JSON 序列化的 ParameterSchema（见第 5 节） |
| `gui_schema` | `google.protobuf.Struct` | 7 | 是 | JSON 序列化的 GuiSchema（面板布局定义） |

**错误条件**:
- `NOT_FOUND`: 插件 ID 不存在于 Registry 中

**示例 JSON**:
```json
{
  "plugin_id": "bilateral.denoise",
  "name": "Bilateral Denoise",
  "version": "1.2.0",
  "category": "enhance",
  "description": "Edge-preserving bilateral noise reduction",
  "parameter_schema": {
    "version": 1,
    "sections": [
      {
        "id": "basic",
        "label": "Basic",
        "collapsible": false,
        "default_collapsed": false,
        "fields": [
          {
            "id": "sigma_color",
            "label": "Color Sigma",
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
            "advanced": false
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
    "min_panel_width": 320
  }
}
```

### 2.2 ImageService

服务定义位于 `proto/image.proto`，Package: `photopipeline.image`。

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

#### 2.2.1 Load

**用途**: 加载图像文件的元信息和基本属性，不解码像素数据。

**请求**: `ImagePath`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `path` | `string` | 1 | 是 | 图像文件绝对路径 | 必须存在且可读 |

**响应**: `ImageInfo`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `id` | `string` | 1 | 是 | 唯一 UUID（会话级） |
| `path` | `string` | 2 | 是 | 原始路径 |
| `filename` | `string` | 3 | 是 | 文件名 |
| `format` | `string` | 4 | 是 | 图像格式字符串 |
| `width` | `uint32` | 5 | 是 | 像素宽度 |
| `height` | `uint32` | 6 | 是 | 像素高度 |
| `file_size_bytes` | `uint64` | 7 | 是 | 文件大小（字节） |
| `pixel_format` | `string` | 8 | 是 | 原始像素格式 |
| `color_space` | `string` | 9 | 是 | 颜色空间描述 |
| `metadata` | `MetadataInfo` | 10 | 否 | 摘要元数据 |

**MetadataInfo**:

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `make` | `optional string` | 1 | 否 | 相机制造商 |
| `model` | `optional string` | 2 | 否 | 相机型号 |
| `lens_model` | `optional string` | 3 | 否 | 镜头型号 |
| `date_time_original` | `optional string` | 4 | 否 | 拍摄时间 (ISO 8601) |
| `exposure_time` | `optional string` | 5 | 否 | 曝光时间 (如 "1/125") |
| `f_number` | `optional string` | 6 | 否 | 光圈值 (如 "5.6") |
| `iso` | `optional uint32` | 7 | 否 | ISO 感光度 |
| `focal_length` | `optional string` | 8 | 否 | 焦距 (如 "50mm") |
| `latitude` | `optional double` | 9 | 否 | GPS 纬度 |
| `longitude` | `optional double` | 10 | 否 | GPS 经度 |
| `altitude` | `optional double` | 11 | 否 | GPS 海拔 (米) |

**错误条件**:
- `NOT_FOUND`: 文件路径不存在
- `INTERNAL`: 无法读取或解析文件头

**示例请求**:
```json
{"path": "/home/user/photos/DSC001.ARW"}
```

**示例响应**:
```json
{
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
    "iso": 400,
    "focal_length": "24mm"
  }
}
```

#### 2.2.2 Decode

**用途**: 将图像文件解码为原始像素数据，通过流返回。

**请求**: `DecodeRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `path` | `string` | 1 | 是 | 图像文件路径 | 必须存在 |
| `pixel_format` | `optional string` | 2 | 否 | 目标像素格式 | `"u8"`/`"u16"`/`"f16"`/`"f32"` |
| `max_width` | `optional uint32` | 3 | 否 | 最大宽度 | 超过时等比缩放 |
| `max_height` | `optional uint32` | 4 | 否 | 最大高度 | 超过时等比缩放 |
| `read_metadata` | `bool` | 5 | 是 | 是否同时提取元数据 | |
| `apply_transfer` | `bool` | 6 | 是 | 是否应用传输函数解码 | 线性化 |

**响应流**: `PixelDataChunk`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `offset` | `uint32` | 1 | 是 | 数据偏移（字节） |
| `data` | `bytes` | 2 | 是 | 分块数据 |
| `total_size` | `uint32` | 3 | 是 | 解压后总大小（字节） |
| `is_last` | `bool` | 4 | 是 | 是否为最后一块 |

**分块策略**: 每块大小 = `min(256KB, total_size)`，流按需提供背压。

**错误条件**:
- `NOT_FOUND`: 文件不存在
- `INTERNAL`: 解码失败

#### 2.2.3 Encode

**用途**: 将像素缓冲区编码为指定图像格式并写入文件。

**请求**: `EncodeRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `pixel_data` | `bytes` | 1 | 是 | 原始像素数据 | |
| `width` | `uint32` | 2 | 是 | 图像宽度 | |
| `height` | `uint32` | 3 | 是 | 图像高度 | |
| `layout` | `string` | 4 | 是 | 通道布局 | `"gray"`/`"gray_alpha"`/`"rgb"`/`"rgba"` |
| `pixel_format` | `string` | 5 | 是 | 像素格式 | `"u8"`/`"u16"`/`"u32"`/`"f16"`/`"f32"` |
| `output_path` | `string` | 6 | 是 | 输出文件路径 | |
| `format` | `string` | 7 | 是 | 目标格式 | `"heif"`/`"avif"`/`"jxl"`/`"png"`/`"tiff"`/`"jpeg"`/`"webp"` |
| `quality` | `optional float` | 8 | 否 | 编码质量 | `0.0-100.0`，默认 95.0 |
| `lossless` | `bool` | 9 | 是 | 是否无损 | |
| `bit_depth` | `uint32` | 10 | 是 | 输出位深度 | 默认 10 |
| `chroma_subsampling` | `optional string` | 11 | 否 | 色度子采样 | `"yuv444"`/`"yuv422"`/`"yuv420"` |
| `encoder` | `optional string` | 12 | 否 | 编码器选择 | 如 `"rav1e"` |
| `effort` | `optional uint32` | 13 | 否 | 编码努力级别 | 0-10 |
| `metadata` | `MetadataInfo` | 14 | 是 | 要嵌入的元数据 | |

**响应流**: `EncodeProgress`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `fraction` | `float` | 1 | 是 | 进度分数 `[0.0, 1.0]` |
| `message` | `string` | 2 | 是 | 进度描述 |
| `bytes_written` | `uint64` | 3 | 是 | 已写入字节数 |
| `done` | `bool` | 4 | 是 | 是否完成 |

#### 2.2.4 GetThumbnail

**用途**: 生成图像文件的 JPEG 缩略图。

**请求**: `ThumbnailRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `path` | `string` | 1 | 是 | 图像文件路径 | |
| `max_size` | `uint32` | 2 | 是 | 最大边长（像素） | 默认 256，最小 1 |

**响应**: `ImageData`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `data` | `bytes` | 1 | 是 | JPEG 编码的缩略图数据 |
| `width` | `uint32` | 2 | 是 | 缩略图宽度 |
| `height` | `uint32` | 3 | 是 | 缩略图高度 |
| `format` | `string` | 4 | 是 | 格式，固定为 `"jpeg"` |

### 2.3 PipelineService

服务定义位于 `proto/pipeline.proto`，Package: `photopipeline.pipeline`。

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

#### 2.3.1 CreatePipeline

**用途**: 创建并存储一条管线定义。

**请求**: `PipelineSpec`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `name` | `string` | 1 | 是 | 管线名称 |
| `nodes` | `repeated PipelineNode` | 2 | 是 | 管线节点列表 |
| `edges` | `repeated PipelineEdge` | 3 | 否 | 连接边 |
| `params` | `map<string, google.protobuf.Struct>` | 4 | 否 | 全局参数（按节点 ID） |
| `batch` | `BatchConfig` | 5 | 否 | 批量处理配置 |

**PipelineNode**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `id` | `string` | 1 | 是 | 节点唯一 ID | 管线内唯一 |
| `plugin_id` | `string` | 2 | 是 | 对应的插件 ID | 必须存在于 Registry |
| `label` | `string` | 3 | 否 | 节点显示标签 | 为空则使用 id |
| `enabled` | `bool` | 4 | 是 | 是否启用 | 默认 true |
| `params` | `google.protobuf.Struct` | 5 | 否 | 节点参数覆盖 | |

**PipelineEdge**:

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `from` | `string` | 1 | 是 | 源节点 ID |
| `to` | `string` | 2 | 是 | 目标节点 ID |

**BatchConfig**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|---|
| `parallel` | `int32` | 1 | 是 | 并行度 | 1 |
| `output_pattern` | `string` | 2 | 否 | 输出文件名模式 | `"{name}_out"` |
| `on_conflict` | `string` | 3 | 否 | 冲突处理策略 | `"skip"` |
| `resume` | `bool` | 4 | 是 | 是否支持断点续传 | false |

**响应**: `PipelineId`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `id` | `string` | 1 | UUID v4 格式的管线 ID |

**错误条件**:
- `INVALID_ARGUMENT`: 管线验证失败（空节点、无效边、插件不存在）

#### 2.3.2 Execute

**用途**: 执行已创建的管线，通过流返回进度。

**请求**: `ExecuteRequest`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `pipeline_id` | `string` | 1 | 是 | CreatePipeline 返回的 ID |
| `image_path` | `string` | 2 | 是 | 输入图像路径 |
| `output_path` | `string` | 3 | 是 | 输出文件路径（可为空） |

**响应流**: `ExecuteProgress`

| 字段 | 类型 | 编号 | 必填 | 描述 |
|---|---|---|---|---|
| `stage` | `Stage` (enum) | 1 | 是 | 执行阶段 |
| `node_id` | `string` | 2 | 否 | 当前节点 ID |
| `node_label` | `string` | 3 | 否 | 当前节点标签 |
| `fraction` | `float` | 4 | 是 | 进度分数 `[0.0, 1.0]` |
| `message` | `string` | 5 | 是 | 进度消息 |
| `elapsed_ms` | `int64` | 6 | 是 | 已用毫秒 |

**Stage 枚举**:

| 值 | 名称 | 描述 |
|---|---|---|
| 0 | `LOADING` | 正在加载输入图像 |
| 1 | `DECODING` | 正在解码图像数据 |
| 2 | `PROCESSING` | 正在执行节点 |
| 3 | `ENCODING` | 正在编码输出 |
| 4 | `DONE` | 完成 |
| 5 | `ERROR` | 出错 |

#### 2.3.3 Validate

**用途**: 验证管线定义，不执行。

**请求**: `PipelineSpec`（结构同 2.3.1）

**响应**: `ValidationResult`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `valid` | `bool` | 1 | 是否存在 Error 级别的验证问题 |
| `issues` | `repeated ValidationIssue` | 2 | 验证问题列表 |

**ValidationIssue**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `severity` | `Severity` (enum) | 1 | 严重程度 |
| `param` | `string` | 2 | 关联参数路径 |
| `message` | `string` | 3 | 问题描述 |

**Severity 枚举**:

| 值 | 名称 | 描述 |
|---|---|---|
| 0 | `INFO` | 信息 |
| 1 | `WARNING` | 警告（不阻止执行） |
| 2 | `ERROR` | 错误（阻止执行） |

#### 2.3.4 GetNodeSchema (在 PipelineService 中)

与 PluginService.GetNodeSchema 功能相同，但在 PipelineService 中实现。返回特定插件的参数模式和 GUI 布局模式。

**请求**: `PluginId`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `id` | `string` | 1 | 插件唯一标识 |

**响应**: `NodeSchema`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `plugin_id` | `string` | 1 | 插件 ID |
| `name` | `string` | 2 | 显示名称 |
| `version` | `string` | 3 | 版本 |
| `category` | `string` | 4 | 类别 |
| `description` | `string` | 5 | 描述 |
| `parameter_schema` | `google.protobuf.Struct` | 6 | ParameterSchema JSON |
| `gui_schema` | `google.protobuf.Struct` | 7 | GuiSchema JSON |

### 2.4 BatchService

服务定义位于 `proto/batch.proto`，Package: `photopipeline.batch`。

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

#### 2.4.1 SubmitBatch

**用途**: 提交批量处理任务。

**请求**: `BatchSpec`

| 字段 | 类型 | 编号 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|---|
| `pipeline_config_path` | `string` | 1 | 是 | 管线配置 TOML/JSON 文件路径 | |
| `file_pattern` | `string` | 2 | 是 | 输入文件匹配模式 | `"*.*"` |
| `output_dir` | `string` | 3 | 是 | 输出目录 | `"."` |
| `parallel` | `int32` | 4 | 是 | 并行度 | 1 |
| `resume` | `bool` | 5 | 是 | 断点续传 | false |

**响应**: `BatchId`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `id` | `string` | 1 | 批次 UUID |

#### 2.4.2 GetProgress

**请求**: `BatchId`

**响应流**: `BatchProgress`

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `status` | `Status` (enum) | 1 | 批次状态 |
| `total_files` | `int32` | 2 | 总文件数 |
| `completed_files` | `int32` | 3 | 已完成文件数 |
| `failed_files` | `int32` | 4 | 失败文件数 |
| `current_file` | `string` | 5 | 当前处理的文件 |
| `fraction` | `float` | 6 | 进度分数 |
| `progress_details` | `string` | 7 | 进度详情文本 |

**Status 枚举**:

| 值 | 名称 | 描述 |
|---|---|---|
| 0 | `PENDING` | 待处理 |
| 1 | `RUNNING` | 运行中 |
| 2 | `DONE` | 完成 |
| 3 | `CANCELED` | 已取消 |
| 4 | `ERROR` | 出错 |

#### 2.4.3 Cancel

**请求**: `BatchId`

**响应**: `google.protobuf.Empty`

**错误条件**:
- `NOT_FOUND`: 批次 ID 不存在

### 2.5 ExecutionService（v2 设计）

以下为规划中的 v2 执行服务，合并并简化现有 PipelineService.Execute 和 BatchService。

```protobuf
syntax = "proto3";
package photopipeline.execution;

service ExecutionService {
  rpc Run(RunRequest) returns (stream RunEvent);
  rpc Cancel(CancelRequest) returns (google.protobuf.Empty);
}
```

#### 2.5.1 Run

**RunRequest**:

| 字段 | 类型 | 编号 | 必填 | 描述 | 约束 |
|---|---|---|---|---|---|
| `config_path` | `string` | 1 | 是 | 管线配置 JSON 文件路径 | 必须存在 |
| `image_paths` | `repeated string` | 2 | 是 | 输入图像路径列表 | 单张或批量 |
| `output_dir` | `string` | 3 | 是 | 输出目录 | |
| `output_pattern` | `string` | 4 | 否 | 输出文件名模式 | 默认 `"{name}_processed.{ext}"` |
| `filter` | `string` | 5 | 否 | 节点执行过滤器 | 正则，只执行匹配名称的节点 |
| `metrics` | `bool` | 6 | 否 | 是否上报指标数据 | 默认 false |

**RunEvent (oneof)**:

| 编号 | 变体 | 描述 |
|---|---|---|
| 1 | `ProgressUpdate` | 节点进度更新 |
| 2 | `MetricSnapshot` | 资源使用指标（仅当 metrics=true） |
| 3 | `StageTransition` | 阶段转换事件 |
| 4 | `ErrorEvent` | 错误事件 |
| 5 | `DoneEvent` | 完成事件 |
| 6 | `Heartbeat` | 心跳（每 5 秒） |

**ProgressUpdate**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `node_id` | `string` | 1 | 当前节点 ID |
| `node_label` | `string` | 2 | 当前节点标签 |
| `fraction` | `float` | 3 | 进度分数 `[0.0, 1.0]` |
| `message` | `string` | 4 | 进度描述 |
| `elapsed_ms` | `int64` | 5 | 阶段已用毫秒 |

**MetricSnapshot**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `cpu_percent` | `float` | 1 | CPU 使用率 (%) |
| `memory_used_mb` | `uint64` | 2 | 已用内存 (MB) |
| `memory_total_mb` | `uint64` | 3 | 系统总内存 (MB) |
| `gpu_metrics` | `GpuMetrics` | 4 | GPU 指标（可选） |
| `elapsed_seconds` | `float` | 5 | 总用时（秒） |
| `bytes_processed` | `uint64` | 6 | 已处理字节数 |
| `throughput_mbps` | `float` | 7 | 吞吐量 (MB/s) |

**GpuMetrics**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `gpu_utilization` | `float` | 1 | GPU 利用率 (%) |
| `gpu_memory_used_mb` | `uint64` | 2 | GPU 已用内存 (MB) |
| `gpu_memory_total_mb` | `uint64` | 3 | GPU 总内存 (MB) |
| `temperature_celsius` | `float` | 4 | GPU 温度 |

**StageTransition**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `stage` | `Stage` (enum) | 1 | 阶段 ID |
| `previous_stage` | `Stage` (enum) | 2 | 前一阶段 |
| `timestamp_ms` | `int64` | 3 | 时间戳 |

**Stage 枚举**:

| 值 | 名称 |
|---|---|
| 0 | `LOADING` |
| 1 | `DECODING` |
| 2 | `PROCESSING` |
| 3 | `ENCODING` |
| 4 | `DONE` |
| 5 | `ERROR` |

**ErrorEvent**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `code` | `string` | 1 | 错误码 |
| `message` | `string` | 2 | 错误消息 |
| `node_id` | `optional string` | 3 | 关联节点（可选） |
| `details` | `google.protobuf.Struct` | 4 | 附加上下文 |

**DoneEvent**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `output_paths` | `repeated string` | 1 | 输出文件路径列表 |
| `total_bytes` | `uint64` | 2 | 总输出字节数 |
| `total_seconds` | `float` | 3 | 总用时（秒） |
| `node_stats` | `repeated NodeResult` | 4 | 各节点统计数据 |

**NodeResult**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `node_id` | `string` | 1 | 节点 ID |
| `node_label` | `string` | 2 | 节点标签 |
| `status` | `string` | 3 | `"completed"`/`"failed"`/`"skipped"` |
| `elapsed_ms` | `uint64` | 4 | 执行用时 (ms) |
| `cpu_time_ms` | `uint64` | 5 | CPU 时间 (ms) |
| `gpu_time_ms` | `optional uint64` | 6 | GPU 时间 (ms) |
| `peak_memory_mb` | `uint64` | 7 | 峰值内存 (MB) |
| `input_pixels` | `uint64` | 8 | 输入像素数 |
| `output_pixels` | `uint64` | 9 | 输出像素数 |

#### 2.5.2 Cancel

**CancelRequest**:

| 字段 | 类型 | 编号 | 描述 |
|---|---|---|---|
| `pipeline_id` | `string` | 1 | 要取消的管线 ID |

**响应**: `google.protobuf.Empty`

---

## 3. CLI 接口

CLI 使用 `clap` derive 模式定义，入口文件 `cli/src/main.rs`。

### 3.1 命令体系

```
photopipeline <SUBCOMMAND>

SUBCOMMANDS:
  pipeline    Pipeline commands
  plugin      Plugin management commands
  batch       Batch processing commands
  help        Print help information
```

### 3.2 Pipeline 子命令

#### `pipeline run`

```
photopipeline pipeline run -c <CONFIG> -i <INPUT> -o <OUTPUT>
```

| 参数 | 短标志 | 长标志 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|---|
| `config` | `-c` | `--config` | 是 | TOML 管线配置文件路径 | |
| `input` | `-i` | `--input` | 是 | 输入图像文件路径 | |
| `output` | `-o` | `--output` | 是 | 输出文件路径 | |

**退出码**:

| 退出码 | 含义 |
|---|---|
| 0 | 成功 |
| 1 | 管线执行失败 |
| 2 | 文件未找到 |
| 3 | 配置无效 |

**stdout**: 进度信息（tracing 日志输出）
**stderr**: 错误详情

#### `pipeline validate`

```
photopipeline pipeline validate -c <CONFIG>
```

| 参数 | 短标志 | 长标志 | 必填 | 描述 |
|---|---|---|---|---|
| `config` | `-c` | `--config` | 是 | TOML 管线配置文件路径 |

**退出码**: 0 = 有效, 1 = 无效

### 3.3 Plugin 子命令

#### `plugin list`

```
photopipeline plugin list
```

无参数。

**stdout**: 列出所有已注册插件（ID、名称、版本、类别）

**退出码**: 0

#### `plugin info <PLUGIN_ID>`

```
photopipeline plugin info <PLUGIN_ID>
```

| 参数 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `plugin_id` | positional | 是 | 插件唯一标识 |

**stdout**: 插件详细信息（ID、名称、版本、类别、描述、参数模式、GUI 模式）

**退出码**: 0 = 找到, 1 = 未找到

### 3.4 Batch 子命令

#### `batch run`

```
photopipeline batch run -c <CONFIG> -p <PATTERN> -o <OUTPUT>
```

| 参数 | 短标志 | 长标志 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|---|
| `config` | `-c` | `--config` | 是 | TOML 管线配置文件路径 | |
| `pattern` | `-p` | `--pattern` | 是 | 输入文件 glob 模式 | `"*.ARW"` |
| `output` | `-o` | `--output` | 是 | 输出目录 | `"./output/"` |

#### `batch validate`

```
photopipeline batch validate -c <CONFIG> -p <PATTERN>
```

| 参数 | 短标志 | 长标志 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|---|
| `config` | `-c` | `--config` | 是 | TOML 管线配置文件路径 | |
| `pattern` | `-p` | `--pattern` | 是 | 输入文件 glob 模式 | `"*.ARW"` |

---

## 4. PipelineConfig 文件格式

管线配置使用 JSON（主格式）或 TOML 文件。

### 4.1 顶层结构

```json
{
  "name": "string (必填)",
  "version": "string (可选)",
  "description": "string (可选)",
  "pipelines": [ ... ],
  "images": [ ... ],
  "output": { ... },
  "groups": [ ... ],
  "execution": { ... }
}
```

### 4.2 字段详解

#### PipelineConfig

| 字段 | 类型 | 必填 | 描述 | 约束 |
|---|---|---|---|---|
| `name` | `string` | 是 | 配置方案名称 | 非空 |
| `version` | `string` | 否 | 语义化版本 | 如 `"1.0"` |
| `description` | `string` | 否 | 描述文本 | |
| `pipelines` | `PipelineTemplate[]` | 是 | 管线定义数组 | 至少 1 个 |

PipelineTemplate 对应 `PipelineTemplate` Rust 结构体 (`crates/engine/src/graph.rs`):

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `metadata` | `TemplateMetadata` | 否 | 模板元信息 | `{}` |
| `nodes` | `TemplateNode[]` | 是 | 节点列表 | |
| `edges` | `TemplateEdge[]` | 否 | 连接边 | `[]` |
| `overrides` | `ImageOverride[]` | 否 | 图像级参数覆盖 | `[]` |
| `groups` | `ParamGroup[]` | 否 | 条件参数组 | `[]` |
| `batch` | `BatchConfig` | 否 | 批量处理配置 | |

**TemplateMetadata**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `name` | `string` | 否 | 管线名称 |
| `version` | `string` | 否 | 管线版本 |
| `description` | `string` | 否 | 管线描述 |

**TemplateNode**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `id` | `string` | 是 | 节点唯一 ID | |
| `plugin` | `string` | 是 | 插件 ID | |
| `label` | `string` | 否 | 显示标签 | 使用 `id` |
| `enabled` | `bool` | 否 | 是否启用 | `true` |
| `params` | `object` | 否 | 参数覆盖 | `null` |

**TemplateEdge**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `from` | `string` | 是 | 源节点 ID |
| `to` | `string` | 是 | 目标节点 ID |

**ImageOverride**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `image` | `string` | 是 | 图像名称/路径 |
| `params` | `map<string, object>` | 否 | 按节点 ID 的参数覆盖 |

**ParamGroup**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `name` | `string` | 是 | 组名称 |
| `condition` | `string` | 是 | 条件表达式字符串 |
| `params` | `map<string, object>` | 否 | 按节点 ID 的参数 |

**条件表达式语法** (`condition` 字段):

```
// 等值比较
exif.make == "Canon"

// 数值比较
exif.iso >= 800
exif.focal_length <= 200

// GPS 范围
gps.near(lat: 34.05, lon: -118.24, radius_km: 10)

// 逻辑组合
(exif.iso >= 800) and (exif.make == "Sony")
(exif.iso >= 800) or (exif.focal_length <= 24)

// 三元表达式
exif.iso >= 800 ? "high" : "low"

// 图片属性引用  
image.width > 4000
```

**BatchConfig**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `parallel` | `int` | 否 | 并行文件数 | 1 |
| `output_pattern` | `string` | 否 | 输出文件名模式 | |
| `on_conflict` | `string` | 否 | 冲突处理 | |
| `resume` | `bool` | 否 | 断点续传 | false |

### 4.3 完整示例

```json
{
  "name": "Raw to HEIF Pipeline",
  "version": "1.0",
  "description": "Convert RAW files to 10-bit HEIF with denoising",
  "pipelines": [
    {
      "metadata": {
        "name": "RAW→HEIF",
        "version": "2.1",
        "description": "Standard raw development pipeline"
      },
      "nodes": [
        {
          "id": "raw_decoder",
          "plugin": "raw.decoder",
          "label": "RAW Decode",
          "enabled": true,
          "params": {
            "demosaic": "AMaZE",
            "white_balance": "camera"
          }
        },
        {
          "id": "denoise",
          "plugin": "bilateral.denoise",
          "label": "Denoise",
          "enabled": true,
          "params": {
            "sigma_color": 10.0,
            "sigma_spatial": 3.0
          }
        },
        {
          "id": "sharpen",
          "plugin": "unsharp.mask",
          "label": "Sharpen",
          "enabled": true,
          "params": {
            "amount": 1.2,
            "radius": 1.0,
            "threshold": 0
          }
        },
        {
          "id": "colorspace",
          "plugin": "colorspace.convert",
          "label": "Convert to Display P3",
          "enabled": true,
          "params": {
            "target": "display_p3",
            "intent": "perceptual"
          }
        },
        {
          "id": "encoder",
          "plugin": "heif.encoder",
          "label": "HEIF Encode",
          "enabled": true,
          "params": {
            "quality": 95.0,
            "bit_depth": 10,
            "chroma_subsampling": "yuv444"
          }
        }
      ],
      "edges": [
        {"from": "raw_decoder", "to": "denoise"},
        {"from": "denoise", "to": "sharpen"},
        {"from": "sharpen", "to": "colorspace"},
        {"from": "colorspace", "to": "encoder"}
      ],
      "groups": [
        {
          "name": "High ISO",
          "condition": "exif.iso >= 800",
          "params": {
            "denoise": {
              "sigma_color": 25.0,
              "sigma_spatial": 5.0
            }
          }
        },
        {
          "name": "Low ISO",
          "condition": "exif.iso < 800",
          "params": {
            "denoise": {
              "sigma_color": 5.0,
              "sigma_spatial": 1.0
            }
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
  "execution": {
    "tile_threshold_pixels": 8847360,
    "max_ram_mb": 16384,
    "timeout_seconds": 3600
  }
}
```

---

## 5. ParameterSchema 格式

`ParameterSchema` 定义插件的用户可配置参数。通过 gRPC 以 JSON 格式传输，前端据此动态渲染 UI。

### 5.1 顶层结构

```json
{
  "version": 1,
  "sections": [ ParameterSection, ... ]
}
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `version` | `u32` | Schema 格式版本，当前为 1 |
| `sections` | `ParameterSection[]` | 参数分组列表 |

### 5.2 ParameterSection

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

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `id` | `string` | 是 | 节唯一 ID | |
| `label` | `string` | 是 | 节显示标签 | |
| `description` | `string` | 否 | 节描述 | `null` |
| `icon` | `string` | 否 | 图标名称 | `null` |
| `collapsible` | `bool` | 是 | 是否可折叠 | |
| `default_collapsed` | `bool` | 是 | 默认折叠状态 | |
| `fields` | `ParameterField[]` | 是 | 字段列表 | |

### 5.3 ParameterField

```json
{
  "id": "brightness",
  "label": "Brightness",
  "description": "Adjust image brightness",
  "help_url": "https://docs.example.com/brightness",
  "type": "float",
  "min": -1.0,
  "max": 1.0,
  "step": 0.01,
  "precision": 2,
  "unit": "EV",
  "logarithmic": false,
  "style": "slider",
  "default": 0.0,
  "required": false,
  "advanced": false,
  "allow_override": true,
  "supports_expression": false
}
```

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `id` | `string` | 是 | 字段唯一 ID | |
| `label` | `string` | 是 | 字段显示标签 | |
| `description` | `string` | 否 | 字段描述文本 | `null` |
| `help_url` | `string` | 否 | 帮助文档链接 | `null` |
| `type` | `string` (flatten) | 是 | 值类型标签（见 5.4） | |
| `...` | (类型特定字段) | | 见 5.4 各类型定义 | |
| `default` | `any` | 是 | 默认值 | |
| `required` | `bool` | 是 | 是否必填 | |
| `advanced` | `bool` | 是 | 是否为高级参数 | |
| `allow_override` | `bool` | 是 | 是否允许外部覆盖 | `true` |
| `supports_expression` | `bool` | 是 | 是否支持表达式 | `false` |

### 5.4 ParameterType 类型全集（21 种）

#### 5.4.1 string

序列化标签: `"string"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `max_length` | `usize` | 是 | 最大字符数 | |
| `pattern` | `string` | 否 | 正则验证模式 | `null` |
| `placeholder` | `string` | 否 | 占位文本 | `null` |

```json
{
  "type": "string",
  "max_length": 256,
  "pattern": "[a-zA-Z0-9_]+",
  "placeholder": "Enter name"
}
```

#### 5.4.2 integer

序列化标签: `"integer"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `min` | `i64` | 是 | 最小值 | |
| `max` | `i64` | 是 | 最大值 | |
| `step` | `i64` | 是 | 步进值 | |
| `unit` | `string` | 否 | 单位标签 | `null` |
| `style` | `IntegerWidget` | 否 | 控件样式 | `"spin_box"` |

**IntegerWidget 枚举**: `"spin_box"`, `"slider"`, `"combo"`

```json
{
  "type": "integer",
  "min": 0,
  "max": 255,
  "step": 1,
  "unit": "px",
  "style": "slider"
}
```

#### 5.4.3 float

序列化标签: `"float"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `min` | `f64` | 是 | 最小值 | |
| `max` | `f64` | 是 | 最大值 | |
| `step` | `f64` | 是 | 步进值 | |
| `precision` | `u8` | 是 | 小数位数 | |
| `unit` | `string` | 否 | 单位标签 | `null` |
| `logarithmic` | `bool` | 否 | 是否对数刻度 | `false` |
| `style` | `FloatWidget` | 否 | 控件样式 | `"spin_box"` |

**FloatWidget 枚举**: `"spin_box"`, `"slider"`, `"combo_slider"`, `"drag_input"`

```json
{
  "type": "float",
  "min": 0.0,
  "max": 100.0,
  "step": 0.1,
  "precision": 2,
  "unit": "%",
  "logarithmic": false,
  "style": "slider"
}
```

#### 5.4.4 boolean

序列化标签: `"boolean"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `label_true` | `string` | 否 | 真值标签 | `null` |
| `label_false` | `string` | 否 | 假值标签 | `null` |

```json
{
  "type": "boolean",
  "label_true": "On",
  "label_false": "Off"
}
```

#### 5.4.5 enum

序列化标签: `"enum"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `options` | `EnumOption[]` | 是 | 选项列表 | |
| `display` | `EnumDisplay` | 否 | 显示样式 | `"dropdown"` |

**EnumDisplay 枚举**: `"dropdown"`, `"radio_group"`, `"button_group"`, `"segmented_control"`, `"tabs"`, `"popup_card"`

**EnumOption**:

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `value` | `string` | 是 | 选项值 | |
| `label` | `string` | 是 | 选项标签 | |
| `description` | `string` | 否 | 选项描述 | `null` |
| `icon` | `string` | 否 | 选项图标 | `null` |
| `tags` | `string[]` | 是 | 标签 | `[]` |
| `recommended` | `bool` | 否 | 是否推荐 | `false` |

```json
{
  "type": "enum",
  "options": [
    {"value": "amaze", "label": "AMaZE", "tags": ["quality"], "recommended": true},
    {"value": "lmmse", "label": "LMMSE", "tags": ["speed"], "recommended": false},
    {"value": "vng4", "label": "VNG4", "tags": ["balanced"], "recommended": false}
  ],
  "display": "dropdown"
}
```

#### 5.4.6 color

序列化标签: `"color"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `mode` | `ColorMode` | 否 | 颜色模式 | `"RGB"` |
| `show_alpha` | `bool` | 否 | 是否显示 Alpha 通道 | `false` |

**ColorMode 枚举**: `"RGB"`, `"RGBA"`, `"HSL"`, `"HSV"`, `"Lab"`

```json
{
  "type": "color",
  "mode": "RGB",
  "show_alpha": true
}
```

#### 5.4.7 file_path

序列化标签: `"file_path"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `kind` | `FilePathKind` | 否 | 路径类型 | `"file"` |
| `filters` | `[string, string][]` | 否 | 文件类型过滤器 | `[]` |
| `must_exist` | `bool` | 否 | 是否必须存在 | `false` |

**FilePathKind 枚举**: `"file"`, `"directory"`, `"save_file"`

```json
{
  "type": "file_path",
  "kind": "file",
  "filters": [["LUT Files", "*.cube"], ["All Files", "*.*"]],
  "must_exist": true
}
```

#### 5.4.8 coordinate

序列化标签: `"coordinate"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `alt_required` | `bool` | 否 | 高度是否必填 | `false` |
| `direction_required` | `bool` | 否 | 方向是否必填 | `false` |

```json
{
  "type": "coordinate",
  "alt_required": true,
  "direction_required": false
}
```

#### 5.4.9 slider

序列化标签: `"slider"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `min` | `f64` | 是 | 最小值 | |
| `max` | `f64` | 是 | 最大值 | |
| `step` | `f64` | 否 | 步进值 | 1.0 |
| `show_ticks` | `bool` | 否 | 是否显示刻度 | `false` |
| `ticks` | `f64[]` | 否 | 自定义刻度位置 | `null` |
| `show_value` | `bool` | 否 | 是否显示数值 | `true` |
| `orientation` | `SliderOrientation` | 否 | 方向 | `"horizontal"` |
| `style` | `SliderStyle` | 否 | 样式 | `"continuous"` |

**SliderOrientation**: `"horizontal"`, `"vertical"`
**SliderStyle**: `"continuous"`, `"discrete"`, `"range"`, `"dual_handle"`

#### 5.4.10 combo_slider

序列化标签: `"combo_slider"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `min` | `f64` | 是 | 最小值 | |
| `max` | `f64` | 是 | 最大值 | |
| `step` | `f64` | 否 | 步进值 | 1.0 |
| `presets` | `[string, f64][]` | 是 | 预设值列表 | |
| `unit` | `string` | 否 | 单位 | `null` |

```json
{
  "type": "combo_slider",
  "min": 0.0,
  "max": 10.0,
  "step": 0.5,
  "presets": [["Low", 1.0], ["Medium", 5.0], ["High", 9.0]],
  "unit": "dB"
}
```

#### 5.4.11 expression

序列化标签: `"expression"`

| 额外字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `variables` | `VariableDef[]` | 是 | 可用变量列表 |

**VariableDef**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `name` | `string` | 是 | 变量名 |
| `description` | `string` | 是 | 变量描述 |
| `var_type` | `string` | 是 | 变量类型（`"number"`, `"string"` 等） |
| `example` | `string` | 否 | 示例值 |

```json
{
  "type": "expression",
  "variables": [
    {"name": "iso", "description": "ISO value", "var_type": "number", "example": "400"},
    {"name": "focal_length", "description": "Focal length in mm", "var_type": "number", "example": "50"}
  ]
}
```

#### 5.4.12 preset

序列化标签: `"preset"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `preset_schema_ref` | `string` | 是 | 预设模式引用 | |
| `builtin_presets` | `NamedPreset[]` | 是 | 内置预设列表 | |
| `allow_custom` | `bool` | 否 | 是否允许自定义 | `false` |
| `allow_import` | `bool` | 否 | 是否允许导入 | `false` |

**NamedPreset**:

| 字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `name` | `string` | 是 | 预设名称 |
| `description` | `string` | 否 | 描述 |
| `params` | `map<string, any>` | 是 | 参数值 |

#### 5.4.13 array

序列化标签: `"array"`

| 额外字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `element` | `ParameterField` | 是 | 元素字段定义 |
| `min_items` | `usize` | 是 | 最小项数 |
| `max_items` | `usize` | 否 | 最大项数 |

#### 5.4.14 map_widget

序列化标签: `"map_widget"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `show_track` | `bool` | 否 | 是否显示轨迹 | `false` |
| `show_photos` | `bool` | 否 | 是否显示照片标记 | `false` |
| `allow_manual_pin` | `bool` | 否 | 是否允许手动标记 | `false` |

#### 5.4.15 before_after

序列化标签: `"before_after"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `zoom_levels` | `f64[]` | 是 | 可用缩放级别 | |
| `show_histogram` | `bool` | 否 | 是否显示直方图 | `false` |

#### 5.4.16 separator

序列化标签: `"separator"`

| 额外字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `label` | `string` | 否 | 分隔线标签 | `null` |

#### 5.4.17 section (嵌套)

序列化标签: `"section"`

| 额外字段 | 类型 | 必填 | 描述 |
|---|---|---|---|
| `fields` | `ParameterField[]` | 是 | 嵌套字段列表 |

### 5.5 GuiSchema

GuiSchema 定义前端面板布局。

```json
{
  "layout": { ... },
  "icon": "denoise",
  "color": "#4A90D9",
  "preview": "live",
  "aux_views": ["histogram", "waveform"],
  "min_panel_width": 320
}
```

| 字段 | 类型 | 必填 | 描述 | 默认值 |
|---|---|---|---|---|
| `layout` | `GuiLayout` | 是 | 面板布局 | |
| `icon` | `string` | 否 | 面板图标 | `null` |
| `color` | `string` | 否 | 主题色 | `null` |
| `preview` | `PreviewMode` | 是 | 预览模式 | `"none"` |
| `aux_views` | `AuxView[]` | 是 | 辅助视图 | `[]` |
| `min_panel_width` | `u32` | 是 | 最小面板宽度 | 320 |

**GuiLayout** (enum):

```json
// 模式1: 标准布局
{"Standard": {
  "sections": [
    {"param_section_id": "basic", "title_visible": true, "style": "card"}
  ]
}}

// 模式2: 自定义行布局
{"Custom": {
  "rows": [
    {
      "height": "normal",
      "cells": [
        {"param_field_id": "brightness", "width_fraction": 0.5, "label_position": "top"}
      ]
    }
  ]
}}
```

**PreviewMode 枚举**:
- `"none"` — 无预览
- `"live"` — 实时预览
- `"manual_refresh"` — 手动刷新
- `BeforeAfter { default_split, orientation, lock_zoom }` — 分割对比
- `Tiled { rows, cols }` — 平铺对比

**AuxView 枚举**: `histogram`, `waveform`, `vectorscope`, `gamut_diagram`, `map`, `focus_peaking`, `clipping_warning`, `metadata_table`, `progress_bar`, `status_text`

---

## 6. 内部 Rust API

### 6.1 Plugin Trait 及能力 Trait

位置: `crates/plugin/src/trait_def.rs`

#### 6.1.1 Plugin（基础 Trait）

所有插件必须实现的基础 trait。

```rust
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &PluginId;
    fn name(&self) -> &str;
    fn version(&self) -> PluginVersion;
    fn category(&self) -> PluginCategory;
    fn description(&self) -> &str;
    fn tags(&self) -> &[String];
    fn requires_pixel_access(&self) -> bool;
    fn produces_pixel_output(&self) -> bool { false }
    fn supported_hardware(&self) -> HardwareRequirement;
    fn parameter_schema(&self) -> &ParameterSchema;
    fn gui_schema(&self) -> &GuiSchema;

    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self) -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>>;
}
```

| 方法 | 参数 | 返回 | 描述 |
|---|---|---|---|
| `id()` | 无 | `&PluginId` | 唯一标识符 |
| `name()` | 无 | `&str` | 显示名称 |
| `version()` | 无 | `PluginVersion` | 版本号 |
| `category()` | 无 | `PluginCategory` | 类别 |
| `description()` | 无 | `&str` | 描述文本 |
| `tags()` | 无 | `&[String]` | 标签列表 |
| `requires_pixel_access()` | 无 | `bool` | 是否需要像素数据 |
| `produces_pixel_output()` | 无 | `bool` | 是否产生新像素输出；默认 `false` |
| `supported_hardware()` | 无 | `HardwareRequirement` | 硬件需求 |
| `parameter_schema()` | 无 | `&ParameterSchema` | 参数模式定义 |
| `gui_schema()` | 无 | `&GuiSchema` | GUI 布局定义 |
| `initialize()` | `cfg: &PluginConfig` | `PluginResult<()>` | 初始化插件 |
| `shutdown()` | 无 | `PluginResult<()>` | 关闭插件 |
| `validate()` | `params: &ParameterSet` | `PluginResult<Vec<ValidationIssue>>` | 验证参数 |

#### 6.1.2 ProgressSink Trait

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

| 方法 | 参数 | 返回 | 描述 |
|---|---|---|---|
| `set_progress()` | `fraction: f32`, `message: &str` | 无 | 报告进度 |
| `is_canceled()` | 无 | `bool` | 检查是否取消 |

#### 6.1.3 MetadataProcessor Trait

```rust
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;

    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet)
        -> PluginResult<Metadata>;

    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata,
        params: &ParameterSet) -> PluginResult<MetadataWriteReport>;
}
```

| 方法 | 参数 | 返回 | 描述 |
|---|---|---|---|
| `metadata_scope()` | 无 | `Vec<MetadataScope>` | 读写范围 (EXIF/XMP/IPTC/GPS/All) |
| `read_metadata()` | `target: &MetadataTarget`, `params: &ParameterSet` | `PluginResult<Metadata>` | 读取元数据 |
| `write_metadata()` | `target: &mut MetadataTarget`, `metadata: &Metadata`, `params: &ParameterSet` | `PluginResult<MetadataWriteReport>` | 写入元数据 |

#### 6.1.4 PixelProcessor Trait

```rust
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self) -> Vec<ColorSpace>;
    fn required_gpu_backend(&self) -> Option<GpuBackend>;

    async fn process_pixels(&self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>)
        -> PluginResult<ProcessingStats>;
}
```

| 方法 | 参数 | 返回 | 描述 |
|---|---|---|---|
| `supported_input_formats()` | 无 | `Vec<PixelFormat>` | 支持的输入像素格式 |
| `supported_output_formats()` | 无 | `Vec<PixelFormat>` | 支持的输出像素格式 |
| `supported_color_spaces()` | 无 | `Vec<ColorSpace>` | 支持的颜色空间 |
| `required_gpu_backend()` | 无 | `Option<GpuBackend>` | 所需 GPU 后端 |
| `process_pixels()` | `input: &PixelBuffer`, `output: &mut PixelBuffer`, `params: &ParameterSet`, `progress: Box<dyn ProgressSink>` | `PluginResult<ProcessingStats>` | 执行像素处理 |

#### 6.1.5 FormatProcessor Trait

```rust
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn format_id(&self) -> ImageFormat;

    fn can_decode(&self, data: &FormatProbe) -> bool;
    async fn decode(&self, data: &[u8], options: &DecodeOptions)
        -> PluginResult<DecodedImage>;

    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata,
        options: &EncodeOptions) -> PluginResult<Vec<u8>>;
}
```

| 方法 | 参数 | 返回 | 描述 |
|---|---|---|---|
| `supported_extensions()` | 无 | `Vec<(&str, &str)>` | 支持的文件扩展名列表 |
| `format_id()` | 无 | `ImageFormat` | 格式标识 |
| `can_decode()` | `data: &FormatProbe` | `bool` | 是否能解码 |
| `decode()` | `data: &[u8]`, `options: &DecodeOptions` | `PluginResult<DecodedImage>` | 解码 |
| `can_encode()` | `format: &ImageFormat` | `bool` | 是否能编码 |
| `encode()` | `image: &PixelBuffer`, `metadata: &Metadata`, `options: &EncodeOptions` | `PluginResult<Vec<u8>>` | 编码为字节数组 |

#### 6.1.6 GpuProcessor Trait（v2 预留）

```rust
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;

    async fn process_gpu(&self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>)
        -> PluginResult<ProcessingStats>;
}
```

#### 6.1.7 AiProcessor Trait

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

**ModelInfo**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `name` | `String` | 模型名称 |
| `version` | `String` | 模型版本 |
| `source` | `ModelSource` | 模型来源 |
| `input_shape` | `Vec<usize>` | 输入张量形状 |
| `output_shape` | `Vec<usize>` | 输出张量形状 |
| `memory_mb` | `u64` | 内存需求 (MB) |
| `description` | `String` | 描述 |

**ModelSource 枚举**:
- `Bundled` — 内置于插件
- `ExternalFile(String)` — 外部文件路径
- `HuggingFace { repo, file }` — HuggingFace 仓库
- `Url(String)` — URL 下载

#### 6.1.8 ExternalToolProcessor Trait（v2 预留）

```rust
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;

    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(&self, input_paths: &[PathBuf], output_path: &Path,
        params: &ParameterSet) -> PluginResult<()>;
}
```

### 6.2 Registry API

位置: `crates/plugin/src/registry.rs`

```rust
pub struct Registry {
    entries: DashMap<PluginId, RegistryEntry>,
    manifests: DashMap<PluginId, PluginManifest>,
    load_order: RwLock<Vec<PluginId>>,
    metadata_processors: DashMap<PluginId, Arc<dyn MetadataProcessor>>,
    pixel_processors: DashMap<PluginId, Arc<dyn PixelProcessor>>,
    format_processors: DashMap<PluginId, Arc<dyn FormatProcessor>>,
    ai_processors: DashMap<PluginId, Arc<dyn AiProcessor>>,
}
```

#### 公共方法

| 方法 | 签名 | 描述 |
|---|---|---|
| `new()` | `fn new() -> Self` | 创建空 Registry |
| `register()` | `fn register(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()>` | 注册基础插件 |
| `unregister()` | `fn unregister(&self, id: &str) -> Option<Arc<dyn Plugin>>` | 注销插件（包括所有处理器） |
| `get()` | `fn get(&self, id: &str) -> Option<Arc<dyn Plugin>>` | 按 ID 获取通用插件引用 |
| `get_metadata_processor()` | `fn get_metadata_processor(&self, id: &str) -> Option<Arc<dyn MetadataProcessor>>` | 按 ID 获取元数据处理器 |
| `get_pixel_processor()` | `fn get_pixel_processor(&self, id: &str) -> Option<Arc<dyn PixelProcessor>>` | 按 ID 获取像素处理器 |
| `get_format_processor()` | `fn get_format_processor(&self, id: &str) -> Option<Arc<dyn FormatProcessor>>` | 按 ID 获取格式处理器 |
| `iter_format_processors()` | `fn iter_format_processors(&self) -> impl Iterator<Item = Arc<dyn FormatProcessor>>` | 遍历所有格式处理器 |
| `query()` | `fn query(&self, q: &PluginQuery) -> Vec<Arc<dyn Plugin>>` | 按条件查询插件 |
| `by_category()` | `fn by_category(&self, cat: PluginCategory) -> Vec<Arc<dyn Plugin>>` | 按类别查询 |
| `all()` | `fn all(&self) -> Vec<Arc<dyn Plugin>>` | 获取所有插件 |
| `manifest()` | `fn manifest(&self, id: &str) -> Option<PluginManifest>` | 按 ID 获取插件清单 |
| `manifests()` | `fn manifests(&self) -> Vec<PluginManifest>` | 获取所有插件清单 |
| `categories()` | `fn categories(&self) -> Vec<PluginCategory>` | 获取所有已注册类别 |
| `is_loaded()` | `fn is_loaded(&self, id: &str) -> bool` | 检查插件是否已加载 |
| `register_metadata_processor()` | `fn register_metadata_processor(&self, p: Arc<dyn MetadataProcessor>) -> PluginResult<()>` | 注册元数据处理器 |
| `register_pixel_processor()` | `fn register_pixel_processor(&self, p: Arc<dyn PixelProcessor>) -> PluginResult<()>` | 注册像素处理器 |
| `register_format_processor()` | `fn register_format_processor(&self, p: Arc<dyn FormatProcessor>) -> PluginResult<()>` | 注册格式处理器 |
| `register_ai_processor()` | `fn register_ai_processor(&self, p: Arc<dyn AiProcessor>) -> PluginResult<()>` | 注册 AI 处理器 |

**PluginQuery 结构体**:

| 字段 | 类型 | 描述 | 默认值 |
|---|---|---|---|
| `category` | `Option<PluginCategory>` | 按类别过滤 | `None` |
| `tags` | `Vec<String>` | 按标签过滤（AND） | `[]` |
| `requires_pixel` | `Option<bool>` | 按像素访问过滤 | `None` |
| `keyword` | `Option<String>` | 按名称/描述关键词搜索 | `None` |
| `enabled_only` | `bool` | 是否只返回启用插件 | `false` |

### 6.3 ParameterSchema API

位置: `crates/plugin/src/schema.rs`

#### ParameterSchema

```rust
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `empty()` | `fn empty() -> Self` | 创建空 schema (version=1) |
| `field()` | `fn field(&self, section_id: &str, field_id: &str) -> Option<&ParameterField>` | 按节 ID 和字段 ID 查找 |
| `defaults()` | `fn defaults(&self) -> ParameterSet` | 提取所有默认值生成 ParameterSet |
| `all_fields()` | `fn all_fields(&self) -> Vec<&ParameterField>` | 展平返回所有字段引用 |

### 6.4 ParameterSet API

位置: `crates/plugin/src/schema.rs`

```rust
pub struct ParameterSet {
    pub values: HashMap<String, serde_json::Value>,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `new()` | `fn new() -> Self` | 创建空集合 |
| `insert()` | `fn insert(&mut self, key: String, value: Value)` | 插入键值对 |
| `get()` | `fn get(&self, key: &str) -> Option<&serde_json::Value>` | 获取原始值 |
| `get_str()` | `fn get_str(&self, key: &str) -> Option<&str>` | 获取字符串值 |
| `get_i64()` | `fn get_i64(&self, key: &str) -> Option<i64>` | 获取整数 |
| `get_f64()` | `fn get_f64(&self, key: &str) -> Option<f64>` | 获取浮点数 |
| `get_bool()` | `fn get_bool(&self, key: &str) -> Option<bool>` | 获取布尔值 |
| `merge()` | `fn merge(&mut self, other: &ParameterSet)` | 浅合并（后者覆盖） |
| `iter()` | `fn iter(&self) -> impl Iterator<Item = (&String, &Value)>` | 遍历所有键值对 |

### 6.5 Engine API

位置: `crates/engine/src/`

#### 6.5.1 PipelineGraph

```rust
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<(PortId, PortId)>,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `new()` | `fn new() -> Self` | 创建空图 |
| `add_node()` | `fn add_node(&mut self, plugin_id: String, label: String) -> NodeId` | 添加节点（自动创建输入/输出端口） |
| `remove_node()` | `fn remove_node(&mut self, node_id: NodeId) -> bool` | 移除节点及相关边 |
| `connect()` | `fn connect(&mut self, from: PortId, to: PortId) -> Result<(), PluginError>` | 连接端口（含环检测） |
| `disconnect()` | `fn disconnect(&mut self, from: PortId, to: PortId) -> bool` | 断开端口 |
| `topological_order()` | `fn topological_order(&self) -> Result<Vec<NodeId>, PluginError>` | 获取拓扑排序（Kahn 算法） |
| `has_cycle()` | `fn has_cycle(&self) -> bool` | 检测环 |
| `validate_graph()` | `fn validate_graph(&self) -> Result<(), Vec<String>>` | 验证图结构 |
| `from_template()` | `fn from_template(template: &PipelineTemplate) -> Self` | 从模板创建图 |
| `node()` | `fn node(&self, id: NodeId) -> Option<&PipelineNode>` | 按 ID 获取节点 |
| `node_mut()` | `fn node_mut(&mut self, id: NodeId) -> Option<&mut PipelineNode>` | 按 ID 获取可变节点 |
| `port_owner()` | `fn port_owner(&self, port_id: PortId) -> Option<NodeId>` | 按端口 ID 查找所属节点 |

**PipelineNode**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `id` | `NodeId` (UUID) | 节点 UUID |
| `label` | `String` | 显示标签 |
| `plugin_id` | `PluginId` | 对应插件 ID |
| `enabled` | `bool` | 是否启用 |
| `position` | `(f64, f64)` | 图形位置（编辑器用） |
| `inputs` | `Vec<PortId>` | 输入端口列表 |
| `outputs` | `Vec<PortId>` | 输出端口列表 |
| `parameter_overrides` | `Option<ParameterSet>` | 参数覆盖 |

#### 6.5.2 PipelineTemplate

```rust
pub struct PipelineTemplate {
    pub metadata: TemplateMetadata,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub overrides: Vec<ImageOverride>,
    pub groups: Vec<ParamGroup>,
    pub batch: Option<BatchConfig>,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `validate()` | `fn validate(&self) -> Result<(), String>` | 验证模板（非空节点、有效边） |
| `into_graph()` | `fn into_graph(self) -> Self` | 转换为可执行的 PipelineGraph |

#### 6.5.3 NodeExecutor

```rust
pub struct NodeExecutor {
    pub registry: Arc<Registry>,
    pub resolver: Arc<ParameterResolver>,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `new()` | `fn new(registry: Arc<Registry>, resolver: Arc<ParameterResolver>) -> Self` | 创建执行器 |
| `execute()` | `async fn execute(&self, graph: &PipelineGraph, image_info: &ImageInfo, buffer: Option<PixelBuffer>, metadata: &Metadata, progress: Box<dyn ProgressSink>) -> PluginResult<ExecutionResult>` | 按拓扑序执行所有节点 |

**NodeStatus 枚举**:

| 变体 | 描述 |
|---|---|
| `Pending` | 待处理 |
| `Running` | 运行中 |
| `Completed(ProcessingStats)` | 已完成 |
| `Failed(String)` | 失败 |
| `Skipped` | 已跳过 |

**NodeRunState**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `status` | `NodeStatus` | 当前状态 |
| `started_at` | `Option<DateTime<Utc>>` | 开始时间 |

**ExecutionContext**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `image_info` | `ImageInfo` | 图像信息 |
| `buffer` | `Option<PixelBuffer>` | 当前像素缓冲 |
| `encoded_output` | `Option<Vec<u8>>` | 编码输出字节 |
| `metadata` | `Metadata` | 元数据 |
| `node_states` | `HashMap<NodeId, NodeRunState>` | 节点运行状态 |

**ExecutionResult**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `buffer` | `Option<PixelBuffer>` | 最终像素缓冲区 |
| `encoded_output` | `Option<Vec<u8>>` | 最终编码字节 |
| `metadata` | `Metadata` | 最终元数据 |
| `node_states` | `HashMap<NodeId, NodeRunState>` | 各节点状态 |

#### 6.5.4 ParameterResolver

```rust
pub struct ParameterResolver {
    pub template_params: HashMap<NodeId, ParameterSet>,
    pub group_overrides: Vec<(GroupCondition, HashMap<NodeId, ParameterSet>)>,
    pub image_overrides: HashMap<(ImageId, NodeId), ParameterSet>,
    pub expr_engine: ExpressionEngine,
}
```

| 方法 | 签名 | 描述 |
|---|---|---|
| `new()` | `fn new() -> Self` | 创建空解析器 |
| `set_template_params()` | `fn set_template_params(&mut self, node_id: NodeId, params: ParameterSet)` | 设置模板级参数 |
| `add_group_override()` | `fn add_group_override(&mut self, condition: GroupCondition, params: HashMap<NodeId, ParameterSet>)` | 添加条件参数组 |
| `set_image_override()` | `fn set_image_override(&mut self, image_id: ImageId, node_id: NodeId, params: ParameterSet)` | 设置图像级覆盖 |
| `resolve()` | `fn resolve(&self, node_id: NodeId, image_id: ImageId, schema: &ParameterSchema, metadata: &Metadata, image_info: &ImageInfo) -> ParameterSet` | 五级解析：默认→模板→分组→图像→表达式 |
| `resolve_single()` | `fn resolve_single(&self, node_id: NodeId, schema: &ParameterSchema) -> ParameterSet` | 简单解析（仅默认+模板） |
| `evaluate_condition()` | `fn evaluate_condition(&self, condition: &GroupCondition, metadata: &Metadata, image_info: &ImageInfo) -> bool` | 评估条件 |

**参数解析优先级** (高到低):

1. 表达式 (`supports_expression` 字段)
2. 图像级覆盖 (`ImageOverride`)
3. 条件组覆盖 (`ParamGroup`，匹配 condition)
4. 模板级参数 (`TemplateNode.params`)
5. 插件默认值 (`ParameterSchema.defaults()`)

不可覆盖字段（`allow_override = false`）在模板快照后锁定，组和图像覆盖不影响它们。

**GroupCondition 枚举**:

| 变体 | 描述 |
|---|---|
| `Always` | 永真 |
| `ExifEq { tag, value }` | EXIF 字段等值比较 |
| `ExifGte { tag, value }` | EXIF 数值 >= |
| `ExifLte { tag, value }` | EXIF 数值 <= |
| `GpsNear { lat, lon, radius_km }` | GPS 位置半径 |
| `And(Vec<GroupCondition>)` | 逻辑与 |
| `Or(Vec<GroupCondition>)` | 逻辑或 |
| `Expression(String)` | 表达式计算 |

**ExpressionEngine**:

```rust
pub struct ExpressionEngine;

impl ExpressionEngine {
    pub fn evaluate(&self, expr: &str, metadata: &Metadata, image_info: &ImageInfo)
        -> Result<serde_json::Value, String>;
}
```

支持语法:
- `${exif.<field>}` — EXIF 变量 (`iso`, `make`, `model`, `lens`, `focal_length`, `aperture`, `shutter`)
- `${image.<field>}` — 图像变量 (`filename`, `width`, `height`, `filesize`)
- 字面量数字和字符串
- 比较运算符: `==`, `!=`, `>`, `<`, `>=`, `<=`
- 三元表达式: `condition ? true_val : false_val`

### 6.6 Core Types

位置: `crates/core/src/`

#### 6.6.1 图像类型 (`image.rs`)

**PixelBuffer**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `width` | `u32` | 像素宽度 |
| `height` | `u32` | 像素高度 |
| `layout` | `ChannelLayout` | 通道布局 |
| `format` | `PixelFormat` | 像素格式 |
| `color_space` | `ColorSpace` | 颜色空间 |
| `icc_profile` | `Option<Vec<u8>>` | ICC 配置文件 |
| `data` | `AlignedBuffer` | 64 字节对齐的像素数据 |

方法: `new()`, `byte_size()`, `pixel_count()`, `u16_samples()`

**PixelFormat 枚举**: `U8`, `U16`, `U32`, `F16`, `F32`
方法: `bytes_per_channel()`, `is_float()`, `is_high_precision()`, `max_value_u16()`

**ChannelLayout 枚举**: `Gray(1)`, `GrayAlpha(2)`, `RGB(3)`, `RGBA(4)`, `Planar(u8)`, `Custom(u8)`
方法: `channel_count()`, `is_interleaved()`

**AlignedBuffer**: 对齐像素数据容器，支持 `as_u16_slice()`, `as_f32_slice()` 零拷贝转换。

**TileLayout**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `image_width/height` | `u32` | 图像尺寸 |
| `tile_size` | `u32` | 瓦片大小 |
| `tiles_x/tiles_y` | `u32` | 瓦片行列数 |
| `overlap` | `u32` | 瓦片重叠像素 |
| `total_tiles` | `u32` | 总瓦片数 |

方法: `new()`, `tile_spec()`, `iter_tiles()`

**DecodeOptions**:

| 字段 | 类型 | 默认值 |
|---|---|---|
| `pixel_format` | `Option<PixelFormat>` | `None` |
| `max_width/max_height` | `Option<u32>` | `None` |
| `read_metadata` | `bool` | `true` |
| `apply_transfer` | `bool` | `false` |
| `icc_profile` | `Option<Vec<u8>>` | `None` |

**EncodeOptions**:

| 字段 | 类型 | 默认值 |
|---|---|---|
| `format` | `ImageFormat` | `HEIF` |
| `quality` | `Option<f32>` | `Some(95.0)` |
| `lossless` | `bool` | `false` |
| `bit_depth` | `u8` | `10` |
| `chroma_subsampling` | `Option<ChromaSubsampling>` | `Some(Yuv444)` |
| `encoder` | `Option<String>` | `None` |
| `effort` | `Option<u8>` | `None` |
| `compression` | `Option<String>` | `None` |
| `embed_profile` | `Option<bool>` | `None` |

**ChromaSubsampling**: `Yuv444`, `Yuv422`, `Yuv420`

#### 6.6.2 颜色类型 (`color.rs`)

**ColorSpace**:

| 字段 | 类型 | 描述 |
|---|---|---|
| `primaries` | `ColorPrimaries` | 色域 |
| `transfer` | `TransferFunction` | 传输函数 |
| `white_point` | `WhitePoint` | 白点 |
| `hdr_nits` | `Option<f32>` | HDR 峰值亮度 |

预定义常量: `SRGB`, `ADOBE_RGB`, `DISPLAY_P3`, `REC2020_PQ`, `ACES_CG`, `LINEAR_SRGB`

方法: `is_hdr()`, `primaries_to_xyz_matrix()`, `conversion_matrix_to()`, `generate_icc_profile()`

**ColorPrimaries**: `BT709`, `BT2020`, `DisplayP3`, `SRGB`, `AdobeRGB`, `ProPhoto`, `ACES`, `ACEScg`, `CIEXYZ`, `DCIP3`, `Rec2100`

**TransferFunction**:
- 标准: `Linear`, `SRGB`, `Gamma22/24/26/28`
- HDR: `PQ` (ST.2084), `HLG` (ARIB STD-B67)
- 相机 Log: `SLog3`, `LogC`
- 自定义: `Custom(f64)`

方法: `decode_to_linear()`, `encode_from_linear()`

**WhitePoint**: `D50`, `D55`, `D60`, `D65`, `D75`, `DCI`, `E`, `Custom(f32, f32)`

**RenderingIntent**: `Perceptual`, `RelativeColorimetric`, `Saturation`, `AbsoluteColorimetric`

**GamutMapping**: `Clip`, `Compress`, `LuminancePreserve`

**ColorConversionSpec**: 完整颜色转换规范，支持 ICC 和 OCIO 配置。

#### 6.6.3 元数据类型 (`metadata.rs`)

**Metadata**: `{ exif, xmp, iptc, gps, custom }`

**ExifData**: 27 个字段，涵盖相机、镜头、曝光、GPS、颜色等 EXIF 标准标签。

**XmpData**: 12 个字段，支持自定义 namespace 扩展。

**IptcData**: 11 个字段，新闻行业标准元数据。

**GpsData**: 23 个字段，含坐标、海拔、方向、速度、目的地等。

方法: `has_coordinates()`, `coordinate_tuple()`

**GpxTrack**: GPX 轨迹支持，含 `interpolate_at()` 时间插值。

#### 6.6.4 基础类型 (`types.rs`)

**类型别名**: `PluginId = String`, `NodeId = Uuid`, `ImageId = Uuid`, `BatchId = Uuid`, `PortId = Uuid`, `GroupId = Uuid`

**PluginVersion**: `{ major: u32, minor: u32, patch: u32, pre: Option<String> }`

**VersionRequirement**: `{ min_version, max_version? }`，方法: `is_satisfied_by()`

**PluginCategory**: `Input`, `Metadata`, `Color`, `Transform`, `Enhance`, `Merge`, `Format`, `External`, `Custom(String)`

**ImageFormat**: `HEIF`, `HEIC`, `AVIF`, `JXL`, `PNG`, `TIFF`, `JPEG`, `WEBP`, `OpenEXR`, `RAW`, `DNG`, `PPM`, `PGM`, `BMP`, `Unknown(String)`

**GpuBackend**: `None`, `CUDA`, `Metal`, `Vulkan`, `Auto`
**AiBackend**: `ONNX`, `TensorRT`, `CoreML`, `OpenVINO`, `Burn`

**GpuContext**: `{ backend, device_name, total_memory_mb, available_memory_mb, compute_units }`

**HardwareInfo**: `{ cpu_cores, cpu_threads, total_ram_mb, gpus }`

**ProcessingStats**: `{ elapsed_ms, cpu_time_ms, gpu_time_ms?, peak_memory_mb, input_pixels, output_pixels }`

**ImageInfo**: `{ id, path, filename, format, width, height, file_size_bytes, pixel_format, color_space }`

**Widget 枚举组**:
- `ColorMode`: `RGB`, `RGBA`, `HSL`, `HSV`, `Lab`
- `FilePathKind`: `File`, `Directory`, `SaveFile`
- `SliderOrientation`: `Horizontal`, `Vertical`
- `SliderStyle`: `Continuous`, `Discrete`, `Range`, `DualHandle`
- `FloatWidget`: `SpinBox`, `Slider`, `ComboSlider`, `DragInput`
- `IntegerWidget`: `SpinBox`, `Slider`, `Combo`
- `EnumDisplay`: `Dropdown`, `RadioGroup`, `ButtonGroup`, `SegmentedControl`, `Tabs`, `PopupCard`

---

## 7. ProgressSink / ProgressReporter Trait

位置: `crates/plugin/src/trait_def.rs`

`ProgressSink` 是插件系统和执行引擎之间的进度报告抽象，允许编译期多态（trait object），同时 CLI 和 gRPC 共用执行代码。

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

### 实现变体

| 实现 | 用途 | `set_progress` 行为 | `is_canceled` 行为 |
|---|---|---|---|
| `ChannelProgressSink` (server) | gRPC Execute 流 | 发送 `ExecuteProgress` 到 `mpsc::Sender` | 读取 `AtomicBool` |
| `NoopProgress` (batch) | 批量处理静默模式 | 无操作 | 读取 `AtomicBool` |
| `InlineProgress` (executor) | 子处理器内部 | 无操作 | 读取 `AtomicBool` |
| CLI 实现 | CLI 输出 | 打印到 stderr | 监听 Ctrl+C |

### 取消流程

```
用户请求 Cancel
  → gRPC: set cancel AtomicBool = true
  → executor 轮询 progress.is_canceled()
  → 返回 PluginError::Canceled
  → gRPC 流发送 Error stage
  → 流关闭
```

---

## 8. Config File Loading

```rust
fn load_config(path: &str) -> Result<PipelineConfig>
```

### 行为

1. **自动检测格式**: 根据文件扩展名选择解析器
   - `.json` → `serde_json::from_str`
   - `.toml` → `toml::from_str`
   - 其他 → 尝试 JSON，失败则尝试 TOML
2. **验证结构**: 加载后调用 `PipelineTemplate::validate()`:
   - 检查 `nodes` 非空
   - 检查 `edges` 引用的节点存在
3. **返回**: `Ok(PipelineConfig)` 或 `Err(PluginError::Config(msg))`

### 管线执行入口

```rust
// Server: build_template() 将 proto PipelineSpec → PipelineTemplate → PipelineGraph
let template = build_template(&spec);
template.validate()?;
let graph = template.into_graph();

// CLI: 直接读取 JSON/TOML 文件
let config: PipelineTemplate = load_config(path)?;
config.validate()?;
let graph = config.into_graph();
```

---

## 9. 错误代码体系

位置: `crates/core/src/error.rs`

### PluginError 枚举（17 种）

| 错误变体 | 错误码 | 描述 | 何时发生 |
|---|---|---|---|
| `NotFound(PluginId)` | `PLUGIN_NOT_FOUND` | 插件未找到 | Registry 查询无结果 |
| `AlreadyLoaded { plugin }` | `PLUGIN_ALREADY_LOADED` | 插件已加载 | 重复注册 |
| `LoadFailed { plugin, reason }` | `PLUGIN_LOAD_FAILED` | 加载失败 | 初始化异常 |
| `VersionMismatch { plugin, actual, required }` | `PLUGIN_VERSION_MISMATCH` | 版本不匹配 | 依赖检查 |
| `InvalidParameter { plugin, field, message }` | `INVALID_PARAMETER` | 参数无效 | 验证失败 |
| `MissingTool { plugin, tool, required }` | `MISSING_TOOL` | 外部工具缺失 | 外部工具检查 |
| `GpuNotAvailable { plugin, backend }` | `GPU_NOT_AVAILABLE` | GPU 不可用 | CUDA/Metal 检查 |
| `GpuOutOfMemory { plugin, needed, available }` | `GPU_OUT_OF_MEMORY` | GPU 内存不足 | 显存分配 |
| `ExpressionError { plugin, field, error }` | `EXPRESSION_ERROR` | 表达式错误 | 表达式计算 |
| `Timeout { plugin, elapsed, timeout }` | `TIMEOUT` | 处理超时 | 执行时限 |
| `Internal { plugin, message }` | `INTERNAL_ERROR` | 内部错误 | 插件 panic/异常 |
| `Canceled { plugin }` | `CANCELED` | 用户取消 | `is_canceled() == true` |
| `Io { plugin, error }` | `IO_ERROR` | IO 错误 | 文件读写 |
| `ValidationFailed(String)` | `VALIDATION_FAILED` | 管线验证失败 | `template.validate()` |
| `NodeExecutionFailed { node, message }` | `NODE_EXECUTION_FAILED` | 节点执行失败 | 处理器返回错误 |
| `CircularDependency` | `CIRCULAR_DEPENDENCY` | 循环依赖 | 图拓扑排序 |
| `FileNotFound(String)` | `FILE_NOT_FOUND` | 文件未找到 | `is_file()` 返回 false |
| `UnsupportedFormat(String)` | `UNSUPPORTED_FORMAT` | 不支持的格式 | 解码/编码 |
| `EncodingFailed(String)` | `ENCODING_FAILED` | 编码失败 | `encode()` 错误 |
| `DecodingFailed(String)` | `DECODING_FAILED` | 解码失败 | `decode()` 错误 |
| `Config(String)` | `CONFIG_ERROR` | 配置错误 | JSON/TOML 解析 |
| `Other(String)` | `OTHER` | 其他错误 | 通用 |

### ValidationIssue 枚举

| 变体 | 严重程度 | 描述 |
|---|---|---|
| `Error { param, message }` | ERROR | 阻止执行 |
| `Warning { param, message }` | WARNING | 可继续但需注意 |
| `Info { param, message }` | INFO | 参考信息 |

### gRPC 状态码映射

| PluginError | gRPC Status |
|---|---|
| `NotFound`, `FileNotFound` | `NOT_FOUND` |
| `InvalidParameter`, `ValidationFailed` | `INVALID_ARGUMENT` |
| `AlreadyLoaded` | `ALREADY_EXISTS` |
| `Canceled` | `CANCELLED` |
| `Timeout` | `DEADLINE_EXCEEDED` |
| `GpuNotAvailable`, `GpuOutOfMemory` | `RESOURCE_EXHAUSTED` |
| `UnsupportedFormat` | `UNIMPLEMENTED` |
| `EncodingFailed`, `DecodingFailed`, `Io`, `Internal`, `NodeExecutionFailed` | `INTERNAL` |
| 其他 | `INTERNAL` |

---

## 10. 接口交互流程

### 10.1 序列图

#### 10.1.1 前端通过 gRPC 浏览插件

```
┌──────┐         ┌──────────┐         ┌──────────────┐
│ GUI  │         │  Server  │         │   Registry   │
└──┬───┘         └────┬─────┘         └──────┬───────┘
   │                  │                      │
   │  ListPlugins()   │                      │
   ├─────────────────►│                      │
   │                  │  manifests()         │
   │                  ├─────────────────────►│
   │                  │  Vec<PluginManifest> │
   │                  │◄─────────────────────┤
   │ PluginCatalogResponse                   │
   │◄─────────────────┤                      │
   │                  │                      │
   │  GetNodeSchema   │                      │
   │  (id="bilateral   │                      │
   │   .denoise")     │                      │
   ├─────────────────►│                      │
   │                  │  get("bilateral      │
   │                  │       .denoise")     │
   │                  ├─────────────────────►│
   │                  │  Arc<dyn Plugin>     │
   │                  │◄─────────────────────┤
   │                  │  parameter_schema()  │
   │                  │  gui_schema()        │
   │                  │  → serialize to      │
   │                  │    protobuf Struct   │
   │ NodeSchemaResponse                     │
   │◄─────────────────┤                      │
```

#### 10.1.2 管线执行流程

```
┌──────┐     ┌──────────┐     ┌──────────────┐     ┌─────────┐     ┌─────────┐
│ GUI  │     │  Server  │     │ NodeExecutor │     │Registry │     │Plugin   │
└──┬───┘     └────┬─────┘     └──────┬───────┘     └────┬────┘     └────┬────┘
   │              │                  │                   │               │
   │ CreatePipeline│                 │                   │               │
   │ (PipelineSpec)│                 │                   │               │
   ├─────────────►│                  │                   │               │
   │              │ build_template() │                   │               │
   │              │ → validate()     │                   │               │
   │              │ → into_graph()   │                   │               │
   │  PipelineId  │                  │                   │               │
   │◄─────────────┤                  │                   │               │
   │              │                  │                   │               │
   │ Execute(id,  │                  │                   │               │
   │   path, out) │                  │                   │               │
   ├─────────────►│                  │                   │               │
   │              │ load_image()     │                   │               │
   │              │ → Stage:LOADING █│                   │               │
   │◄═════════════╪══════════════════╪═══════════════════╪═══════════════╪══ 流开始
   │              │ execute(graph,   │                   │               │
   │              │   image, buf,    │                   │               │
   │              │   meta, progress)│                   │               │
   │              ├─────────────────►│                   │               │
   │              │                  │ topological_order │               │
   │              │                  │     ↓             │               │
   │              │                  │ for each node:     │               │
   │              │                  │  ├ resolve params  │               │
   │              │                  │  ├ validate params │               │
   │              │                  │  └──► Stage:       │               │
   │              │                  │       PROCESSING █ │               │
   │◄═════════════╪══════════════════╪═══════════════════╪═══════════════╪══
   │              │                  │       process()    │               │
   │              │                  │       ├────────────────────────────►│
   │              │                  │       │ process_pixels(buf, params) │
   │              │                  │       │◄────────────────────────────┤
   │              │                  │       │    ProcessingStats          │
   │              │                  │  → Stage:DONE █ │               │
   │◄═════════════╪══════════════════╪═══════════════════╪═══════════════╪══
   │              │                  │  save output      │               │
   │              │                  │◄──────────────────┤               │
   │              │  ExecutionResult │                   │               │
   │              │◄─────────────────┤                   │               │
   │◄═════════════╪══════════════════╪═══════════════════╪═══════════════╪══ 流结束
```

#### 10.1.3 参数解析层级

```
查询 NodeSchema (GetNodeSchema)
  → 获取 ParameterSchema.defaults()     ← 第 1 层: 插件默认值
  → GUI 渲染参数面板
  → 用户编辑 → PipelineSpec.nodes[].params ← 第 2 层: 模板级参数
  → 匹配 GroupCondition                  ← 第 3 层: 条件组覆盖
  → ImageOverride (每图像覆盖)           ← 第 4 层: 图像级覆盖
  → ExpressionEngine.evaluate()          ← 第 5 层: 表达式求值
  → 最终 ParameterSet → process_pixels()
```

### 10.2 数据流

#### 整体数据流

```
┌─────────────────────────────────────────────────────────────┐
│                    数据流概览                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  插件开发者                                                  │
│    │ 编写 Plugin impl                                        │
│    │ → 定义 ParameterSchema (crates/plugin/src/schema.rs)    │
│    │ → 注册到 Registry (crates/plugin/src/registry.rs)      │
│    ▼                                                        │
│  Registry (运行时)                                           │
│    │ gRPC ListPlugins / GetNodeSchema                        │
│    ▼                                                        │
│  前端 (GUI/CLI)                                              │
│    │ 渲染参数面板 (动态 UI，基于 ParameterSchema)              │
│    │ 用户编辑参数                                            │
│    │ 导出 JSON 配置文件                                      │
│    ▼                                                        │
│  PipelineConfig JSON 文件 (文件系统)                          │
│    │ load_config()                                          │
│    ▼                                                        │
│  后端 (Engine)                                               │
│    │ PipelineTemplate → PipelineGraph                       │
│    │ NodeExecutor.execute()                                 │
│    │  → ParameterResolver.resolve()                         │
│    │  → 对每个节点:                                          │
│    │     PixelProcessor::process_pixels() /                  │
│    │     FormatProcessor::encode() /                         │
│    │     MetadataProcessor::write_metadata()                 │
│    ▼                                                        │
│  输出 (PixelBuffer / Vec<u8>)                                │
│    │ 写入磁盘 / 流回前端                                      │
│    ▼                                                        │
│  最终图像文件                                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 从插件定义到前端渲染的数据流

```
┌──────────────┐    ParameterSchema     ┌──────────────┐
│ Plugin impl  │ ─────────────────────► │ Registry     │
│ (Rust struct)│                        │ (DashMap)    │
└──────────────┘                        └──────┬───────┘
                                               │
                                      serde_json::to_value
                                               │
                                               ▼
                                        JSON String
                                               │
                                  json_to_prost_value
                                               │
                                               ▼
┌──────────────┐    google.protobuf.Struct   ┌──────────────┐
│ Frontend     │ ◄───────────────────────── │ gRPC Server  │
│ (C#/JS/...)  │   GetNodeSchema Response    │ (tonic)      │
└──────┬───────┘                             └──────────────┘
       │
       │ 解析 "type" 标签字段
       │ 将每个 ParameterField 映射到 UI 控件
       │  string   → TextBox
       │  integer  → NumericUpDown / Slider
       │  float    → Slider / SpinBox
       │  boolean  → ToggleSwitch
       │  enum     → ComboBox / RadioGroup
       │  color    → ColorPicker
       │  file_path → FilePicker
       │  slider   → TrackBar
       │  
       ▼
┌──────────────┐
│ 参数面板 UI   │
└──────────────┘
```

#### 从前端配置到后端执行的数据流

```
┌──────────────┐    PipelineConfig JSON     ┌──────────────┐
│ Frontend     │ ──────────────────────────►│ File System  │
│ (保存配置)    │                            │ (.json/.toml) │
└──────────────┘                            └──────┬───────┘
                                                   │
                                     load_config / from_str
                                                   │
                                                   ▼
┌──────────────┐                          ┌──────────────┐
│ CLI run      │ ──────────────►         │ Engine        │
│ gRPC Execute │   template.into_graph()  │ PipelineGraph │
└──────────────┘                          └──────┬───────┘
                                                 │
                                  topological_order()
                                  ParameterResolver::resolve()
                                                 │
                                                 ▼
┌──────────────┐    Box<dyn ProgressSink>  ┌──────────────┐
│ Frontend     │ ◄──────────────────────── │ NodeExecutor │
│ (进度/取消)   │   stream ExecuteProgress  │              │
└──────────────┘                           └──────┬───────┘
                                                  │
                          process_pixels / encode / write_metadata
                                                  │
                                                  ▼
                                          ExecutionResult
                                          { buffer, encoded_output, metadata, node_states }
```

---

## 附录 A: 类型别名和常量

| 别名 | 基础类型 | 描述 |
|---|---|---|
| `PluginId` | `String` | 插件标识符，点分隔命名 |
| `NodeId` | `Uuid` | 节点 UUID |
| `ImageId` | `Uuid` | 图像 UUID |
| `BatchId` | `Uuid` | 批次 UUID |
| `PortId` | `Uuid` | 端口 UUID |
| `GroupId` | `Uuid` | 组 UUID |
| `PluginResult<T>` | `Result<T, PluginError>` | 插件系统使用的 Result 类型 |

## 附录 B: 瓦片处理阈值

- 默认阈值: 8,847,360 像素 (~4096×2160, 约 4K)
- 超过阈值时自动启用 `TileEngine` 分块处理
- 默认 tile_size: 512，默认 overlap: 64

## 附录 C: 依赖关系

```
photopipeline-server
 ├── photopipeline-core
 ├── photopipeline-plugin
 ├── photopipeline-engine
 ├── tonic (gRPC)
 └── tokio (async runtime)

photopipeline-cli
 ├── photopipeline-core
 ├── photopipeline-plugin
 ├── photopipeline-engine
 ├── photopipeline-plugins (内置插件集)
 ├── clap (CLI 解析)
 └── tokio

photopipeline-engine
 ├── photopipeline-core
 ├── photopipeline-plugin
 └── uuid

photopipeline-plugin
 ├── photopipeline-core
 ├── dashmap
 ├── parking_lot
 └── async-trait

photopipeline-core
 ├── serde / serde_json
 ├── uuid
 ├── chrono
 ├── strum
 └── thiserror
```

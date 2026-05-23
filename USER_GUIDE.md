# Photopipeline 用户指南

> 本文档面向 Photopipeline 的最终用户，涵盖从安装、CLI 操作到高级管线配置的完整工作流。

---

## 目录

1. [入门](#1-入门)
   - 1.1 [平台要求](#11-平台要求)
   - 1.2 [安装](#12-安装)
   - 1.3 [首次运行](#13-首次运行)
   - 1.4 [Hello World 管线](#14-hello-world-管线)
2. [核心概念](#2-核心概念)
   - 2.1 [管线](#21-管线)
   - 2.2 [节点](#22-节点)
   - 2.3 [插件](#23-插件)
   - 2.4 [参数](#24-参数)
   - 2.5 [参数优先级](#25-参数优先级)
   - 2.6 [表达式](#26-表达式)
3. [CLI 命令参考](#3-cli-命令参考)
   - 3.1 [全局选项](#31-全局选项)
   - 3.2 [`pipeline` 子命令](#32-pipeline-子命令)
   - 3.3 [`plugin` 子命令](#33-plugin-子命令)
   - 3.4 [`batch` 子命令](#34-batch-子命令)
   - 3.5 [gRPC 服务端](#35-grpc-服务端)
4. [管线配置详解](#4-管线配置详解)
   - 4.1 [TOML 格式规则](#41-toml-格式规则)
   - 4.2 [`[metadata]` 段](#42-metadata-段)
   - 4.3 [`[[nodes]]` 段](#43-nodes-段)
   - 4.4 [`[[edges]]` 段](#44-edges-段)
   - 4.5 [各插件参数完整参考](#45-各插件参数完整参考)
   - 4.6 [`[[overrides]]` 段](#46-overrides-段)
   - 4.7 [`[[groups]]` 段](#47-groups-段)
   - 4.8 [`[batch]` 段](#48-batch-段)
5. [表达式语言参考](#5-表达式语言参考)
   - 5.1 [语法总览](#51-语法总览)
   - 5.2 [变量命名空间](#52-变量命名空间)
   - 5.3 [运算符](#53-运算符)
   - 5.4 [三元运算（嵌套）](#54-三元运算嵌套)
   - 5.5 [字面值](#55-字面值)
   - 5.6 [完整表达式示例](#56-完整表达式示例)
6. [批量处理](#6-批量处理)
   - 6.1 [工作流](#61-工作流)
   - 6.2 [输出模板](#62-输出模板)
   - 6.3 [并行度优化](#63-并行度优化)
   - 6.4 [断点续传](#64-断点续传)
7. [常用管线配方](#7-常用管线配方)
   - 7.1 [HDR 处理管线](#71-hdr-处理管线)
   - 7.2 [AI 降噪管线](#72-ai-降噪管线)
   - 7.3 [GPS 标注管线](#73-gps-标注管线)
   - 7.4 [色彩空间转换管线](#74-色彩空间转换管线)
   - 7.5 [格式转换管线](#75-格式转换管线)
   - 7.6 [缩略图批量管线](#76-缩略图批量管线)
   - 7.7 [元数据清理管线](#77-元数据清理管线)
   - 7.8 [多实例自适应管线](#78-多实例自适应管线)
8. [疑难排解](#8-疑难排解)
9. [性能调优](#9-性能调优)
10. [附录](#10-附录)

---

## 1. 入门

### 1.1 平台要求

| 依赖 | 最低版本 | 用途 |
|---|---|---|
| Rust | 1.90+ | 编译 Rust workspace |
| CMake | 3.20+ | Halide / OIIO 构建（CI 平台，本地非必须） |
| pkg-config | — | 系统库定位 |
| libheif-dev | 1.12+ | HEIF / AVIF 编解码 |
| libjxl-dev | 0.8+ | JPEG XL 编解码 |
| liblcms2-dev | 2.0+ | ICC 色彩管理 |
| exiftool | 12.00+ | EXIF / XMP / IPTC 元数据（可选） |
| heif-enc | — | HEIF 命令行编码器（可选） |
| cjxl | — | JPEG XL 命令行编码器（可选） |

> **注意：** `exiftool`、`heif-enc`、`cjxl` 为外部命令行工具，仅在对应插件的后端模式为 external_tool 时使用。内置解析器无需它们。

### 1.2 安装

#### 方法一：Cargo Install（推荐）

```bash
cargo install photopipeline
```

> 该方法发布至 [crates.io](https://crates.io) 后方可使用。

#### 方法二：源码编译

```bash
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
cargo build --release --workspace
```

编译后二进制文件位于：
- `target/release/photopipeline` — CLI 主程序
- `target/release/photopipeline-server` — gRPC 服务端

#### 系统依赖安装

**Ubuntu / Debian**
```bash
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl
```

**macOS (Homebrew)**
```bash
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool
```

**Windows (vcpkg)**
```powershell
vcpkg install libheif libjxl lcms2
```

### 1.3 首次运行

```bash
# 打印帮助信息
photopipeline --help

# 列出所有已注册插件
photopipeline plugin list

# 查看指定插件详情
photopipeline plugin info photopipeline.plugins.colorspace
```

**预期输出示例：**

```
Plugin: Color Space
  ID:           photopipeline.plugins.colorspace
  Version:      1.0.0
  Category:     color
  Description:  Color space conversion with ICC profile support
  Tags:         [color, colorspace, conversion, icc, lcms2, halide, srgb, rec2020, hdr]
  Pixel access: true
  Min RAM:      256 MB
```

### 1.4 Hello World 管线

创建文件 `hello.toml`：

```toml
[metadata]
name = "我的第一条管线"
version = "1.0"
description = "从 RAW 到 16 位 JXL 的完整管线"

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
params = { gps_mode = "manual", latitude = 30.5728, longitude = 104.0668 }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = { bit_depth = "16", effort = 7 }

[[edges]]
from = "raw"
to = "exif"

[[edges]]
from = "exif"
to = "gps"

[[edges]]
from = "gps"
to = "output"
```

```bash
# 验证管线配置
photopipeline pipeline validate -c hello.toml

# 运行管线处理单张图片
photopipeline pipeline run \
  -c hello.toml \
  -i DSC0001.ARW \
  -o result.jxl
```

---

## 2. 核心概念

### 2.1 管线

管线（Pipeline）是一个**有向无环图（DAG）**，由节点（Node）和有向边（Edge）组成。数据按边的方向从上游节点流向下游节点。

```
source ──→ exif ──→ gps ──→ color ──→ denoise ──→ output
```

管线引擎按**拓扑排序**顺序执行各节点，自动检测环并拒绝无效图结构。

### 2.2 节点

节点是管线中的最小执行单元。每个节点绑定一个插件。节点属性：

| 属性 | 类型 | 说明 |
|---|---|---|
| `id` | 字符串 | 节点唯一标识符（管线内唯一） |
| `plugin` | 字符串 | 绑定的插件 ID |
| `label` | 字符串 | GUI 中显示的标签（可选） |
| `enabled` | 布尔 | 是否启用（默认 `true`） |
| `params` | 映射表 | 参数覆盖（可选） |

### 2.3 插件

插件按类别分为：

| 类别 | 说明 | 示例 |
|---|---|---|
| Input | 读取/解码输入文件 | `raw_input` |
| Metadata | 仅读写元数据，不访问像素 | `exif_rw`、`gps_set`、`time_shift` |
| Color | 色彩空间转换与调色 | `colorspace`、`lut3d` |
| Transform | 几何变换 | `transform` |
| Enhance | 画质增强 | `lens_correct`、`ai_denoise` |
| Format | 格式编码/解码 | `heif_encoder`、`jxl_encoder` |
| External | 外部工具集成 | （预留） |
| Merge | 多图像合成 | （预留） |

**关键区分：** Metadata 类插件的 `requires_pixel_access()` 返回 `false`，因此管线引擎跳过像素内存分配。仅在管线中包含至少一个 Pixel/Color/Transform/Enhance 类插件时才会分配像素缓冲区。

### 2.4 参数

每个插件的参数由其 `ParameterSchema` 声明式定义。参数类型包括：

- **基础类型：** 字符串、整数、浮点数、布尔值、枚举
- **GUI 控件：** 滑块、组合框、颜色选择器、文件路径选择器、地图选取器
- **复合类型：** 数组、预设、表达式、前/后对比预览
- **布局类：** 分隔线、嵌套段落

> 完整参数类型参考见 [附录 C](#103-parameterType-参数类型速查)。

### 2.5 参数优先级

管线引擎采用**四级参数优先级系统**，高优先级覆盖低优先级：

```
级别 3: 图像覆盖 (image override)     ← 最高优先级
  └─ 级别 2: 分组覆盖 (group override) ← 后匹配者胜出
      └─ 级别 1: 模板默认 (template default)
          └─ 级别 0: 插件内置 (plugin builtin) ← 最低优先级
```

**合并规则：**
1. 从插件内置默认值开始（`ParameterSchema::defaults()`）。
2. 应用模板中 `[[nodes]].params` 的值，覆盖同键。
3. 按定义顺序评估每个分组条件，匹配则合并其参数（后匹配覆盖先匹配）。
4. 应用图像覆盖（`[[overrides]]`），覆盖所有同键。
5. 在最终参数集中求值所有 `${ }` 表达式。

### 2.6 表达式

参数值可包含由 `${ }` 包裹的行内表达式，管线引擎在参数解析阶段自动求值：

```toml
params = {
  denoise_strength = "${exif.iso > 1600 ? 90 : 50}"
}
```

表达式支持三元运算、比较运算符、字符串/数字字面值、变量引用。

> 完整表达式参考见 [第 5 章](#5-表达式语言参考)。

---

## 3. CLI 命令参考

### 3.1 全局选项

```
photopipeline [OPTIONS] <COMMAND>

OPTIONS:
  -h, --help       打印帮助信息
  -v, --verbose    详细输出（调试时使用）
```

### 3.2 `pipeline` 子命令

#### `pipeline run` — 执行管线

```bash
photopipeline pipeline run \
  -c, --config <CONFIG>    # TOML 管线配置文件路径
  -i, --input <INPUT>      # 输入图像文件路径
  -o, --output <OUTPUT>    # 输出文件路径
```

**示例：**

```bash
# 基本用法
photopipeline pipeline run \
  -c examples/hdr_pipeline.toml \
  -i ~/photos/DSC0001.ARW \
  -o ~/output/DSC0001.jxl

# 使用相对路径
photopipeline pipeline run \
  -c my_config.toml \
  -i input.jpg \
  -o output.heif
```

#### `pipeline validate` — 验证管线配置

```bash
photopipeline pipeline validate \
  -c, --config <CONFIG>    # TOML 管线配置文件路径
```

**输出示例：**

```
✓ Pipeline validation passed
  Nodes: 5
  Edges: 4
  No cycles detected
```

验证失败时的输出：

```
✗ Pipeline validation failed
  - edge references unknown node 'nonexistent'
  - cycle detected: circular dependency
```

### 3.3 `plugin` 子命令

#### `plugin list` — 列出所有插件

```bash
photopipeline plugin list
```

**输出示例：**

```
Registered plugins (14 total):

  Input (1):
    photopipeline.plugins.raw_input           v1.0.0    RAW Input

  Metadata (3):
    photopipeline.plugins.exif_rw             v1.0.0    EXIF Reader/Writer
    photopipeline.plugins.gps_set             v1.0.0    GPS Coordinate Manager
    photopipeline.plugins.time_shift          v1.0.0    Time Shift

  Color (2):
    photopipeline.plugins.colorspace          v1.0.0    Color Space
    photopipeline.plugins.lut3d               v1.0.0    3D LUT

  Transform (1):
    photopipeline.plugins.transform            v1.0.0    Transform

  Enhance (2):
    photopipeline.plugins.lens_correct        v1.0.0    Lens Correction
    photopipeline.plugins.ai_denoise          v1.0.0    AI Denoise

  Format (5):
    photopipeline.plugins.heif_encoder        v1.0.0    HEIF Encoder
    photopipeline.plugins.jxl_encoder         v1.0.0    JPEG XL Encoder
    photopipeline.plugins.avif_encoder        v1.0.0    AVIF Encoder
    photopipeline.plugins.tiff_encoder        v1.0.0    TIFF Encoder
    photopipeline.plugins.png_encoder         v1.0.0    PNG Encoder
```

#### `plugin info <PLUGIN_ID>` — 查看插件详情

```bash
photopipeline plugin info photopipeline.plugins.ai_denoise
```

**输出包含：** 插件元信息、参数 Schema（所有字段及类型/默认值/约束）、GUI Schema（布局/预览模式/辅助视图）。

### 3.4 `batch` 子命令

#### `batch run` — 批量处理

```bash
photopipeline batch run \
  -c, --config <CONFIG>       # TOML 管线配置文件路径
  -p, --pattern <PATTERN>     # 文件匹配模式 (glob) [默认: *.ARW]
  -o, --output <OUTPUT_DIR>   # 输出目录 [默认: ./output/]
```

**示例：**

```bash
# 处理目录下所有 ARW 文件
photopipeline batch run \
  -c pipeline.toml \
  -p "~/photos/2024/**/*.ARW" \
  -o "~/processed/"

# 处理特定日期范围
photopipeline batch run \
  -c pipeline.toml \
  -p "~/photos/DSC_202405*.ARW" \
  -o "~/processed/2024-05/"

# 使用输出模板（按日期组织）
photopipeline batch run \
  -c pipeline.toml \
  -p "~/photos/*.ARW" \
  -o "~/processed/{date}/{filename}.heif"
```

#### `batch validate` — 批量验证

```bash
photopipeline batch validate \
  -c, --config <CONFIG>       # TOML 管线配置文件路径
  -p, --pattern <PATTERN>     # 文件匹配模式 [默认: *.ARW]
```

验证包括：管线结构校验、文件存在性校验、插件可用性校验、参数合法性校验。

### 3.5 gRPC 服务端

```bash
# 启动 gRPC 服务端（监听 localhost:50051）
photopipeline-server

# 自定义监听地址
photopipeline-server --addr 0.0.0.0:50051
```

服务端提供三个 gRPC 服务：
- **PipelineService** — 管线创建、执行、验证、Schema 查询
- **ImageService** — 图像加载、解码、编码、缩略图
- **BatchService** — 批量任务提交、进度查询、取消

---

## 4. 管线配置详解

### 4.1 TOML 格式规则

- 使用标准 [TOML v1.0](https://toml.io/) 格式
- 管线至少需要一个 `[[nodes]]` 条目
- 节点 ID 必须唯一，使用 `snake_case`
- 插件 ID 格式为 `photopipeline.plugins.{name}`
- 参数值遵循 JSON 类型映射：字符串用引号包裹，数值直接书写，布尔为 `true`/`false`
- 表达式用 `${ }` 包裹，可嵌套在三元运算符中

### 4.2 `[metadata]` 段

```toml
[metadata]
name = "My Pipeline"          # string, 可选
version = "1.0"               # string, 可选
description = "描述文本"       # string, 可选
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `name` | string | 否 | 管线名称，显示在 CLI 和 GUI 中 |
| `version` | string | 否 | 管线版本号（不与插件版本关联） |
| `description` | string | 否 | 多行描述，支持 Markdown |

### 4.3 `[[nodes]]` 段

```toml
[[nodes]]
id = "node_id"               # string, 必填
plugin = "full.plugin.id"    # string, 必填
label = "显示名称"            # string, 可选
enabled = true                # bool, 可选, 默认 true
params = { ... }              # table, 可选
```

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | string | 是 | — | 节点唯一标识符，在管线内唯一 |
| `plugin` | string | 是 | — | 插件完整 ID |
| `label` | string | 否 | `id` 值 | GUI 中显示的节点名称 |
| `enabled` | bool | 否 | `true` | `false` 时节点被跳过 |
| `params` | table | 否 | `{}` | 参数覆盖表（键 = 参数 ID，值 = 参数值） |

### 4.4 `[[edges]]` 段

```toml
[[edges]]
from = "source_node_id"     # string, 必填
to = "target_node_id"       # string, 必填
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `from` | string | 是 | 源节点 ID（数据的输出端） |
| `to` | string | 是 | 目标节点 ID（数据的输入端） |

**约束：**
- 边必须有向（无环）
- 不允许自环（from == to）
- 不允许重复边
- 形成环的边将被拒绝

### 4.5 各插件参数完整参考

#### 4.5.1 RAW 输入 (`photopipeline.plugins.raw_input`)

```toml
[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = {
  raw_mode = "auto",               # "auto" | "dcraw" | "libraw" | "rawtherapee"
  output_format = "f32",           # "u16" | "f32"
  half_size = false,               # 半分辨率解码
  apply_white_balance = true,      # 应用相机白平衡
  dcraw_path = "dcraw",            # dcraw 二进制路径
  dcraw_extra_args = "",           # 附加 dcraw 命令行参数
}
```

**支持的 RAW 格式：** Sony (.arw)、Canon (.cr2/.cr3)、Nikon (.nef)、Adobe DNG (.dng)、Fujifilm (.raf)、Olympus (.orf)、Panasonic (.rw2)、Pentax (.pef)、Hasselblad (.3fr)、Mamiya (.mef)、Leaf (.mos)、Epson (.erf)、Phase One (.iiq)、Samsung (.srw)、Sigma (.x3f)。

#### 4.5.2 EXIF 读写 (`photopipeline.plugins.exif_rw`)

```toml
[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"
params = {
  read_exif = true,              # 读取 EXIF
  read_xmp = true,               # 读取 XMP
  read_iptc = true,              # 读取 IPTC
  read_gps = true,               # 读取 GPS
  overwrite_original = false,    # 覆盖原文件
  preserve_makernote = true,     # 保留 MakerNote
  write_exif = "all",            # "all" | "selected" | "none"
  exiftool_path = "exiftool",    # exiftool 路径
  exiftool_args = "",            # 附加参数
}
```

#### 4.5.3 GPS 坐标 (`photopipeline.plugins.gps_set`)

```toml
[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = {
  gps_mode = "gpx_track",           # "manual" | "gpx_track" | "clear"
  latitude = 30.5728,               # 纬度 (-90~90)
  longitude = 104.0668,             # 经度 (-180~180)
  altitude = 500.0,                 # 海拔 (m)
  gpx_file = "track.gpx",           # GPX 轨迹文件
  time_offset_seconds = 0,          # 相机-GPS 时间偏移（秒）
  max_interpolation_gap = 300,      # 最大插值间隔（秒）
}
```

**GPS 模式说明：**
- `manual` — 使用手动指定的经纬度
- `gpx_track` — 从 GPX 文件按时间戳插值坐标
- `clear` — 清除现有 GPS 数据

#### 4.5.4 时间偏移 (`photopipeline.plugins.time_shift`)

```toml
[[nodes]]
id = "time"
plugin = "photopipeline.plugins.time_shift"
params = {
  shift_hours = 0,               # 偏移小时 (-23~23)
  shift_minutes = 0,             # 偏移分钟 (-59~59)
  shift_seconds = 0,             # 偏移秒 (-59~59)
  source_timezone = "UTC",       # 源时区
  target_timezone = "local",     # 目标时区
  increment_per_image = 0.0,     # 每张递增秒数
  batch_image_index = 0,         # 批次中图片索引
}
```

#### 4.5.5 色彩空间 (`photopipeline.plugins.colorspace`)

```toml
[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "auto",                  # 可选值见下方
  target_color_space = "rec2020_pq",            # 可选值见下方
  rendering_intent = "relative_colorimetric",   # "perceptual" | "relative_colorimetric" | "saturation" | "absolute_colorimetric"
  black_point_compensation = true,              # 黑点补偿
  gamut_mapping = "compress",                   # "clip" | "compress" | "luminance_preserve"
  embed_icc = true,                             # 嵌入 ICC Profile
  icc_profile_path = "",                        # 自定义 ICC 文件路径
}
```

**色彩空间可选值：** `srgb`、`display_p3`、`adobe_rgb`、`pro_photo`、`bt2020`、`rec2020_pq`、`aces_cg`、`linear_srgb`、`auto`

#### 4.5.6 3D LUT (`photopipeline.plugins.lut3d`)

```toml
[[nodes]]
id = "lut"
plugin = "photopipeline.plugins.lut3d"
params = {
  lut_path = "/path/to/lut.cube",        # LUT 文件路径
  lut_format = "cube",                   # "cube" | "3dl" | "look" | "csp"
  intensity = 100.0,                     # 混合强度 (0~100)
  input_color_space = "srgb",            # LUT 期望输入色彩空间
  clamp_output = true,                   # 钳制输出
  interpolation_method = "tetrahedral",  # "trilinear" | "tetrahedral"
}
```

#### 4.5.7 几何变换 (`photopipeline.plugins.transform`)

```toml
[[nodes]]
id = "xform"
plugin = "photopipeline.plugins.transform"
params = {
  resize_mode = "long_edge",      # "none" | "absolute" | "percentage" | "long_edge" | "short_edge" | "megapixels"
  target_width = 1920,            # 目标宽度 (px)
  target_height = 1080,           # 目标高度 (px)
  scale_percent = 100.0,          # 缩放百分比 (1~1000)
  long_edge_px = 2048,            # 长边目标像素
  angle = 0.0,                    # 旋转角度 (-360~360)
  flip_horizontal = false,        # 水平翻转
  flip_vertical = false,          # 垂直翻转
  crop_enabled = false,           # 启用裁剪
  crop_x = 0,                     # 裁剪 X 起点
  crop_y = 0,                     # 裁剪 Y 起点
  crop_width = 1920,              # 裁剪宽度
  crop_height = 1080,             # 裁剪高度
  filter_type = "lanczos3",       # "bilinear" | "lanczos3" | "nearest"
}
```

#### 4.5.8 镜头校正 (`photopipeline.plugins.lens_correct`)

```toml
[[nodes]]
id = "lens"
plugin = "photopipeline.plugins.lens_correct"
params = {
  correction_mode = "auto",          # "auto" | "manual" | "off"
  correct_distortion = true,         # 校正畸变
  correct_tca = true,                # 校正横向色差 (TCA)
  correct_vignetting = true,         # 校正暗角
  correct_geometry = false,          # 独立几何校正
  lensfun_db_path = "/usr/share/lensfun", # LensFun 数据库路径
  camera_make = "",                  # 相机品牌（manual 模式）
  camera_model = "",                 # 相机型号（manual 模式）
  lens_model = "",                   # 镜头型号（manual 模式）
}
```

#### 4.5.9 AI 降噪 (`photopipeline.plugins.ai_denoise`)

```toml
[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = {
  denoise_model = "standard_v2",        # "lightweight_v1" | "standard_v2" | "high_quality_v2" | "raw_denoise_v1"
  denoise_strength = 50.0,              # 降噪强度 (0~100)
  detail_preservation = 50.0,           # 细节保留 (0~100)
  color_noise_reduction = 75.0,         # 色彩噪声降低 (0~100)
  ai_backend = "onnx_cpu",              # "onnx_cpu" | "onnx_cuda" | "tensorrt" | "coreml" | "openvino"
  tile_size = 0,                        # 分块大小 (0=自动)
  use_fp16 = true,                      # 半精度推理
}
```

**AI 后端选择建议：**

| 硬件 | 推荐后端 | 原因 |
|---|---|---|
| NVIDIA GPU | `tensorrt` | 利用 Tensor Cores |
| Apple Silicon | `coreml` | 利用 ANE |
| Intel CPU/iGPU | `openvino` | 优化推理 |
| AMD GPU / 其他 | `onnx_cuda` | 通用 GPU 路径 |
| 仅 CPU | `onnx_cpu` | 无 GPU 时唯一选择 |

#### 4.5.10 HEIF 编码器 (`photopipeline.plugins.heif_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.heif_encoder"
params = {
  quality = 95.0,              # 品质 (0~100)
  lossless = false,            # 无损模式
  bit_depth = "10",            # 位深: "8" | "10"
  chroma_subsampling = "444",  # 色度采样: "444" | "422" | "420"
  encoder_effort = 4,          # 编码器 effort (0~10)
  heif_enc_path = "heif-enc",  # heif-enc 二进制路径
}
```

#### 4.5.11 JPEG XL 编码器 (`photopipeline.plugins.jxl_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = {
  quality = 90.0,              # 品质 (0~100); -1=无损
  lossless = false,            # 数学无损
  bit_depth = "16",            # 位深: "8" | "10" | "12" | "16"
  effort = 7,                  # effort (1~9)
  modular = false,             # 模块模式（合成图像）
  cjxl_path = "cjxl",          # cjxl 二进制路径
}
```

#### 4.5.12 AVIF 编码器 (`photopipeline.plugins.avif_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.avif_encoder"
params = {
  quality = 85.0,              # 品质 (0~100)
  speed = 6,                   # 速度 (0=慢/最佳, 10=快)
  bit_depth = "10",            # 位深: "8" | "10" | "12"
  chroma_subsampling = "444",  # 色度采样: "444" | "422" | "420"
  lossless = false,            # 无损
  avifenc_path = "avifenc",    # avifenc 二进制路径
}
```

#### 4.5.13 TIFF 编码器 (`photopipeline.plugins.tiff_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.tiff_encoder"
params = {
  compression = "deflate",     # "none" | "lzw" | "deflate" | "packbits"
  bigtiff = true,              # BigTIFF 格式（>4GB 文件）
  embed_icc = true,            # 嵌入 ICC Profile
  pixel_format = "u16",        # "u8" | "u16" | "f32"
}
```

#### 4.5.14 PNG 编码器 (`photopipeline.plugins.png_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.png_encoder"
params = {
  compression_level = 6,       # Deflate 压缩级别 (0=存储, 9=最佳)
  bit_depth = "16",            # 位深: "8" | "16"
  embed_icc = true,            # 嵌入 ICC Profile (iCCP chunk)
  include_exif = false,        # 包含 EXIF (eXIf chunk)
  color_type = "rgb",          # "rgb" | "rgba" | "gray" | "graya"
}
```

### 4.6 `[[overrides]]` 段

逐图参数覆盖，优先级最高。

```toml
[[overrides]]
image = "DSC0003.ARW"
[overrides.params.gps]
  latitude = 30.5728
  longitude = 104.0668

[[overrides]]
image = "DSC0007.ARW"
[overrides.params.ai_denoise]
  denoise_strength = 95
  detail_preservation = 25
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|
| `image` | string | 是 | 文件名（不含路径），与 glob 匹配结果比较 |
| `params.<node_id>` | table | 否 | 该节点 ID 下的参数覆盖表 |

### 4.7 `[[groups]]` 段

根据条件规则对匹配图片应用参数。**按定义顺序求值，后匹配者覆盖先匹配者。**

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
|---|---|---|
| `name` | string | 是 | 分组名称 |
| `condition` | string | 是 | 条件表达式（见下文） |
| `params.<node_id>` | table | 否 | 该组匹配时应用的参数 |

**支持的条件类型：**

| 类型 | 格式 | 示例 |
|---|---|---|
| EXIF 等于 | `exif.<tag> = "<value>"` | `exif.make = "Canon"` |
| EXIF 大于等于 | `exif.<tag> >= <number>` | `exif.iso >= 1600` |
| EXIF 小于等于 | `exif.<tag> <= <number>` | `exif.iso <= 400` |
| GPS 邻近 | `gps_near(<lat>, <lon>, <radius_km>)` | `gps_near(30.57, 104.07, 10)` |
| 任意表达式 | `${表达式}` | `${exif.iso > 1600}` |
| 逻辑与 | 多组实现 | 多个分组条目 = AND |
| 逻辑或 | 多组实现 | 多个分组条目 = OR |

> **注意：** TOML 中不直接支持 `and()` 和 `or()` 函数。组合逻辑通过配置多个分组实现。AND 逻辑需要所有分组都匹配，OR 逻辑只需任意分组匹配。

### 4.8 `[batch]` 段

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
| `output_pattern` | string | — | 输出路径模板（支持占位符） |
| `on_conflict` | string | — | 输出冲突策略：`"skip"` 或 `"overwrite"` |
| `resume` | bool | `false` | 是否支持断点续传 |

---

## 5. 表达式语言参考

### 5.1 语法总览

表达式由 `${` 和 `}` 界定，可嵌套。管线引擎在参数解析的最终阶段自动求值所有表达式。

```
${<expression>}
```

表达式的类型取决于其内容：
- **变量引用：** `${exif.iso}` → 返回数值/字符串
- **比较表达式：** `${exif.iso > 1600}` → 返回 `"true"` 或 `"false"`
- **三元表达式：** `${exif.iso > 1600 ? 0.9 : 0.4}` → 返回 `"0.9"` 或 `"0.4"`
- **字面值：** `${3.14}` → 返回 `"3.14"`；`${'hello'}` → 返回 `"hello"`

### 5.2 变量命名空间

#### `exif.*` 命名空间

| 变量 | 类型 | 说明 | 示例值 |
|---|---|---|---|
| `exif.iso` | number | ISO 感光度 | `800` |
| `exif.aperture` | number | 光圈值 | `2.8` |
| `exif.shutter` | number/string | 快门速度 | `0.002` 或 `"1/500"` |
| `exif.focal_length` | string | 焦距 | `"50mm"` |
| `exif.make` | string | 相机制造商 | `"Canon"` |
| `exif.model` | string | 相机型号 | `"EOS R5"` |
| `exif.lens` | string | 镜头型号 | `"24-70mm f/2.8"` |

> **默认行为：** 当图片无 EXIF 数据时，数值变量返回 `"0"`，字符串变量返回 `""`。分组条件中无 EXIF 数据的分组被视为不匹配。

#### `image.*` 命名空间

| 变量 | 类型 | 说明 | 示例值 |
|---|---|---|---|
| `image.filename` | string | 文件名 | `"DSC0001.ARW"` |
| `image.width` | number | 像素宽度 | `6000` |
| `image.height` | number | 像素高度 | `4000` |
| `image.filesize` | number | 文件大小（字节） | `52428800` |

### 5.3 运算符

| 运算符 | 说明 | 示例 |
|---|---|---|
| `>` | 大于 | `${exif.iso > 1600}` |
| `<` | 小于 | `${exif.iso < 400}` |
| `>=` | 大于等于 | `${exif.iso >= 800}` |
| `<=` | 小于等于 | `${exif.iso <= 3200}` |
| `==` | 等于 | `${exif.make == "Sony"}` |
| `!=` | 不等于 | `${exif.make != "Canon"}` |
| `? :` | 三元条件 | `${cond ? a : b}` |

> **数值比较：** 浮点数比较使用 epsilon 容差（`f64::EPSILON`）。
> **字符串比较：** 精确匹配，区分大小写。
> **交叉类型比较：** 数值与字符串比较始终返回 `false`。

### 5.4 三元运算（嵌套）

三元运算符支持嵌套（通过括号平衡匹配实现）：

```toml
# 多级降噪强度
params = {
  strength = "${exif.iso > 6400 ? 95 : ${exif.iso > 3200 ? 85 : ${exif.iso > 1600 ? 70 : 40}}}"
}

# 按相机品牌选择输出格式
params = {
  format = "${exif.make == 'Sony' ? 'f32' : 'u16'}"
}

# 复合条件嵌套
params = {
  quality = "${exif.iso >= 800 && exif.iso <= 3200 ? 75 : ${exif.iso > 3200 ? 95 : 40}}"
}
```

> **注意：** 逻辑 AND (`&&`) 和 OR (`||`) 通过嵌套三元运算实现。如 `a && b ? x : y` = `${a ? ${b ? x : y} : y}`。

### 5.5 字面值

| 类型 | 语法 | 示例 |
|---|---|---|
| 整数 | 直接书写 | `400`、`1600` |
| 浮点数 | 直接书写 | `3.14`、`0.5` |
| 字符串（双引号） | `"text"` | `"Canon"` |
| 字符串（单引号） | `'text'` | `'Sony'` |

### 5.6 完整表达式示例

```toml
# 自动降噪强度
denoise_strength = "${exif.iso > 1600 ? 90 : 50}"

# 按厂商选择格式
output_format = "${exif.make == 'Sony' ? 'f32' : 'u16'}"

# GPS 点名判断
location = "${exif.make == 'DJI' ? 'Drone' : 'Ground'}"

# 图像尺寸判断
max_size = "${image.width > 4000 ? 4096 : ${image.width > 2000 ? 2048 : 1024}}"

# 文件大小判断（字节）
quality = "${image.filesize > 50000000 ? 90 : 100}"

# 在分组条件中使用
[[groups]]
condition = "${exif.iso >= 1600}"
[groups.params.ai_denoise]
  denoise_strength = 90

[[groups]]
condition = "${exif.iso >= 3200}"
[groups.params.ai_denoise]
  denoise_strength = 95
```

---

## 6. 批量处理

### 6.1 工作流

```
1. 加载图片列表    → 按 glob 模式匹配文件
2. 提取 EXIF 快照  → 读取每张图片的元数据
3. 加载 GPX 轨迹   →（可选）按时间戳插值 GPS 坐标
4. 自动分组        → 按 ISO / GPS 聚类 / 时间间隔分组
5. 参数解析        → 四级优先级合并，表达式求值
6. 逐图执行        → 并行处理每张图片
7. 进度报告        → 流式进度，断点续传
```

### 6.2 输出模板

输出路径中使用占位符实现按元数据自动组织输出：

| 占位符 | 说明 | 示例值 |
|---|---|---|
| `{filename}` | 输入文件名（不含扩展名） | `DSC0001` |
| `{ext}` | 输入文件扩展名 | `ARW` |
| `{date}` | EXIF 日期（YYYY-MM-DD） | `2024-05-15` |
| `{camera}` | 相机型号 | `EOS_R5` |
| `{iso}` | ISO 值 | `800` |

**模板示例：**

```toml
[batch]
parallel = 4

# 按日期组织
output_pattern = "output/{date}/{filename}.heif"

# 按相机型号组织
output_pattern = "{camera}/{filename}.jxl"

# 按日期 + ISO
output_pattern = "archive/{date}/{iso}/{filename}.heif"
```

### 6.3 并行度优化

| 场景 | 推荐 `parallel` 值 | 原因 |
|---|---|---|
| 纯元数据管线 | CPU 核心数 × 2 | I/O 密集型，无 CPU 瓶颈 |
| CPU 像素处理 | CPU 核心数 | 计算密集型 |
| GPU 像素处理 | 2–4 | GPU 并发瓶颈 |
| 内存受限（< 8 GB） | 1–2 | 避免 OOM |

### 6.4 断点续传

设置 `resume = true` 后，批处理会在每个文件处理完成后记录进度。若批处理中断，再次执行时会自动跳过已完成的文件。

```toml
[batch]
resume = true
on_conflict = "skip"     # 跳过已存在的输出文件
```

---

## 7. 常用管线配方

### 7.1 HDR 处理管线

```toml
[metadata]
name = "HDR 处理管线"
version = "1.0"
description = "从 RAW 到 Rec.2020 PQ (HDR) 完整流程"

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
params = { bit_depth = "10", chroma_subsampling = "444", quality = 95, encoder_effort = 8 }

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

### 7.2 AI 降噪管线

```toml
[metadata]
name = "AI 降噪管线"
version = "1.0"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"

[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = {
  denoise_model = "high_quality_v2",
  denoise_strength = 75,
  detail_preservation = 40,
  ai_backend = "onnx_cuda",
  use_fp16 = true,
}

[[nodes]]
id = "lens"
plugin = "photopipeline.plugins.lens_correct"
params = { correction_mode = "auto" }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = { bit_depth = "16", effort = 7 }

[[edges]]
from = "raw"
to = "exif"

[[edges]]
from = "exif"
to = "denoise"

[[edges]]
from = "denoise"
to = "lens"

[[edges]]
from = "lens"
to = "output"

# 按 ISO 自适应降噪强度
[[groups]]
condition = "exif.iso >= 3200"
[groups.params.ai_denoise]
  denoise_strength = 90

[[groups]]
condition = "exif.iso >= 6400"
[groups.params.ai_denoise]
  denoise_strength = 95
  detail_preservation = 25
```

### 7.3 GPS 标注管线

```toml
[metadata]
name = "GPS 标注管线"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"

[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = {
  gps_mode = "gpx_track",
  gpx_file = "hiking_track.gpx",
  time_offset_seconds = 0,
  max_interpolation_gap = 300,
}

[[nodes]]
id = "time"
plugin = "photopipeline.plugins.time_shift"
params = {
  shift_hours = 0,
  source_timezone = "UTC",
  target_timezone = "Asia/Shanghai",
}

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.tiff_encoder"
params = { compression = "deflate", pixel_format = "u16", embed_icc = true }

[[edges]]
from = "raw"
to = "exif"

[[edges]]
from = "exif"
to = "gps"

[[edges]]
from = "gps"
to = "time"

[[edges]]
from = "time"
to = "output"
```

### 7.4 色彩空间转换管线

```toml
[metadata]
name = "色彩空间转换管线"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "srgb",
  target_color_space = "linear_srgb",
  rendering_intent = "relative_colorimetric",
  embed_icc = true,
}

[[nodes]]
id = "lut"
plugin = "photopipeline.plugins.lut3d"
params = {
  lut_path = "cinematic_look.cube",
  intensity = 80.0,
  interpolation_method = "tetrahedral",
}

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.png_encoder"
params = { bit_depth = "16", embed_icc = true }

[[edges]]
from = "raw"
to = "color"

[[edges]]
from = "color"
to = "lut"

[[edges]]
from = "lut"
to = "output"
```

### 7.5 格式转换管线

```toml
[metadata]
name = "批量格式转换 ARW→JXL"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = { bit_depth = "16", effort = 9, lossless = true }

[[edges]]
from = "raw"
to = "output"

[batch]
parallel = 4
output_pattern = "converted/{filename}.jxl"
on_conflict = "skip"
resume = true
```

```bash
photopipeline batch run \
  -c convert.toml \
  -p "~/photos/*.ARW" \
  -o "./converted/"
```

### 7.6 缩略图批量管线

```toml
[metadata]
name = "缩略图生成器"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { half_size = true, output_format = "u16" }

[[nodes]]
id = "xform"
plugin = "photopipeline.plugins.transform"
params = {
  resize_mode = "long_edge",
  long_edge_px = 2048,
  filter_type = "lanczos3",
}

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.heif_encoder"
params = { quality = 80, bit_depth = "10", chroma_subsampling = "444" }

[[edges]]
from = "raw"
to = "xform"

[[edges]]
from = "xform"
to = "output"

[batch]
parallel = 4
output_pattern = "thumbnails/{filename}.heif"
on_conflict = "skip"
```

### 7.7 元数据清理管线

```toml
[metadata]
name = "元数据清理 + 版权添加"

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"
params = {
  read_exif = true,
  read_xmp = true,
  read_gps = false,
  preserve_makernote = true,
  write_exif = "selected",
}

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.png_encoder"
params = { bit_depth = "16", embed_icc = false, include_exif = false }

[[edges]]
from = "exif"
to = "output"
```

> **注意：** 该管线没有输入节点（raw_input）。它仅操作元数据，从原始文件读取 EXIF，可以选择保留或清除某些标签后重新编码。`include_exif = false` 确保输出的 PNG 不包含嵌入的 EXIF 数据。

### 7.8 多实例自适应管线

```toml
[metadata]
name = "自适应管线"

[[nodes]]
id = "raw"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32" }

[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"

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
to = "denoise"

[[edges]]
from = "denoise"
to = "color"

[[edges]]
from = "color"
to = "output"

[batch]
parallel = 2
output_pattern = "{date}/{filename}.jxl"
on_conflict = "skip"
resume = true
```

---

## 8. 疑难排解

### Q: 我的图片没有 EXIF 数据，表达式会报错吗？

不会。缺少 EXIF 数据时，`exif.*` 变量返回默认值（数值返回 `"0"`，字符串返回 `""`）。分组规则中的 EXIF 条件在无 EXIF 数据时返回 `false`，分组不匹配。

### Q: 如何验证管线配置是否正确？

```bash
photopipeline pipeline validate -c my_pipeline.toml
```

成功时输出节点数和边数。失败时报告具体错误（未知节点引用、环检测等）。

### Q: 管线中的节点可以并行执行吗？

当前执行引擎按拓扑排序顺序串行执行各节点。不互相依赖的节点理论上可并行，完整并行执行将在后续版本中实现。

### Q: GPU 支持需要额外配置吗？

不需要。`colorspace` 和 `lut3d` 插件自动尝试使用 GPU（`preferred_backend = Auto`），不可用时自动回退到 CPU。`ai_denoise` 插件通过 `ai_backend` 参数指定推理后端。

### Q: AI 降噪模型从哪里下载？

内置模型需要手动下载。`standard_v2` 模型可从 HuggingFace（`photopipeline/denoise-standard-v2`）获取。插件在首次使用时检测模型文件是否存在，若不存在则报错并提示下载 URL。将 `.onnx` 文件放入项目模型目录即可。

### Q: 如何处理 RAW + JPEG 配对？

当前管线一次处理一个文件。对于 RAW + JPEG 配对，可在 `[[overrides]]` 中对 RAW 文件指定特殊参数，对 JPEG 文件使用不同的输入插件。完整配对支持计划中。

### Q: TIFF 编码器支持 OpenEXR 格式吗？

TIFF 编码器仅支持经典 TIFF 和 BigTIFF。OpenEXR 支持通过 `photopipeline-oiio` crate（feature-gated）提供，需启用 OIIO FFI 绑定。

### Q: 如何让管线跳过某些图片？

- 设置节点 `enabled = false` 跳过特定处理步骤。
- 使用 `[batch]` 中 `on_conflict = "skip"` 跳过已存在的输出文件。
- 使用 glob 模式过滤输入文件：`-p "DSC_2024*.ARW"`。

### Q: 编码器报 "tool not found" 错误

hex-enc / cjxl / avifenc 等外部编码器未安装。解决方法：
- Ubuntu: `sudo apt install libheif-examples libjxl-tools`
- macOS: `brew install libheif jpeg-xl`
- 或仅在内部编码器模式下使用插件（将 `*_path` 参数留空即可）

---

## 9. 性能调优

### 9.1 Tile 分块大小

| 图像尺寸 | 推荐 tile_size | 原因 |
|---|---|---|
| < 2000px | 256 | 减少开销 |
| 2000–6000px | 512 | 默认平衡 |
| > 6000px | 1024 | 减少分块数 |

在 TOML 管线的 `ai_denoise` 节点中设置 `tile_size` 参数；在 `colorspace`、`lut3d`、`transform` 等 Halide 驱动插件中，分块由 TileEngine 自动管理（默认 1024，64 重叠）。

### 9.2 GPU 选择

在 gRPC 服务端配置中可指定 GPU 后端：

```bash
photopipeline-server --gpu-backend cuda
```

### 9.3 内存估算

`所需内存 ≈ 像素数 × 通道数 × 字节数(channel) × (分块数 + 2)`

例如：4096×2160 f32 RGBA = ~135MB，分 16 块处理，每块约 1MB，并行 4 块 ≈ 4MB + 输出 135MB ≈ 140MB。

---

## 10. 附录

### 10.1 色彩空间常量速查

| 常量名 | 原色 | 传递函数 | 白点 | HDR |
|---|---|---|---|---|
| `srgb` | sRGB | sRGB | D65 | 否 |
| `adobe_rgb` | Adobe RGB | Gamma 2.2 | D65 | 否 |
| `display_p3` | Display P3 | sRGB | D65 | 否 |
| `pro_photo` | ProPhoto | Gamma 1.8 | D50 | 否 |
| `bt2020` | BT.2020 | sRGB | D65 | 否 |
| `rec2020_pq` | BT.2020 | PQ (ST.2084) | D65 | 是 |
| `aces_cg` | ACEScg | Linear | D60 | 否 |
| `linear_srgb` | sRGB | Linear | D65 | 否 |

### 10.2 ChromaSubsampling 说明

| 采样 | 说明 | 适用场景 |
|---|---|---|
| `yuv444` | 全分辨率色度，不损失色彩信息 | 专业工作流、归档 |
| `yuv422` | 水平色度减半 | 广播标准 |
| `yuv420` | 水平和垂直色度减半 | Web 分发、体积敏感 |

> **推荐：** 在处理管线中始终使用 `444` 采样，仅最终分发阶段使用 `420`。

### 10.3 ParameterType 参数类型速查

| # | 类型 | JSON 标签 | GUI 控件 | 典型参数 |
|:--:|------|:---:|---|---|
| 1 | String | `"string"` | 文本输入框 | 路径、版权文字 |
| 2 | Integer | `"integer"` | 数字输入/滑块/组合框 | 质量、effort |
| 3 | Float | `"float"` | 数字输入/滑块/拖拽输入 | 强度、亮度 |
| 4 | Boolean | `"boolean"` | 开关 | 启用/禁用 |
| 5 | Enum | `"enum"` | 下拉/单选/分段控件 | 模式选择 |
| 6 | Color | `"color"` | 颜色选择器 | 调色 |
| 7 | FilePath | `"file_path"` | 文件/目录选择器 | LUT 路径 |
| 8 | Coordinate | `"coordinate"` | 经纬度输入 | GPS 坐标 |
| 9 | Slider | `"slider"` | 滑动条 | 降噪强度 |
| 10 | ComboSlider | `"combo_slider"` | 预设滑块 | 品质预设 |
| 11 | Expression | `"expression"` | 表达式编辑器 | 自适应参数 |
| 12 | Preset | `"preset"` | 预设管理器 | 预设方案 |
| 13 | Array | `"array"` | 动态列表 | 标签列表 |
| 14 | MapWidget | `"map_widget"` | 地图选取器 | GPS 位置 |
| 15 | BeforeAfter | `"before_after"` | 前后对比 | 预览 |
| 16 | Separator | `"separator"` | 分隔线 | 界面布局 |
| 17 | Section | `"section"` | 嵌套段落 | 界面组织 |

### 10.4 渲染意图 (RenderingIntent) 速查

| 意图 | 说明 | 适用场景 |
|---|---|---|
| `perceptual` | 保留视觉关系，允许牺牲色彩精度 | 摄影输出 |
| `relative_colorimetric` | 白点匹配 + 色域裁剪 | 精确再现 |
| `saturation` | 保留鲜艳度，色彩可偏移 | 图表、商务 |
| `absolute_colorimetric` | 精确色度再现，包含白点 | 打样 |

### 10.5 GpuBackend 枚举速查

| 后端 | 平台 | 说明 |
|---|---|---|
| `cuda` | Linux/Windows | NVIDIA GPU |
| `metal` | macOS | Apple GPU |
| `vulkan` | 全平台 | 跨平台 API |
| `directx` | Windows | DirectX GPU |
| `opencl` | 全平台 | 旧版跨平台 API |
| `rocm` | Linux | AMD GPU |
| `openvino` | 全平台 | Intel 推理 |
| `auto` | 全平台 | 自动选择最佳 |

### 10.6 AiBackend 枚举速查

| 后端 | 适用硬件 | 说明 |
|---|---|---|
| `onnx_cpu` | 仅 CPU | ONNX Runtime CPU 执行 |
| `onnx_cuda` | NVIDIA GPU | ONNX Runtime CUDA 执行 |
| `tensorrt` | NVIDIA GPU | TensorRT 优化推理 |
| `coreml` | Apple Silicon | CoreML + ANE |
| `openvino` | Intel CPU/GPU | OpenVINO 推理引擎 |

### 10.7 输出格式对比

| 格式 | 最大位深 | 压缩方式 | 透明度 | ICC 支持 | 推荐用途 |
|---|---|---|---|---|---|
| HEIF | 10 bit | x265 有损 | 否 | 是 | HDR 终端分发 |
| JXL | 16 bit (lossless) | 混合（有损+无损） | 是 | 是 | 归档、无损存储 |
| AVIF | 12 bit | AV1 有损 | 是 | 是 | Web 分发 |
| TIFF | 16 bit (packbits) | 无损 | 否 | 是 | 中间存档 |
| PNG | 16 bit (deflate) | 无损 | 是 | 是 | 网络分发、存档 |

---

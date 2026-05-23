# Photopipeline 用户手册

## 目录

1. [安装与配置](#1-安装与配置)
2. [核心概念](#2-核心概念)
3. [CLI 完整命令参考](#3-cli-完整命令参考)
4. [管线 TOML 格式详解](#4-管线-toml-格式详解)
5. [表达式语言参考](#5-表达式语言参考)
6. [逐图配置与分组规则](#6-逐图配置与分组规则)
7. [批量处理工作流](#7-批量处理工作流)
8. [常用管线示例](#8-常用管线示例)
9. [FAQ](#9-faq)

---

## 1. 安装与配置

### 1.1 环境要求

| 依赖 | 最低版本 | 用途 |
|------|:--------:|------|
| Rust | 1.90+ | 编译 Rust workspace |
| GCC / Clang | — | 编译 C/C++ 依赖 |
| CMake | 3.20+ | Halide、OIIO 构建 |
| pkg-config | — | 查找系统库 |
| libheif-dev | 1.12+ | HEIF/AVIF 编码 |
| libjxl-dev | 0.8+ | JPEG XL 编码 |
| liblcms2-dev | 2.0+ | 色彩管理 |
| exiftool | 12.00+ | 元数据读写（可选） |
| heif-enc | — | HEIF 编码 CLI（可选） |
| cjxl | — | JPEG XL 编码 CLI（可选） |

### 1.2 安装步骤

```bash
# Ubuntu / Debian
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl

# macOS
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool

# Windows (via vcpkg)
vcpkg install libheif libjxl lcms2

# 编译 Photopipeline
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
cargo build --release --workspace

# 安装到系统
cargo install --path .
```

---

## 2. 核心概念

### 2.1 管线（Pipeline）

管线是一个有向无环图（DAG），由节点（Node）和边（Edge）组成。每个节点绑定一个插件（Plugin），边定义数据流向。

### 2.2 插件类别

| 类别 | 说明 | 示例 |
|------|------|------|
| Input | 读取输入文件 | `raw_input` |
| Metadata | 读写元数据（不访问像素） | `exif_rw`、`gps_set`、`time_shift` |
| Color | 色彩转换与调色 | `colorspace`、`lut3d` |
| Transform | 几何变换 | `transform` |
| Enhance | 画质增强 | `lens_correct`、`ai_denoise` |
| Format | 格式编码/解码 | `heif_encoder`、`jxl_encoder` |
| External | 外部工具集成 | (预留) |

### 2.3 参数优先级

参数按四级优先级合并，高优先级覆盖低优先级：

```
Level 3: 图像覆盖 (image override)        ← 最高
Level 2: 分组覆盖 (group override)        ← 最后匹配者胜出
Level 1: 模板默认 (template default)
Level 0: 插件内置 (plugin builtin)       ← 最低
```

### 2.4 惰性像素处理

管线引擎在执行前检查每个节点的 `requires_pixel_access()`：
- 若整个管线仅包含 metadata 插件，则零像素分配
- 首个像素处理插件触发像素缓冲区分配
- 像素数据通过 Arc 共享，避免不必要拷贝

---

## 3. CLI 完整命令参考

### 3.1 全局选项

```
photopipeline [OPTIONS] <COMMAND>
```

### 3.2 子命令

#### `pipeline` — 管线操作

```bash
# 运行管线
photopipeline pipeline run -c <CONFIG> -i <INPUT> -o <OUTPUT>

    选项:
      -c, --config <CONFIG>   管线 TOML 配置文件路径
      -i, --input <INPUT>     输入图像文件路径
      -o, --output <OUTPUT>   输出文件路径

# 验证管线配置
photopipeline pipeline validate -c <CONFIG>

    选项:
      -c, --config <CONFIG>   管线 TOML 配置文件路径
```

#### `plugin` — 插件操作

```bash
# 列出所有已注册插件
photopipeline plugin list

# 查看插件详细信息
photopipeline plugin info <PLUGIN_ID>

    参数:
      <PLUGIN_ID>   插件 ID，例如 "photopipeline.plugins.ai_denoise"
```

输出示例：

```
Plugin: AI Denoise
  ID:           photopipeline.plugins.ai_denoise
  Version:      1.0.0
  Category:     enhance
  Description:  AI-powered image denoising using ONNX Runtime
  Tags:         ["ai", "denoise", "onnx", "machine-learning", "gpu", ...]
  Pixel access: true
  Min RAM:      2048 MB
```

#### `batch` — 批量操作

```bash
# 运行批量处理
photopipeline batch run -c <CONFIG> -p <PATTERN> -o <OUTPUT>

    选项:
      -c, --config <CONFIG>        管线 TOML 配置文件路径
      -p, --pattern <PATTERN>      文件匹配模式（glob）[默认: *.ARW]
      -o, --output <OUTPUT_DIR>   输出目录 [默认: ./output/]

# 验证批量管线
photopipeline batch validate -c <CONFIG> -p <PATTERN>

    选项:
      -c, --config <CONFIG>        管线 TOML 配置文件路径
      -p, --pattern <PATTERN>      文件匹配模式 [默认: *.ARW]
```

### 3.3 gRPC Server

```bash
# 启动 gRPC 服务端（监听 localhost:50051）
photopipeline-server
```

---

## 4. 管线 TOML 格式详解

### 4.1 完整结构

```toml
[metadata]          # 可选：管线元信息
name = "My Pipeline"
version = "1.0"
description = "描述文本"

[[nodes]]           # 必须：节点定义（至少一个）
id = "node_id"
plugin = "plugin.id"
label = "显示名称"
enabled = true       # 可选：默认 true
params = { ... }     # 可选：参数覆盖

[[edges]]           # 可选：边定义
from = "node_a"
to = "node_b"

[[overrides]]       # 可选：图像级覆盖
image = "filename.arw"
[overrides.params.node_id]
  param_name = value

[[groups]]          # 可选：分组规则
name = "Group Name"
condition = "exif.iso >= 1600"
[groups.params.node_id]
  param_name = value

[batch]             # 可选：批量处理配置
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

### 4.2 `[metadata]` 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|:--:|------|
| name | string | 否 | 管线名称 |
| version | string | 否 | 管线版本 |
| description | string | 否 | 管线描述 |

### 4.3 `[[nodes]]` 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|:--:|------|
| id | string | 是 | 节点唯一标识符（在管线内唯一） |
| plugin | string | 是 | 插件 ID（如 `photopipeline.plugins.colorspace`） |
| label | string | 否 | GUI 中显示的节点名称 |
| enabled | bool | 否 | 是否启用（默认 `true`） |
| params | table | 否 | 参数覆盖表（key = 参数ID, value = 参数值） |

### 4.4 `[[edges]]` 字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|:--:|------|
| from | string | 是 | 源节点 ID |
| to | string | 是 | 目标节点 ID |

### 4.5 各节点类型参数说明

#### RAW Input (`raw_input`)

```toml
[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"
params = {
  raw_mode = "auto",          # auto | dcraw | libraw | rawtherapee
  output_format = "u16",       # u16 | f32
  half_size = false,           # 半分辨率解码
  apply_white_balance = true,  # 应用相机白平衡
  dcraw_path = "dcraw",        # dcraw 二进制路径
  dcraw_extra_args = "",       # 附加 dcraw 参数
}
```

#### EXIF Reader/Writer (`exif_rw`)

```toml
[[nodes]]
id = "exif"
plugin = "photopipeline.plugins.exif_rw"
params = {
  read_exif = true,             # 读取 EXIF
  read_xmp = true,              # 读取 XMP
  read_iptc = true,             # 读取 IPTC
  read_gps = true,              # 读取 GPS
  overwrite_original = false,   # 覆盖原文件
  preserve_makernote = true,    # 保留 MakerNote
  write_exif = "all",           # all | selected | none
  exiftool_path = "exiftool",   # exiftool 路径
  exiftool_args = "",           # 附加参数
}
```

#### GPS Coordinate Manager (`gps_set`)

```toml
[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = {
  gps_mode = "manual",            # manual | gpx_track | clear
  latitude = 30.5728,             # 纬度（-90 ~ 90）
  longitude = 104.0668,           # 经度（-180 ~ 180）
  altitude = 500.0,               # 海拔（m）
  gpx_file = "track.gpx",         # GPX 轨迹文件
  time_offset_seconds = 0,        # 相机-GPS 时间偏移（秒）
  max_interpolation_gap = 300,    # 最大插值间隔（秒）
}
```

#### Time Shift (`time_shift`)

```toml
[[nodes]]
id = "shift"
plugin = "photopipeline.plugins.time_shift"
params = {
  shift_hours = 0,                # 偏移小时（-23 ~ 23）
  shift_minutes = 0,              # 偏移分钟（-59 ~ 59）
  shift_seconds = 0,              # 偏移秒（-59 ~ 59）
  source_timezone = "UTC",        # 源时区
  target_timezone = "local",      # 目标时区
  increment_per_image = 0.0,      # 每张图片递增秒数
  batch_image_index = 0,          # 批次中图片索引
}
```

#### Color Space (`colorspace`)

```toml
[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "auto",           # auto | srgb | display_p3 | adobe_rgb | pro_photo | bt2020 | aces_cg | linear_srgb
  target_color_space = "rec2020_pq",     # srgb | display_p3 | adobe_rgb | pro_photo | bt2020_pq | linear_srgb
  rendering_intent = "relative_colorimetric",  # relative_colorimetric | perceptual | saturation | absolute_colorimetric
  black_point_compensation = true,       # 黑点补偿
  gamut_mapping = "compress",            # clip | compress | luminance_preserve
  embed_icc = true,                      # 嵌入 ICC Profile
  icc_profile_path = "",                 # 自定义 ICC 文件路径
}
```

#### 3D LUT (`lut3d`)

```toml
[[nodes]]
id = "lut"
plugin = "photopipeline.plugins.lut3d"
params = {
  lut_path = "/path/to/lut.cube",        # LUT 文件路径（.cube / .3dl / .look / .csp）
  lut_format = "cube",                   # cube | 3dl | look | csp
  intensity = 100.0,                     # 混合强度（0-100%）
  input_color_space = "srgb",            # LUT 期望的输入色彩空间
  clamp_output = true,                   # 钳制输出到 0-1
  interpolation_method = "tetrahedral",  # trilinear | tetrahedral
}
```

#### Transform (`transform`)

```toml
[[nodes]]
id = "xform"
plugin = "photopipeline.plugins.transform"
params = {
  resize_mode = "long_edge",    # none | absolute | percentage | long_edge | short_edge | megapixels
  target_width = 1920,          # 目标宽度（px）
  target_height = 1080,         # 目标高度（px）
  scale_percent = 100.0,        # 缩放百分比
  long_edge_px = 2048,          # 长边目标像素
  angle = 0.0,                  # 旋转角度（-360 ~ 360）
  flip_horizontal = false,      # 水平翻转
  flip_vertical = false,        # 垂直翻转
  crop_enabled = false,         # 启用裁剪
  crop_x = 0,                   # 裁剪 X 起点
  crop_y = 0,                   # 裁剪 Y 起点
  crop_width = 1920,            # 裁剪宽度
  crop_height = 1080,           # 裁剪高度
  filter_type = "lanczos3",     # bilinear | lanczos3 | nearest
}
```

#### Lens Correction (`lens_correct`)

```toml
[[nodes]]
id = "lens"
plugin = "photopipeline.plugins.lens_correct"
params = {
  correction_mode = "auto",           # auto | manual | off
  correct_distortion = true,          # 校正畸变
  correct_tca = true,                 # 校正横向色差（TCA）
  correct_vignetting = true,          # 校正暗角
  correct_geometry = false,           # 独立几何校正
  lensfun_db_path = "/usr/share/lensfun",  # LensFun 数据库路径
  camera_make = "",                   # 手动指定相机制造商
  camera_model = "",                  # 手动指定相机型号
  lens_model = "",                    # 手动指定镜头型号
}
```

#### AI Denoise (`ai_denoise`)

```toml
[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = {
  denoise_model = "standard_v2",     # lightweight_v1 | standard_v2 | high_quality_v2 | raw_denoise_v1
  denoise_strength = 50.0,           # 降噪强度（0-100）
  detail_preservation = 50.0,        # 细节保留（0-100）
  color_noise_reduction = 75.0,      # 色彩噪声降低（0-100）
  ai_backend = "onnx_cpu",           # onnx_cpu | onnx_cuda | tensorrt | coreml | openvino
  tile_size = 0,                     # 分块大小（0=自动）
  use_fp16 = true,                   # 半精度推理
}
```

#### HEIF Encoder (`heif_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.heif_encoder"
params = {
  quality = 95.0,              # 品质（0-100）
  lossless = false,            # 无损模式
  bit_depth = "10",            # 位深：8 | 10
  chroma_subsampling = "444",  # 色度采样：444 | 422 | 420
  encoder_effort = 4,          # 编码器 effort（0-10）
  heif_enc_path = "heif-enc",  # heif-enc 二进制路径
}
```

#### JPEG XL Encoder (`jxl_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.jxl_encoder"
params = {
  quality = 90.0,              # 品质（0-100）；-1 = 无损
  lossless = false,            # 数学无损
  bit_depth = "16",            # 位深：8 | 10 | 12 | 16
  effort = 7,                  # effort（1-9）
  modular = false,             # 模块模式（用于合成图像）
  cjxl_path = "cjxl",          # cjxl 二进制路径
}
```

#### AVIF Encoder (`avif_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.avif_encoder"
params = {
  quality = 85.0,              # 品质（0-100）
  speed = 6,                   # 速度（0=慢/最佳, 10=快）
  bit_depth = "10",            # 位深：8 | 10 | 12
  chroma_subsampling = "444",  # 色度采样：444 | 422 | 420
  lossless = false,            # 无损
  avifenc_path = "avifenc",    # avifenc 二进制路径
}
```

#### TIFF Encoder (`tiff_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.tiff_encoder"
params = {
  compression = "none",        # none | lzw | deflate | packbits
  bigtiff = true,              # BigTIFF 格式（>4GB 文件）
  embed_icc = true,            # 嵌入 ICC Profile
  pixel_format = "u16",        # u8 | u16 | f32
}
```

#### PNG Encoder (`png_encoder`)

```toml
[[nodes]]
id = "output"
plugin = "photopipeline.plugins.png_encoder"
params = {
  compression_level = 6,       # Deflate 压缩级别（0=存储, 9=最佳）
  bit_depth = "16",            # 位深：8 | 16
  embed_icc = true,            # 嵌入 ICC Profile（iCCP chunk）
  include_exif = false,        # 包含 EXIF（eXIf chunk）
  color_type = "rgb",          # rgb | rgba | gray | graya
}
```

### 4.6 `[batch]` 配置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|:---:|------|
| parallel | int | 1 | 并行处理数量 |
| output_pattern | string | — | 输出路径模板，支持 `{date}` `{filename}` 等占位符 |
| on_conflict | string | — | 输出冲突处理策略：`skip` / `overwrite` |
| resume | bool | false | 是否支持断点续传 |

---

## 5. 表达式语言参考

### 5.1 语法

表达式包裹在 `${}` 中，支持：

```
${变量引用}
${比较表达式}
${条件 ? 真值 : 假值}
${字面数值}  ${'字符串'}  ${"字符串"}
```

### 5.2 变量

#### `exif.*` 命名空间

| 变量 | 类型 | 说明 | 示例值 |
|------|------|------|------|
| `exif.iso` | number | ISO 感光度 | 800 |
| `exif.aperture` | string | 光圈值 | 2.8 |
| `exif.shutter` | string | 快门速度 | 1/500 |
| `exif.focal_length` | string | 焦距 | 50mm |
| `exif.make` | string | 相机制造商 | Canon |
| `exif.model` | string | 相机型号 | EOS R5 |
| `exif.lens` | string | 镜头型号 | 24-70mm |

#### `image.*` 命名空间

| 变量 | 类型 | 说明 | 示例值 |
|------|------|------|------|
| `image.filename` | string | 文件名 | DSC0001.ARW |
| `image.width` | number | 像素宽度 | 6000 |
| `image.height` | number | 像素高度 | 4000 |
| `image.filesize` | number | 文件大小（字节） | 52428800 |

### 5.3 操作符

| 操作符 | 说明 |
|--------|------|
| `>` | 大于 |
| `<` | 小于 |
| `>=` | 大于等于 |
| `<=` | 小于等于 |
| `==` | 等于（数值比较：浮点 epsilon；字符串比较：精确） |
| `!=` | 不等于 |
| `? :` | 三元条件运算符（可嵌套） |

### 5.4 示例

```toml
# 根据 ISO 自动调整降噪强度
params = {
  denoise_strength = "${exif.iso > 1600 ? 90 : 50}"
}

# 根据机型选择不同配置
params = {
  output_format = "${exif.make == 'Sony' ? 'f32' : 'u16'}"
}

# 复合条件
params = {
  strength = "${exif.iso >= 800 && exif.iso <= 3200 ? 75 : ${exif.iso > 3200 ? 95 : 40}}"
}

# 条件分组
[[groups]]
condition = "${exif.iso >= 1600}"
```

---

## 6. 逐图配置与分组规则

### 6.1 图像级覆盖（`[[overrides]]`）

对特定图片应用不同于模板的参数：

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
```

字段说明：
- `image`：文件名（不含路径），与 glob 匹配到的 `file_name()` 比较
- `params.<node_id>`：该节点 ID 下的参数覆盖表，键为参数 ID

### 6.2 分组覆盖（`[[groups]]`）

根据条件规则对匹配的图片应用参数：

```toml
[[groups]]
name = "High ISO - Heavy Denoise"
condition = "exif.iso >= 1600"
[groups.params.ai_denoise]
  denoise_strength = 90
  detail_preservation = 30

[[groups]]
name = "Night Shots"
condition = "${exif.iso > 3200}"
[groups.params.ai_denoise]
  denoise_strength = 95
  denoise_model = "high_quality_v2"
```

字段说明：
- `name`：分组名称
- `condition`：条件表达式（支持 EXIF 字段比较与 `${}` 表达式）
- `params.<node_id>`：该组匹配时应用的参数

#### 条件类型

| 条件类型 | 格式 | 示例 |
|------|------|------|
| EXIF 等于 | `exif.<tag> = "<value>"` | `exif.make = "Canon"` |
| EXIF 大于等于 | `exif.<tag> >= <number>` | `exif.iso >= 1600` |
| EXIF 小于等于 | `exif.<tag> <= <number>` | `exif.iso <= 400` |
| GPS 邻近 | `gps_near(<lat>, <lon>, <radius_km>)` | `gps_near(30.57, 104.07, 10)` |
| 表达式 | `${表达式}` | `${exif.iso > 1600}` |
| 逻辑与 | `and(条件1, 条件2, ...)` | TOML 中通过多组实现 |
| 逻辑或 | `or(条件1, 条件2, ...)` | TOML 中通过多组实现 |

分组按定义顺序求值，后匹配者的参数覆盖先匹配者的参数。所有分组合并后再与图像覆盖合并（图像覆盖优先级最高）。

### 6.3 优先级交互示例

```toml
# 模板默认
[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = { denoise_strength = 50 }     # Level 1

# 分组覆盖
[[groups]]
condition = "exif.iso >= 1600"
[groups.params.ai_denoise]
  denoise_strength = 80                # Level 2
  detail_preservation = 30             # Level 2

[[groups]]
condition = "exif.iso >= 3200"
[groups.params.ai_denoise]
  denoise_strength = 95                # Level 2 (最后匹配)

# 图像覆盖
[[overrides]]
image = "DSC0003.ARW"
[overrides.params.ai_denoise]
  denoise_strength = 85                # Level 3 (最高)
  detail_preservation = 60             # Level 3
```

对于 `DSC0003.ARW`（ISO=6400）：最终 `denoise_strength=85`（图像覆盖胜出），`detail_preservation=60`（图像覆盖胜出）。

对于 `DSC0005.ARW`（ISO=6400）：分组覆盖生效，`denoise_strength=95`（ISO>=3200 组最后匹配），`detail_preservation=30`。

---

## 7. 批量处理工作流

### 7.1 完整工作流

```
1. 加载图片 → 自动提取 EXIF 快照
2. 加载 GPX 轨迹（可选）→ 按时间戳自动插值 GPS
3. 自动分组：按 ISO / GPS 聚类 / 时间间隔 → 应用分组预设
4. 选择单张图片 → 细调覆盖参数（按需）
5. 验证 → 检查参数完整性
6. 导出 → 并行处理，进度流，断点续传支持
```

### 7.2 批量处理示例

```bash
# 基本批量处理
photopipeline batch run \
  -c batch_pipeline.toml \
  -p "~/photos/2024/*.ARW" \
  -o "~/processed/"

# 输出结构
#   ~/processed/DSC0001.heif
#   ~/processed/DSC0002.heif
#   ...

# 使用输出模板
photopipeline batch run \
  -c batch_pipeline.toml \
  -p "~/photos/*.ARW" \
  -o "~/processed/{date}/{filename}.heif"
```

### 7.3 输出模式占位符

| 占位符 | 说明 |
|------|------|
| `{filename}` | 输入文件名（不含扩展名） |
| `{ext}` | 输入文件扩展名 |
| `{date}` | EXIF 日期（格式：YYYY-MM-DD） |
| `{camera}` | 相机型号（来自 EXIF） |
| `{iso}` | ISO 值 |

### 7.4 并行度建议

| 场景 | 推荐 `parallel` |
|------|:---:|
| 纯 metadata 管线 | CPU 核心数 × 2 |
| CPU 像素处理 | CPU 核心数 |
| GPU 像素处理 | GPU 并发数（通常 2-4） |
| 内存受限（<8GB） | 1-2 |

---

## 8. 常用管线示例

### 8.1 HDR 处理管线

```toml
[metadata]
name = "HDR Processing Pipeline"
version = "1.0"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"
params = { output_format = "f32", apply_white_balance = true }

[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = { gps_mode = "gpx_track", gpx_file = "track.gpx" }

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = {
  source_color_space = "srgb",
  target_color_space = "rec2020_pq",
  rendering_intent = "relative_colorimetric",
  black_point_compensation = true,
  gamut_mapping = "compress",
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
from = "source"
to = "gps"
[[edges]]
from = "gps"
to = "color"
[[edges]]
from = "color"
to = "denoise"
[[edges]]
from = "denoise"
to = "output"
```

### 8.2 降噪管线

```toml
[metadata]
name = "Advanced Denoise Pipeline"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"

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
from = "source"
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

### 8.3 GPS 标注管线

```toml
[metadata]
name = "GPS Tagging Pipeline"

[[nodes]]
id = "source"
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
params = { compression = "deflate", pixel_format = "u16" }

[[edges]]
from = "source"
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

### 8.4 色彩转换管线

```toml
[metadata]
name = "Color Space Conversion Pipeline"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"

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
from = "source"
to = "color"
[[edges]]
from = "color"
to = "lut"
[[edges]]
from = "lut"
to = "output"
```

### 8.5 缩略图批量管线

```toml
[metadata]
name = "Thumbnail Generator"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"
params = { half_size = true }

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
params = { quality = 80, bit_depth = "10" }

[[edges]]
from = "source"
to = "xform"
[[edges]]
from = "xform"
to = "output"

[batch]
parallel = 4
output_pattern = "thumbnails/{filename}.heif"
on_conflict = "skip"
```

---

## 9. FAQ

### Q: 我的图片没有 EXIF 数据，表达式会报错吗？

不会。缺少 EXIF 数据时，`exif.*` 变量返回默认值（数值返回 `0`，字符串返回空字符串），表达式继续执行。分组规则中的 EXIF 条件在无 EXIF 数据时返回 `false`，分组不匹配。

### Q: 如何让管线跳过某些图片？

使用 `enabled = false` 禁用节点，或使用分组规则 + `skip` 策略排除不想要的图片。也可以在 `[batch]` 中设置 `on_conflict = "skip"` 跳过已存在的输出文件。

### Q: 管线中的节点可以并行执行吗？

当前执行引擎按拓扑排序顺序执行。不互相依赖的节点理论上可并行，完整并行执行将在后续版本中实现。

### Q: TIFF 编码器支持 EXR 格式吗？

TIFF 编码器仅支持经典 TIFF 和 BigTIFF。OpenEXR 格式请使用支持的编码器（计划中）。目前可通过 `photopipeline.oiio` 的 FFI 绑定（feature-gated）进行 EXR 编解码。

### Q: AI 降噪模型从哪里下载？

内置的 `standard_v2` 模型需要从 HuggingFace（`photopipeline/denoise-standard-v2`）下载。插件在首次使用时检测模型文件是否存在，若不存在则提示下载 URL。将 `.onnx` 文件放入 `models/` 目录即可。

### Q: 如何处理 RAW + JPEG 配对？

b>目前管线一次处理一个文件。对于 RAW + JPEG 配对，可以在 `[[overrides]]` 中对 RAW 文件指定特殊参数（如使用 `raw_input`），对 JPEG 文件使用不同的输入插件或跳过 raw decode。完整配对支持计划中。

### Q: 支持什么 RAW 格式？

`raw_input` 插件：Sony (.arw)、Canon (.cr2/.cr3)、Nikon (.nef)、Adobe DNG (.dng)、Fujifilm (.raf)、Olympus (.orf)、Panasonic (.rw2)、Pentax (.pef)、Hasselblad (.3fr)、Mamiya (.mef)、Leaf (.mos)、Epson (.erf)。

### Q: 如何验证管线配置是否正确？

```bash
photopipeline pipeline validate -c my_pipeline.toml
```

成功则输出节点数和边数。失败则报告具体错误。

### Q: GPU 支持需要额外配置吗？

不需要。`colorspace` 和 `lut3d` 插件自动尝试使用 GPU（`preferred_backend = Auto`），`ai_denoise` 插件在参数中指定 `ai_backend`。无 GPU 时自动回退到 CPU。

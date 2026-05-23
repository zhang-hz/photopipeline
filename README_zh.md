# Photopipeline — 超高精度图像后处理管线

## 项目简介

Photopipeline 是一个跨平台（C++ + Rust）、16bit+ 精度、插件化架构的图像后处理管线引擎。支持 **Windows / macOS / Linux** 三大平台。核心管线全程保持 16bit+ 精度，通过插件化设计实现按需像素处理与惰性计算，适用于专业摄影后期、批量图像处理及色彩管理工作流。

| 维度 | 设计 |
|------|------|
| 语言 | Rust workspace（核心引擎 + CLI + Server + Linux GUI） |
| 计算管线 | Halide（CPU SIMD + GPU）/ 纯 Rust 后备 |
| 图像 I/O | OpenImageIO + 系统原生库（libheif / libjxl / lcms2） |
| 元数据 | ExifTool 子进程（标准）+ builtin parser（轻量） |
| 色彩管理 | LittleCMS2 + OpenColorIO（VFX 级） |
| GUI ↔ Server | gRPC + protobuf（localhost） |
| 像素格式 | u16 / f32，始终 ≥16bit |
| 主力输出 | JXL 16bit（libjxl effort=7-9）+ HEIF 10bit（x265 veryslow 444 grain） |
| 插件加载 | Native .so/.dll + WASM + ExternalTool + Builtin + Remote |

---

## 核心特性

### 1. 16bit+ 全管线精度

管线中的每个节点始终保持 ≥16bit 整数或 32bit 浮点数精度，绝不因格式转换而丢失数据。主力输出格式支持 16bit JXL（visually lossless）与 10bit HEIF（x265 veryslow 444），默认启用 444 色度采样以避免色度信息损失。

### 2. 插件化架构（14+ 内置插件）

所有功能由插件提供，统一通过 `Plugin` 基础 Trait 管理。内置 14 个插件覆盖 Input、Metadata、Color、Transform、Enhance、Format 六大类别。支持五种插件加载方式：编译内置、动态库、WASM 沙箱、外部工具子进程、远程市场。

### 3. 惰性像素处理（metadata-only 操作零拷贝）

Metadata 类插件（exif_rw、gps_set、time_shift）不访问像素数据，仅读写 Arc 共享的元数据。当管线中仅包含 metadata 插件时，零像素内存分配，零拷贝，处理速度极快。

### 4. 批量处理 + 逐图差异化配置

支持对数百张图片执行相同管线，同时允许对单张图片及分组应用不同的参数覆盖。输出模式支持按日期/文件名自动组织。

### 5. 四级参数优先级

```
image override     (优先级 3, 最高)
  └> group override  (优先级 2, 最后匹配者胜出)
      └> template default (优先级 1)
          └> plugin builtin default (优先级 0, 最低)
```

### 6. GPU 硬件加速

支持 CUDA / Metal / Vulkan / DirectX / OpenCL / ROCm / OpenVINO 等 GPU 后端。AI 降噪插件支持 ONNX / TensorRT / CoreML / OpenVINO 推理后端。

### 7. 主力格式

- **HEIF 10-bit**：x265 编码器，veryslow preset，CRF 18，444 色度采样，film grain 调优
- **JXL 16-bit**：libjxl 编码器，effort 7-9，distance 0.5（visual lossless）

### 8. 表达式引擎

支持在参数值中使用条件表达式：

```
${exif.iso > 1600 ? 0.9 : 0.4}
```

支持的变量命名空间：`exif.*`、`image.*`，支持比较运算符（`>` `<` `>=` `<=` `==` `!=`）和三元运算。

---

## 快速开始

### 环境要求

| 依赖 | 最低版本 |
|------|:--------:|
| Rust | 1.90+ |
| GCC / Clang | — |
| CMake | 3.20+ |
| pkg-config | — |
| libheif-dev | 1.12+ |
| libjxl-dev | 0.8+ |
| liblcms2-dev | 2.0+ |

### 安装

```bash
# 从 crates.io 安装（待发布）
cargo install photopipeline

# 从源码构建
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
cargo build --release --workspace

# 二进制文件
#   target/release/photopipeline      — CLI 主程序
#   target/release/photopipeline-server — gRPC 服务端
```

### 基本使用

#### CLI 命令示例

```bash
# 列出所有已注册插件
photopipeline plugin list

# 查看插件详细信息
photopipeline plugin info photopipeline.plugins.ai_denoise

# 运行单个管线
photopipeline pipeline run \
  -c examples/hdr_pipeline.toml \
  -i DSC0001.ARW \
  -o output/DSC0001.heif

# 验证管线配置
photopipeline pipeline validate -c examples/hdr_pipeline.toml

# 批量处理
photopipeline batch run \
  -c examples/hdr_pipeline.toml \
  -p "*.ARW" \
  -o ./output/

# 验证批量管线
photopipeline batch validate \
  -c examples/hdr_pipeline.toml \
  -p "*.ARW"
```

#### 管线 TOML 配置示例

```toml
[metadata]
name = "HDR Processing Pipeline"
version = "1.0"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"

[[nodes]]
id = "gps"
plugin = "photopipeline.plugins.gps_set"
params = { mode = "manual" }

[[nodes]]
id = "color"
plugin = "photopipeline.plugins.colorspace"
params = { source_color_space = "srgb", target_color_space = "rec2020_pq" }

[[nodes]]
id = "denoise"
plugin = "photopipeline.plugins.ai_denoise"
params = { denoise_strength = 85 }

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

#### 批量处理示例

```toml
# 追加到管线 TOML 中

# 逐图覆盖
[[overrides]]
image = "DSC0003.ARW"
[overrides.params.gps]
  lat = 30.5728
  lon = 104.0668

# 自动分组规则
[[groups]]
name = "High ISO"
condition = "exif.iso >= 1600"
[groups.params.ai_denoise]
  denoise_strength = 90
  detail_preservation = 30

# 批量配置
[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

---

## 架构概览

### 三层架构

```
┌──────────────────────────────────────────────────────┐
│ GUI Layer (platform-native, gRPC client)             │
│  Windows: WinUI 3 (.NET 8)                           │
│  macOS:   SwiftUI                                    │
│  Linux:   GTK4 + Rust                                │
├──────────────────────────────────────────────────────┤
│ Server Layer (Rust, localhost:50051)                  │
│  PipelineExecutor · PluginRegistry · BatchScheduler  │
│  ParameterResolver · ProgressBroker · TileEngine     │
├──────────────────────────────────────────────────────┤
│ Compute Layer                                        │
│  Halide kernels | OIIO | libheif | libjxl | lcms2    │
│  ExifTool subprocess | 商业API stubs                 │
└──────────────────────────────────────────────────────┘
```

### 数据流与精度保证

```
Metadata plugins:   Arc<Metadata> 共享，0拷贝
Metadata → Metadata:  同一 Arc，始终共享
Metadata → Pixel插件:  仅在写入时触发 COW
Pixel → Pixel (单消费者):  Arc 不加写时复制，原地修改
Pixel → Pixel (多消费者):  Arc 共享，只读
GPU → GPU:  通过 GpuHandle 传递，数据留在 VRAM 直到编码
```

### Tile 分块处理

```
4096×2160 f32 RGBA = 135MB/帧
分割: 256×256 tile，每个约 1MB
并行: 16 个 tile 并发（Rayon/GPU）
优势: 降低峰值 VRAM 占用，缓存友好
```

---

## 插件列表

| # | 插件 ID | 名称 | 类别 | 像素访问 | 后端/工具 | 描述 |
|:--:|-----|------|------|:--:|------|------|
| 1 | `exif_rw` | EXIF Reader/Writer | Metadata | ✗ | ExifTool + kamadak-exif | 读写 EXIF/XMP/IPTC/GPS 元数据 |
| 2 | `gps_set` | GPS Coordinate Manager | Metadata | ✗ | ExifTool + geo crate | 手动设置或从 GPX 轨迹插值 GPS 坐标 |
| 3 | `time_shift` | Time Shift | Metadata | ✗ | chrono + ExifTool | 调整拍摄时间戳及时区转换 |
| 4 | `colorspace` | Color Space | Color | ✓ | Halide + lcms2 | 色彩空间转换，ICC 配置，渲染意图 |
| 5 | `lut3d` | 3D LUT | Color | ✓ | Halide | 3D 查找表色彩调色与电影仿真 |
| 6 | `transform` | Transform | Transform | ✓ | Halide | 缩放、旋转、裁剪、翻转（含多种滤波器） |
| 7 | `lens_correct` | Lens Correction | Enhance | ✓ | LensFun + Halide | 镜头畸变/色差/暗角校正 |
| 8 | `ai_denoise` | AI Denoise | Enhance | ✓ | ONNX Runtime | AI 降噪，支持 ONNX/TensorRT/CoreML |
| 9 | `raw_input` | RAW Input | Input | ✗* | dcraw / LibRaw | 读取 RAW 格式（ARW/CR2/CR3/NEF/DNG 等） |
| 10 | `heif_encoder` | HEIF Encoder | Format | ✗ | libheif + x265 | HEIF/HEIC 10-bit 编码 |
| 11 | `jxl_encoder` | JPEG XL Encoder | Format | ✗ | libjxl / cjxl | JPEG XL 16-bit 编码（支持无损） |
| 12 | `avif_encoder` | AVIF Encoder | Format | ✗ | libheif + aom | AVIF（AV1）编码 |
| 13 | `tiff_encoder` | TIFF Encoder | Format | ✗ | OIIO / 内置 | TIFF/BigTIFF 编码（多种压缩） |
| 14 | `png_encoder` | PNG Encoder | Format | ✗ | lodepng / 内置 | PNG 16-bit 编码（含 ICC 嵌入） |

> \* raw_input 产生像素输出但不消费上游像素，metadata 类插件不访问像素数据。

### 编码器品质推荐

| 格式 | 编码器 | 设置 | 品质 |
|------|------|------|:--:|
| HEIF 10-bit | x265 | preset=veryslow, crf=18, 444, tune=grain | ★★★★★ |
| HEIF 10-bit (GPU) | NVENC | Turing+, b-frames, 10bit | ★★★★ |
| HEIF 10-bit (Mac) | VideoToolbox | Apple Silicon HW | ★★★★ |
| JXL 16-bit | libjxl | effort=7-9, distance=0.5（视觉无损） | ★★★★★ |
| JXL lossless | libjxl | effort=7-9, distance=0 | ★★★★★（完美） |

---

## 项目结构

```
photopipeline/
├── ARCHITECTURE.md           # 架构设计文档（英文）
├── ARCHITECTURE_zh.md        # 架构设计文档（中文）
├── README_zh.md              # 本文档
├── USER_GUIDE.md             # 用户手册
├── PLUGIN_DEV.md             # 插件开发指南
├── API_REFERENCE.md          # API 参考
├── CHANGELOG.md              # 变更日志
├── Cargo.toml                # workspace 根
├── justfile                   # 任务运行器（just）
├── .github/
│   └── workflows/
│       ├── build-halide.yml
│       ├── build-rust.yml
│       └── release.yml
├── crates/
│   ├── core/                 # 共享类型
│   ├── plugin/               # 插件 Trait + Registry + Loader
│   ├── engine/               # 管线 DAG + 执行引擎
│   ├── plugins/              # 所有内置插件
│   ├── external/             # 外部工具封装
│   ├── server/               # gRPC 服务端
│   └── oiio/                 # OIIO FFI 绑定（feature-gated）
├── cli/                      # CLI 二进制
├── proto/                    # Protobuf 定义
├── halide_generators/        # Halide 生成器源文件（C++）
├── examples/                 # 管线配置示例
└── gui/
    ├── linux/                # GTK4 + Rust
    ├── windows/              # WinUI 3 (.NET 8)
    └── macos/                # SwiftUI
```

---

## 开发阶段

| Phase | 名称 | 目标 | 产出 |
|:---:|------|------|------|
| 0 | 设计文档 | 规划 | ARCHITECTURE.md |
| 1 | 环境搭建 | 配置 | Rust, dev libs, Git repo, CI 脚手架 |
| 2 | Core Crate | 类型定义 | ImageBuffer, Metadata, ColorSpace, Error |
| 3 | Plugin System | 框架 | Plugin trait, Registry, Loader, Schema |
| 4 | Pipeline Engine | 运行时 | DAG, Executor, ParameterResolver, TileEngine |
| 5 | Builtin Plugins | 功能实现 | 14 个内置插件 |
| 6 | External Tools | 集成 | ExifTool, libvips, 商业 API stubs |
| 7 | CLI | 前端 | 子命令, batch, TOML 管线配置 |
| 8 | gRPC Server | 后端 | Proto 定义, 服务实现, 流式传输 |
| 9 | Halide / OIIO | 计算层 | 生成器文件, FFI, CI 编译脚本 |
| 10 | GUI Linux | 桌面端 | GTK4 管线编辑器 + 预览 + 批量 |
| 11 | GUI Windows | 桌面端 | WinUI 3 项目, gRPC 客户端 |
| 12 | GUI macOS | 桌面端 | SwiftUI 项目, gRPC 客户端 |
| 13 | CI/CD | DevOps | 全平台 GitHub Actions 矩阵, 发布 |
| 14 | 验证 | 测试 | cargo build, lint, test |

---

## 许可证

MIT License — 详见仓库根目录 LICENSE 文件。

# Photopipeline

<p align="center">
  <strong>16 位以上精度 · 跨平台图像处理管线引擎</strong>
</p>

<p align="center">
  <a href="https://github.com/zhang-hz/photopipeline/actions"><img src="https://img.shields.io/github/actions/workflow/status/zhang-hz/photopipeline/build-rust.yml?branch=main&label=CI" alt="CI 状态"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT 许可证"/></a>
  <a href="https://crates.io/crates/photopipeline"><img src="https://img.shields.io/crates/v/photopipeline?color=orange" alt="Crates.io"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.90%2B-orange.svg" alt="Rust 1.90+"/></a>
</p>

---

## 为什么选择 Photopipeline

专业图像工作流对精度有着严苛的要求——传统 8 位管线在面对色调映射、色彩空间转换和
格式转码时会产生累积舍入误差，导致暗部细节丢失、渐变色阶断裂。

Photopipeline 从根本上解决了这一问题：**整条处理管线保证 16 位整数或 32 位浮点
精度**，从 RAW 解码到最终编码，不存在任何隐蔽的位深度降级或意外截断。如果你的相机
记录 14 位 RAW 数据，Photopipeline 能将每一个比特的信息完整保留至色彩调色、
降噪、镜头校正和最终输出。

管线基于**惰性求值 + 写时复制**的内存模型构建：纯元数据操作（GPS 标注、EXIF 调整）
完全不分配像素内存；像素数据通过 `Arc` 智能指针共享，仅在节点真正写入时才触发
复制。结合分块并行处理和 GPU 驻留计算，这一架构显著降低了 CPU 和 GPU 的内存压力。

最后，Photopipeline 天然支持**可扩展插件架构**。所有 14 个内置插件与第三方插件
使用相同的公开 Trait 体系；GUI 面板由插件 Schema 自动生成——新增插件无需编写任何
前端代码即可无缝集成到桌面应用中。

---

## 核心特性

| 特性 | 说明 | 状态 |
|---|---|---|
| **端到端 16 位精度** | 全管线 u16/f32；零隐蔽截断 | 稳定 |
| **插件化架构** | 6 种能力 Trait、Schema 驱动 GUI、5 种加载方式 | 稳定 |
| **惰性像素处理** | 纯元数据操作零像素内存消耗 | 稳定 |
| **四级参数优先级** | 图像覆盖 > 分组覆盖 > 模板默认 > 插件默认 | 稳定 |
| **GPU 硬件加速** | CUDA / Metal / Vulkan / DirectX / OpenCL / ROCm / OpenVINO | Beta |
| **AI 推理后端** | ONNX Runtime / TensorRT / CoreML / OpenVINO / Burn | Beta |
| **分块并行处理** | 256–1024 像素分块；重叠支持；内存高效 | 稳定 |
| **表达式引擎** | 参数中支持 `${exif.iso > 1600 ? 0.9 : 0.4}` | 稳定 |
| **批量处理** | Glob 模式、逐图覆盖、自动分组、断点续传 | 稳定 |
| **TOML 管线配置** | 可读管线定义 + 完整校验 | 稳定 |
| **gRPC 服务端** | 流式 RPC：Execute / Decode / Encode / Progress | Beta |
| **跨平台桌面 GUI** | WinUI 3 (Windows) / SwiftUI (macOS) / GTK4+Rust (Linux) | Alpha |
| **14 个内置插件** | EXIF / GPS / 时移 / 色彩空间 / 3DLUT / 变换 / 镜头校正 / 降噪 / HEIF / JXL / AVIF / TIFF / PNG / RAW | 稳定 |
| **编码器品质梯度** | x265 veryslow 444 / libjxl effort=7–9 / 视觉无损输出 | 稳定 |

---

## 快速开始

### 环境要求

| 依赖 | 最低版本 | 用途 |
|---|---|---|
| Rust | 1.90+ | 编译 Rust workspace |
| CMake | 3.20+ | Halide / OIIO 构建（仅 CI） |
| pkg-config | — | 系统库检测 |
| libheif-dev | 1.12+ | HEIF / AVIF 支持 |
| libjxl-dev | 0.8+ | JPEG XL 支持 |
| liblcms2-dev | 2.0+ | 色彩管理 |
| exiftool | 12.00+ | 元数据读写（可选） |

### 安装

```bash
# 从 crates.io 安装
cargo install photopipeline

# 从源码构建
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
cargo build --release --workspace

# 验证安装
photopipeline --help
photopipeline plugin list
```

### 平台依赖安装

```bash
# Ubuntu / Debian
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl

# macOS (Homebrew)
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool

# Windows (vcpkg)
vcpkg install libheif libjxl lcms2
```

### Hello World 管线

创建 `hello.toml`：

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

---

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                     GUI 层（平台原生）                             │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────┐    │
│  │  WinUI 3     │   │  SwiftUI     │   │  GTK4 + Rust     │    │
│  │  (.NET 8)    │   │  (macOS)     │   │  (Linux)         │    │
│  └──────┬───────┘   └──────┬───────┘   └───────┬──────────┘    │
│         │                  │                    │                │
│         └──────────────────┼────────────────────┘                │
│                   gRPC (localhost:50051)                         │
├─────────────────────────────────────────────────────────────────┤
│                    服务端层（Rust）                                │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐      │
│  │ PipelineExec │  │PluginRegistry│  │  BatchScheduler   │      │
│  └──────┬───────┘  └──────┬───────┘  └────────┬──────────┘      │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌────────┴──────────┐      │
│  │ParamResolver │  │ProgressBroker│  │    TileEngine     │      │
│  └──────────────┘  └──────────────┘  └───────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                    计算层                                         │
│  ┌──────────────┐  ┌──────┐  ┌────────┐  ┌───────┐  ┌──────┐   │
│  │Halide Kernels│  │ OIIO │  │libheif │  │libjxl │  │lcms2 │   │
│  └──────────────┘  └──────┘  └────────┘  └───────┘  └──────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ExifTool (子) │  │原生编解码器   │  │商业 API 存根         │    │
│  └──────────────┘  └──────────────┘  └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 数据流与零拷贝保证

```
Metadata 插件           → Arc<Metadata> 共享，零拷贝
Metadata → Metadata     → 始终共享同一 Arc
Metadata → Pixel 插件   → 单消费者写入时触发写时复制
Pixel → Pixel（单消费者）→ Arc 不重复，原地修改
Pixel → Pixel（多消费者）→ Arc 共享，只读
GPU → GPU               → GpuHandle 传递，数据留在 VRAM 直到编码
```

### 分块处理

```
4096 × 2160 × f32 × RGBA = 135 MB/帧
分割：256 × 256 分块，每块约 1 MB
并行：最多 16 个分块并发（Rayon / GPU 线程组）
优势：降低峰值 VRAM 占用、CPU 缓存友好、可中断
```

---

## 管线配置

管线使用 TOML 格式定义。有效管线至少需要一个 `[[nodes]]` 条目。

### 顶层配置段参考

| 配置段 | 必填 | 用途 |
|---|---|---|
| `[metadata]` | 否 | 管线名称、版本、描述 |
| `[[nodes]]` | 是 | 节点定义（至少一个） |
| `[[edges]]` | 否 | 节点间有向边 |
| `[[overrides]]` | 否 | 逐图参数覆盖 |
| `[[groups]]` | 否 | 条件分组覆盖 |
| `[batch]` | 否 | 批量处理配置 |

### 表达式语言

参数支持由 `${ }` 包裹的行内表达式：

```
${exif.iso > 1600 ? 0.9 : 0.4}
```

**变量：** `exif.iso`、`exif.aperture`、`exif.shutter`、`exif.focal_length`、`exif.make`、`exif.model`、`exif.lens`、`image.filename`、`image.width`、`image.height`、`image.filesize`

**运算符：** `>` `<` `>=` `<=` `==` `!=` `? :`

---

## 插件列表

全部 14 个内置插件与第三方插件使用相同的公开 Trait 基础设施。

| # | 插件 ID | 名称 | 类别 | 像素? | 后端 / 工具 | 品质 |
|:--:|---|---|:--:|:--:|---|---|
| 1 | `exif_rw` | EXIF 读写 | Metadata | | ExifTool + kamadak-exif | ★★★★★ |
| 2 | `gps_set` | GPS 坐标设置 | Metadata | | ExifTool + geo crate | ★★★★★ |
| 3 | `time_shift` | 时间偏移 | Metadata | | chrono + ExifTool | ★★★★★ |
| 4 | `colorspace` | 色彩空间转换 | Color | ✓ | Halide + lcms2 | ★★★★★ |
| 5 | `lut3d` | 3D LUT | Color | ✓ | Halide | ★★★★★ |
| 6 | `transform` | 几何变换 | Transform | ✓ | Halide | ★★★★★ |
| 7 | `lens_correct` | 镜头校正 | Enhance | ✓ | LensFun + Halide | ★★★★ |
| 8 | `ai_denoise` | AI 降噪 | Enhance | ✓ | ONNX / TensorRT / CoreML | ★★★★ |
| 9 | `raw_input` | RAW 输入 | Input | | dcraw / LibRaw | ★★★★★ |
| 10 | `heif_encoder` | HEIF 编码器 | Format | | libheif + x265 | ★★★★★ |
| 11 | `jxl_encoder` | JPEG XL 编码器 | Format | | libjxl | ★★★★★ |
| 12 | `avif_encoder` | AVIF 编码器 | Format | | libheif + aom | ★★★★ |
| 13 | `tiff_encoder` | TIFF 编码器 | Format | | OIIO | ★★★★★ |
| 14 | `png_encoder` | PNG 编码器 | Format | | lodepng | ★★★★★ |

### 编码器品质推荐

| 格式 | 编码器 | 设置 | 品质 |
|---|---|---|---|
| HEIF 10 位 | x265 | preset=veryslow, crf=18, 444, tune=grain | ★★★★★ |
| HEIF 10 位 (GPU) | NVENC | Turing+, b-frames, 10-bit | ★★★★ |
| HEIF 10 位 (Mac) | VideoToolbox | Apple Silicon HW | ★★★★ |
| JXL 16 位 | libjxl | effort=7–9, distance=0.5（视觉无损） | ★★★★★ |
| JXL 无损 | libjxl | effort=7–9, distance=0 | ★★★★★ |

---

## 性能

在 AMD Ryzen 5950X / 64 GB RAM / NVIDIA RTX 4090 上，处理 6000×4000 RAW 图片的基准测试：

| 管线 | 时间 (s) | 峰值内存 (MB) | 显存 (MB) | 吞吐量 |
|---|---|---|---|---|
| 纯元数据（GPS 标注） | 0.3 | 8 | 0 | 3.3 张/秒 |
| RAW → JXL 16 位 | 2.1 | 512 | 0 | 0.48 张/秒 |
| RAW → HEIF 10 位 (CPU) | 4.7 | 1024 | 0 | 0.21 张/秒 |
| RAW → HEIF 10 位 (GPU) | 1.2 | 256 | 512 | 0.83 张/秒 |
| RAW → 降噪 → JXL (GPU) | 2.8 | 384 | 768 | 0.35 张/秒 |
| 批量 100 张（元数据） | 3.1 | 32 | 0 | 32.3 张/秒 |

---

## 文档

| 文档 | 语言 | 内容 |
|---|---|---|
| [README.md](README.md) | English | 项目主文档（英文版） |
| [用户指南](USER_GUIDE.md) | 中文 | 完整用户手册：安装、CLI、管线配置、表达式、批量处理 |
| [插件开发](PLUGIN_DEV.md) | 中文 | 插件开发指南：Trait 参考、Schema 定义、完整教程 |
| [API 参考](API_REFERENCE.md) | 中文 | 按 Crate 组织的完整 API 参考 |
| [架构设计](ARCHITECTURE.md) | English | 架构设计文档 |
| [架构设计](ARCHITECTURE_zh.md) | 中文 | 架构设计文档（中文版） |
| [变更日志](CHANGELOG.md) | English | 版本变更历史 |
| [贡献指南](CONTRIBUTING.md) | English | 贡献指南 |

---

## 项目结构

```
photopipeline/
├── crates/
│   ├── core/            # 共享类型：ImageBuffer、Metadata、ColorSpace、Error
│   ├── plugin/          # 插件 Trait + Registry + Loader + Schema
│   ├── engine/          # 管线 DAG + Executor + ParameterResolver + TileEngine
│   ├── plugins/         # 全部 14 个内置插件
│   ├── external/        # 外部工具封装（ExifTool、libvips、商业 API 存根）
│   └── oiio/            # OIIO FFI 绑定（feature-gated）
├── cli/                 # CLI 二进制（基于 clap）
├── proto/               # Protobuf 服务定义
├── halide_generators/   # Halide C++ 生成器源文件（在 CI 上编译）
├── examples/            # 管线 TOML 配置示例
├── gui/
│   ├── linux/           # GTK4 + Rust
│   ├── windows/         # WinUI 3 (.NET 8)
│   └── macos/           # SwiftUI
├── .github/workflows/   # CI/CD 流水线定义
├── Cargo.toml           # Workspace 根
├── justfile             # 任务运行器
├── README.md            # 英文主文档
├── README_zh.md         # 本文档
├── LICENSE              # MIT 许可证
└── CHANGELOG.md         # 版本历史
```

---

## 贡献

Photopipeline 欢迎社区贡献。详见 [CONTRIBUTING.md](CONTRIBUTING.md)：

- 开发环境搭建
- 编码规范
- Pull Request 流程
- 测试要求
- 发布流程

所有 Rust 代码在合入前必须通过 `cargo clippy -- -D warnings` 和 `cargo fmt --all -- --check`。

---

## 许可证

Photopipeline 基于 [MIT 许可证](LICENSE)。

Copyright (c) 2024–2026 Photopipeline Contributors.

---

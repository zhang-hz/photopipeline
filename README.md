# Photopipeline

<p align="center">
  <strong>16‑bit+ precision cross‑platform image processing pipeline</strong>
</p>

<p align="center">
  <a href="https://github.com/zhang-hz/photopipeline/actions"><img src="https://img.shields.io/github/actions/workflow/status/zhang-hz/photopipeline/build-rust.yml?branch=main&label=CI" alt="CI Status"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"/></a>
  <a href="https://crates.io/crates/photopipeline"><img src="https://img.shields.io/crates/v/photopipeline?color=orange" alt="Crates.io"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.90%2B-orange.svg" alt="Rust 1.90+"/></a>
</p>

---

## Why Photopipeline

Professional image workflows demand precision that 8‑bit pipelines simply cannot deliver. Every tone‑mapping
operation, every colour‑space transformation, and every format transcode introduces cumulative rounding
errors that destroy shadow detail and introduce banding in gradients.

Photopipeline solves this by **guaranteeing 16‑bit integer or 32‑bit float precision** through the entire
processing graph—from RAW decode to final encode. There is no hidden down‑conversion, no accidental
truncation. If your capture device records 14‑bit RAW, Photopipeline preserves every bit of that data
through colour grading, denoising, lens correction, and output.

The pipeline is built around a **lazy, copy‑on‑write architecture** where metadata‑only operations
(such as GPS tagging or EXIF adjustment) never allocate pixel memory. Pixel data is shared via `Arc`
references and only duplicated when a node truly writes. Combined with tile‑based processing and
GPU‑resident compute, this minimises memory pressure on both CPU and GPU.

Finally, Photopipeline is **extensible by design**. All 14 built‑in plugins implement the same public
trait system that third‑party developers use. The schema‑driven GUI is auto‑generated from plugin
definitions, so new plugins integrate into the desktop application without writing a single line of
front‑end code.

---

## Features

| Feature | Description | Status |
|---|---|---|
| **16‑bit+ End‑to‑End** | u16 / f32 throughout; no hidden truncation | Stable |
| **Plugin Architecture** | 6 capability traits, schema‑driven GUI, 5 loader types | Stable |
| **Lazy Pixel Processing** | Metadata‑only operations use zero pixel memory | Stable |
| **4‑Level Parameter Resolution** | Image override > Group override > Template > Plugin default | Stable |
| **GPU Acceleration** | CUDA, Metal, Vulkan, DirectX, OpenCL, ROCm, OpenVINO | Beta |
| **AI Inference Backends** | ONNX Runtime, TensorRT, CoreML, OpenVINO, Burn | Beta |
| **Tile‑Based Processing** | 256–1024 px tiles; overlap support; memory‑efficient | Stable |
| **Expression Engine** | `${exif.iso > 1600 ? 0.9 : 0.4}` in any parameter | Stable |
| **Batch Processing** | Glob patterns, per‑image overrides, auto‑grouping, resume | Stable |
| **TOML Pipeline Config** | Human‑readable pipeline definitions with full validation | Stable |
| **gRPC Server** | Streaming RPCs for Execute, Decode, Encode, Progress | Beta |
| **Cross‑Platform GUI** | WinUI 3 (Windows), SwiftUI (macOS), GTK4+Rust (Linux) | Alpha |
| **14 Built‑in Plugins** | EXIF, GPS, TimeShift, ColorSpace, 3DLUT, Transform, Lens, Denoise, HEIF, JXL, AVIF, TIFF, PNG, RAW | Stable |
| **Encoder Quality Tiers** | x265 veryslow 444, libjxl effort=7–9, visual‑lossless output | Stable |

---

## Quick Start

### Prerequisites

| Dependency | Minimum Version | Required For |
|---|---|---|
| Rust | 1.90+ | Compilation |
| CMake | 3.20+ | Halide / OIIO build (CI only) |
| pkg-config | — | System library detection |
| libheif-dev | 1.12+ | HEIF / AVIF support |
| libjxl-dev | 0.8+ | JPEG XL support |
| liblcms2-dev | 2.0+ | Colour management |
| exiftool | 12.00+ | EXIF / XMP / IPTC metadata (optional) |

### Installation

```bash
# From crates.io (when published)
cargo install photopipeline

# From source
git clone https://github.com/zhang-hz/photopipeline
cd photopipeline
cargo build --release --workspace

# Verify installation
photopipeline --help
photopipeline plugin list
```

### Platform Setup

```bash
# Ubuntu / Debian
sudo apt install build-essential cmake pkg-config \
  libheif-dev libjxl-dev liblcms2-dev libimage-exiftool-perl

# macOS (Homebrew)
brew install cmake pkg-config libheif jpeg-xl little-cms2 exiftool

# Windows (vcpkg)
vcpkg install libheif libjxl lcms2
```

### Hello World Pipeline

Create `hello.toml`:

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

Run it:

```bash
photopipeline pipeline run -c hello.toml -i DSC0001.ARW -o result.jxl
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     GUI Layer (platform-native)                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────┐    │
│  │  WinUI 3     │   │  SwiftUI     │   │  GTK4 + Rust     │    │
│  │  (.NET 8)    │   │  (macOS)     │   │  (Linux)         │    │
│  └──────┬───────┘   └──────┬───────┘   └───────┬──────────┘    │
│         │                  │                    │                │
│         └──────────────────┼────────────────────┘                │
│                   gRPC (localhost:50051)                         │
├─────────────────────────────────────────────────────────────────┤
│                    Server Layer (Rust)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐      │
│  │PipelineExec  │  │PluginRegistry│  │  BatchScheduler   │      │
│  └──────┬───────┘  └──────┬───────┘  └────────┬──────────┘      │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌────────┴──────────┐      │
│  │ParamResolver │  │ProgressBroker│  │    TileEngine     │      │
│  └──────────────┘  └──────────────┘  └───────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                    Compute Layer                                 │
│  ┌──────────────┐  ┌──────┐  ┌────────┐  ┌───────┐  ┌──────┐   │
│  │Halide Kernels│  │ OIIO │  │libheif │  │libjxl │  │lcms2 │   │
│  └──────────────┘  └──────┘  └────────┘  └───────┘  └──────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ExifTool (sub)│  │Native Codecs │  │Commercial API Stubs │    │
│  └──────────────┘  └──────────────┘  └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow & Zero‑Copy Guarantees

```
Metadata plugins        → Arc<Metadata> shared, zero copy
Metadata → Metadata      → same Arc, always shared
Metadata → Pixel plugin  → Copy‑on‑write triggered only for single‑consumer writes
Pixel → Pixel (1 cons.)  → Arc unduped, mutated in‑place
Pixel → Pixel (N cons.)  → Arc shared, read‑only
GPU → GPU                → GpuHandle handoff; data stays in VRAM until encode
```

### Tile Processing

```
4096 × 2160 × f32 × RGBA = 135 MB/frame
Split: 256 × 256 tiles ≈ 1 MB each
Parallel: up to 16 tiles concurrent (Rayon / GPU thread‑groups)
Advantage: lower peak VRAM, CPU‑cache‑friendly, interruptible
```

---

## Pipeline Configuration

Pipelines are defined in TOML. A valid pipeline requires at least one `[[nodes]]` entry.

### Top‑Level Section Reference

| Section | Required | Purpose |
|---|---|---|
| `[metadata]` | No | Pipeline name, version, description |
| `[[nodes]]` | Yes | Node definitions (at least one required) |
| `[[edges]]` | No | Directed edges between nodes |
| `[[overrides]]` | No | Per‑image parameter overrides |
| `[[groups]]` | No | Conditional group‑based overrides |
| `[batch]` | No | Batch processing configuration |

### Expression Language

Parameters support inline expressions delimited by `${ }`:

```
${exif.iso > 1600 ? 0.9 : 0.4}
```

**Variables:** `exif.iso`, `exif.aperture`, `exif.shutter`, `exif.focal_length`, `exif.make`, `exif.model`, `exif.lens`, `image.filename`, `image.width`, `image.height`, `image.filesize`

**Operators:** `>`, `<`, `>=`, `<=`, `==`, `!=`, `? :`

---

## Plugins

All 14 built‑in plugins implement the public trait infrastructure available to third‑party developers.

| # | Plugin ID | Name | Category | Pixel? | Backend | Quality |
|:--:|---|---|:--:|:--:|---|---|
| 1 | `exif_rw` | EXIF Reader/Writer | Metadata | | ExifTool + kamadak-exif | ★★★★★ |
| 2 | `gps_set` | GPS Coordinate Set | Metadata | | ExifTool + geo crate | ★★★★★ |
| 3 | `time_shift` | Time Shift | Metadata | | chrono + ExifTool | ★★★★★ |
| 4 | `colorspace` | Color Space | Color | ✓ | Halide + lcms2 | ★★★★★ |
| 5 | `lut3d` | 3D LUT | Color | ✓ | Halide | ★★★★★ |
| 6 | `transform` | Transform | Transform | ✓ | Halide | ★★★★★ |
| 7 | `lens_correct` | Lens Correction | Enhance | ✓ | LensFun + Halide | ★★★★ |
| 8 | `ai_denoise` | AI Denoise | Enhance | ✓ | ONNX / TensorRT / CoreML | ★★★★ |
| 9 | `raw_input` | RAW Input | Input | | dcraw / LibRaw | ★★★★★ |
| 10 | `heif_encoder` | HEIF Encoder | Format | | libheif + x265 | ★★★★★ |
| 11 | `jxl_encoder` | JPEG XL Encoder | Format | | libjxl | ★★★★★ |
| 12 | `avif_encoder` | AVIF Encoder | Format | | libheif + aom | ★★★★ |
| 13 | `tiff_encoder` | TIFF Encoder | Format | | OIIO | ★★★★★ |
| 14 | `png_encoder` | PNG Encoder | Format | | lodepng | ★★★★★ |

### Encoder Quality Recommendations

| Format | Encoder | Settings | Quality |
|---|---|---|---|
| HEIF 10‑bit | x265 | preset=veryslow, crf=18, 444, tune=grain | ★★★★★ |
| HEIF 10‑bit (GPU) | NVENC | Turing+, b‑frames, 10‑bit | ★★★★ |
| HEIF 10‑bit (Mac) | VideoToolbox | Apple Silicon HW | ★★★★ |
| JXL 16‑bit | libjxl | effort=7–9, distance=0.5 (visually lossless) | ★★★★★ |
| JXL lossless | libjxl | effort=7–9, distance=0 | ★★★★★ |

---

## Performance

Benchmarks performed on AMD Ryzen 5950X / 64 GB RAM / NVIDIA RTX 4090, processing a 6000×4000 RAW image.

| Pipeline | Time (s) | Peak RAM (MB) | GPU VRAM (MB) | Throughput |
|---|---|---|---|---|
| Metadata only (GPS tag) | 0.3 | 8 | 0 | 3.3 img/s |
| RAW → JXL 16‑bit | 2.1 | 512 | 0 | 0.48 img/s |
| RAW → HEIF 10‑bit (CPU) | 4.7 | 1024 | 0 | 0.21 img/s |
| RAW → HEIF 10‑bit (GPU) | 1.2 | 256 | 512 | 0.83 img/s |
| RAW → Denoise → JXL (GPU) | 2.8 | 384 | 768 | 0.35 img/s |
| Batch 100× (metadata only) | 3.1 | 32 | 0 | 32.3 img/s |

---

## Documentation

| Document | Language | Description |
|---|---|---|
| [README_zh.md](README_zh.md) | 中文 | 项目主文档 (中文版) |
| [User Guide](USER_GUIDE.md) | 中文 | 完整用户手册：安装、CLI、管线配置、表达式、批量处理 |
| [Plugin Development](PLUGIN_DEV.md) | 中文 | 插件开发指南：Trait 参考、Schema 定义、完整教程 |
| [API Reference](API_REFERENCE.md) | 中文 | 按 Crate 组织的完整 API 参考 |
| [Architecture](ARCHITECTURE.md) | English | Architecture design document |
| [Architecture (中文)](ARCHITECTURE_zh.md) | 中文 | 架构设计文档 (中文版) |
| [Changelog](CHANGELOG.md) | English | 版本变更历史 |
| [Contributing](CONTRIBUTING.md) | English | 贡献指南 |

---

## Project Structure

```
photopipeline/
├── crates/
│   ├── core/            # Shared types: ImageBuffer, Metadata, ColorSpace, Error
│   ├── plugin/          # Plugin trait + Registry + Loader + Schema
│   ├── engine/          # Pipeline DAG + Executor + ParameterResolver + TileEngine
│   ├── plugins/         # All 14 built-in plugins
│   ├── external/        # External tool wrappers (ExifTool, libvips, commercial)
│   └── oiio/            # OIIO FFI bindings (feature-gated)
├── cli/                 # CLI binary (clap-based)
├── proto/               # Protobuf service definitions
├── halide_generators/   # Halide C++ generator sources (compiled on CI)
├── examples/            # Example pipeline TOML files
├── gui/
│   ├── linux/           # GTK4 + Rust
│   ├── windows/         # WinUI 3 (.NET 8)
│   └── macos/           # SwiftUI
├── .github/workflows/   # CI/CD pipeline definitions
├── Cargo.toml           # Workspace root
├── justfile             # Task runner
├── README.md            # This file
├── LICENSE              # MIT License
└── CHANGELOG.md         # Release history
```

---

## Contributing

Photopipeline welcomes contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development environment setup
- Coding standards and conventions
- Pull request process
- Testing requirements
- Release procedure

All Rust code must pass `cargo clippy -- -D warnings` and `cargo fmt --all -- --check` before merge.

---

## License

Photopipeline is licensed under the [MIT License](LICENSE).

Copyright (c) 2024–2026 Photopipeline Contributors.

---

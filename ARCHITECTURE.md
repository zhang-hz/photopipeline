# Photopipeline — 架构设计文档 v1.0

## 项目概览

超高精度跨平台图像后处理应用。核心管道16bit+精度，plugins按需访问像素。

| 维度 | 设计 |
|------|------|
| 语言 | Rust workspace (核心引擎+CLI+Server+Linux GUI) |
| 计算管线 | Halide (CPU SIMD+GPU) / 纯Rust后备 |
| 图像 I/O | OpenImageIO + 系统原生库(libheif/libjxl/lcms2) |
| 元数据 | ExifTool (标准) + builtin parser (轻量) |
| 色彩管理 | LittleCMS2 + OpenColorIO (VFX级) |
| GUI↔Server | gRPC + protobuf (localhost) |
| 像素格式 | u16 / f32, >=16bit 始终 |
| 主力输出 | JXL 16bit (libjxl effort=7-9) + HEIF 10bit (x265 veryslow 444 grain) |
| 插件加载 | Native .so/.dll + WASM + ExternalTool + Builtin + Remote |

---

## 1. 环境评估与编译策略

### 1.1 开发环境

| 资源 | 值 | 备注 |
|------|-----|------|
| CPU | i5-1145G7 4C/8T 2.6GHz | Rust编译可行 |
| RAM | 3.3GB (可用836MB) | 无法本地编译Halide/OIIO/等大型C++项目 |
| Disk | 14GB | 勉强足够 |
| GPU | 无本地GPU | GPU测试在用户Windows/macOS机器 |
| 系统 | Ubuntu Linux x86_64 | |
| 系统库(已装) | libheif 1.21, libjxl 0.11, liblcms2 2.17 | 仅运行时,需dev包 |
| Rust | not installed | 需1.90+ |

### 1.2 编译分工

| 组件 | 位置 | 理由 |
|------|:--:|------|
| Rust workspace (core/cli/server/linux gui) | 本地 cargo build | 4C/8T足够 |
| Halide generators (C++) | GitHub Actions | 编译需8GB+ RAM, 本地内存不足 |
| OIIO (C++) | GitHub Actions | 同上,依赖繁多 |
| protobuf .proto → .rs | 本地 | 轻量 |
| Windows GUI (WinUI 3) | GitHub Actions | 本地无Windows |
| macOS GUI (SwiftUI) | GitHub Actions | 本地无macOS |
| GPU测试 | 用户本地机器 | 本地无GPU |
| system dev libs | 本地 apt install | 预编译秒装 |

---

## 2. 核心架构 (三层分离)

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

### 2.1 gRPC Services

```protobuf
service PipelineService {
  rpc CreatePipeline(PipelineSpec) returns (PipelineId);
  rpc Execute(ExecuteRequest) returns (stream ExecuteProgress);
  rpc Validate(PipelineSpec) returns (ValidationResult);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}

service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);
  rpc Decode(ImagePath) returns (stream PixelDataChunk);
  rpc GetThumbnail(ImagePath) returns (ImageData);
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);
}

service BatchService {
  rpc SubmitBatch(BatchSpec) returns (BatchId);
  rpc GetProgress(BatchId) returns (stream BatchProgress);
  rpc Cancel(BatchId) returns (Empty);
}
```

---

## 3. Plugin Architecture

### 3.1 Trait Hierarchy

```
                       ┌──────────┐
                       │  Plugin   │  base trait (all plugins)
                       └─────┬────┘
          ┌──────────┐  ┌────┴────┐  ┌──────────────┐
          │Metadata  │  │  Pixel   │  │   Format     │
          │Processor │  │Processor │  │  Processor   │
          └──────────┘  └────┬────┘  └──────────────┘
                       ┌─────┴─────┐
                       │GpuProcessor│  ← extension
                       └─────┬─────┘
                       ┌─────┴─────┐
                       │AiProcessor │  ← extension
                       └────────────┘
          ┌───────────────┐
          │ExternalTool   │  ← passthrough
          │Processor      │
          └───────────────┘
```

### 3.2 Base Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Any + Debug {
    fn id(&self)                        -> &PluginId;
    fn name(&self)                      -> &str;
    fn version(&self)                   -> PluginVersion;
    fn category(&self)                  -> PluginCategory;
    fn description(&self)               -> &str;
    fn tags(&self)                      -> &[String];
    fn requires_pixel_access(&self)     -> bool;   // false = never touch pixels
    fn produces_pixel_output(&self)     -> bool;

    fn parameter_schema(&self)          -> &ParameterSchema;
    fn gui_schema(&self)                -> &GuiSchema;

    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self)                         -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet)
        -> PluginResult<Vec<ValidationIssue>>;
}
```

### 3.3 Capability Traits

```rust
// metadata_only, zero pixel access
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;
    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet)
        -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet)
        -> PluginResult<MetadataWriteReport>;
}

// pixel processing, 16bit+, GPU optional
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self)  -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self)   -> Vec<ColorSpace>;
    fn required_gpu_backend(&self)     -> Option<GpuBackend>;
    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>
    ) -> PluginResult<ProcessingStats>;
}

// encode/decode
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn format_id(&self)                       -> ImageFormat;
    fn supported_extensions(&self)            -> Vec<(&str, &str)>;
    fn can_decode(&self, data: &FormatProbe)  -> bool;
    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn decode(&self, data: &[u8], opts: &DecodeOptions)
        -> PluginResult<DecodedImage>;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata, opts: &EncodeOptions)
        -> PluginResult<Vec<u8>>;
}

// GPU compute
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;
    async fn process_gpu(
        &self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>
    ) -> PluginResult<ProcessingStats>;
}

// AI / ONNX
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;
    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor>;
}

// External tool passthrough
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;
    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(&self, input: &[PathBuf], output: &PathBuf, params: &ParameterSet)
        -> PluginResult<()>;
}
```

### 3.4 Plugin Category enum

```rust
pub enum PluginCategory {
    Input,
    Metadata,
    Color,
    Transform,
    Enhance,
    Merge,
    Format,
    External,
    Custom(String),
}
```

### 3.5 Plugin Loaders

| Loader | 格式 | 热重载 | 用途 |
|--------|------|:--:|------|
| Builtin | 编译进binary | ✗ | 核心始终可用的plugin |
| Native | .so/.dll/.dylib | ✗ | 高性能第三方plugin |
| WASM | .wasm | ✓ | 安全沙箱第三方plugin |
| ExternalTool | 子进程调用 | ✗ | ExifTool/ffmpeg等外部工具 |
| Remote | URL下载安装 | ✗ | 插件市场分发 |

---

## 4. Parameter System (Schema-Driven)

### 4.1 Four-Level Priority

```
image override     (priority 3, highest)
  └> group override  (priority 2, last-matching wins)
      └> template default (priority 1)
          └> plugin builtin default (priority 0, lowest)
```

### 4.2 Schema Types

```rust
pub enum ParameterType {
    String { max_length, pattern, placeholder },
    Integer { min, max, step, unit, style: IntegerWidget },
    Float   { min, max, step, precision, unit, logarithmic, style: FloatWidget },
    Boolean { label_true, label_false },
    Enum    { options: Vec<EnumOption>, display: EnumDisplay },
    Color   { mode: ColorMode, show_alpha },
    FilePath{ kind, filters, must_exist },
    Coordinate { alt_required, direction_required },
    Slider  { min, max, step, show_ticks, ticks, style },
    ComboSlider { min, max, step, presets, unit },
    Expression { variables, return_type },
    Preset  { preset_schema_ref, builtin_presets, allow_custom },
    Array   { element_schema, min_items, max_items },
    MapWidget { show_track, show_photos, allow_manual_pin },
    BeforeAfterPreview { zoom_levels, show_histogram },
    Separator{ label },
    Section  { fields },
}
```

### 4.3 GUI Schema

```rust
pub struct GuiSchema {
    pub layout:          GuiLayout,
    pub icon:            Option<String>,
    pub color:           Option<String>,
    pub preview:         PreviewMode,
    pub aux_views:       Vec<AuxView>,
    pub min_panel_width: u32,
}
```

---

## 5. Data Flow & Precision

### 5.1 ImageBuffer (COW, lazy allocation)

```rust
pub struct ImageBuffer {
    pub metadata:     Arc<RwLock<Metadata>>,
    // None = metadata-only mode, no pixel memory allocated
    pixel_data:       Option<Arc<PixelData>>,
    pub pixel_format: PixelFormat,
    pub color_space:  ColorSpace,
    pub icc_profile:  Option<Arc<Vec<u8>>>,
}

pub struct PixelData {
    pub buffer: AlignedBuffer,   // page-aligned for GPU mapping
    pub width:  u32,
    pub height: u32,
    pub layout: ChannelLayout,
}
```

### 5.2 Zero-Copy Pass

```
Metadata plugins:   Arc<Metadata> shared, 0 copy
Metadata → Metadata:  same Arc, always
Metadata → Pixel plugin:  COW triggered only on write by single consumer
Pixel → Pixel (single consumer):  Arc unduped, mutate in-place
Pixel → Pixel (multi consumer):   Arc shared, read-only
GPU → GPU:  GpuHandle passed, data stays in VRAM until encode
```

### 5.3 Tile Processing

```
4096×2160 f32 RGBA = 135MB/frame
Split: 256×256 tiles, ~1MB each
Parallel: 16 tiles concurrent (Rayon/GPU)
Advantage: lower peak VRAM, cache-friendly
```

---

## 6. Pipeline Engine

### 6.1 DAG Model

```rust
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<(PortId, PortId)>,
}

pub struct PipelineNode {
    pub id:        NodeId,
    pub label:     String,
    pub plugin_id: PluginId,
    pub enabled:   bool,
    pub position:  (f64, f64),
    pub inputs:    Vec<PortId>,
    pub outputs:   Vec<PortId>,
}
```

### 6.2 Execution Strategy

1. Parse DAG → topological sort
2. For each node (parallel where possible):
   a. Check `requires_pixel_access()` — skip PixelBuffer if false
   b. `ParameterResolver::resolve(image, node)` → merge 4-level params
   c. Evaluate expressions in params
   d. Call `process_*()` with resolved params
   e. Collect ProcessingStats
3. Lazy pixel allocation: only allocate when first pixel-affecting plugin runs
4. Tile splitting for large images

---

## 7. Plugin Catalog

### 7.1 Built-in Plugins

| # | ID | Category | Pixel Access? | Backend |
|---|-----|----------|:--:|------|
| 1 | exif_rw | Metadata | ✗ | ExifTool + kamadak-exif |
| 2 | xmp_iptc | Metadata | ✗ | ExifTool |
| 3 | gps_set | Metadata | ✗ | ExifTool + geo crate |
| 4 | time_shift | Metadata | ✗ | chrono + ExifTool |
| 5 | colorspace_convert | Color | ✓ | Halide + lcms2 |
| 6 | lut_3d | Color | ✓ | Halide |
| 7 | resize | Transform | ✓ | Halide |
| 8 | rotate | Transform | ✓ | Halide |
| 9 | crop | Transform | ✓ | Halide |
| 10 | lens_correct | Correct | ✓ | LensFun + Halide |
| 11 | ai_denoise | Enhance | ✓ | ONNX Runtime |
| 12 | heif_encoder | Format | ✓ | libheif + x265 |
| 13 | jxl_encoder | Format | ✓ | libjxl |
| 14 | avif_encoder | Format | ✓ | libheif + aom |
| 15 | tiff_encoder | Format | ✓ | OIIO |
| 16 | png_encoder | Format | ✓ | lodepng |
| 17 | exr_encoder | Format | ✓ | OIIO |

### 7.2 Encoder Quality Recommendations

| Format | Encoder | Settings | Quality |
|------|------|------|:--:|
| HEIF 10-bit | x265 | preset=veryslow, crf=18, 444, tune=grain | ★★★★★ |
| HEIF 10-bit (GPU) | NVENC | Turing+, b-frames, 10bit | ★★★★ |
| HEIF 10-bit (Mac) | VideoToolbox | Apple Silicon HW | ★★★★ |
| JXL 16-bit | libjxl | effort=7-9, distance=0.5(visually lossless) | ★★★★★ |
| JXL lossless | libjxl | effort=7-9, distance=0 | ★★★★★ (perfect) |

---

## 8. GUI Design

### 8.1 Main Layout (3 pane)

```
┌──────────────┬──────────────────────────┬──────────────────────┐
│  FILMSTRIP   │        PREVIEW           │    PLUGIN CONTROLS   │
│  (thumbnail  │   Before │ After         │  (full detail per    │
│   list +     │   split/compare           │   selected node,     │
│   group tree)│                           │   inherits schema)  │
│              ├──────────────────────────┤                      │
│  BATCH       │   PIPELINE OVERVIEW      │                      │
│  PROGRESS    │   mini DAG + status      │                      │
│              ├──────────────────────────┤                      │
│  [▶ Start]  │   INFO BAR               │                      │
└──────────────┴──────────────────────────┴──────────────────────┘
```

### 8.2 Per-Plugin GUI (schema-driven)

Each plugin panel includes:
- Context bar (Template / Group / Image override toggle)
- Full parameter controls from ParameterSchema
- Override status badges per field (🟡override / ⬜inherited)
- Expression editor when supports_expression=true
- Preview area from GuiSchema
- Aux views from GuiSchema (histogram, map, waveform, etc.)

### 8.3 Plugin Panels (see ARCHITECTURE.md appendix for full designs)

Detailed designs exist for: Source, EXIF Metadata, GPS Set, Time Shift,
Color Space, 3D LUT, Transform (resize/rotate/crop), Lens Correction,
AI Denoise, HEIF 10-bit Encoder, JXL 16-bit Encoder.

---

## 9. Batch Processing

### 9.1 Workflow

1. Load images → auto-extract EXIF snapshots
2. Load GPX track (optional) → auto-interpolate GPS by timestamp
3. Auto-group: by ISO, GPS cluster, time gap → apply group presets
4. Select individual images → fine-tune overrides if needed
5. Validate → check parameter completeness
6. Export → parallel processing, progress stream, resume support

### 9.2 Configuration File (TOML)

```toml
[metadata]
name = "My HDR Pipeline"
version = "1.0"

[[nodes]]
id = "source"
plugin = "input"

[[nodes]]
id = "gps"
plugin = "gps_set"
params = { mode = "gpx_interpolate", track = "track.gpx" }

[[nodes]]
id = "color"
plugin = "colorspace_convert"
params = { to = "rec2020_pq", intent = "relative_colorimetric" }

[[nodes]]
id = "output"
plugin = "heif_encoder"
params = { encoder = "x265", crf = 18, chroma = "444", bit_depth = 10 }

[[edges]]
from = "source"
to = "gps"
[[edges]]
from = "gps"
to = "color"
[[edges]]
from = "color"
to = "output"

# per-image overrides (only diff from template)
[[overrides]]
image = "DSC0003.ARW"
params.gps = { lat = 30.5728, lon = 104.0668 }

# auto grouping
[[groups]]
name = "High ISO"
condition = "exif.iso >= 1600"
params.ai_denoise = { strength = 0.9 }

[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

---

## 10. CI/CD Strategy

### 10.1 GitHub Actions Workflows

```
┌─────────────────────────────────────────┐
│ build-halide.yml                        │
│  matrix: [linux, windows, macos]        │
│  output: halide artifacts per platform  │
├─────────────────────────────────────────┤
│ build-oiio.yml                          │
│  matrix: [linux, windows, macos]        │
│  output: oiio artifacts per platform    │
├─────────────────────────────────────────┤
│ build-rust.yml                          │
│  matrix: [linux-x86_64, linux-aarch64,  │
│           windows-x86_64, macos-arm64,  │
│           macos-x86_64]                 │
│  setup-rust + apt/choco/brew deps       │
│  cargo build --workspace                │
├─────────────────────────────────────────┤
│ build-gui-windows.yml (dotnet build)    │
│ build-gui-macos.yml (xcodebuild)        │
├─────────────────────────────────────────┤
│ release.yml                             │
│  needs: [build-*, test]                 │
│  pack: AppImage / MSIX / DMG            │
│  upload to GitHub Releases              │
└─────────────────────────────────────────┘
```

### 10.2 Local Development Workflow

```
Developer (this machine, 4C/8T, 3.3GB):
  1. write Rust code
  2. cargo build (workspace, ~2min incremental)
  3. cargo test (local)
  4. git push → GitHub Actions handles:
     - Halide/OIIO compilation
     - Cross-platform builds
     - GUI packaging
```

---

## 11. Project Structure

```
photopipeline/
├── ARCHITECTURE.md           # this document
├── Cargo.toml                # workspace root
├── justfile                   # task runner (just)
├── .github/
│   └── workflows/
│       ├── build-halide.yml
│       ├── build-oiio.yml
│       ├── build-rust.yml
│       ├── build-gui.yml
│       └── release.yml
├── crates/
│   ├── core/                 # Shared types
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── image.rs
│   │       ├── color.rs
│   │       ├── metadata.rs
│   │       ├── error.rs
│   │       └── types.rs
│   ├── plugin/               # Plugin trait + registry + loaders
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── trait.rs
│   │       ├── registry.rs
│   │       ├── loader.rs
│   │       ├── schema.rs
│   │       └── gui_schema.rs
│   ├── engine/               # Pipeline DAG + execution
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs
│   │       ├── executor.rs
│   │       ├── params.rs
│   │       └── tile.rs
│   ├── plugins/              # All built-in plugins
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── exif.rs
│   │       ├── gps.rs
│   │       ├── time_shift.rs
│   │       ├── colorspace.rs
│   │       ├── lut3d.rs
│   │       ├── transform.rs
│   │       ├── lens_correct.rs
│   │       ├── ai_denoise.rs
│   │       ├── heif_encoder.rs
│   │       └── jxl_encoder.rs
│   ├── external/             # External tool wrappers
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── exiftool.rs
│   │       ├── libvips.rs
│   │       └── commercial.rs
│   └── server/               # gRPC server
│       └── src/
│           ├── lib.rs
│           ├── main.rs
│           └── services/
├── cli/                      # CLI binary
│   └── src/
│       ├── main.rs
│       └── commands/
├── proto/                    # Protobuf definitions
│   ├── pipeline.proto
│   ├── image.proto
│   └── batch.proto
├── halide_generators/        # Halide source (compiled on CI)
│   ├── colorspace_generator.cpp
│   ├── resize_generator.cpp
│   └── CMakeLists.txt
└── gui/
    ├── linux/                # GTK4 + Rust
    ├── windows/              # WinUI 3 (.NET 8)
    └── macos/                # SwiftUI
```

---

## 12. Development Phases

| Phase | Name | Target | Output |
|:---:|------|------|------|
| 0 | Design Doc | Plan | ARCHITECTURE.md |
| 1 | Environment | Setup | Rust, dev libs, Git init, CI scaffold |
| 2 | Core Crate | Types | ImageBuffer, Metadata, ColorSpace, Error |
| 3 | Plugin System | Framework | Plugin trait, Registry, Loader, Schema |
| 4 | Pipeline Engine | Runtime | DAG, Executor, ParameterResolver, TileEngine |
| 5 | Builtin Plugins | Functionality | All 17 plugins |
| 6 | External Tools | Integration | ExifTool, libvips, commercial stubs |
| 7 | CLI | Frontend | Subcommands, batch, TOML pipeline |
| 8 | gRPC Server | Backend | Proto defs, service impls, streaming |
| 9 | Halide/OIIO | Compute | Generator files, FFI, CI compile scripts |
| 10 | GUI Linux | Desktop | GTK4 pipeline editor + preview + batch |
| 11 | GUI Windows | Desktop | WinUI 3 project, gRPC client |
| 12 | GUI macOS | Desktop | SwiftUI project, gRPC client |
| 13 | CI/CD | DevOps | Full GitHub Actions matrix, release |
| 14 | Verify | Test | cargo build, lint, test |

---

*End of ARCHITECTURE.md*

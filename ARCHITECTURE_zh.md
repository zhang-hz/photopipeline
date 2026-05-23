# Photopipeline — 架构设计文档 v1.0

## 项目概览

超高精度跨平台图像后处理应用。核心管线 16bit+ 精度，插件按需访问像素。

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

## 1. 环境评估与编译策略

### 1.1 开发环境

| 资源 | 值 | 备注 |
|------|-----|------|
| CPU | i5-1145G7 4C/8T 2.6GHz | Rust 编译可行 |
| RAM | 3.3GB（可用 836MB） | 无法本地编译 Halide/OIIO 等大型 C++ 项目 |
| Disk | 14GB | 勉强足够 |
| GPU | 无本地 GPU | GPU 测试在用户 Windows/macOS 机器 |
| 系统 | Ubuntu Linux x86_64 | |
| 系统库（已装） | libheif 1.21, libjxl 0.11, liblcms2 2.17 | 仅运行时，需 dev 包 |
| Rust | 未安装 | 需 1.90+ |

### 1.2 编译分工

| 组件 | 位置 | 理由 |
|------|:--:|------|
| Rust workspace（core/cli/server/linux gui） | 本地 cargo build | 4C/8T 足够 |
| Halide generators（C++） | GitHub Actions | 编译需 8GB+ RAM，本地内存不足 |
| OIIO（C++） | GitHub Actions | 同上，依赖繁多 |
| protobuf .proto → .rs | 本地 | 轻量 |
| Windows GUI（WinUI 3） | GitHub Actions | 本地无 Windows |
| macOS GUI（SwiftUI） | GitHub Actions | 本地无 macOS |
| GPU 测试 | 用户本地机器 | 本地无 GPU |
| system dev libs | 本地 apt install | 预编译秒装 |

---

## 2. 核心架构（三层分离）

```
┌──────────────────────────────────────────────────────┐
│ GUI Layer（平台原生，gRPC 客户端）                    │
│  Windows: WinUI 3 (.NET 8)                           │
│  macOS:   SwiftUI                                    │
│  Linux:   GTK4 + Rust                                │
├──────────────────────────────────────────────────────┤
│ Server Layer（Rust，localhost:50051）                  │
│  PipelineExecutor · PluginRegistry · BatchScheduler  │
│  ParameterResolver · ProgressBroker · TileEngine     │
├──────────────────────────────────────────────────────┤
│ Compute Layer                                        │
│  Halide kernels | OIIO | libheif | libjxl | lcms2    │
│  ExifTool subprocess | 商业 API stubs                │
└──────────────────────────────────────────────────────┘
```

### 2.1 gRPC 服务

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

## 3. 插件架构

### 3.1 Trait 层次结构

```
                        ┌──────────┐
                        │  Plugin   │  基础 trait（所有插件）
                        └─────┬────┘
           ┌──────────┐  ┌────┴────┐  ┌──────────────┐
           │Metadata  │  │  Pixel   │  │   Format     │
           │Processor │  │Processor │  │  Processor   │
           └──────────┘  └────┬────┘  └──────────────┘
                        ┌─────┴─────┐
                        │GpuProcessor│  ← 扩展
                        └─────┬─────┘
                        ┌─────┴─────┐
                        │AiProcessor │  ← 扩展
                        └────────────┘
           ┌───────────────┐
           │ExternalTool   │  ← 透传
           │Processor      │
           └───────────────┘
```

### 3.2 基础 Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Any + Debug {
    fn id(&self)                        -> &PluginId;
    fn name(&self)                      -> &str;
    fn version(&self)                   -> PluginVersion;
    fn category(&self)                  -> PluginCategory;
    fn description(&self)               -> &str;
    fn tags(&self)                      -> &[String];
    fn requires_pixel_access(&self)     -> bool;   // false = 永不触碰像素
    fn produces_pixel_output(&self)     -> bool;

    fn parameter_schema(&self)          -> &ParameterSchema;
    fn gui_schema(&self)                -> &GuiSchema;

    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self)                         -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet)
        -> PluginResult<Vec<ValidationIssue>>;
}
```

### 3.3 能力 Trait

```rust
// 仅元数据，零像素访问
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;
    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet)
        -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet)
        -> PluginResult<MetadataWriteReport>;
}

// 像素处理，16bit+，可选 GPU
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

// 编解码
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

// GPU 计算
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

// 外部工具透传
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

### 3.4 Plugin Category 枚举

```rust
pub enum PluginCategory {
    Input,       // 输入
    Metadata,    // 元数据
    Color,       // 色彩
    Transform,   // 变换
    Enhance,     // 增强
    Merge,       // 合成
    Format,      // 格式
    External,    // 外部
    Custom(String), // 自定义
}
```

### 3.5 插件加载器

| Loader | 格式 | 热重载 | 用途 |
|--------|------|:--:|------|
| Builtin | 编译进 binary | ✗ | 核心始终可用的插件 |
| Native | .so / .dll / .dylib | ✗ | 高性能第三方插件 |
| WASM | .wasm | ✓ | 安全沙箱第三方插件 |
| ExternalTool | 子进程调用 | ✗ | ExifTool / ffmpeg 等外部工具 |
| Remote | URL 下载安装 | ✗ | 插件市场分发 |

---

## 4. 参数系统（Schema 驱动）

### 4.1 四级优先级

```
image override     （优先级 3，最高）
  └> group override  （优先级 2，最后匹配者胜出）
      └> template default （优先级 1）
          └> plugin builtin default （优先级 0，最低）
```

### 4.2 Schema 类型

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

## 5. 数据流与精度

### 5.1 ImageBuffer（COW，惰性分配）

```rust
pub struct ImageBuffer {
    pub metadata:     Arc<RwLock<Metadata>>,
    // None = 仅元数据模式，不分配像素内存
    pixel_data:       Option<Arc<PixelData>>,
    pub pixel_format: PixelFormat,
    pub color_space:  ColorSpace,
    pub icc_profile:  Option<Arc<Vec<u8>>>,
}

pub struct PixelData {
    pub buffer: AlignedBuffer,   // 页对齐，支持 GPU 映射
    pub width:  u32,
    pub height: u32,
    pub layout: ChannelLayout,
}
```

### 5.2 零拷贝传递

```
Metadata plugins:   Arc<Metadata> 共享，0 拷贝
Metadata → Metadata:  同一 Arc，始终共享
Metadata → Pixel 插件:  仅在单消费者写入时触发 COW
Pixel → Pixel（单消费者）:  Arc 不加写时复制，原地修改
Pixel → Pixel（多消费者）:  Arc 共享，只读
GPU → GPU:  GpuHandle 传递，数据留在 VRAM 直到编码
```

### 5.3 分块处理

```
4096×2160 f32 RGBA = 135MB/帧
分割: 256×256 tile，每个约 1MB
并行: 16 个 tile 并发（Rayon/GPU）
优势: 降低峰值 VRAM 占用，缓存友好
```

---

## 6. 管线引擎

### 6.1 DAG 模型

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

### 6.2 执行策略

1. 解析 DAG → 拓扑排序
2. 对每个节点（尽可能并行）：
   a. 检查 `requires_pixel_access()` — 若为 false 则跳过 PixelBuffer
   b. `ParameterResolver::resolve(image, node)` → 合并 4 级参数
   c. 求值参数中的表达式
   d. 使用解析后的参数调用 `process_*()`
   e. 收集 ProcessingStats
3. 惰性像素分配：仅在第一个像素处理插件运行时分配
4. 大图像分块处理

---

## 7. 插件目录

### 7.1 内置插件

| # | ID | 类别 | 像素访问 | 后端 |
|:--:|-----|----------|:--:|------|
| 1 | exif_rw | Metadata | ✗ | ExifTool + kamadak-exif |
| 2 | (`xmp_iptc`) | Metadata | ✗ | ExifTool |
| 3 | gps_set | Metadata | ✗ | ExifTool + geo crate |
| 4 | time_shift | Metadata | ✗ | chrono + ExifTool |
| 5 | colorspace | Color | ✓ | Halide + lcms2 |
| 6 | lut_3d | Color | ✓ | Halide |
| 7 | transform | Transform | ✓ | Halide |
| 8 | (`rotate`) | Transform | ✓ | Halide |
| 9 | (`crop`) | Transform | ✓ | Halide |
| 10 | lens_correct | Enhance | ✓ | LensFun + Halide |
| 11 | ai_denoise | Enhance | ✓ | ONNX Runtime |
| 12 | heif_encoder | Format | ✓ | libheif + x265 |
| 13 | jxl_encoder | Format | ✓ | libjxl |
| 14 | avif_encoder | Format | ✓ | libheif + aom |
| 15 | tiff_encoder | Format | ✓ | OIIO |
| 16 | png_encoder | Format | ✓ | lodepng |
| 17 | raw_input | Input | ✓ | dcraw / LibRaw |

> 注：v0.1.0 已实现 14 个（exif_rw, gps_set, time_shift, colorspace, lut3d, transform, lens_correct, ai_denoise, raw_input, heif_encoder, jxl_encoder, avif_encoder, tiff_encoder, png_encoder）。

### 7.2 编码器品质推荐

| 格式 | 编码器 | 设置 | 品质 |
|------|------|------|:--:|
| HEIF 10-bit | x265 | preset=veryslow, crf=18, 444, tune=grain | ★★★★★ |
| HEIF 10-bit（GPU） | NVENC | Turing+, b-frames, 10bit | ★★★★ |
| HEIF 10-bit（Mac） | VideoToolbox | Apple Silicon HW | ★★★★ |
| JXL 16-bit | libjxl | effort=7-9, distance=0.5（视觉无损） | ★★★★★ |
| JXL 无损 | libjxl | effort=7-9, distance=0 | ★★★★★（完美） |

---

## 8. GUI 设计

### 8.1 主布局（三栏）

```
┌──────────────┬──────────────────────────┬──────────────────────┐
│  胶片条      │        预览区             │    插件控制面板       │
│  （缩略图    │   处理前 │ 处理后         │  （选中节点的完整     │
│   列表 +     │   分割/对比               │   详情，继承 schema） │
│   分组树）   │                           │                      │
│              ├──────────────────────────┤                      │
│  批量进度    │   管线总览                │                      │
│              │   迷你 DAG + 状态         │                      │
│              ├──────────────────────────┤                      │
│  [▶ 开始]   │   信息栏                  │                      │
└──────────────┴──────────────────────────┴──────────────────────┘
```

### 8.2 每个插件的 GUI（Schema 驱动）

每个插件面板包含：
- 上下文栏（模板 / 分组 / 图像覆盖切换按钮）
- 来自 ParameterSchema 的完整参数控件
- 每个字段的覆盖状态标记（🟡override / ⬜inherited）
- 当 `supports_expression=true` 时的表达式编辑器
- 来自 GuiSchema 的预览区域
- 来自 GuiSchema 的辅助视图（直方图、地图、波形图等）

---

## 9. 批量处理

### 9.1 工作流

1. 加载图片 → 自动提取 EXIF 快照
2. 加载 GPX 轨迹（可选）→ 按时间戳自动插值 GPS
3. 自动分组：按 ISO、GPS 聚类、时间间隔 → 应用分组预设
4. 选择单张图片 → 按需细调覆盖参数
5. 验证 → 检查参数完整性
6. 导出 → 并行处理、进度流、断点续传支持

### 9.2 配置文件（TOML）

```toml
[metadata]
name = "我的 HDR 管线"
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

# 逐图覆盖（仅与模板的差异）
[[overrides]]
image = "DSC0003.ARW"
params.gps = { lat = 30.5728, lon = 104.0668 }

# 自动分组
[[groups]]
name = "高 ISO"
condition = "exif.iso >= 1600"
params.ai_denoise = { strength = 0.9 }

[batch]
parallel = 4
output_pattern = "output/{date}/{filename}.heif"
on_conflict = "skip"
resume = true
```

---

## 10. CI/CD 策略

### 10.1 GitHub Actions 工作流

```
┌─────────────────────────────────────────┐
│ build-halide.yml                        │
│  matrix: [linux, windows, macos]        │
│  output: 各平台 halide 构建产物          │
├─────────────────────────────────────────┤
│ build-oiio.yml                          │
│  matrix: [linux, windows, macos]        │
│  output: 各平台 oiio 构建产物            │
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

### 10.2 本地开发工作流

```
开发者（本机，4C/8T，3.3GB）：
  1. 编写 Rust 代码
  2. cargo build（workspace，增量编译约 2 分钟）
  3. cargo test（本地）
  4. git push → GitHub Actions 处理：
     - Halide/OIIO 编译
     - 跨平台构建
     - GUI 打包
```

---

## 11. 项目结构

```
photopipeline/
├── ARCHITECTURE.md           # 本文档（英文原版）
├── ARCHITECTURE_zh.md        # 本文档（中文翻译）
├── README_zh.md              # 项目主文档
├── USER_GUIDE.md             # 用户手册
├── PLUGIN_DEV.md             # 插件开发指南
├── API_REFERENCE.md          # API 参考
├── CHANGELOG.md              # 变更日志
├── Cargo.toml                # workspace 根
├── justfile                   # 任务运行器（just）
├── .github/
│   └── workflows/
│       ├── build-halide.yml
│       ├── build-oiio.yml
│       ├── build-rust.yml
│       ├── build-gui.yml
│       └── release.yml
├── crates/
│   ├── core/                 # 共享类型
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── image.rs
│   │       ├── color.rs
│   │       ├── metadata.rs
│   │       ├── error.rs
│   │       └── types.rs
│   ├── plugin/               # 插件 Trait + Registry + Loader
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── trait_def.rs
│   │       ├── registry.rs
│   │       ├── loader.rs
│   │       ├── schema.rs
│   │       └── gui_schema.rs
│   ├── engine/               # 管线 DAG + 执行
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs
│   │       ├── executor.rs
│   │       ├── params.rs
│   │       └── tile.rs
│   ├── plugins/              # 所有内置插件
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── exif_rw.rs
│   │       ├── gps_set.rs
│   │       ├── time_shift.rs
│   │       ├── colorspace.rs
│   │       ├── lut3d.rs
│   │       ├── transform.rs
│   │       ├── lens_correct.rs
│   │       ├── ai_denoise.rs
│   │       ├── heif_encoder.rs
│   │       ├── jxl_encoder.rs
│   │       ├── avif_encoder.rs
│   │       ├── tiff_encoder.rs
│   │       ├── png_encoder.rs
│   │       └── raw_input.rs
│   ├── external/             # 外部工具封装
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── exiftool.rs
│   │       ├── libvips.rs
│   │       └── commercial.rs
│   ├── server/               # gRPC 服务端
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── main.rs
│   │       └── services/
│   │           ├── mod.rs
│   │           ├── pipeline.rs
│   │           ├── image.rs
│   │           └── batch.rs
│   └── oiio/                 # OIIO FFI 绑定（feature-gated）
│       └── src/
│           └── lib.rs
├── cli/                      # CLI 二进制
│   ├── Cargo.toml
│   ├── tests/
│   │   └── integration_test.rs
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       └── commands/
│           ├── mod.rs
│           ├── pipeline.rs
│           ├── plugin.rs
│           └── batch.rs
├── proto/                    # Protobuf 定义
│   ├── pipeline.proto
│   ├── image.proto
│   └── batch.proto
├── halide_generators/        # Halide 源文件（在 CI 上编译）
│   ├── CMakeLists.txt
│   ├── colorspace_generator.cpp
│   ├── resize_generator.cpp
│   └── tonemap_generator.cpp
├── examples/
│   └── hdr_pipeline.toml
└── gui/
    ├── linux/                # GTK4 + Rust
    ├── windows/              # WinUI 3 (.NET 8)
    └── macos/                # SwiftUI
```

---

## 12. 开发阶段

| Phase | 名称 | 目标 | 产出 |
|:---:|------|------|------|
| 0 | 设计文档 | 规划 | ARCHITECTURE.md |
| 1 | 环境搭建 | 配置 | Rust、dev libs、Git 初始化、CI 脚手架 |
| 2 | Core Crate | 类型定义 | ImageBuffer、Metadata、ColorSpace、Error |
| 3 | Plugin System | 框架 | Plugin trait、Registry、Loader、Schema |
| 4 | Pipeline Engine | 运行时 | DAG、Executor、ParameterResolver、TileEngine |
| 5 | Builtin Plugins | 功能实现 | 14+ 个内置插件 |
| 6 | External Tools | 集成 | ExifTool、libvips、商业 API stubs |
| 7 | CLI | 前端 | 子命令、batch、TOML 管线配置 |
| 8 | gRPC Server | 后端 | Proto 定义、服务实现、流式传输 |
| 9 | Halide/OIIO | 计算层 | 生成器文件、FFI、CI 编译脚本 |
| 10 | GUI Linux | 桌面端 | GTK4 管线编辑器 + 预览 + 批量 |
| 11 | GUI Windows | 桌面端 | WinUI 3 项目、gRPC 客户端 |
| 12 | GUI macOS | 桌面端 | SwiftUI 项目、gRPC 客户端 |
| 13 | CI/CD | DevOps | 全平台 GitHub Actions 矩阵、发布 |
| 14 | 验证 | 测试 | cargo build、lint、test |

---

*文档结束*

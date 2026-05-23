# Photopipeline API 参考

> 按 Crate 组织的完整公共 API 文档。涵盖 `photopipeline-core`、`photopipeline-plugin`、`photopipeline-engine`、CLI、gRPC 服务和 Protobuf 消息定义。

---

## 目录

1. [Core Crate — `photopipeline-core`](#1-core-crate--photopipeline-core)
   - 1.1 [类型别名](#11-类型别名)
   - 1.2 [PixelBuffer 与图像数据](#12-pixelbuffer-与图像数据)
   - 1.3 [AlignedBuffer 与 GpuBuffer](#13-alignedbuffer-与-gpubuffer)
   - 1.4 [PixelFormat 枚举](#14-pixelformat-枚举)
   - 1.5 [ChannelLayout 枚举](#15-channellayout-枚举)
   - 1.6 [Tile 分块类型](#16-tile-分块类型)
   - 1.7 [色彩空间类型](#17-色彩空间类型)
   - 1.8 [元数据类型](#18-元数据类型)
   - 1.9 [通用类型](#19-通用类型)
   - 1.10 [错误类型](#110-错误类型)
   - 1.11 [GUI Schema 类型](#111-gui-schema-类型)
2. [Plugin Crate — `photopipeline-plugin`](#2-plugin-crate--photopipeline-plugin)
   - 2.1 [基础 Plugin Trait](#21-基础-plugin-trait)
   - 2.2 [能力 Trait（6 种）](#22-能力-trait6-种)
   - 2.3 [ProgressSink](#23-progresssink)
   - 2.4 [ParameterSchema](#24-parameterschema)
   - 2.5 [ParameterType（18 种）](#25-parametertype18-种)
   - 2.6 [ParameterSet](#26-parameterset)
   - 2.7 [Registry](#27-registry)
   - 2.8 [PluginLoader](#28-pluginloader)
   - 2.9 [PluginManifest 与 PluginQuery](#29-pluginmanifest-与-pluginquery)
   - 2.10 [ModelInfo 与 ModelSource](#210-modelinfo-与-modelsource)
   - 2.11 [GUI 面板类型](#211-gui-面板类型)
3. [Engine Crate — `photopipeline-engine`](#3-engine-crate--photopipeline-engine)
   - 3.1 [PipelineGraph (DAG)](#31-pipelinegraph-dag)
   - 3.2 [PipelineTemplate](#32-pipelinetemplate)
   - 3.3 [NodeExecutor](#33-nodeexecutor)
   - 3.4 [ParameterResolver](#34-parameterresolver)
   - 3.5 [ExpressionEngine](#35-expressionengine)
   - 3.6 [TileEngine](#36-tileengine)
4. [CLI 命令接口](#4-cli-命令接口)
5. [gRPC 服务接口](#5-grpc-服务接口)
6. [Protobuf 消息定义](#6-protobuf-消息定义)

---

## 1. Core Crate — `photopipeline-core`

**路径：** `crates/core/src/`
**导出：** `color`、`error`、`image`、`metadata`、`types`

### 1.1 类型别名

```rust
pub type PluginId = String;
pub type NodeId = Uuid;
pub type ImageId = Uuid;
pub type BatchId = Uuid;
pub type PortId = Uuid;
pub type GroupId = Uuid;
```

### 1.2 PixelBuffer 与图像数据

#### PixelBuffer

```rust
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub layout: ChannelLayout,
    pub format: PixelFormat,
    pub color_space: ColorSpace,
    pub icc_profile: Option<Vec<u8>>,
    pub data: AlignedBuffer,
}
```

| 方法 | 签名 | 说明 |
|---|---|---|
| `new` | `(width, height, layout, format) -> Self` | 根据参数分配对齐缓冲区 |
| `byte_size` | `() -> usize` | 返回缓冲区字节大小 |
| `pixel_count` | `() -> u64` | 返回 `width × height` |
| `u16_samples` | `(channel: usize) -> Option<&[u16]>` | 返回 planar 格式的单通道 u16 切片 |
| `gpu_handle` | `() -> Option<GpuBufferHandle>` | 返回 GPU 缓冲区句柄（当前返回 `None`） |

#### GpuBufferHandle

```rust
#[derive(Debug, Clone)]
pub struct GpuBufferHandle {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: GpuBackend,
}
```

#### ImageDimensions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}
```

| 方法 | 说明 |
|---|---|
| `pixel_count() -> u64` | 返回像素总数 |

### 1.3 AlignedBuffer 与 GpuBuffer

#### AlignedBuffer

```rust
#[derive(Debug, Clone)]
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,
}
```

| 方法 | 说明 |
|---|---|
| `new(size, alignment) -> Self` | 创建零填充对齐字节缓冲区 |
| `as_u16_slice() -> &[u16]` | 通过 `bytemuck` 转换为 u16 切片 |
| `as_f32_slice() -> &[f32]` | 通过 `bytemuck` 转换为 f32 切片 |

#### GpuBuffer

```rust
#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: GpuBackend,
}
```

#### Tensor

```rust
#[derive(Debug, Clone)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorDtype {
    F32,
    F16,
    I8,
    U8,
}
```

### 1.4 PixelFormat 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum PixelFormat {
    U8,
    U16,
    U32,
    F16,
    F32,
}
```

| 方法 | 返回 | 说明 |
|---|---|---|
| `bytes_per_channel()` | `usize` | U8=1, U16/F16=2, U32/F32=4 |
| `is_float()` | `bool` | F16 或 F32 返回 true |
| `is_high_precision()` | `bool` | 非 U8 返回 true |
| `max_value_u16()` | `u16` | U8=255, 其他=65535 |

### 1.5 ChannelLayout 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelLayout {
    Gray,         // 1 通道
    GrayAlpha,    // 2 通道
    RGB,          // 3 通道
    RGBA,         // 4 通道
    Planar(u8),   // n 通道平面排列
    Custom(u8),   // n 通道自定义排列
}
```

| 方法 | 返回 | 说明 |
|---|---|---|
| `channel_count()` | `u8` | 返回通道数 |
| `is_interleaved()` | `bool` | 交错排列（Gray/GrayAlpha/RGB/RGBA）返回 true |

### 1.6 Tile 分块类型

#### TileCoord / TileSpec / TileLayout

```rust
pub struct TileCoord { pub x: u32, pub y: u32; }

pub struct TileSpec {
    pub coord: TileCoord,
    pub x_offset: u32,
    pub y_offset: u32,
    pub width: u32,
    pub height: u32,
}

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

| TileLayout 方法 | 签名 | 说明 |
|---|---|---|
| `new` | `(image_w, image_h, tile_size, overlap) -> Self` | 计算分块布局 |
| `tile_spec` | `(x, y) -> TileSpec` | 获取指定分块的规格 |
| `iter_tiles` | `() -> impl Iterator<Item = TileSpec>` | 遍历所有分块 |

### 1.7 色彩空间类型

#### ColorPrimaries（11 种）

```rust
pub enum ColorPrimaries {
    BT709, BT2020, DisplayP3, SRGB, AdobeRGB,
    ProPhoto, ACES, ACEScg, CIEXYZ, DCIP3, Rec2100,
}
```

#### TransferFunction（11 种）

```rust
pub enum TransferFunction {
    Linear, SRGB, Gamma22, Gamma24, Gamma26, Gamma28,
    PQ, HLG, SLog3, LogC, Custom(f64),
}
```

#### WhitePoint（8 种）

```rust
pub enum WhitePoint {
    D50, D55, D60, D65, D75, DCI, E, Custom(f32, f32),
}
```

#### ColorSpace

```rust
pub struct ColorSpace {
    pub primaries: ColorPrimaries,
    pub transfer: TransferFunction,
    pub white_point: WhitePoint,
    pub hdr_nits: Option<f32>,
}
```

| 常量 / 方法 | 说明 |
|---|---|
| `ColorSpace::SRGB` | sRGB 原色，sRGB 传递函数，D65 白点 |
| `ColorSpace::ADOBE_RGB` | Adobe RGB 原色，Gamma 2.2，D65 |
| `ColorSpace::DISPLAY_P3` | Display P3 原色，sRGB 传递函数，D65 |
| `ColorSpace::REC2020_PQ` | BT.2020 原色，PQ 传递函数，D65，1000 nit |
| `ColorSpace::ACES_CG` | ACEScg 原色，Linear，D60 |
| `ColorSpace::LINEAR_SRGB` | sRGB 原色，Linear，D65 |
| `is_hdr() -> bool` | hdr_nits > 203 返回 true |

#### RenderingIntent / GamutMapping / ColorConversionSpec

```rust
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

pub enum GamutMapping {
    Clip,
    Compress,
    LuminancePreserve,
}

pub struct ColorConversionSpec {
    pub source: ColorSpace,
    pub target: ColorSpace,
    pub intent: RenderingIntent,
    pub black_point_compensation: bool,
    pub gamut_mapping: GamutMapping,
    pub icc_profile: Option<Vec<u8>>,
    pub ocio_config: Option<String>,
    pub ocio_display: Option<String>,
    pub ocio_view: Option<String>,
}
```

#### ColorRGB / ColorRGBA

```rust
pub struct ColorRGB { pub r: f32, pub g: f32, pub b: f32 }
pub struct ColorRGBA { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl ColorRGB {
    pub const BLACK: Self;
    pub const WHITE: Self;
    pub fn luminance(&self) -> f32; // 0.2126*r + 0.7152*g + 0.0722*b
}
```

### 1.8 元数据类型

#### Metadata

```rust
pub struct Metadata {
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub iptc: Option<IptcData>,
    pub gps: Option<GpsData>,
    pub custom: Vec<CustomTag>,
}
```

#### ExifData

```rust
pub struct ExifData {
    // 设备和软件
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub serial_number: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub image_description: Option<String>,
    pub orientation: Option<u16>,

    // 时间
    pub date_time_original: Option<DateTime<Utc>>,
    pub date_time_digitized: Option<DateTime<Utc>>,
    pub sub_sec_time_original: Option<String>,
    pub offset_time_original: Option<String>,

    // 曝光
    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<u16>,
    pub aperture_value: Option<String>,
    pub shutter_speed_value: Option<String>,
    pub brightness_value: Option<String>,
    pub exposure_bias: Option<String>,
    pub metering_mode: Option<u16>,
    pub flash: Option<u16>,
    pub exposure_program: Option<u16>,
    pub white_balance: Option<u16>,

    // 图像
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub color_space: Option<u16>,
    pub bits_per_sample: Option<Vec<u16>>,
    pub compression: Option<u16>,

    // 厂商
    pub maker_note: Option<Vec<u8>>,
    pub raw_tags: Vec<RawExifTag>,
}
```

#### XmpData / IptcData

```rust
pub struct XmpData {
    pub creator: Option<String>,
    pub rights: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
    pub modify_date: Option<DateTime<Utc>>,
    pub rating: Option<u8>,
    pub label: Option<String>,
    pub subject: Vec<String>,
    pub raw_properties: Vec<RawXmpProperty>,
}

pub struct IptcData {
    pub creator: Option<String>,
    pub headline: Option<String>,
    pub caption: Option<String>,
    pub keywords: Vec<String>,
    pub copyright_notice: Option<String>,
    pub date_created: Option<DateTime<Utc>>,
    pub time_created: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub raw_tags: Vec<RawIptcTag>,
}
```

#### GpsData

```rust
pub struct GpsData {
    pub latitude: Option<f64>,
    pub latitude_ref: Option<String>,
    pub longitude: Option<f64>,
    pub longitude_ref: Option<String>,
    pub altitude: Option<f64>,
    pub altitude_ref: Option<i8>,
    pub timestamp: Option<DateTime<Utc>>,
    pub img_direction: Option<f64>,
    pub img_direction_ref: Option<String>,
    pub map_datum: Option<String>,
    pub satellites: Option<String>,
    pub status: Option<String>,
    pub measure_mode: Option<String>,
    pub dop: Option<f64>,
    pub speed: Option<f64>,
    pub speed_ref: Option<String>,
    pub track: Option<f64>,
    pub track_ref: Option<String>,
    pub dest_bearing: Option<f64>,
    pub dest_bearing_ref: Option<String>,
    pub dest_distance: Option<f64>,
    pub dest_latitude: Option<f64>,
    pub dest_longitude: Option<f64>,
    pub processing_method: Option<String>,
    pub area_information: Option<String>,
    pub date_stamp: Option<String>,
}
```

| 方法 | 说明 |
|---|---|
| `has_coordinates() -> bool` | 纬度和经度均为 `Some` |
| `coordinate_tuple() -> Option<(f64, f64)>` | 返回 (纬度, 经度) 元组 |

#### GpxTrack / GpxPoint

```rust
pub struct GpxTrack {
    pub name: Option<String>,
    pub points: Vec<GpxPoint>,
    pub duration_seconds: Option<f64>,
    pub distance_meters: Option<f64>,
}

pub struct GpxPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub speed: Option<f64>,
    pub bearing: Option<f64>,
}

impl GpxTrack {
    pub fn interpolate_at(&self, timestamp: &DateTime<Utc>) -> Option<GpxPoint>;
}
```

`interpolate_at()` 使用线性插值，处理前后边界点（单一方向外推）和方位角跨 0° 情况。

#### MetadataScope / MetadataTarget / MetadataWriteReport

```rust
pub enum MetadataScope { EXIF, XMP, IPTC, GPS, All }

pub struct MetadataTarget {
    pub path: String,
    pub format: ImageFormat,
}

pub struct MetadataWriteReport {
    pub tags_written: u32,
    pub tags_skipped: u32,
    pub warnings: Vec<String>,
}
```

### 1.9 通用类型

#### PluginVersion / VersionRequirement

```rust
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl PluginVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self;
}
impl Display for PluginVersion;      // "1.2.3" 或 "1.2.3-alpha"
impl PartialOrd for PluginVersion;   // 支持 < > <= >=

pub struct VersionRequirement {
    pub min_version: PluginVersion,
    pub max_version: Option<PluginVersion>,
}

impl VersionRequirement {
    pub fn is_satisfied_by(&self, version: &PluginVersion) -> bool;
}
impl Display for VersionRequirement; // ">=1.0.0,<2.0.0"
```

#### PluginCategory / GpuBackend / AiBackend

```rust
#[derive(Display, EnumString)]
pub enum PluginCategory {
    Input, Metadata, Color, Transform,
    Enhance, Merge, Format, External, Custom(String),
}

#[derive(Display)]
pub enum GpuBackend {
    None, CUDA, Metal, Vulkan, DirectX,
    OpenCL, ROCm, OpenVINO, Auto,
}

#[derive(Display)]
pub enum AiBackend {
    ONNX, TensorRT, CoreML, OpenVINO, Burn,
}
```

#### GpuContext / HardwareInfo / ProcessingStats

```rust
pub struct GpuContext {
    pub backend: GpuBackend,
    pub device_name: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub compute_units: u32,
}

pub struct HardwareInfo {
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_ram_mb: u64,
    pub gpus: Vec<GpuContext>,
}

pub struct ProcessingStats {
    pub elapsed_ms: u64,
    pub cpu_time_ms: u64,
    pub gpu_time_ms: Option<u64>,
    pub peak_memory_mb: u64,
    pub input_pixels: u64,
    pub output_pixels: u64,
}
```

#### ImageFormat

```rust
#[derive(Display)]
pub enum ImageFormat {
    HEIF, HEIC, AVIF, JXL, PNG, TIFF,
    JPEG, WEBP, OpenEXR, RAW, DNG,
    PPM, PGM, BMP, Unknown(String),
}
```

#### ImageInfo / DecodeOptions / EncodeOptions / FormatProbe

```rust
pub struct ImageInfo {
    pub id: ImageId,
    pub path: String,
    pub filename: String,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub pixel_format: PixelFormat,
    pub color_space: ColorSpace,
}

pub struct DecodeOptions {
    pub pixel_format: Option<PixelFormat>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub read_metadata: bool,        // 默认 true
    pub apply_transfer: bool,        // 默认 false
    pub icc_profile: Option<Vec<u8>>,
}

pub struct DecodedImage {
    pub buffer: PixelBuffer,
    pub metadata: Metadata,
    pub format: ImageFormat,
}

pub struct EncodeOptions {
    pub format: ImageFormat,         // 默认 HEIF
    pub quality: Option<f32>,        // 默认 Some(95.0)
    pub lossless: bool,              // 默认 false
    pub bit_depth: u8,               // 默认 10
    pub chroma_subsampling: Option<ChromaSubsampling>,
    pub encoder: Option<String>,
    pub effort: Option<u8>,
}

pub enum ChromaSubsampling { Yuv444, Yuv422, Yuv420 }

pub struct FormatProbe {
    pub path: Option<PathBuf>,
    pub extension: Option<String>,
    pub magic_bytes: Option<Vec<u8>>,
    pub mime_type: Option<String>,
}
```

#### HardwareRequirement / PluginConfig

```rust
pub struct HardwareRequirement {
    pub requires_cpu: bool,          // 默认 true
    pub requires_gpu: bool,          // 默认 false
    pub min_ram_mb: u64,             // 默认 256
    pub preferred_backend: Option<GpuBackend>, // 默认 None
}

pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, String>,
}
```

### 1.10 错误类型

#### PluginError（23 变体）

```rust
#[derive(Debug, Error)]
pub enum PluginError {
    NotFound(PluginId),
    AlreadyLoaded { plugin: PluginId },
    LoadFailed { plugin: PluginId, reason: String },
    VersionMismatch { plugin: PluginId, actual: PluginVersion, required: VersionRequirement },
    InvalidParameter { plugin: PluginId, field: String, message: String },
    MissingTool { plugin: PluginId, tool: String, required: String },
    GpuNotAvailable { plugin: PluginId, backend: GpuBackend },
    GpuOutOfMemory { plugin: PluginId, needed: u64, available: u64 },
    ExpressionError { plugin: PluginId, field: String, error: String },
    Timeout { plugin: PluginId, elapsed: f64, timeout: f64 },
    Internal { plugin: PluginId, message: String },
    Canceled { plugin: PluginId },
    Io { plugin: PluginId, error: std::io::Error },
    ValidationFailed(String),
    NodeExecutionFailed { node: String, message: String },
    CircularDependency,
    FileNotFound(String),
    UnsupportedFormat(String),
    EncodingFailed(String),
    DecodingFailed(String),
    Config(String),
    Other(String),
}

pub type PluginResult<T> = Result<T, PluginError>;
```

#### ValidationIssue

```rust
pub enum ValidationIssue {
    Error   { param: String, message: String },
    Warning { param: String, message: String },
    Info    { param: String, message: String },
}
impl Display for ValidationIssue; // "ERROR(param): message" 等
```

### 1.11 GUI Schema 类型

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub preview: PreviewMode,
    pub aux_views: Vec<AuxView>,
    pub min_panel_width: u32,           // 默认 320
}

pub enum GuiLayout {
    Standard { sections: Vec<GuiSection> },
    Custom { rows: Vec<GuiRow> },
}

pub struct GuiSection {
    pub param_section_id: String,
    pub title_visible: bool,
    pub style: SectionStyle,           // Default | Card | AccentCard | CollapsibleCard
}

pub struct GuiRow {
    pub height: RowHeight,             // Compact | Normal | Expanded | Custom(u32)
    pub cells: Vec<GuiCell>,
}

pub struct GuiCell {
    pub param_field_id: String,
    pub width_fraction: f64,
    pub label_position: LabelPosition, // Top | Left | None
}

pub enum PreviewMode {
    None,
    Live,
    ManualRefresh,
    BeforeAfter { default_split: f32, orientation: SplitOrientation, lock_zoom: bool },
    Tiled { rows: u32, cols: u32 },
}

pub enum AuxView {
    Histogram, Waveform, Vectorscope, GamutDiagram, Map,
    FocusPeaking, ClippingWarning, MetadataTable, ProgressBar, StatusText,
}

pub enum SectionStyle { Default, Card, AccentCard, CollapsibleCard }
pub enum SliderStyle { Continuous, Discrete, Range, DualHandle }
pub enum SliderOrientation { Horizontal, Vertical }
pub enum FloatWidget { SpinBox, Slider, ComboSlider, DragInput }
pub enum IntegerWidget { SpinBox, Slider, Combo }
pub enum EnumDisplay { Dropdown, RadioGroup, ButtonGroup, SegmentedControl, Tabs, PopupCard }
pub enum ColorMode { RGB, RGBA, HSL, HSV, Lab }
pub enum FilePathKind { File, Directory, SaveFile }
pub enum SplitOrientation { Horizontal, Vertical }
pub enum LabelPosition { Top, Left, None }
pub enum RowHeight { Compact, Normal, Expanded, Custom(u32) }
```

---

## 2. Plugin Crate — `photopipeline-plugin`

**路径：** `crates/plugin/src/`

### 2.1 基础 Plugin Trait

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

### 2.2 能力 Trait（6 种）

```rust
// 1. 元数据处理
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;
    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet) -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet) -> PluginResult<MetadataWriteReport>;
}

// 2. 像素处理
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self) -> Vec<ColorSpace>;
    fn required_gpu_backend(&self) -> Option<GpuBackend>;
    async fn process_pixels(&self, input: &PixelBuffer, output: &mut PixelBuffer, params: &ParameterSet, progress: Box<dyn ProgressSink>) -> PluginResult<ProcessingStats>;
}

// 3. 格式编解码
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn format_id(&self) -> ImageFormat;
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn can_decode(&self, data: &FormatProbe) -> bool;
    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn decode(&self, data: &[u8], options: &DecodeOptions) -> PluginResult<DecodedImage>;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata, options: &EncodeOptions) -> PluginResult<Vec<u8>>;
}

// 4. GPU 计算
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;
    async fn process_gpu(&self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer, params: &ParameterSet, progress: Box<dyn ProgressSink>) -> PluginResult<ProcessingStats>;
}

// 5. AI 推理
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;
    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor>;
}

// 6. 外部工具
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;
    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(&self, input_paths: &[PathBuf], output_path: &PathBuf, params: &ParameterSet) -> PluginResult<()>;
}
```

### 2.3 ProgressSink

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

### 2.4 ParameterSchema

```rust
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}

impl ParameterSchema {
    pub fn empty() -> Self;
    pub fn field(&self, section_id: &str, field_id: &str) -> Option<&ParameterField>;
    pub fn defaults(&self) -> ParameterSet;
    pub fn all_fields(&self) -> Vec<&ParameterField>;
}
```

```rust
pub struct ParameterSection {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub fields: Vec<ParameterField>,
}
```

```rust
pub struct ParameterField {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub help_url: Option<String>,
    pub field_type: ParameterType,       // #[serde(flatten)]
    pub default: serde_json::Value,
    pub required: bool,
    pub advanced: bool,
    pub allow_override: bool,            // 默认 true
    pub supports_expression: bool,       // 默认 false
}
```

### 2.5 ParameterType（18 种）

```rust
#[serde(tag = "type")]
pub enum ParameterType {
    #[serde(rename = "string")]
    String { max_length: usize, pattern: Option<String>, placeholder: Option<String> },
    #[serde(rename = "integer")]
    Integer { min: i64, max: i64, step: i64, unit: Option<String>, style: IntegerWidget },
    #[serde(rename = "float")]
    Float { min: f64, max: f64, step: f64, precision: u8, unit: Option<String>, logarithmic: bool, style: FloatWidget },
    #[serde(rename = "boolean")]
    Boolean { label_true: Option<String>, label_false: Option<String> },
    #[serde(rename = "enum")]
    Enum { options: Vec<EnumOption>, display: EnumDisplay },
    #[serde(rename = "color")]
    Color { mode: ColorMode, show_alpha: bool },
    #[serde(rename = "file_path")]
    FilePath { kind: FilePathKind, filters: Vec<(String, String)>, must_exist: bool },
    #[serde(rename = "coordinate")]
    Coordinate { alt_required: bool, direction_required: bool },
    #[serde(rename = "slider")]
    Slider { min: f64, max: f64, step: f64, show_ticks: bool, ticks: Option<Vec<f64>>, show_value: bool, orientation: SliderOrientation, style: SliderStyle },
    #[serde(rename = "combo_slider")]
    ComboSlider { min: f64, max: f64, step: f64, presets: Vec<(String, f64)>, unit: Option<String> },
    #[serde(rename = "expression")]
    Expression { variables: Vec<VariableDef> },
    #[serde(rename = "preset")]
    Preset { preset_schema_ref: String, builtin_presets: Vec<NamedPreset>, allow_custom: bool, allow_import: bool },
    #[serde(rename = "array")]
    Array { element: Box<ParameterField>, min_items: usize, max_items: Option<usize> },
    #[serde(rename = "map_widget")]
    MapWidget { show_track: bool, show_photos: bool, allow_manual_pin: bool },
    #[serde(rename = "before_after")]
    BeforeAfter { zoom_levels: Vec<f64>, show_histogram: bool },
    #[serde(rename = "separator")]
    Separator { label: Option<String> },
    #[serde(rename = "section")]
    Section { fields: Vec<ParameterField> },
}
```

### 2.6 ParameterSet

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParameterSet {
    pub values: HashMap<String, serde_json::Value>,
}

impl ParameterSet {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: String, value: Value);
    pub fn get(&self, key: &str)      -> Option<&Value>;
    pub fn get_str(&self, key: &str)  -> Option<&str>;
    pub fn get_i64(&self, key: &str)  -> Option<i64>;
    pub fn get_f64(&self, key: &str)  -> Option<f64>;
    pub fn get_bool(&self, key: &str) -> Option<bool>;
    pub fn merge(&mut self, other: &ParameterSet);  // 浅合并（other 覆盖 self）
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)>;
}
```

### 2.7 Registry

```rust
pub struct Registry { /* 内部: DashMap + RwLock */ }

impl Registry {
    pub fn new() -> Self;

    // 基础注册/获取
    pub fn register(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()>;
    pub fn unregister(&self, id: &PluginId) -> Option<Arc<dyn Plugin>>;
    pub fn get(&self, id: &PluginId) -> Option<Arc<dyn Plugin>>;

    // 能力处理器注册/获取
    pub fn register_metadata_processor(&self, p: Arc<dyn MetadataProcessor>) -> PluginResult<()>;
    pub fn get_metadata_processor(&self, id: &PluginId) -> Option<Arc<dyn MetadataProcessor>>;
    pub fn register_pixel_processor(&self, p: Arc<dyn PixelProcessor>) -> PluginResult<()>;
    pub fn get_pixel_processor(&self, id: &PluginId) -> Option<Arc<dyn PixelProcessor>>;
    pub fn register_format_processor(&self, p: Arc<dyn FormatProcessor>) -> PluginResult<()>;
    pub fn get_format_processor(&self, id: &PluginId) -> Option<Arc<dyn FormatProcessor>>;
    pub fn register_gpu_processor(&self, p: Arc<dyn GpuProcessor>) -> PluginResult<()>;
    pub fn get_gpu_processor(&self, id: &PluginId) -> Option<Arc<dyn GpuProcessor>>;
    pub fn register_ai_processor(&self, p: Arc<dyn AiProcessor>) -> PluginResult<()>;
    pub fn get_ai_processor(&self, id: &PluginId) -> Option<Arc<dyn AiProcessor>>;
    pub fn register_external_tool_processor(&self, p: Arc<dyn ExternalToolProcessor>) -> PluginResult<()>;
    pub fn get_external_tool_processor(&self, id: &PluginId) -> Option<Arc<dyn ExternalToolProcessor>>;

    // 查询
    pub fn query(&self, q: &PluginQuery) -> Vec<Arc<dyn Plugin>>;
    pub fn by_category(&self, cat: PluginCategory) -> Vec<Arc<dyn Plugin>>;
    pub fn all(&self) -> Vec<Arc<dyn Plugin>>;

    // Manifest
    pub fn manifest(&self, id: &PluginId) -> Option<PluginManifest>;
    pub fn manifests(&self) -> Vec<PluginManifest>;
    pub fn categories(&self) -> Vec<PluginCategory>;
    pub fn is_loaded(&self, id: &PluginId) -> bool;
}
```

### 2.8 PluginLoader

```rust
#[async_trait]
pub trait PluginLoader: Send + Sync {
    fn name(&self) -> &str;
    fn supported_extensions(&self) -> Vec<&str>;
    async fn probe(&self, path: &Path) -> PluginResult<Option<PluginManifest>>;
    async fn load(&self, manifest: &PluginManifest, path: &Path) -> PluginResult<Box<dyn Plugin>>;
    fn can_hot_reload(&self) -> bool;
}

// 内置实现
pub struct BuiltinPluginLoader;        // 不支持从路径加载
pub struct NativePluginLoader;         // 读取 .toml manifest
pub struct ExternalToolPluginLoader;   // TBD

pub struct PluginLoaderManager {
    pub fn new() -> Self;
    pub fn add_search_path(&mut self, path: PathBuf);
    pub async fn discover_and_load(&self, registry: &Registry) -> PluginResult<Vec<PluginId>>;
}
```

### 2.9 PluginManifest 与 PluginQuery

```rust
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: PluginVersion,
    pub category: PluginCategory,
    pub description: String,
    pub tags: Vec<String>,
    pub requires_pixel_access: bool,
    pub requires_network: bool,
    pub requires_filesystem: bool,
    pub min_ram_mb: u64,
    pub dependencies: HashMap<String, String>,
}

pub struct PluginQuery {
    pub category: Option<PluginCategory>,
    pub tags: Vec<String>,             // AND 匹配
    pub requires_pixel: Option<bool>,
    pub keyword: Option<String>,       // 搜索名称和描述
    pub enabled_only: bool,
}
```

### 2.10 ModelInfo 与 ModelSource

```rust
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub source: ModelSource,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub memory_mb: u64,
    pub description: String,
}

pub enum ModelSource {
    Bundled,
    ExternalFile(String),
    HuggingFace { repo: String, file: String },
    Url(String),
}

pub struct ToolAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub error: Option<String>,
}
```

### 2.11 GUI 面板类型

```rust
pub struct NodePanelDefinition {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_category: String,
    pub sections: Vec<PanelSection>,
    pub context_bar: ContextBarConfig,
    pub help_text: Option<String>,
}

pub struct PanelSection {
    pub id: String,
    pub label: String,
    pub widget: PanelWidget,
    pub collapsible: bool,
    pub default_collapsed: bool,
}

#[serde(tag = "type")]
pub enum PanelWidget {
    TextInput { param_id, placeholder, max_length },
    NumberInput { param_id, min, max, step, precision, unit },
    Slider { param_id, min, max, step, show_value },
    Toggle { param_id, label_on, label_off },
    Dropdown { param_id, options: Vec<DropdownOption> },
    SegmentedControl { param_id, options: Vec<DropdownOption> },
    CardSelector { param_id, options: Vec<CardOption> },
    FilePicker { param_id, kind, filters },
    ColorPicker { param_id, show_alpha },
    CoordinateInput { param_id_lat, param_id_lon, param_id_alt },
    ComboSlider { param_id, presets, min, max, unit },
    MapWidget { param_id_lat, param_id_lon, show_track, show_photos, allow_manual_pin },
    ExpressionEditor { param_id, variables, example },
    BeforeAfterPreview { zoom_levels, show_histogram },
    NestedFields { fields: Vec<PanelSection> },
    Label { text },
}
```

---

## 3. Engine Crate — `photopipeline-engine`

**路径：** `crates/engine/src/`
**导出：** `executor`、`graph`、`params`、`tile`

### 3.1 PipelineGraph (DAG)

```rust
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<(PortId, PortId)>,
}

pub struct PipelineNode {
    pub id: NodeId,
    pub label: String,
    pub plugin_id: PluginId,
    pub enabled: bool,                             // 默认 true
    pub position: (f64, f64),                      // 默认 (0, 0)
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub parameter_overrides: Option<ParameterSet>,
}
```

| 方法 | 签名 | 说明 |
|---|---|---|
| `new` | `() -> Self` | 创建空图 |
| `add_node` | `(plugin_id, label) -> NodeId` | 添加节点，返回 UUID |
| `remove_node` | `(node_id) -> bool` | 移除节点及关联边 |
| `connect` | `(from_port, to_port) -> Result<(), PluginError>` | 添加有向边（含环检测） |
| `disconnect` | `(from_port, to_port) -> bool` | 移除边 |
| `topological_order` | `() -> Result<Vec<NodeId>, PluginError>` | Kahn 算法拓扑排序 |
| `has_cycle` | `() -> bool` | 检测环 |
| `validate_graph` | `() -> Result<(), Vec<String>>` | 完整校验（端口、ID、环） |
| `from_template` | `(template: &PipelineTemplate) -> Self` | 从模板构建图 |
| `node` | `(id) -> Option<&PipelineNode>` | 按 ID 查询 |
| `node_mut` | `(id) -> Option<&mut PipelineNode>` | 可变引用 |
| `port_owner` | `(port_id) -> Option<NodeId>` | 查找端口所属节点 |

**连接约束：**
- 不允许自环（from == to）
- 不允许同节点多端口互联
- 不允许重复边
- 形成环的边将被拒绝（`CircularDependency` 错误）

### 3.2 PipelineTemplate

```rust
pub struct PipelineTemplate {
    pub metadata: TemplateMetadata,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub overrides: Vec<ImageOverride>,
    pub groups: Vec<ParamGroup>,
    pub batch: Option<BatchConfig>,
}

pub struct TemplateMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

pub struct TemplateNode {
    pub id: String,
    pub plugin: String,
    pub label: Option<String>,
    pub enabled: bool,                               // 默认 true
    pub params: Option<HashMap<String, serde_json::Value>>,
}

pub struct TemplateEdge {
    pub from: String,
    pub to: String,
}

pub struct ImageOverride {
    pub image: String,
    pub params: HashMap<String, ParameterSet>,        // key = node_id
}

pub struct ParamGroup {
    pub name: String,
    pub condition: String,
    pub params: HashMap<String, ParameterSet>,        // key = node_id
}

pub struct BatchConfig {
    pub parallel: usize,                              // 默认 1
    pub output_pattern: Option<String>,
    pub on_conflict: Option<String>,                  // "skip" | "overwrite"
    pub resume: bool,                                 // 默认 false
}
```

| PipelineTemplate 方法 | 说明 |
|---|---|
| `validate() -> Result<(), String>` | 校验节点数和边引用 |
| `into_graph() -> PipelineGraph` | 转换为可执行的图 |

### 3.3 NodeExecutor

```rust
pub struct NodeExecutor {
    pub registry: Arc<Registry>,
    pub resolver: Arc<ParameterResolver>,
}

impl NodeExecutor {
    pub fn new(registry: Arc<Registry>, resolver: Arc<ParameterResolver>) -> Self;

    pub async fn execute(
        &self,
        graph: &PipelineGraph,
        image_info: &ImageInfo,
        buffer: Option<PixelBuffer>,
        metadata: &Metadata,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ExecutionResult>;
}
```

**执行流程：**
1. 拓扑排序图节点
2. 按顺序遍历每个节点：
   - 检查取消状态
   - 跳过 `enabled = false` 的节点
   - 通过 `ParameterResolver::resolve()` 合并参数
   - 调用 `plugin.validate()` 校验参数
   - 按 `requires_pixel_access()` 分派到 `process_pixel_node()` 或 `process_metadata_node()`
3. 返回 `ExecutionResult`（含最终 PixelBuffer、Metadata 和节点状态）

```rust
pub struct ExecutionContext {
    pub image_info: ImageInfo,
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

pub struct NodeRunState {
    pub status: NodeStatus,
    pub started_at: Option<DateTime<Utc>>,
}

pub enum NodeStatus {
    Pending,
    Running,
    Completed(ProcessingStats),
    Failed(String),
    Skipped,
}

pub struct ExecutionResult {
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}
```

### 3.4 ParameterResolver

```rust
pub struct ParameterResolver {
    pub template_params: HashMap<NodeId, ParameterSet>,
    pub group_overrides: Vec<(GroupCondition, HashMap<NodeId, ParameterSet>)>,
    pub image_overrides: HashMap<(ImageId, NodeId), ParameterSet>,
    pub expr_engine: ExpressionEngine,
}
```

| 方法 | 签名 | 说明 |
|---|---|---|
| `new` | `() -> Self` | 创建空解析器 |
| `set_template_params` | `(node_id, params)` | 设置模板级参数 |
| `add_group_override` | `(condition, node_params)` | 添加分组覆盖 |
| `set_image_override` | `(image_id, node_id, params)` | 设置图像级覆盖 |
| `resolve` | `(node_id, image_id, schema, metadata, image_info) -> ParameterSet` | 四级合并 + 表达式求值 |
| `resolve_single` | `(node_id, schema) -> ParameterSet` | 仅插件默认 + 模板（用于单图） |

**四级合并顺序：** Plugin defaults → Template defaults → Group overrides (last match wins) → Image overrides → Expression evaluation

#### GroupCondition

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

**Haversine 公式：** `GpsNear` 使用精确的 Haversine 球面距离计算（地球半径 6371 km）。

### 3.5 ExpressionEngine

```rust
pub struct ExpressionEngine;

impl ExpressionEngine {
    pub fn evaluate(
        &self,
        expr: &str,                                  // 含 ${ } 的表达式
        metadata: &Metadata,
        image_info: &ImageInfo,
    ) -> Result<serde_json::Value, String>;
}
```

**支持的语法：**
- 变量：`exif.iso`、`exif.aperture`、`exif.shutter`、`exif.focal_length`、`exif.make`、`exif.model`、`exif.lens`、`image.filename`、`image.width`、`image.height`、`image.filesize`
- 运算符：`>`、`<`、`>=`、`<=`、`==`、`!=`
- 三元：`condition ? true_value : false_value`（支持嵌套）
- 字面值：`123`、`3.14`、`"string"`、`'string'`
- 浮点比较使用 epsilon 容差

### 3.6 TileEngine

```rust
pub struct TileEngine {
    pub default_tile_size: u32,        // 默认 1024
    pub overlap: u32,                  // 默认 64
    pub max_parallel: usize,           // 默认 = CPU 核心数
}

impl TileEngine {
    pub fn new(default_tile_size: u32, overlap: u32, max_parallel: usize) -> Self;

    pub async fn process_tiled(
        &self,
        processor: &dyn PixelProcessor,
        input: &PixelBuffer,
        params: &ParameterSet,
        progress: &dyn ProgressSink,
    ) -> PluginResult<PixelBuffer>;
}
```

**分块处理模式：** 将大图按 `tile_size` 分割为独立块，每块创建子 PixelBuffer，依次调用 `processor.process_pixels()`，最后将结果 blit 回输出缓冲区。

---

## 4. CLI 命令接口

**路径：** `cli/src/main.rs`

### 命令结构

```
photopipeline [OPTIONS] <COMMAND>

COMMANDS:
  pipeline    管线运行与验证
  plugin      插件列表与信息
  batch       批量处理
  help        打印帮助信息
```

### `pipeline` 子命令

```
photopipeline pipeline run
  -c, --config <CONFIG>      管线 TOML 配置文件路径
  -i, --input <INPUT>        输入图像文件路径
  -o, --output <OUTPUT>      输出文件路径

photopipeline pipeline validate
  -c, --config <CONFIG>      管线 TOML 配置文件路径
```

### `plugin` 子命令

```
photopipeline plugin list                    列出所有已注册插件

photopipeline plugin info <PLUGIN_ID>        查看插件详情
  <PLUGIN_ID>   插件 ID，如 photopipeline.plugins.colorspace
```

### `batch` 子命令

```
photopipeline batch run
  -c, --config <CONFIG>      管线 TOML 配置文件路径
  -p, --pattern <PATTERN>    文件匹配模式 (glob) [默认: *.ARW]
  -o, --output <OUTPUT_DIR>  输出目录 [默认: ./output/]

photopipeline batch validate
  -c, --config <CONFIG>      管线 TOML 配置文件路径
  -p, --pattern <PATTERN>    文件匹配模式 [默认: *.ARW]
```

### 配置加载

```rust
// cli/src/config.rs
pub fn load_template(content: &str) -> Result<PipelineTemplate, String>;
```

---

## 5. gRPC 服务接口

**服务端入口：** `crates/server/src/main.rs`
**监听地址：** `localhost:50051`

### PipelineService

```protobuf
service PipelineService {
  rpc CreatePipeline(PipelineSpec) returns (PipelineId);
  rpc Execute(ExecuteRequest) returns (stream ExecuteProgress);
  rpc Validate(PipelineSpec) returns (ValidationResult);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}
```

| RPC | 输入 → 输出 | 说明 |
|---|---|---|
| `CreatePipeline` | `PipelineSpec` → `PipelineId` | 创建管线，返回 UUID |
| `Execute` | `ExecuteRequest` → stream `ExecuteProgress` | 流式执行管线 |
| `Validate` | `PipelineSpec` → `ValidationResult` | 验证管线配置 |
| `GetNodeSchema` | `PluginId` → `NodeSchema` | 获取插件参数/GUI Schema |

### ImageService

```protobuf
service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);
  rpc Decode(DecodeRequest) returns (stream PixelDataChunk);
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);
  rpc GetThumbnail(ThumbnailRequest) returns (ImageData);
}
```

| RPC | 输入 → 输出 | 说明 |
|---|---|---|
| `Load` | `ImagePath` → `ImageInfo` | 加载图片元数据 |
| `Decode` | `DecodeRequest` → stream `PixelDataChunk` | 流式解码 |
| `Encode` | `EncodeRequest` → stream `EncodeProgress` | 流式编码 |
| `GetThumbnail` | `ThumbnailRequest` → `ImageData` | 获取缩略图 |

### BatchService

```protobuf
service BatchService {
  rpc SubmitBatch(BatchSpec) returns (BatchId);
  rpc GetProgress(BatchId) returns (stream BatchProgress);
  rpc Cancel(BatchId) returns (Empty);
}
```

| RPC | 输入 → 输出 | 说明 |
|---|---|---|
| `SubmitBatch` | `BatchSpec` → `BatchId` | 提交批量任务 |
| `GetProgress` | `BatchId` → stream `BatchProgress` | 流式获取进度 |
| `Cancel` | `BatchId` → `Empty` | 取消批量任务 |

---

## 6. Protobuf 消息定义

### pipeline.proto

```protobuf
message PluginId {
  string id = 1;
}

message PipelineSpec {
  string name = 1;
  repeated PipelineNode nodes = 2;
  repeated PipelineEdge edges = 3;
  map<string, google.protobuf.Struct> params = 4;
  BatchConfig batch = 5;
}

message PipelineNode {
  string id = 1;
  string plugin_id = 2;
  string label = 3;
  bool enabled = 4;
  google.protobuf.Struct params = 5;
}

message PipelineEdge {
  string from = 1;
  string to = 2;
}

message PipelineId {
  string id = 1;
}

message ExecuteRequest {
  string pipeline_id = 1;
  string image_path = 2;
  string output_path = 3;
}

message ExecuteProgress {
  enum Stage { LOADING=0; DECODING=1; PROCESSING=2; ENCODING=3; DONE=4; ERROR=5; }
  Stage stage = 1;
  string node_id = 2;
  string node_label = 3;
  float fraction = 4;
  string message = 5;
  int64 elapsed_ms = 6;
}

message ValidationResult {
  bool valid = 1;
  repeated ValidationIssue issues = 2;
}

message ValidationIssue {
  enum Severity { INFO=0; WARNING=1; ERROR=2; }
  Severity severity = 1;
  string param = 2;
  string message = 3;
}

message NodeSchema {
  string plugin_id = 1;
  string name = 2;
  string version = 3;
  string category = 4;
  string description = 5;
  google.protobuf.Struct parameter_schema = 6;
  google.protobuf.Struct gui_schema = 7;
}

message BatchConfig {
  int32 parallel = 1;
  string output_pattern = 2;
  string on_conflict = 3;
  bool resume = 4;
}
```

### image.proto

```protobuf
message ImagePath { string path = 1; }

message ImageInfo {
  string id = 1;
  string path = 2;
  string filename = 3;
  string format = 4;
  uint32 width = 5;
  uint32 height = 6;
  uint64 file_size_bytes = 7;
  string pixel_format = 8;
  string color_space = 9;
  MetadataInfo metadata = 10;
}

message MetadataInfo {
  optional string make = 1;
  optional string model = 2;
  optional string lens_model = 3;
  optional string date_time_original = 4;
  optional string exposure_time = 5;
  optional string f_number = 6;
  optional uint32 iso = 7;
  optional string focal_length = 8;
  optional double latitude = 9;
  optional double longitude = 10;
  optional double altitude = 11;
}

message DecodeRequest {
  string path = 1;
  optional string pixel_format = 2;
  optional uint32 max_width = 3;
  optional uint32 max_height = 4;
  bool read_metadata = 5;
  bool apply_transfer = 6;
}

message PixelDataChunk {
  uint32 offset = 1;
  bytes data = 2;
  uint32 total_size = 3;
  bool is_last = 4;
}

message EncodeRequest {
  bytes pixel_data = 1;
  uint32 width = 2;
  uint32 height = 3;
  string layout = 4;
  string pixel_format = 5;
  string output_path = 6;
  string format = 7;
  optional float quality = 8;
  bool lossless = 9;
  uint32 bit_depth = 10;
  optional string chroma_subsampling = 11;
  optional string encoder = 12;
  optional uint32 effort = 13;
  MetadataInfo metadata = 14;
}

message EncodeProgress {
  float fraction = 1;
  string message = 2;
  uint64 bytes_written = 3;
  bool done = 4;
}

message ThumbnailRequest { string path = 1; uint32 max_size = 2; }

message ImageData {
  bytes data = 1;
  uint32 width = 2;
  uint32 height = 3;
  string format = 4;
}
```

### batch.proto

```protobuf
message BatchSpec {
  string pipeline_config_path = 1;
  string file_pattern = 2;
  string output_dir = 3;
  int32 parallel = 4;
  bool resume = 5;
}

message BatchId { string id = 1; }

message BatchProgress {
  enum Status { PENDING=0; RUNNING=1; DONE=2; CANCELED=3; ERROR=4; }
  Status status = 1;
  int32 total_files = 2;
  int32 completed_files = 3;
  int32 failed_files = 4;
  string current_file = 5;
  float fraction = 6;
  string progress_details = 7;
}
```

---

*API Reference — Photopipeline v0.1.0*

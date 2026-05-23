# Photopipeline API 参考

## 目录

1. [Core Crate — 核心类型](#1-core-crate--核心类型)
2. [Plugin Crate — 插件框架](#2-plugin-crate--插件框架)
3. [Engine Crate — 管线引擎](#3-engine-crate--管线引擎)
4. [CLI — 命令与参数](#4-cli--命令与参数)
5. [gRPC 服务接口](#5-grpc-服务接口)
6. [Protobuf 消息定义](#6-protobuf-消息定义)

---

## 1. Core Crate — 核心类型

Crate 名称：`photopipeline-core`
路径：`crates/core/src/`

### 1.1 类型别名

```rust
pub type PluginId = String;
pub type NodeId = Uuid;
pub type ImageId = Uuid;
pub type BatchId = Uuid;
pub type PortId = Uuid;
pub type GroupId = Uuid;
```

### 1.2 ImageBuffer、PixelBuffer、AlignedBuffer

```rust
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub layout: ChannelLayout,         // Gray | GrayAlpha | RGB | RGBA | Planar(u8) | Custom(u8)
    pub format: PixelFormat,           // U8 | U16 | U32 | F16 | F32
    pub color_space: ColorSpace,
    pub icc_profile: Option<Vec<u8>>,
    pub data: AlignedBuffer,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32, layout: ChannelLayout, format: PixelFormat) -> Self;
    pub fn byte_size(&self) -> usize;
    pub fn pixel_count(&self) -> u64;
    pub fn u16_samples(&self, channel: usize) -> Option<&[u16]>;
    pub fn gpu_handle(&self) -> Option<GpuBufferHandle>;
}
```

```rust
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,
}

impl AlignedBuffer {
    pub fn new(size: usize, alignment: usize) -> Self;
    pub fn as_u16_slice(&self) -> &[u16];
    pub fn as_f32_slice(&self) -> &[f32];
}
```

```rust
pub struct GpuBuffer {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: GpuBackend,
}
```

### 1.3 PixelFormat

```rust
pub enum PixelFormat {
    U8,
    U16,
    U32,
    F16,
    F32,
}

impl PixelFormat {
    pub fn bytes_per_channel(&self) -> usize;
    pub fn is_float(&self) -> bool;
    pub fn is_high_precision(&self) -> bool;  // 非 U8
    pub fn max_value_u16(&self) -> u16;
}
```

### 1.4 ChannelLayout

```rust
pub enum ChannelLayout {
    Gray,          // 1 通道
    GrayAlpha,     // 2 通道
    RGB,           // 3 通道
    RGBA,          // 4 通道
    Planar(u8),    // n 通道平面
    Custom(u8),    // n 通道自定义
}

impl ChannelLayout {
    pub fn channel_count(&self) -> u8;
    pub fn is_interleaved(&self) -> bool;
}
```

### 1.5 Tile / 分块

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

impl TileLayout {
    pub fn new(image_width: u32, image_height: u32, tile_size: u32, overlap: u32) -> Self;
    pub fn tile_spec(&self, x: u32, y: u32) -> TileSpec;
    pub fn iter_tiles(&self) -> impl Iterator<Item = TileSpec> + '_;
}
```

### 1.6 Color（颜色与色彩空间）

```rust
pub enum ColorPrimaries {
    BT709, BT2020, DisplayP3, SRGB, AdobeRGB,
    ProPhoto, ACES, ACEScg, CIEXYZ, DCIP3, Rec2100,
}

pub enum TransferFunction {
    Linear, SRGB, Gamma22, Gamma24, Gamma26, Gamma28,
    PQ, HLG, SLog3, LogC, Custom(f64),
}

pub enum WhitePoint {
    D50, D55, D60, D65, D75, DCI, E, Custom(f32, f32),
}

pub struct ColorSpace {
    pub primaries: ColorPrimaries,
    pub transfer: TransferFunction,
    pub white_point: WhitePoint,
    pub hdr_nits: Option<f32>,
}

impl ColorSpace {
    pub const SRGB: Self;
    pub const ADOBE_RGB: Self;
    pub const DISPLAY_P3: Self;
    pub const REC2020_PQ: Self;
    pub const ACES_CG: Self;
    pub const LINEAR_SRGB: Self;
    pub fn is_hdr(&self) -> bool;  // hdr_nits > 203
}
```

```rust
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

pub enum GamutMapping {
    Clip, Compress, LuminancePreserve,
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

### 1.7 Metadata（元数据）

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
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub serial_number: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub image_description: Option<String>,
    pub orientation: Option<u16>,

    pub date_time_original: Option<DateTime<Utc>>,
    pub date_time_digitized: Option<DateTime<Utc>>,
    pub sub_sec_time_original: Option<String>,
    pub offset_time_original: Option<String>,

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

    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub color_space: Option<u16>,
    pub bits_per_sample: Option<Vec<u16>>,
    pub compression: Option<u16>,

    pub maker_note: Option<Vec<u8>>,
    pub raw_tags: Vec<RawExifTag>,
}
```

#### XmpData

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
```

#### IptcData

```rust
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
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub img_direction: Option<f64>,
    pub satellites: Option<String>,
    pub speed: Option<f64>,
    pub track: Option<f64>,
    pub dest_bearing: Option<f64>,
    pub dest_distance: Option<f64>,
    pub dest_latitude: Option<f64>,
    pub dest_longitude: Option<f64>,
    pub map_datum: Option<String>,
    // ... 其他 GPS 字段
}

impl GpsData {
    pub fn has_coordinates(&self) -> bool;
    pub fn coordinate_tuple(&self) -> Option<(f64, f64)>;
}
```

#### GPX

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

### 1.8 Types（通用类型）

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
```

```rust
pub struct VersionRequirement {
    pub min_version: PluginVersion,
    pub max_version: Option<PluginVersion>,
}

impl VersionRequirement {
    pub fn is_satisfied_by(&self, version: &PluginVersion) -> bool;
}
```

```rust
pub enum PluginCategory {
    Input, Metadata, Color, Transform,
    Enhance, Merge, Format, External, Custom(String),
}
```

```rust
pub enum GpuBackend {
    None, CUDA, Metal, Vulkan, DirectX, OpenCL, ROCm, OpenVINO, Auto,
}

pub enum AiBackend {
    ONNX, TensorRT, CoreML, OpenVINO, Burn,
}

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
```

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

```rust
pub enum ImageFormat {
    HEIF, HEIC, AVIF, JXL, PNG, TIFF, JPEG, WEBP,
    OpenEXR, RAW, DNG, PPM, PGM, BMP, Unknown(String),
}
```

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
```

```rust
pub struct ImageDimensions { pub width: u32, pub height: u32; }

pub enum ChromaSubsampling { Yuv444, Yuv422, Yuv420 }

pub struct DecodeOptions {
    pub pixel_format: Option<PixelFormat>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub read_metadata: bool,
    pub apply_transfer: bool,
    pub icc_profile: Option<Vec<u8>>,
}

pub struct DecodedImage {
    pub buffer: PixelBuffer,
    pub metadata: Metadata,
    pub format: ImageFormat,
}

pub struct EncodeOptions {
    pub format: ImageFormat,
    pub quality: Option<f32>,
    pub lossless: bool,
    pub bit_depth: u8,
    pub chroma_subsampling: Option<ChromaSubsampling>,
    pub encoder: Option<String>,
    pub effort: Option<u8>,
}

pub struct FormatProbe {
    pub path: Option<PathBuf>,
    pub extension: Option<String>,
    pub magic_bytes: Option<Vec<u8>>,
    pub mime_type: Option<String>,
}
```

```rust
pub struct HardwareRequirement {
    pub requires_cpu: bool,
    pub requires_gpu: bool,
    pub min_ram_mb: u64,
    pub preferred_backend: Option<GpuBackend>,
}
```

```rust
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,
}

pub enum TensorDtype { F32, F16, I8, U8 }
```

### 1.9 Error（错误类型）

```rust
#[derive(Debug, Error)]
pub enum PluginError {
    NotFound(PluginId),
    AlreadyLoaded { plugin: PluginId },
    LoadFailed { plugin: PluginId, reason: String },
    VersionMismatch { plugin, actual, required },
    InvalidParameter { plugin, field, message },
    MissingTool { plugin, tool, required },
    GpuNotAvailable { plugin, backend: GpuBackend },
    GpuOutOfMemory { plugin, needed, available },
    ExpressionError { plugin, field, error },
    Timeout { plugin, elapsed, timeout },
    Internal { plugin, message },
    Canceled { plugin },
    Io { plugin, error: std::io::Error },
    ValidationFailed(String),
    NodeExecutionFailed { node, message },
    CircularDependency,
    FileNotFound(String),
    UnsupportedFormat(String),
    EncodingFailed(String),
    DecodingFailed(String),
    Config(String),
    Other(String),
}

pub type PluginResult<T> = Result<T, PluginError>;

pub enum ValidationIssue {
    Error { param: String, message: String },
    Warning { param: String, message: String },
    Info { param: String, message: String },
}
```

### 1.10 GuiSchema 相关类型

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub preview: PreviewMode,
    pub aux_views: Vec<AuxView>,
    pub min_panel_width: u32,
}

pub enum GuiLayout {
    Standard { sections: Vec<GuiSection> },
    Custom { rows: Vec<GuiRow> },
}

pub struct GuiSection {
    pub param_section_id: String,
    pub title_visible: bool,
    pub style: SectionStyle,
}

pub enum SectionStyle { Default, Card, AccentCard, CollapsibleCard }

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
```

---

## 2. Plugin Crate — 插件框架

Crate 名称：`photopipeline-plugin`
路径：`crates/plugin/src/`

### 2.1 基础 Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    fn id(&self) -> &PluginId;
    fn name(&self) -> &str;
    fn version(&self) -> PluginVersion;
    fn category(&self) -> PluginCategory;
    fn description(&self) -> &str;
    fn tags(&self) -> &[String];
    fn requires_pixel_access(&self) -> bool;
    fn produces_pixel_output(&self) -> bool;
    fn supported_hardware(&self) -> HardwareRequirement;

    fn parameter_schema(&self) -> &ParameterSchema;
    fn gui_schema(&self) -> &GuiSchema;

    async fn initialize(&mut self, cfg: &PluginConfig) -> PluginResult<()>;
    async fn shutdown(&mut self) -> PluginResult<()>;
    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>>;
}
```

### 2.2 能力 Trait

```rust
// 元数据处理 — 零像素访问
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;
    async fn read_metadata(&self, target: &MetadataTarget, params: &ParameterSet) -> PluginResult<Metadata>;
    async fn write_metadata(&self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet) -> PluginResult<MetadataWriteReport>;
}

// 像素处理 — 16bit+ 精度
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self) -> Vec<ColorSpace>;
    fn required_gpu_backend(&self) -> Option<GpuBackend>;
    async fn process_pixels(&self, input: &PixelBuffer, output: &mut PixelBuffer, params: &ParameterSet, progress: Box<dyn ProgressSink>) -> PluginResult<ProcessingStats>;
}

// 格式编解码
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn format_id(&self) -> ImageFormat;
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn can_decode(&self, data: &FormatProbe) -> bool;
    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn decode(&self, data: &[u8], opts: &DecodeOptions) -> PluginResult<DecodedImage>;
    async fn encode(&self, image: &PixelBuffer, metadata: &Metadata, opts: &EncodeOptions) -> PluginResult<Vec<u8>>;
}

// GPU 计算
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;
    async fn process_gpu(&self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer, params: &ParameterSet, progress: Box<dyn ProgressSink>) -> PluginResult<ProcessingStats>;
}

// AI 推理
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;
    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor>;
}

// 外部工具
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

pub struct ParameterSection {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub fields: Vec<ParameterField>,
}

pub struct ParameterField {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub help_url: Option<String>,
    pub field_type: ParameterType,
    pub default: serde_json::Value,
    pub required: bool,
    pub advanced: bool,
    pub allow_override: bool,
    pub supports_expression: bool,
}
```

### 2.5 ParameterType（18 种）

```rust
pub enum ParameterType {
    String { max_length: usize, pattern: Option<String>, placeholder: Option<String> },
    Integer { min: i64, max: i64, step: i64, unit: Option<String>, style: IntegerWidget },
    Float { min: f64, max: f64, step: f64, precision: u8, unit: Option<String>, logarithmic: bool, style: FloatWidget },
    Boolean { label_true: Option<String>, label_false: Option<String> },
    Enum { options: Vec<EnumOption>, display: EnumDisplay },
    Color { mode: ColorMode, show_alpha: bool },
    FilePath { kind: FilePathKind, filters: Vec<(String, String)>, must_exist: bool },
    Coordinate { alt_required: bool, direction_required: bool },
    Slider { min: f64, max: f64, step: f64, show_ticks: bool, ticks: Option<Vec<f64>>, show_value: bool, orientation: SliderOrientation, style: SliderStyle },
    ComboSlider { min: f64, max: f64, step: f64, presets: Vec<(String, f64)>, unit: Option<String> },
    Expression { variables: Vec<VariableDef> },
    Preset { preset_schema_ref: String, builtin_presets: Vec<NamedPreset>, allow_custom: bool, allow_import: bool },
    Array { element: Box<ParameterField>, min_items: usize, max_items: Option<usize> },
    MapWidget { show_track: bool, show_photos: bool, allow_manual_pin: bool },
    BeforeAfter { zoom_levels: Vec<f64>, show_histogram: bool },
    Separator { label: Option<String> },
    Section { fields: Vec<ParameterField> },
}
```

### 2.6 ParameterSet

```rust
pub struct ParameterSet {
    pub values: HashMap<String, serde_json::Value>,
}

impl ParameterSet {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: String, value: serde_json::Value);
    pub fn get(&self, key: &str) -> Option<&serde_json::Value>;
    pub fn get_str(&self, key: &str) -> Option<&str>;
    pub fn get_i64(&self, key: &str) -> Option<i64>;
    pub fn get_f64(&self, key: &str) -> Option<f64>;
    pub fn get_bool(&self, key: &str) -> Option<bool>;
    pub fn merge(&mut self, other: &ParameterSet);
    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)>;
}
```

### 2.7 Registry

```rust
pub struct Registry { /* 内部 DashMap */ }

impl Registry {
    pub fn new() -> Self;
    pub fn register(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()>;
    pub fn unregister(&self, id: &PluginId) -> Option<Arc<dyn Plugin>>;
    pub fn get(&self, id: &PluginId) -> Option<Arc<dyn Plugin>>;
    pub fn query(&self, q: &PluginQuery) -> Vec<Arc<dyn Plugin>>;
    pub fn by_category(&self, cat: PluginCategory) -> Vec<Arc<dyn Plugin>>;
    pub fn all(&self) -> Vec<Arc<dyn Plugin>>;
    pub fn manifest(&self, id: &PluginId) -> Option<PluginManifest>;
    pub fn manifests(&self) -> Vec<PluginManifest>;
    pub fn categories(&self) -> Vec<PluginCategory>;
    pub fn is_loaded(&self, id: &PluginId) -> bool;

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
pub struct BuiltinPluginLoader;
pub struct NativePluginLoader;
pub struct ExternalToolPluginLoader;

pub struct PluginLoaderManager {
    pub fn new() -> Self;
    pub fn add_search_path(&mut self, path: PathBuf);
    pub async fn discover_and_load(&self, registry: &Registry) -> PluginResult<Vec<PluginId>>;
}
```

### 2.9 Manifest 与 Query

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
    pub tags: Vec<String>,
    pub requires_pixel: Option<bool>,
    pub keyword: Option<String>,
    pub enabled_only: bool,
}
```

### 2.10 GuiSchema 相关（plugin crate 扩展）

```rust
pub struct NodePanelDefinition {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_category: String,
    pub sections: Vec<PanelSection>,
    pub context_bar: ContextBarConfig,
    pub help_text: Option<String>,
}

pub struct PanelSection { ... }
pub enum PanelWidget { TextInput, NumberInput, Slider, Toggle, Dropdown, SegmentedControl, CardSelector, FilePicker, ColorPicker, CoordinateInput, ComboSlider, MapWidget, ExpressionEditor, BeforeAfterPreview, NestedFields, Label }

pub struct ContextBarConfig {
    pub show_template_selector: bool,
    pub show_override_selector: bool,
    pub allow_per_image_override: bool,
}
```

---

## 3. Engine Crate — 管线引擎

Crate 名称：`photopipeline-engine`
路径：`crates/engine/src/`

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
    pub enabled: bool,
    pub position: (f64, f64),
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub parameter_overrides: Option<ParameterSet>,
}

impl PipelineGraph {
    pub fn new() -> Self;
    pub fn add_node(&mut self, plugin_id: String, label: String) -> NodeId;
    pub fn remove_node(&mut self, node_id: NodeId) -> bool;
    pub fn connect(&mut self, from_port: PortId, to_port: PortId) -> Result<(), PluginError>;
    pub fn disconnect(&mut self, from_port: PortId, to_port: PortId) -> bool;
    pub fn topological_order(&self) -> Result<Vec<NodeId>, PluginError>;
    pub fn has_cycle(&self) -> bool;
    pub fn validate_graph(&self) -> Result<(), Vec<String>>;
    pub fn from_template(template: &PipelineTemplate) -> Self;
    pub fn node(&self, id: NodeId) -> Option<&PipelineNode>;
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut PipelineNode>;
    pub fn port_owner(&self, port_id: PortId) -> Option<NodeId>;
}
```

### 3.2 PipelineTemplate (TOML 配置)

```rust
pub struct PipelineTemplate {
    pub metadata: TemplateMetadata,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub overrides: Vec<ImageOverride>,
    pub groups: Vec<ParamGroup>,
    pub batch: Option<BatchConfig>,
}

impl PipelineTemplate {
    pub fn validate(&self) -> Result<(), String>;
    pub fn into_graph(self) -> PipelineGraph;
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
    pub enabled: bool,
    pub params: Option<HashMap<String, serde_json::Value>>,
}

pub struct TemplateEdge {
    pub from: String,
    pub to: String,
}

pub struct ImageOverride {
    pub image: String,
    pub params: HashMap<String, ParameterSet>,
}

pub struct ParamGroup {
    pub name: String,
    pub condition: String,
    pub params: HashMap<String, ParameterSet>,
}

pub struct BatchConfig {
    pub parallel: usize,
    pub output_pattern: Option<String>,
    pub on_conflict: Option<String>,
    pub resume: bool,
}
```

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

```rust
pub enum NodeStatus {
    Pending,
    Running,
    Completed(ProcessingStats),
    Failed(String),
    Skipped,
}

pub struct NodeRunState {
    pub status: NodeStatus,
    pub started_at: Option<DateTime<Utc>>,
}

pub struct ExecutionContext {
    pub image_info: ImageInfo,
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}

pub struct ExecutionResult {
    pub buffer: Option<PixelBuffer>,
    pub metadata: Metadata,
    pub node_states: HashMap<NodeId, NodeRunState>,
}
```

### 3.4 ParameterResolver（四级参数优先级）

```rust
pub struct ParameterResolver {
    pub template_params: HashMap<NodeId, ParameterSet>,
    pub group_overrides: Vec<(GroupCondition, HashMap<NodeId, ParameterSet>)>,
    pub image_overrides: HashMap<(ImageId, NodeId), ParameterSet>,
    pub expr_engine: ExpressionEngine,
}

impl ParameterResolver {
    pub fn new() -> Self;
    pub fn set_template_params(&mut self, node_id: NodeId, params: ParameterSet);
    pub fn add_group_override(&mut self, condition: GroupCondition, params: HashMap<NodeId, ParameterSet>);
    pub fn set_image_override(&mut self, image_id: ImageId, node_id: NodeId, params: ParameterSet);

    pub fn resolve(
        &self, node_id: NodeId, image_id: ImageId,
        schema: &ParameterSchema, metadata: &Metadata, image_info: &ImageInfo,
    ) -> ParameterSet;

    pub fn resolve_single(&self, node_id: NodeId, schema: &ParameterSchema) -> ParameterSet;
}
```

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

### 3.5 ExpressionEngine

```rust
pub struct ExpressionEngine;

impl ExpressionEngine {
    pub fn evaluate(
        &self, expr: &str, metadata: &Metadata, image_info: &ImageInfo,
    ) -> Result<serde_json::Value, String>;
}
```

支持的表达式变量：
- `exif.iso`、`exif.aperture`、`exif.shutter`、`exif.focal_length`
- `exif.make`、`exif.model`、`exif.lens`
- `image.filename`、`image.width`、`image.height`、`image.filesize`
- 字面值：`123`、`3.14`、`"string"`、`'string'`
- 比较运算符：`>`、`<`、`>=`、`<=`、`==`、`!=`
- 三元运算符：`condition ? true_value : false_value`

### 3.6 TileEngine

```rust
pub struct TileEngine {
    pub default_tile_size: u32,    // 默认 1024
    pub overlap: u32,              // 默认 64
    pub max_parallel: usize,       // 自动检测 CPU 核心数
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

### 3.7 数据流与精度保证

```
Metadata plugins:     Arc<Metadata> 共享，0拷贝
Metadata → Metadata:  同一 Arc，始终共享
Metadata → Pixel:     写入时触发 COW（单消费者）
Pixel → Pixel（单消费者）：Arc 不做写时复制，原地修改
Pixel → Pixel（多消费者）：Arc 共享，只读
GPU → GPU:            GpuHandle 传递，VRAM 中保持
```

---

## 4. CLI — 命令与参数

路径：`cli/src/main.rs`

### 全局结构

```
photopipeline [GLOBAL_FLAGS] <COMMAND>

COMMANDS:
  pipeline  管线运行与验证
  plugin    插件列表与信息
  batch     批量处理
  help      打印帮助信息
```

### `pipeline` 子命令

```bash
photopipeline pipeline run \
  -c, --config <CONFIG>          # TOML 管线配置文件路径
  -i, --input <INPUT>            # 输入图像文件路径
  -o, --output <OUTPUT>          # 输出文件路径

photopipeline pipeline validate \
  -c, --config <CONFIG>          # TOML 管线配置文件路径
```

### `plugin` 子命令

```bash
photopipeline plugin list              # 列出所有已注册插件

photopipeline plugin info <PLUGIN_ID>  # 查看指定插件详情
```

### `batch` 子命令

```bash
photopipeline batch run \
  -c, --config <CONFIG>          # TOML 管线配置文件路径
  -p, --pattern <PATTERN>        # 文件匹配模式（glob，默认 "*.ARW"）
  -o, --output <OUTPUT_DIR>      # 输出目录（默认 "./output/"）

photopipeline batch validate \
  -c, --config <CONFIG>          # TOML 管线配置文件路径
  -p, --pattern <PATTERN>        # 文件匹配模式（默认 "*.ARW"）
```

### 配置加载

```rust
// cli/src/config.rs
pub fn load_template(content: &str) -> Result<PipelineTemplate, String>;
```

从 TOML 字符串反序列化为 `PipelineTemplate` 结构。

---

## 5. gRPC 服务接口

服务端入口：`crates/server/src/main.rs`
监听地址：`0.0.0.0:50051`

### 5.1 PipelineService

```protobuf
service PipelineService {
  rpc CreatePipeline(PipelineSpec) returns (PipelineId);
  rpc Execute(ExecuteRequest) returns (stream ExecuteProgress);
  rpc Validate(PipelineSpec) returns (ValidationResult);
  rpc GetNodeSchema(PluginId) returns (NodeSchema);
}
```

**Rust 实现：** `PipelineServiceImpl`（`crates/server/src/services/pipeline.rs`）

| RPC | 输入 | 输出 | 说明 |
|-----|------|------|------|
| `CreatePipeline` | `PipelineSpec` | `PipelineId` | 创建管线，返回 UUID |
| `Execute` | `ExecuteRequest` | `stream ExecuteProgress` | 流式执行管线 |
| `Validate` | `PipelineSpec` | `ValidationResult` | 验证管线配置 |
| `GetNodeSchema` | `PluginId` | `NodeSchema` | 获取插件参数 Schema |

### 5.2 ImageService

```protobuf
service ImageService {
  rpc Load(ImagePath) returns (ImageInfo);
  rpc Decode(DecodeRequest) returns (stream PixelDataChunk);
  rpc Encode(EncodeRequest) returns (stream EncodeProgress);
  rpc GetThumbnail(ThumbnailRequest) returns (ImageData);
}
```

**Rust 实现：** `ImageServiceImpl`（`crates/server/src/services/image.rs`）

| RPC | 输入 | 输出 | 说明 |
|-----|------|------|------|
| `Load` | `ImagePath` | `ImageInfo` | 加载图片信息 |
| `Decode` | `DecodeRequest` | `stream PixelDataChunk` | 流式解码 |
| `Encode` | `EncodeRequest` | `stream EncodeProgress` | 流式编码 |
| `GetThumbnail` | `ThumbnailRequest` | `ImageData` | 获取缩略图 |

### 5.3 BatchService

```protobuf
service BatchService {
  rpc SubmitBatch(BatchSpec) returns (BatchId);
  rpc GetProgress(BatchId) returns (stream BatchProgress);
  rpc Cancel(BatchId) returns (Empty);
}
```

**Rust 实现：** `BatchServiceImpl`（`crates/server/src/services/batch.rs`）

| RPC | 输入 | 输出 | 说明 |
|-----|------|------|------|
| `SubmitBatch` | `BatchSpec` | `BatchId` | 提交批量任务 |
| `GetProgress` | `BatchId` | `stream BatchProgress` | 流式获取进度 |
| `Cancel` | `BatchId` | `Empty` | 取消批量任务 |

---

## 6. Protobuf 消息定义

### 6.1 pipeline.proto

```protobuf
message PluginId { string id = 1; }

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

message PipelineId { string id = 1; }

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

### 6.2 image.proto

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

### 6.3 batch.proto

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

### 6.4 SharedState

```rust
#[derive(Default)]
pub struct SharedState {}
```

服务器共享状态（当前为空，为未来扩展预留：活跃管线列表、批量任务队列、进度缓存等）。

---

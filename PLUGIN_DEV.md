# Photopipeline 插件开发指南

## 目录

1. [插件系统概述](#1-插件系统概述)
2. [Plugin Trait 体系](#2-plugin-trait-体系)
3. [6 种能力 Trait](#3-6-种能力-trait)
4. [Registry 与加载机制](#4-registry-与加载机制)
5. [ParameterSchema 完整参数类型说明](#5-parameterschema-完整参数类型说明)
6. [GuiSchema 说明](#6-guischema-说明)
7. [开发新插件 Step-by-Step 教程](#7-开发新插件-step-by-step-教程)
8. [完整代码示例](#8-完整代码示例)
9. [插件打包与分发](#9-插件打包与分发)
10. [测试插件的方法](#10-测试插件的方法)

---

## 1. 插件系统概述

Photopipeline 插件系统采用 **Trait 分层 + Schema 驱动** 架构：

```
                        ┌──────────┐
                        │  Plugin   │  基础 Trait（所有插件必须实现）
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

**设计原则：**
- 每个插件必须实现基础 `Plugin` trait
- 根据功能选择性实现 1-6 个能力 trait
- 一个插件可以同时实现多个能力 trait（如 `ai_denoise` 同时实现 `PixelProcessor` + `AiProcessor`）
- 参数和 GUI 由 Schema 声明式定义，前端自动渲染
- 支持 5 种插件加载方式：Builtin / Native / WASM / ExternalTool / Remote

---

## 2. Plugin Trait 体系

### 2.1 基础 Plugin Trait

所有插件必须实现此 trait，定义插件的基本元信息：

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

#### 各方法说明

| 方法 | 说明 |
|------|------|
| `id()` | 全局唯一的插件标识符（如 `"photopipeline.plugins.ai_denoise"`） |
| `name()` | 用户友好的显示名称 |
| `version()` | 语义化版本号（`PluginVersion { major, minor, patch, pre }`） |
| `category()` | 插件类别（见下文 `PluginCategory`） |
| `description()` | 单行功能描述（显示在插件面板标题栏） |
| `tags()` | 搜索标签列表 |
| `requires_pixel_access()` | `true` 表示需要像素数据；`false` 表示纯元数据操作 |
| `produces_pixel_output()` | 是否输出新的/修改的像素缓冲区 |
| `supported_hardware()` | 硬件需求（最小 RAM / GPU 需求等） |
| `parameter_schema()` | 返回 `ParameterSchema` 静态引用，定义所有参数 |
| `gui_schema()` | 返回 `GuiSchema` 静态引用，定义 GUI 布局 |
| `initialize()` | 插件初始化（加载模型、连接外部工具等） |
| `shutdown()` | 资源清理 |
| `validate()` | 参数校验，返回 `ValidationIssue` 列表 |

#### PluginCategory

```rust
pub enum PluginCategory {
    Input,        // 输入/解码
    Metadata,     // 元数据读写
    Color,        // 色彩转换/调色
    Transform,    // 几何变换
    Enhance,      // 画质增强
    Merge,        // 合成/融合
    Format,       // 格式编码
    External,     // 外部工具
    Custom(String), // 自定义
}
```

---

## 3. 6 种能力 Trait

### 3.1 MetadataProcessor — 元数据处理

用于仅操作元数据（不访问像素）的插件：

```rust
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;

    async fn read_metadata(
        &self, target: &MetadataTarget, params: &ParameterSet,
    ) -> PluginResult<Metadata>;

    async fn write_metadata(
        &self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport>;
}
```

**何时实现：** 插件只需要读取或写入 EXIF/XMP/IPTC/GPS 元数据，不需要像素数据。

**示例插件：** `exif_rw`、`gps_set`、`time_shift`

**关键点：**
- `requires_pixel_access()` 必须返回 `false`
- `metadata_scope()` 声明操作范围（`EXIF` / `XMP` / `IPTC` / `GPS` / `All`）
- `read_metadata()` 从文件读取元数据
- `write_metadata()` 将元数据写回文件

### 3.2 PixelProcessor — 像素处理

用于访问和修改像素数据的插件：

```rust
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self)  -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self)   -> Vec<ColorSpace>;
    fn required_gpu_backend(&self)     -> Option<GpuBackend>;

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}
```

**何时实现：** 插件需要逐像素处理图像数据（色彩转换、缩放、滤镜等）。

**示例插件：** `colorspace`、`lut3d`、`transform`、`lens_correct`、`ai_denoise`

**关键点：**
- `requires_pixel_access()` 必须返回 `true`
- `process_pixels()` 接收输入 buffer 和可变输出 buffer
- 通过 `progress: Box<dyn ProgressSink>` 报告进度
- 返回 `ProcessingStats`（耗时、峰值内存、处理像素数等）
- `required_gpu_backend()` 返回 `None` 表示纯 CPU，`Some(GpuBackend::Auto)` 表示优先 GPU

### 3.3 FormatProcessor — 格式编解码

用于特定图像格式的编码和解码：

```rust
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
```

**何时实现：** 插件需要支持特定图像格式的读取和/或写入。

**示例插件：** `raw_input`、`heif_encoder`、`jxl_encoder`、`avif_encoder`、`tiff_encoder`、`png_encoder`

**关键点：**
- `format_id()` 返回格式枚举（`ImageFormat::HEIF` 等）
- `supported_extensions()` 返回扩展名与 MIME 类型对
- `can_decode()` / `can_encode()` 用于格式探测
- `decode()` 从字节数组解码为 `DecodedImage`（含 PixelBuffer + Metadata）
- `encode()` 将 PixelBuffer + Metadata 编码为字节数组

### 3.4 GpuProcessor — GPU 计算

用于在 GPU 上直接处理的插件（避免 CPU↔GPU 数据往返）：

```rust
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;

    async fn process_gpu(
        &self, ctx: &GpuContext, input: &GpuBuffer, output: &mut GpuBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}
```

**何时实现：** 插件在 GPU VRAM 中直接操作数据，无需拷贝到 CPU。

**支持的 GPU 后端：**
- `GpuBackend::CUDA` — NVIDIA CUDA
- `GpuBackend::Metal` — Apple Metal
- `GpuBackend::Vulkan` — 跨平台 Vulkan
- `GpuBackend::DirectX` — Windows DirectX
- `GpuBackend::OpenCL` — 通用 OpenCL
- `GpuBackend::ROCm` — AMD ROCm
- `GpuBackend::OpenVINO` — Intel OpenVINO
- `GpuBackend::Auto` — 自动选择最佳后端

### 3.5 AiProcessor — AI 推理

用于加载和运行 AI 模型的插件：

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

**何时实现：** 插件需要加载 AI 模型并执行推理。

**示例插件：** `ai_denoise`

**AI 后端：**
- `AiBackend::ONNX` — ONNX Runtime（CPU/CUDA）
- `AiBackend::TensorRT` — NVIDIA TensorRT
- `AiBackend::CoreML` — Apple CoreML / ANE
- `AiBackend::OpenVINO` — Intel OpenVINO
- `AiBackend::Burn` — Rust 原生 Burn 框架

**ModelInfo 结构：**

```rust
pub struct ModelInfo {
    pub name: String,           // 模型名称
    pub version: String,        // 模型版本
    pub source: ModelSource,    // 模型来源
    pub input_shape: Vec<usize>,// 输入形状
    pub output_shape: Vec<usize>,// 输出形状
    pub memory_mb: u64,         // 预估内存占用
    pub description: String,    // 描述
}

pub enum ModelSource {
    Bundled,                        // 内嵌于包内
    ExternalFile(String),           // 本地文件
    HuggingFace { repo, file },     // HuggingFace Hub
    Url(String),                    // HTTP 下载
}
```

### 3.6 ExternalToolProcessor — 外部工具

用于通过子进程调用外部工具的插件：

```rust
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;

    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(
        &self,
        input_paths: &[PathBuf],
        output_path: &PathBuf,
        params: &ParameterSet,
    ) -> PluginResult<()>;
}
```

**何时实现：** 插件作为外部命令行工具的包装器（如 ExifTool、ffmpeg 等）。

**关键点：**
- `trusted()` 返回 `true` 表示工具受信任，可安全自动执行
- `check_available()` 检查工具是否在系统路径中可用
- `execute()` 通过文件系统路径传递数据

---

## 4. Registry 与加载机制

### 4.1 Plugin Registry

`Registry` 是线程安全的全局插件注册表（基于 `DashMap` + `RwLock`）：

```rust
pub struct Registry {
    entries: DashMap<PluginId, RegistryEntry>,
    manifests: DashMap<PluginId, PluginManifest>,
    load_order: RwLock<Vec<PluginId>>,
    metadata_processors: DashMap<PluginId, Arc<dyn MetadataProcessor>>,
    pixel_processors: DashMap<PluginId, Arc<dyn PixelProcessor>>,
    format_processors: DashMap<PluginId, Arc<dyn FormatProcessor>>,
    gpu_processors: DashMap<PluginId, Arc<dyn GpuProcessor>>,
    ai_processors: DashMap<PluginId, Arc<dyn AiProcessor>>,
    external_tool_processors: DashMap<PluginId, Arc<dyn ExternalToolProcessor>>,
}
```

**核心方法：**

| 方法 | 说明 |
|------|------|
| `register(plugin)` | 注册基础插件（存储为 `Arc<dyn Plugin>`） |
| `register_metadata_processor(p)` | 注册元数据处理器 |
| `register_pixel_processor(p)` | 注册像素处理器 |
| `register_format_processor(p)` | 注册格式处理器 |
| `register_gpu_processor(p)` | 注册 GPU 处理器 |
| `register_ai_processor(p)` | 注册 AI 处理器 |
| `register_external_tool_processor(p)` | 注册外部工具处理器 |
| `unregister(id)` | 移除插件（清理所有关联） |
| `get(id)` | 获取基础插件引用 |
| `get_metadata_processor(id)` | 获取元数据处理器 |
| `get_pixel_processor(id)` | 获取像素处理器 |
| `get_format_processor(id)` | 获取格式处理器 |
| `get_gpu_processor(id)` | 获取 GPU 处理器 |
| `get_ai_processor(id)` | 获取 AI 处理器 |
| `get_external_tool_processor(id)` | 获取外部工具处理器 |
| `query(q)` | 按条件查询插件（分类/标签/关键词/像素需求） |
| `by_category(cat)` | 按分类查询 |
| `all()` | 获取所有插件 |
| `manifests()` | 获取所有插件 Manifest 列表 |
| `manifest(id)` | 获取单个 Manifest |

### 4.2 插件加载器

| Loader | 格式 | 热重载 | 用途 |
|--------|------|:--:|------|
| BuiltinPluginLoader | 编译进 binary | ✗ | 核心始终可用的插件 |
| NativePluginLoader | .so / .dll / .dylib | ✗ | 高性能第三方插件 |
| (WASM) | .wasm | ✓ | 安全沙箱第三方插件 |
| ExternalToolPluginLoader | 子进程调用 | ✗ | ExifTool / ffmpeg 等外部工具 |
| (Remote) | URL 下载安装 | ✗ | 插件市场分发 |

### 4.3 PluginManifest

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
```

### 4.4 PluginQuery

```rust
pub struct PluginQuery {
    pub category: Option<PluginCategory>,  // 按分类筛选
    pub tags: Vec<String>,                  // 标签匹配（AND）
    pub requires_pixel: Option<bool>,       // 按像素需求筛选
    pub keyword: Option<String>,            // 关键词搜索（名称/描述）
    pub enabled_only: bool,                 // 仅启用的插件
}
```

---

## 5. ParameterSchema 完整参数类型说明

### 5.1 ParameterSchema 结构

```rust
pub struct ParameterSchema {
    pub version: u32,                  // Schema 版本号
    pub sections: Vec<ParameterSection>, // 参数段落列表
}
```

### 5.2 ParameterSection

```rust
pub struct ParameterSection {
    pub id: String,                  // 段落 ID（用于 GUI 引用）
    pub label: String,               // 段落标题
    pub description: Option<String>,  // 段落描述
    pub icon: Option<String>,         // 图标名称
    pub collapsible: bool,            // 是否可折叠
    pub default_collapsed: bool,      // 默认折叠状态
    pub fields: Vec<ParameterField>,  // 参数字段列表
}
```

### 5.3 ParameterField

```rust
pub struct ParameterField {
    pub id: String,                  // 字段 ID（全局唯一）
    pub label: String,               // 字段标签
    pub description: Option<String>,  // 字段描述/帮助文本
    pub help_url: Option<String>,     // 在线帮助 URL
    pub field_type: ParameterType,    // 参数类型（见下方 18 种）
    pub default: serde_json::Value,   // 默认值
    pub required: bool,               // 是否必填
    pub advanced: bool,               // 是否为高级参数（默认折叠）
    pub allow_override: bool,         // 是否允许外部覆盖
    pub supports_expression: bool,    // 是否支持表达式
}
```

### 5.4 18 种 ParameterType

| # | 类型 | JSON type 标签 | 用途 | 关键属性 |
|:--:|------|:---:|------|------|
| 1 | **String** | `"string"` | 文本输入 | `max_length`, `pattern`（正则）, `placeholder` |
| 2 | **Integer** | `"integer"` | 整数值 | `min`, `max`, `step`, `unit`, `style`（SpinBox/Slider/Combo） |
| 3 | **Float** | `"float"` | 浮点值 | `min`, `max`, `step`, `precision`, `unit`, `logarithmic`, `style`（SpinBox/Slider/ComboSlider/DragInput） |
| 4 | **Boolean** | `"boolean"` | 开关 | `label_true`, `label_false` |
| 5 | **Enum** | `"enum"` | 枚举下拉 | `options: Vec<EnumOption>`, `display`（Dropdown/RadioGroup/ButtonGroup/SegmentedControl/Tabs/PopupCard） |
| 6 | **Color** | `"color"` | 颜色选择器 | `mode`（RGB/RGBA/HSL/HSV/Lab）, `show_alpha` |
| 7 | **FilePath** | `"file_path"` | 文件路径选择 | `kind`（File/Directory/SaveFile）, `filters`, `must_exist` |
| 8 | **Coordinate** | `"coordinate"` | 经纬度坐标 | `alt_required`, `direction_required` |
| 9 | **Slider** | `"slider"` | 滑块 | `min`, `max`, `step`, `show_ticks`, `ticks`, `show_value`, `orientation`, `style`（Continuous/Discrete/Range/DualHandle） |
| 10 | **ComboSlider** | `"combo_slider"` | 预设滑块 | `min`, `max`, `step`, `presets: Vec<(label, value)>`, `unit` |
| 11 | **Expression** | `"expression"` | 表达式编辑器 | `variables: Vec<VariableDef>`（名称/描述/类型/示例） |
| 12 | **Preset** | `"preset"` | 预设管理器 | `preset_schema_ref`, `builtin_presets`, `allow_custom`, `allow_import` |
| 13 | **Array** | `"array"` | 动态列表 | `element: Box<ParameterField>`（元素 schema）, `min_items`, `max_items` |
| 14 | **MapWidget** | `"map_widget"` | 地图选取器 | `show_track`, `show_photos`, `allow_manual_pin` |
| 15 | **BeforeAfter** | `"before_after"` | 前后对比预览 | `zoom_levels`, `show_histogram` |
| 16 | **Separator** | `"separator"` | 分隔线 | `label`（可选标题） |
| 17 | **Section** | `"section"` | 嵌套段落 | `fields: Vec<ParameterField>`（子字段） |
| 18 | **Nested** | — | 通过 `PanelSection` 的 `NestedFields` widget | `fields: Vec<PanelSection>` |

### 5.5 EnumOption

```rust
pub struct EnumOption {
    pub value: String,             // 枚举值
    pub label: String,             // 显示标签
    pub description: Option<String>,// 选项描述
    pub icon: Option<String>,       // 选项图标
    pub tags: Vec<String>,          // 过滤标签
    pub recommended: bool,          // 推荐选项（加星标记）
}
```

### 5.6 ParameterSet — 参数值容器

```rust
pub struct ParameterSet {
    pub values: HashMap<String, serde_json::Value>,
}

impl ParameterSet {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: String, value: Value);
    pub fn get(&self, key: &str)        -> Option<&Value>;
    pub fn get_str(&self, key: &str)    -> Option<&str>;
    pub fn get_i64(&self, key: &str)    -> Option<i64>;
    pub fn get_f64(&self, key: &str)    -> Option<f64>;
    pub fn get_bool(&self, key: &str)   -> Option<bool>;
    pub fn merge(&mut self, other: &ParameterSet);  // 浅合并
}
```

### 5.7 VariableDef（表达式变量定义）

```rust
pub struct VariableDef {
    pub name: String,              // 变量名
    pub description: String,        // 变量说明
    pub var_type: String,           // 类型（"number" | "string" | "boolean"）
    pub example: Option<String>,    // 示例值
}
```

### 5.8 NamedPreset

```rust
pub struct NamedPreset {
    pub name: String,                              // 预设名称
    pub description: Option<String>,                // 预设描述
    pub params: HashMap<String, serde_json::Value>,  // 预设参数键值对
}
```

---

## 6. GuiSchema 说明

### 6.1 GuiSchema 结构

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,           // 面板布局定义
    pub icon: Option<String>,         // 面板图标
    pub color: Option<String>,        // 面板主题色（十六进制）
    pub preview: PreviewMode,         // 预览模式
    pub aux_views: Vec<AuxView>,      // 辅助视图列表
    pub min_panel_width: u32,         // 最小面板宽度（px）
}
```

### 6.2 GuiLayout

```rust
pub enum GuiLayout {
    Standard { sections: Vec<GuiSection> },  // 标准段落布局
    Custom { rows: Vec<GuiRow> },            // 自定义行列布局
}
```

#### GuiSection

```rust
pub struct GuiSection {
    pub param_section_id: String,    // 引用 ParameterSection 的 id
    pub title_visible: bool,         // 是否显示标题
    pub style: SectionStyle,         // 段落样式
}
```

**SectionStyle：** `Default` / `Card` / `AccentCard` / `CollapsibleCard`

### 6.3 PreviewMode

```rust
pub enum PreviewMode {
    None,                         // 无预览
    Live,                         // 实时预览
    ManualRefresh,                // 手动刷新预览
    BeforeAfter {                 // 前后对比
        default_split: f32,        // 默认分割位置（0.0-1.0）
        orientation: SplitOrientation,
        lock_zoom: bool,           // 锁定缩放
    },
    Tiled { rows: u32, cols: u32 }, // 多宫格预览
}
```

### 6.4 AuxView（辅助视图）

```rust
pub enum AuxView {
    Histogram,        // 直方图
    Waveform,         // 波形图
    Vectorscope,      // 矢量图
    GamutDiagram,     // 色域图
    Map,              // 地图（GPS 数据）
    FocusPeaking,     // 对焦峰值
    ClippingWarning,  // 过曝/欠曝警告
    MetadataTable,    // 元数据表格
    ProgressBar,      // 进度条
    StatusText,       // 状态文本
}
```

### 6.5 GuiSchema 示例

```rust
static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection {
                    param_section_id: "basic".into(),
                    title_visible: true,
                    style: SectionStyle::Card,
                },
                GuiSection {
                    param_section_id: "advanced".into(),
                    title_visible: true,
                    style: SectionStyle::CollapsibleCard,
                },
            ],
        },
        icon: Some("sparkles".into()),
        color: Some("#3b82f6".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: photopipeline_core::SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![AuxView::Histogram, AuxView::ProgressBar],
        min_panel_width: 340,
    }
});
```

---

## 7. 开发新插件 Step-by-Step 教程

### Step 1：确定插件类型

决定你的插件属于哪个品类，需要实现哪些能力 trait：

| 场景 | Category | 能力 Trait |
|------|----------|------------|
| 读取新图像格式 | Format / Input | Plugin + FormatProcessor |
| 写入新图像格式 | Format | Plugin + FormatProcessor |
| 元数据处理 | Metadata | Plugin + MetadataProcessor |
| 像素滤镜/变换 | Enhance / Transform / Color | Plugin + PixelProcessor |
| AI 推理 | Enhance | Plugin + PixelProcessor + AiProcessor |
| GPU 计算 | 任意 | Plugin + GpuProcessor |
| 调用外部工具 | External | Plugin + ExternalToolProcessor |

### Step 2：创建插件结构体

在 `crates/plugins/src/` 下创建新模块文件，定义插件结构体：

```rust
use photopipeline_core::*;
use photopipeline_plugin::*;

#[derive(Debug, Clone)]
pub struct MyPlugin {
    id: String,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.my_plugin".to_string() }
    }
}
```

### Step 3：定义 ParameterSchema

使用 `LazyLock` 创建静态 schema，确保只初始化一次：

```rust
static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "basic".into(),
                label: "Basic Settings".into(),
                description: Some("Basic parameters".into()),
                icon: Some("wrench".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "strength".into(),
                        label: "Strength".into(),
                        description: Some("Effect strength (0-100)".into()),
                        help_url: None,
                        field_type: ParameterType::Slider {
                            min: 0.0, max: 100.0, step: 1.0,
                            show_ticks: true,
                            ticks: Some(vec![0.0, 50.0, 100.0]),
                            show_value: true,
                            orientation: Default::default(),
                            style: Default::default(),
                        },
                        default: serde_json::json!(50.0),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                ],
            },
        ],
    }
});
```

### Step 4：定义 GuiSchema

```rust
static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection {
                    param_section_id: "basic".into(),
                    title_visible: true,
                    style: SectionStyle::Card,
                },
            ],
        },
        icon: Some("star".into()),
        color: Some("#3b82f6".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: photopipeline_core::SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![],
        min_panel_width: 320,
    }
});
```

### Step 5：定义标签

```rust
static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec!["effect".into(), "my_plugin".into()]
});
```

### Step 6：实现 Plugin trait

```rust
#[async_trait]
impl Plugin for MyPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "My Plugin" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Enhance }
    fn description(&self) -> &str { "My custom image effect" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { true }
    fn produces_pixel_output(&self) -> bool { true }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement { min_ram_mb: 256, ..Default::default() }
    }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(v) = params.get_f64("strength") {
            if v < 0.0 || v > 100.0 {
                issues.push(ValidationIssue::Error {
                    param: "strength".into(),
                    message: "Strength must be between 0 and 100".into(),
                });
            }
        }
        Ok(issues)
    }
}
```

### Step 7：实现能力 Trait（以 PixelProcessor 为例）

```rust
#[async_trait]
impl PixelProcessor for MyPlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }

    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }

    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![ColorSpace::SRGB, ColorSpace::LINEAR_SRGB]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        None  // CPU only
    }

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "processing");

        let strength = params.get_f64("strength").unwrap_or(50.0);

        // 在此处实现像素处理逻辑
        // ...

        output.data.data.copy_from_slice(&input.data.data);
        output.width = input.width;
        output.height = input.height;
        output.layout = input.layout;
        output.format = input.format;
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        let pixels = input.pixel_count();
        progress.set_progress(1.0, "done");

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: 0,
            input_pixels: pixels,
            output_pixels: pixels,
        })
    }
}
```

### Step 8：注册插件

在 `crates/plugins/src/lib.rs` 的 `register_all` 函数中添加注册代码：

```rust
{
    let p: Arc<my_plugin::MyPlugin> = Arc::new(my_plugin::MyPlugin::new());
    let _ = registry.register(p.clone() as Arc<dyn Plugin>);
    let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    // 如果实现了 AiProcessor，也注册：
    // let _ = registry.register_ai_processor(p.clone() as Arc<dyn AiProcessor>);
}
```

同时在 `lib.rs` 顶部添加模块声明：
```rust
pub mod my_plugin;
```

### Step 9：编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_plugin_basics() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.name(), "My Plugin");
        assert_eq!(plugin.category(), PluginCategory::Enhance);
        assert!(plugin.requires_pixel_access());
    }

    #[test]
    fn test_validation_strength_range() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("strength".into(), serde_json::json!(150.0));
        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(!issues.is_empty());
    }
}
```

---

## 8. 完整代码示例

### 示例 1：Metadata 插件 — 添加版权信息

```rust
// crates/plugins/src/copyright_stamper.rs

use async_trait::async_trait;
use std::sync::LazyLock;
use std::process::Command;

use photopipeline_core::*;
use photopipeline_plugin::*;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "copyright".into(),
                label: "Copyright".into(),
                description: Some("Add copyright metadata".into()),
                icon: Some("copyright".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "copyright_text".into(),
                        label: "Copyright Text".into(),
                        description: Some("Copyright string to embed".into()),
                        help_url: None,
                        field_type: ParameterType::String {
                            max_length: 512,
                            pattern: None,
                            placeholder: Some("© 2024 Your Name".into()),
                        },
                        default: serde_json::json!(""),
                        required: true,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                ],
            },
        ],
    }
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection {
                    param_section_id: "copyright".into(),
                    title_visible: true,
                    style: SectionStyle::Card,
                },
            ],
        },
        icon: Some("copyright".into()),
        color: Some("#64748b".into()),
        preview: PreviewMode::None,
        aux_views: vec![],
        min_panel_width: 320,
    }
});

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec!["metadata".into(), "copyright".into(), "exif".into()]
});

#[derive(Debug, Clone)]
pub struct CopyrightStamper {
    id: String,
}

impl CopyrightStamper {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.copyright_stamper".to_string() }
    }
}

#[async_trait]
impl Plugin for CopyrightStamper {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "Copyright Stamper" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Metadata }
    fn description(&self) -> &str { "Embed copyright notice in image metadata" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { false }
    fn produces_pixel_output(&self) -> bool { false }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement { min_ram_mb: 64, ..Default::default() }
    }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let text = params.get_str("copyright_text").unwrap_or("");
        if text.is_empty() {
            issues.push(ValidationIssue::Error {
                param: "copyright_text".into(),
                message: "Copyright text is required".into(),
            });
        }
        Ok(issues)
    }
}

#[async_trait]
impl MetadataProcessor for CopyrightStamper {
    fn metadata_scope(&self) -> Vec<MetadataScope> {
        vec![MetadataScope::EXIF, MetadataScope::XMP]
    }

    async fn read_metadata(
        &self, _target: &MetadataTarget, _params: &ParameterSet,
    ) -> PluginResult<Metadata> {
        Ok(Metadata::default())
    }

    async fn write_metadata(
        &self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport> {
        let text = params.get_str("copyright_text").unwrap_or("");

        let mut cmd = Command::new("exiftool");
        cmd.arg("-overwrite_original");
        cmd.arg(format!("-Copyright={}", text));
        cmd.arg(format!("-XMP:Rights={}", text));
        cmd.arg(&target.path);

        let output = cmd.output().map_err(|e| PluginError::Io {
            plugin: self.id.clone(), error: e,
        })?;

        let tags_written = if output.status.success() { 2 } else { 0 };
        let warnings = if !output.status.success() {
            vec![String::from_utf8_lossy(&output.stderr).to_string()]
        } else { vec![] };

        Ok(MetadataWriteReport { tags_written, tags_skipped: 0, warnings })
    }
}
```

### 示例 2：Pixel 插件 — 亮度/对比度调整

```rust
// crates/plugins/src/brightness_contrast.rs

use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::*;
use photopipeline_plugin::*;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "adjustments".into(),
                label: "Adjustments".into(),
                description: Some("Brightness and contrast controls".into()),
                icon: Some("sun".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "brightness".into(),
                        label: "Brightness".into(),
                        description: Some("Brightness adjustment (-100 to 100)".into()),
                        help_url: None,
                        field_type: ParameterType::Slider {
                            min: -100.0, max: 100.0, step: 1.0,
                            show_ticks: true,
                            ticks: Some(vec![-100.0, -50.0, 0.0, 50.0, 100.0]),
                            show_value: true,
                            orientation: Default::default(),
                            style: Default::default(),
                        },
                        default: serde_json::json!(0.0),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "contrast".into(),
                        label: "Contrast".into(),
                        description: Some("Contrast adjustment (0-200, 100 = unchanged)".into()),
                        help_url: None,
                        field_type: ParameterType::Slider {
                            min: 0.0, max: 200.0, step: 1.0,
                            show_ticks: true,
                            ticks: Some(vec![0.0, 50.0, 100.0, 150.0, 200.0]),
                            show_value: true,
                            orientation: Default::default(),
                            style: Default::default(),
                        },
                        default: serde_json::json!(100.0),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: true,
                    },
                ],
            },
        ],
    }
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection {
                    param_section_id: "adjustments".into(),
                    title_visible: true,
                    style: SectionStyle::AccentCard,
                },
            ],
        },
        icon: Some("sun".into()),
        color: Some("#f59e0b".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: photopipeline_core::SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![AuxView::Histogram],
        min_panel_width: 340,
    }
});

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec!["color".into(), "brightness".into(), "contrast".into(), "adjust".into()]
});

#[derive(Debug, Clone)]
pub struct BrightnessContrastPlugin {
    id: String,
}

impl BrightnessContrastPlugin {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.brightness_contrast".to_string() }
    }
}

#[async_trait]
impl Plugin for BrightnessContrastPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "Brightness/Contrast" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Color }
    fn description(&self) -> &str { "Adjust image brightness and contrast" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { true }
    fn produces_pixel_output(&self) -> bool { true }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement { min_ram_mb: 128, ..Default::default() }
    }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> { Ok(()) }
    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(v) = params.get_f64("contrast") {
            if v < 0.0 || v > 200.0 {
                issues.push(ValidationIssue::Error {
                    param: "contrast".into(),
                    message: "Contrast must be between 0 and 200".into(),
                });
            }
        }
        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for BrightnessContrastPlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }

    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F32]
    }

    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![ColorSpace::SRGB, ColorSpace::LINEAR_SRGB]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        None
    }

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "applying brightness/contrast");

        let brightness = params.get_f64("brightness").unwrap_or(0.0) / 100.0; // -1.0 to 1.0
        let contrast = params.get_f64("contrast").unwrap_or(100.0) / 100.0;   // 0.0 to 2.0

        let total_bytes = input.data.data.len();
        output.data.data.copy_from_slice(&input.data.data);

        match input.format {
            PixelFormat::F32 => {
                let pixels = output.data.as_f32_slice();
                // F32 processing: value = (value + brightness) * contrast
                // (simplified — use interior mutability with unsafe for real impl)
            }
            _ => {
                // U8/U16 integer processing with lookup table or direct math
                let bytes_per_sample = input.format.bytes_per_channel();
                let channels = input.layout.channel_count() as usize;
                for i in (0..total_bytes).step_by(bytes_per_sample) {
                    // Apply brightness/contrast per sample
                    // (simplified placeholder)
                }
            }
        }

        output.width = input.width;
        output.height = input.height;
        output.layout = input.layout;
        output.format = input.format;
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        progress.set_progress(1.0, "done");

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: (total_bytes * 2) as u64 / (1024 * 1024),
            input_pixels: input.pixel_count(),
            output_pixels: input.pixel_count(),
        })
    }
}
```

---

## 9. 插件打包与分发

### 9.1 Builtin 方式（内置）

将插件源文件放入 `crates/plugins/src/`，在 `lib.rs` 中注册。插件随主程序一起编译。

**适用场景：** 核心功能、稳定可靠、不需要独立分发的插件。

### 9.2 Native 动态库方式

1. 创建独立的 Rust `cdylib` crate
2. 在 `Cargo.toml` 中设置 `[lib] crate-type = ["cdylib"]`
3. 插件提供 `plugin_entry` 导出函数返回 `Box<dyn Plugin>`
4. 在插件 `.so`/`.dylib`/`.dll` 同目录放置 `.toml` manifest 文件：

```toml
# my_plugin.toml
id = "com.vendor.my_plugin"
name = "My Plugin"
version = { major = 1, minor = 0, patch = 0 }
category = "enhance"
description = "A native plugin example"
tags = ["enhance"]
requires_pixel_access = true
requires_network = false
requires_filesystem = false
min_ram_mb = 256
```

5. 安装：将 `.so` 和 `.toml` 放入 Photopipeline 的 plugins 搜索路径

### 9.3 WASM 方式（计划中）

1. 编译 Rust 插件到 `wasm32-wasip1` 或 `wasm32-unknown-unknown`
2. 通过 Wasmtime/Wasmer 运行时加载
3. 支持热重载（修改 .wasm 文件后自动重载）
4. 沙箱安全隔离，限制文件系统和网络访问

### 9.4 ExternalTool 方式

创建调用外部工具的子进程包装器：

```rust
impl ExternalToolProcessor for MyToolPlugin {
    fn tool_id(&self) -> &str { "my_tool" }
    fn tool_version_requirement(&self) -> VersionRequirement {
        VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: None,
        }
    }
    fn trusted(&self) -> bool { false }

    async fn check_available(&self) -> PluginResult<ToolAvailability> {
        // 执行 `my_tool --version` 检查可用性
    }

    async fn execute(
        &self, input_paths: &[PathBuf], output_path: &PathBuf,
        params: &ParameterSet,
    ) -> PluginResult<()> {
        // 构建子进程命令并执行
    }
}
```

---

## 10. 测试插件的方法

### 10.1 单元测试

在插件模块文件底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_identity() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.id(), "photopipeline.plugins.my_plugin");
        assert_eq!(plugin.name(), "My Plugin");
        assert_eq!(plugin.version(), PluginVersion::new(1, 0, 0));
    }

    #[test]
    fn test_plugin_schema_has_fields() {
        let plugin = MyPlugin::new();
        let schema = plugin.parameter_schema();
        assert!(!schema.sections.is_empty());
        let defaults = schema.defaults();
        assert!(defaults.get("strength").is_some());
    }

    #[test]
    fn test_plugin_validation() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("strength".into(), serde_json::json!(50.0));

        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_plugin_validation_rejects_bad_value() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("strength".into(), serde_json::json!(-10.0));

        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(!issues.is_empty());
    }
}
```

注意：异步测试需要 `tokio-test` 依赖（或使用 `#[tokio::test]`）。

### 10.2 集成测试

在 `cli/tests/integration_test.rs` 中添加测试：

```rust
#[test]
fn test_my_plugin_registered() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let plugin = registry.get(&"photopipeline.plugins.my_plugin".to_string());
    assert!(plugin.is_some());
}

#[test]
fn test_my_plugin_in_pipeline_template() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "my_node".into(),
                plugin: "photopipeline.plugins.my_plugin".into(),
                label: Some("My Plugin".into()),
                enabled: true,
                params: Some({
                    let mut m = HashMap::new();
                    m.insert("strength".into(), serde_json::json!(75.0));
                    m
                }),
            },
        ],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    assert!(template.validate().is_ok());
    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 1);
}
```

### 10.3 运行测试

```bash
# 运行所有测试
cargo test --workspace --exclude photopipeline-server

# 运行特定模块测试
cargo test -p photopipeline-plugins -- my_plugin

# 运行集成测试
cargo test -p photopipeline-cli --test integration_test

# 同时运行静态分析
cargo clippy --workspace --exclude photopipeline-server -- -D warnings
cargo fmt --all -- --check
```

### 10.4 CLI 手动测试

```bash
# 1. 构建后验证插件注册
photopipeline plugin list | grep my_plugin

# 2. 查看插件详情
photopipeline plugin info photopipeline.plugins.my_plugin

# 3. 创建测试管线 TOML
cat > test_plugin.toml << 'EOF'
[metadata]
name = "Test My Plugin"

[[nodes]]
id = "source"
plugin = "photopipeline.plugins.raw_input"

[[nodes]]
id = "effect"
plugin = "photopipeline.plugins.my_plugin"
params = { strength = 75.0 }

[[nodes]]
id = "output"
plugin = "photopipeline.plugins.png_encoder"

[[edges]]
from = "source"
to = "effect"

[[edges]]
from = "effect"
to = "output"
EOF

# 4. 验证配置
photopipeline pipeline validate -c test_plugin.toml

# 5. 运行管线
photopipeline pipeline run -c test_plugin.toml -i test.jpg -o output.png
```

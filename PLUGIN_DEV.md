# Photopipeline 插件开发指南

> 本文档面向需要为 Photopipeline 开发自定义插件的开发者。涵盖 Trait 体系、Schema 定义、参数类型、完整教程和测试方法。

---

## 目录

1. [插件系统概述](#1-插件系统概述)
2. [Plugin Trait 体系](#2-plugin-trait-体系)
3. [能力 Trait 完整参考](#3-能力-trait-完整参考)
4. [Registry 与加载机制](#4-registry-与加载机制)
5. [ParameterSchema 完整参考](#5-parameterschema-完整参考)
6. [GuiSchema 完整参考](#6-guischema-完整参考)
7. [开发新插件 — Step-by-Step](#7-开发新插件--step-by-step)
8. [完整代码示例](#8-完整代码示例)
9. [插件打包与分发](#9-插件打包与分发)
10. [测试插件](#10-测试插件)
11. [最佳实践](#11-最佳实践)

---

## 1. 插件系统概述

Photopipeline 插件系统采用 **Trait 分层 + Schema 驱动** 架构。所有功能——包括 14 个内置插件——通过同一套公开 Trait 体系实现。

### 设计原则

1. **分层 Trait：** 每个插件必须实现基础 `Plugin` trait；根据功能选择性实现 1–6 种能力 trait。
2. **Schema 驱动 GUI：** 参数通过 `ParameterSchema` 声明式定义，GUI 面板由 Schema 自动生成。
3. **多态加载：** 支持 5 种加载方式——Builtin（编译内置）、Native（动态库）、WASM（沙箱）、ExternalTool（子进程）、Remote（市场分发）。
4. **组合优于继承：** 单个插件可同时实现多个能力 trait。例如 `ai_denoise` 同时实现 `PixelProcessor` + `AiProcessor`。

### Trait 层次图

```
                         ┌──────────┐
                         │  Plugin   │  基础 Trait（所有插件必须实现）
                         └─────┬────┘
            ┌──────────┐  ┌────┴────┐  ┌──────────────┐
            │Metadata  │  │  Pixel   │  │   Format     │
            │Processor │  │Processor │  │  Processor   │
            └──────────┘  └────┬────┘  └──────────────┘
                         ┌─────┴─────┐
                         │GpuProcessor│  ← GPU 计算扩展
                         └─────┬─────┘
                         ┌─────┴─────┐
                         │AiProcessor │  ← AI 推理扩展
                         └────────────┘
            ┌───────────────┐
            │ExternalTool   │  ← 外部工具透传
            │Processor      │
            └───────────────┘
```

### 数据流

```
文件 ──→ FormatProcessor.decode() ──→ PixelBuffer ──→ PixelProcessor.process_pixels() ──→ ... ──→ FormatProcessor.encode() ──→ 文件
                                        │
                            Metadata       │
                            Processor ─────┘ (共享 Metadata，零拷贝)
```

---

## 2. Plugin Trait 体系

### 2.1 基础 Plugin Trait

所有插件必须实现此 Trait，定义插件的基本元信息和生命周期。

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

| 方法 | 返回类型 | 说明 |
|---|---|---|
| `id()` | `&PluginId` | 全局唯一标识符，格式 `"photopipeline.plugins.{name}"` |
| `name()` | `&str` | 用户友好显示名称 |
| `version()` | `PluginVersion` | 语义化版本 `{ major, minor, patch, pre }` |
| `category()` | `PluginCategory` | 插件类别（Input / Metadata / Color / Transform / Enhance / Merge / Format / External / Custom） |
| `description()` | `&str` | 单行描述，显示在 GUI 面板标题栏 |
| `tags()` | `&[String]` | 搜索标签列表，用于 PluginQuery |
| `requires_pixel_access()` | `bool` | `true` = 需要像素数据；`false` = 纯元数据操作 |
| `produces_pixel_output()` | `bool` | 是否输出新/修改的像素缓冲区 |
| `supported_hardware()` | `HardwareRequirement` | 硬件需求：`requires_cpu`、`requires_gpu`、`min_ram_mb`、`preferred_backend` |
| `parameter_schema()` | `&ParameterSchema` | 返回静态 Schema 引用 |
| `gui_schema()` | `&GuiSchema` | 返回静态 GUI Schema 引用 |
| `initialize()` | `PluginResult<()>` | 异步初始化（加载模型、连接外部工具等） |
| `shutdown()` | `PluginResult<()>` | 异步资源清理 |
| `validate()` | `PluginResult<Vec<ValidationIssue>>` | 参数校验，返回 Issue 列表 |

**`requires_pixel_access()` 与 `produces_pixel_output()` 的关系：**

| `requires_pixel_access` | `produces_pixel_output` | 典型插件 |
|:---:|:---:|---|
| `false` | `false` | `exif_rw`、`gps_set`、`time_shift` |
| `true` | `true` | `colorspace`、`lut3d`、`transform`、`ai_denoise` |
| `true` | `false` | 输入插件（`raw_input` 产生像素输出但不消费上游像素） |
| `false` | `true` | （极少见——产生像素但不读取像素） |

### PluginCategory 枚举

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
    Custom(String), // 自定义类别
}
```

---

## 3. 能力 Trait 完整参考

### 3.1 MetadataProcessor — 元数据处理

用于仅操作元数据（不访问像素）的插件。

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

**何时实现：** 插件只读写 EXIF / XMP / IPTC / GPS 元数据，不接触像素数据。

**内置示例：** `exif_rw`、`gps_set`、`time_shift`

**关键契约：**
- `requires_pixel_access()` 必须返回 `false`
- `metadata_scope()` 声明操作范围
- `read_metadata()` 从 `MetadataTarget` 读取元数据
- `write_metadata()` 将元数据写回目标文件，返回 `MetadataWriteReport`

### 3.2 PixelProcessor — 像素处理

用于访问和修改像素数据的插件。

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

**何时实现：** 插件需要逐像素处理图像数据。

**内置示例：** `colorspace`、`lut3d`、`transform`、`lens_correct`、`ai_denoise`

**关键契约：**
- `requires_pixel_access()` 必须返回 `true`
- `process_pixels()` 接收不可变输入和可变输出 buffer
- 通过 `progress: Box<dyn ProgressSink>` 报告进度
- 返回 `ProcessingStats`（耗时、峰值内存、处理像素数等）
- `required_gpu_backend()` 返回 `None` 表示纯 CPU 处理

### 3.3 FormatProcessor — 格式编解码

用于特定图像格式的编码和解码。

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

**内置示例：** `raw_input`、`heif_encoder`、`jxl_encoder`、`avif_encoder`、`tiff_encoder`、`png_encoder`

**关键契约：**
- `format_id()` 返回 `ImageFormat` 枚举值
- `supported_extensions()` 返回扩展名与 MIME 类型的元组列表
- `can_decode()` / `can_encode()` 用于格式探测
- `decode()` 从字节数组解码为 `DecodedImage`（含 PixelBuffer + Metadata）
- `encode()` 将 PixelBuffer + Metadata 编码为字节数组

### 3.4 GpuProcessor — GPU 计算

用于在 GPU VRAM 中直接操作的插件，避免 CPU↔GPU 数据往返。

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

**支持的 GPU 后端：**

| 枚举值 | 说明 | 平台 |
|---|---|---|
| `GpuBackend::CUDA` | NVIDIA CUDA | Linux / Windows |
| `GpuBackend::Metal` | Apple Metal | macOS |
| `GpuBackend::Vulkan` | Vulkan 跨平台 | 全平台 |
| `GpuBackend::DirectX` | DirectX | Windows |
| `GpuBackend::OpenCL` | OpenCL 通用 | 全平台 |
| `GpuBackend::ROCm` | AMD ROCm | Linux |
| `GpuBackend::OpenVINO` | Intel OpenVINO | 全平台 |
| `GpuBackend::Auto` | 自动选择最佳 | 全平台 |
| `GpuBackend::None` | 不可用 | — |

### 3.5 AiProcessor — AI 推理

用于加载和运行 AI 模型的插件。

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

**内置示例：** `ai_denoise`

**AI 后端：**

| 后端 | 说明 |
|---|---|
| `AiBackend::ONNX` | ONNX Runtime（CPU / CUDA） |
| `AiBackend::TensorRT` | NVIDIA TensorRT |
| `AiBackend::CoreML` | Apple CoreML / ANE |
| `AiBackend::OpenVINO` | Intel OpenVINO |
| `AiBackend::Burn` | Rust 原生 Burn 框架 |

**ModelInfo 结构：**

```rust
pub struct ModelInfo {
    pub name: String,           // 模型名称
    pub version: String,        // 模型版本
    pub source: ModelSource,    // 模型来源
    pub input_shape: Vec<usize>,// 输入张量形状
    pub output_shape: Vec<usize>,// 输出张量形状
    pub memory_mb: u64,         // 预估内存占用
    pub description: String,    // 模型描述
}

pub enum ModelSource {
    Bundled,                        // 内嵌于包内
    ExternalFile(String),           // 本地文件路径
    HuggingFace { repo, file },     // HuggingFace Hub 自动下载
    Url(String),                    // HTTP 远程下载
}
```

### 3.6 ExternalToolProcessor — 外部工具

用于通过子进程调用外部命令行工具的插件。

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

**何时实现：** 插件作为外部命令行工具的包装器。

**关键契约：**
- `trusted()` 返回 `true` 表示工具受信任，可安全自动执行
- `check_available()` 检查工具是否在系统 PATH 中可用
- `execute()` 通过文件系统路径传递数据（适用于不支持流式数据的工具）

### 3.7 ProgressSink

```rust
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}
```

在 `process_pixels()` 和 `process_gpu()` 中定期调用 `set_progress()` 报告进度，调用 `is_canceled()` 检查取消信号。

---

## 4. Registry 与加载机制

### 4.1 Registry

`Registry` 是线程安全的全局插件注册表，基于 `DashMap` + `RwLock` 实现。

**核心方法：**

| 方法 | 说明 |
|---|---|
| `register(plugin: Arc<dyn Plugin>)` | 注册基础插件 |
| `register_metadata_processor(p)` | 注册 Metadata 处理器 |
| `register_pixel_processor(p)` | 注册 Pixel 处理器 |
| `register_format_processor(p)` | 注册 Format 处理器 |
| `register_gpu_processor(p)` | 注册 GPU 处理器 |
| `register_ai_processor(p)` | 注册 AI 处理器 |
| `register_external_tool_processor(p)` | 注册 External Tool 处理器 |
| `unregister(id)` | 移除插件（清理所有关联） |
| `get(id)` | 获取基础 Plugin 引用 |
| `get_pixel_processor(id)` | 获取 Pixel 处理器 |
| `get_format_processor(id)` | 获取 Format 处理器 |
| `query(q: &PluginQuery)` | 条件查询 |
| `by_category(cat)` | 按分类查询 |
| `all()` | 获取所有插件 |

**PluginQuery 查询条件：**

```rust
pub struct PluginQuery {
    pub category: Option<PluginCategory>,  // 按分类筛选
    pub tags: Vec<String>,                  // 标签匹配（AND）
    pub requires_pixel: Option<bool>,       // 按像素需求筛选
    pub keyword: Option<String>,            // 关键词搜索
    pub enabled_only: bool,                 // 仅已启用
}
```

### 4.2 插件加载器

| 加载器 | 格式 | 热重载 | 适用场景 |
|---|---|---|---|
| `BuiltinPluginLoader` | 编译内嵌 | ✗ | 核心始终可用的插件 |
| `NativePluginLoader` | .so / .dll / .dylib | ✗ | 高性能第三方插件 |
| WASM (计划中) | .wasm | ✓ | 安全沙箱第三方插件 |
| `ExternalToolPluginLoader` | 子进程调用 | ✗ | ExifTool / ffmpeg 等外部工具 |
| Remote (计划中) | URL 下载安装 | ✗ | 插件市场分发 |

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

---

## 5. ParameterSchema 完整参考

### 5.1 结构

```rust
pub struct ParameterSchema {
    pub version: u32,                  // Schema 版本号
    pub sections: Vec<ParameterSection>, // 参数段落列表
}
```

**方法：**
- `empty()` — 创建空 Schema
- `field(section_id, field_id)` — 按 ID 查找字段
- `defaults()` — 返回全部字段默认值的 `ParameterSet`
- `all_fields()` — 获取所有字段引用

### 5.2 ParameterSection

```rust
pub struct ParameterSection {
    pub id: String,                  // 段落 ID
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
    pub field_type: ParameterType,    // 参数类型
    pub default: serde_json::Value,   // 默认值
    pub required: bool,               // 是否必填
    pub advanced: bool,               // 是否高级参数（默认折叠）
    pub allow_override: bool,         // 是否允许外部覆盖
    pub supports_expression: bool,    // 是否支持表达式
}
```

### 5.4 18 种 ParameterType

#### 1. String — 文本输入

```rust
ParameterType::String {
    max_length: usize,
    pattern: Option<String>,     // 正则表达式验证
    placeholder: Option<String>, // 占位文本
}
```

#### 2. Integer — 整数值

```rust
ParameterType::Integer {
    min: i64,
    max: i64,
    step: i64,
    unit: Option<String>,        // 单位标签，如 "px"
    style: IntegerWidget,        // SpinBox | Slider | Combo
}
```

#### 3. Float — 浮点值

```rust
ParameterType::Float {
    min: f64,
    max: f64,
    step: f64,
    precision: u8,               // 小数位数
    unit: Option<String>,        // 单位标签
    logarithmic: bool,           // 对数滑块
    style: FloatWidget,          // SpinBox | Slider | ComboSlider | DragInput
}
```

#### 4. Boolean — 开关

```rust
ParameterType::Boolean {
    label_true: Option<String>,   // 开启标签
    label_false: Option<String>,  // 关闭标签
}
```

#### 5. Enum — 枚举

```rust
ParameterType::Enum {
    options: Vec<EnumOption>,     // 选项列表
    display: EnumDisplay,         // Dropdown | RadioGroup | ButtonGroup | SegmentedControl | Tabs | PopupCard
}
```

#### 6. Color — 颜色选择器

```rust
ParameterType::Color {
    mode: ColorMode,              // RGB | RGBA | HSL | HSV | Lab
    show_alpha: bool,             // 是否显示 alpha 通道
}
```

#### 7. FilePath — 文件路径选择器

```rust
ParameterType::FilePath {
    kind: FilePathKind,           // File | Directory | SaveFile
    filters: Vec<(String, String)>, // (标签, 模式) 如 ("LUT Files", "*.cube")
    must_exist: bool,             // 路径必须存在
}
```

#### 8. Coordinate — 经纬度坐标

```rust
ParameterType::Coordinate {
    alt_required: bool,           // 海拔必填
    direction_required: bool,     // 方向必填
}
```

#### 9. Slider — 滑块

```rust
ParameterType::Slider {
    min: f64,
    max: f64,
    step: f64,
    show_ticks: bool,             // 显示刻度
    ticks: Option<Vec<f64>>,      // 自定义刻度位置
    show_value: bool,             // 显示当前值
    orientation: SliderOrientation, // Horizontal | Vertical
    style: SliderStyle,           // Continuous | Discrete | Range | DualHandle
}
```

#### 10. ComboSlider — 预设滑块

```rust
ParameterType::ComboSlider {
    min: f64,
    max: f64,
    step: f64,
    presets: Vec<(String, f64)>,  // 预设标签-值对
    unit: Option<String>,
}
```

#### 11. Expression — 表达式编辑器

```rust
ParameterType::Expression {
    variables: Vec<VariableDef>,  // 可用变量列表
}
```

**VariableDef：**

```rust
pub struct VariableDef {
    pub name: String,             // 变量名
    pub description: String,       // 变量说明
    pub var_type: String,          // 类型: "number" | "string" | "boolean"
    pub example: Option<String>,   // 示例值
}
```

#### 12. Preset — 预设管理器

```rust
ParameterType::Preset {
    preset_schema_ref: String,       // 引用参数 Schema ID
    builtin_presets: Vec<NamedPreset>, // 内置预设
    allow_custom: bool,              // 允许用户自定义
    allow_import: bool,              // 允许导入外部预设
}
```

#### 13. Array — 动态列表

```rust
ParameterType::Array {
    element: Box<ParameterField>,  // 元素 Schema
    min_items: usize,              // 最小条目数
    max_items: Option<usize>,      // 最大条目数
}
```

#### 14. MapWidget — 地图选取器

```rust
ParameterType::MapWidget {
    show_track: bool,              // 显示轨迹
    show_photos: bool,             // 显示照片位置
    allow_manual_pin: bool,        // 允许手动钉选
}
```

#### 15. BeforeAfter — 前后对比预览

```rust
ParameterType::BeforeAfter {
    zoom_levels: Vec<f64>,         // 可用缩放级别
    show_histogram: bool,          // 显示直方图
}
```

#### 16. Separator — 分隔线

```rust
ParameterType::Separator {
    label: Option<String>,         // 分隔线标签（可选）
}
```

#### 17. Section — 嵌套段落

```rust
ParameterType::Section {
    fields: Vec<ParameterField>,   // 子字段列表
}
```

### 5.5 ParameterSet — 参数值容器

```rust
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
    pub fn merge(&mut self, other: &ParameterSet);  // 浅合并
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)>;
}
```

---

## 6. GuiSchema 完整参考

### 6.1 GuiSchema 结构

```rust
pub struct GuiSchema {
    pub layout: GuiLayout,           // 面板布局
    pub icon: Option<String>,         // 面板图标
    pub color: Option<String>,        // 面板主题色（十六进制）
    pub preview: PreviewMode,         // 预览模式
    pub aux_views: Vec<AuxView>,      // 辅助视图
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

**GuiSection：**

```rust
pub struct GuiSection {
    pub param_section_id: String,    // 引用 ParameterSection::id
    pub title_visible: bool,         // 是否显示标题
    pub style: SectionStyle,         // Default | Card | AccentCard | CollapsibleCard
}
```

**GuiRow / GuiCell：**

```rust
pub struct GuiRow {
    pub height: RowHeight,           // Compact | Normal | Expanded | Custom(u32)
    pub cells: Vec<GuiCell>,
}

pub struct GuiCell {
    pub param_field_id: String,      // 引用 ParameterField::id
    pub width_fraction: f64,         // 列宽比例
    pub label_position: LabelPosition, // Top | Left | None
}
```

### 6.3 PreviewMode

```rust
pub enum PreviewMode {
    None,
    Live,                            // 实时预览
    ManualRefresh,                   // 手动刷新
    BeforeAfter {                    // 前/后对比
        default_split: f32,
        orientation: SplitOrientation,
        lock_zoom: bool,
    },
    Tiled { rows: u32, cols: u32 }, // 多宫格
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

---

## 7. 开发新插件 — Step-by-Step

### Step 1：确定插件类型

| 场景 | Category | 能力 Trait |
|---|---|---|
| 读取新图像格式 | Input / Format | `Plugin` + `FormatProcessor` |
| 写入新图像格式 | Format | `Plugin` + `FormatProcessor` |
| 元数据处理 | Metadata | `Plugin` + `MetadataProcessor` |
| 像素滤镜/变换 | Enhance / Transform / Color | `Plugin` + `PixelProcessor` |
| AI 推理 | Enhance | `Plugin` + `PixelProcessor` + `AiProcessor` |
| GPU 计算 | 任意 | `Plugin` + `GpuProcessor` |
| 调用外部工具 | External | `Plugin` + `ExternalToolProcessor` |

### Step 2：创建插件结构体

在 `crates/plugins/src/` 下创建新文件 `my_plugin.rs`：

```rust
use photopipeline_core::*;
use photopipeline_plugin::*;

#[derive(Debug, Clone)]
pub struct MyPlugin {
    id: String,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.my_plugin".to_string(),
        }
    }
}
```

### Step 3：定义静态 Schema

使用 `LazyLock` 确保只初始化一次：

```rust
use std::sync::LazyLock;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "main".into(),
                label: "主要设置".into(),
                description: Some("核心参数".into()),
                icon: Some("wrench".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "strength".into(),
                        label: "强度".into(),
                        description: Some("效果强度 (0–100)".into()),
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
                        supports_expression: true,
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
                    param_section_id: "main".into(),
                    title_visible: true,
                    style: SectionStyle::Card,
                },
            ],
        },
        icon: Some("star".into()),
        color: Some("#3b82f6".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![AuxView::Histogram],
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

### Step 6：实现 Plugin Trait

```rust
#[async_trait]
impl Plugin for MyPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "My Plugin" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Enhance }
    fn description(&self) -> &str { "自定义图像效果" }
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
                    message: "强度必须在 0 到 100 之间".into(),
                });
            }
        }
        Ok(issues)
    }
}
```

### Step 7：实现能力 Trait

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
        None // CPU only
    }

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "processing");

        let strength = params.get_f64("strength").unwrap_or(50.0) / 100.0;

        // 在此处实现像素处理逻辑
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

在 `crates/plugins/src/lib.rs` 中添加：

```rust
pub mod my_plugin;

// 在 register_all() 函数中添加：
{
    let p: Arc<my_plugin::MyPlugin> = Arc::new(my_plugin::MyPlugin::new());
    let _ = registry.register(p.clone() as Arc<dyn Plugin>);
    let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
}
```

### Step 9：验证注册

```bash
cargo build
photopipeline plugin list | grep my_plugin
photopipeline plugin info photopipeline.plugins.my_plugin
```

---

## 8. 完整代码示例

### 示例 1：Metadata 插件 — 版权印章

完整实现见 `crates/plugins/src/exif_rw.rs`（参考模式），核心要点：

1. `requires_pixel_access()` 返回 `false`
2. 实现 `MetadataProcessor::write_metadata()` 通过 `exiftool` 子进程写入
3. Schema 包含一个必填的 String 类型字段 `copyright_text`
4. 返回 `MetadataWriteReport` 统计写入/跳过的标签数

### 示例 2：Pixel 插件 — 亮度/对比度

完整实现见 `crates/plugins/src/colorspace.rs`（参考模式），核心要点：

1. `requires_pixel_access()` 返回 `true`
2. Schema 包含 Float 类型的 `brightness` 和 `contrast` 参数
3. `process_pixels()` 依据 `PixelFormat` 分派处理
4. 通过 `AlignedBuffer` 的 `as_u16_slice()` / `as_f32_slice()` 安全访问像素数据

### 示例 3：Format 插件 — 编码器

完整实现见 `crates/plugins/src/heif_encoder.rs`，核心要点：

1. 实现 `FormatProcessor` trait
2. `format_id()` 返回 `ImageFormat::HEIF`
3. `encode()` 调用 libheif 编码为字节数组
4. `can_encode()` 返回 `true` 以标记支持的格式

---

## 9. 插件打包与分发

### 9.1 Builtin 方式

将插件源文件放入 `crates/plugins/src/`，在 `lib.rs` 的 `register_all()` 中注册。插件随主程序编译。

**优势：** 始终可用，无加载失败风险。

### 9.2 Native 动态库方式

1. 创建独立 Rust `cdylib` crate：

   ```toml
   # Cargo.toml
   [lib]
   crate-type = ["cdylib"]
   ```

2. 提供 `plugin_entry` 导出函数返回 `Box<dyn Plugin>`。

3. 在同目录放置 `.toml` manifest 文件：

   ```toml
   id = "com.vendor.my_plugin"
   name = "My Plugin"
   version = { major = 1, minor = 0, patch = 0 }
   category = "enhance"
   description = "A native plugin example"
   tags = ["enhance", "custom"]
   requires_pixel_access = true
   requires_network = false
   requires_filesystem = false
   min_ram_mb = 256
   ```

4. 安装：将 `.so`/`.dll`/`.dylib` 和 `.toml` 放入 plugins 搜索路径。

### 9.3 ExternalTool 方式

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
        // 执行 my_tool --version 检查可用性
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

## 10. 测试插件

### 10.1 单元测试

在插件模块文件底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_identity() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.id(), "photopipeline.plugins.my_plugin");
        assert_eq!(plugin.name(), "My Plugin");
    }

    #[test]
    fn test_schema_defaults() {
        let plugin = MyPlugin::new();
        let defaults = plugin.parameter_schema().defaults();
        assert!(defaults.get_f64("strength").is_some());
        assert_eq!(defaults.get_f64("strength"), Some(50.0));
    }

    #[test]
    fn test_validation_accepts_valid() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("strength".into(), serde_json::json!(75.0));
        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validation_rejects_out_of_range() {
        let plugin = MyPlugin::new();
        let mut params = ParameterSet::new();
        params.insert("strength".into(), serde_json::json!(150.0));
        let issues = tokio_test::block_on(plugin.validate(&params)).unwrap();
        assert!(!issues.is_empty());
    }
}
```

### 10.2 集成测试

在 `cli/tests/integration_test.rs` 中添加：

```rust
#[test]
fn test_my_plugin_registered() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    assert!(registry.is_loaded(&"photopipeline.plugins.my_plugin".into()));
}
```

### 10.3 运行测试

```bash
# 全部测试
cargo test --workspace --exclude photopipeline-server

# 指定模块
cargo test -p photopipeline-plugins -- my_plugin

# 集成测试
cargo test -p photopipeline-cli --test integration_test

# 同时运行静态检查
cargo clippy --workspace --exclude photopipeline-server -- -D warnings
cargo fmt --all -- --check
```

---

## 11. 最佳实践

### 命名约定

| 元素 | 约定 | 示例 |
|---|---|---|
| 插件 ID | `photopipeline.plugins.{snake_case}` | `photopipeline.plugins.my_effect` |
| 参数 ID | `snake_case` | `denoise_strength` |
| Schema section ID | `snake_case` | `basic_settings` |
| 静态变量 | `SCREAMING_SNAKE_CASE` | `PARAMETER_SCHEMA` |

### 错误处理

- 始终通过 `PluginResult<T>` 返回错误。
- 校验返回 `Vec<ValidationIssue>` 而非早期 `Err`。
- 避免 `unwrap()` / `expect()`，使用 `?` 传播错误。
- 使用 `PluginError::Internal { plugin, message }` 处理意外错误。

### 性能

- Metadata 插件应设置 `requires_pixel_access() = false` 以启用零拷贝路径。
- Pixel 处理器应通过 `ProgressSink::is_canceled()` 检查取消，支持可中断处理。
- 大图像使用 `TileEngine` 分块处理（默认 1024px + 64px 重叠）。
- 避免在 `process_pixels()` 中执行同步 I/O 操作。

### GPU 用法

- GPU 插件应实现 `GpuProcessor`，在 VRAM 中直接操作。
- 提供 CPU 后备路径（通过 `required_gpu_backend()` 控制）。
- 在 `process_gpu()` 中处理 `GpuBackend::Auto`，自动选择可用 GPU。

### Schema 设计

- 将常用参数放在非折叠段落，高级参数放在折叠段落。
- 为每个参数提供有意义的 `description`。
- 设置合理的 `default` 值——这是 90% 用户将使用的值。
- 对数值参数（denoise_strength、quality 等）设置 `supports_expression = true`。

### Pre‑commit 检查清单

- [ ] `requires_pixel_access()` 与 `produces_pixel_output()` 正确设置
- [ ] `ParameterSchema` 完整定义所有参数
- [ ] `GuiSchema` 包含适当的预览模式和辅助视图
- [ ] `validate()` 覆盖边界值和错误输入
- [ ] 单元测试覆盖 identity、schema、validation
- [ ] 在 `register_all()` 中注册所有能力 trait
- [ ] `cargo build` 通过，`cargo test` 通过，`clippy` 无警告

---

use async_trait::async_trait;
use photopipeline_core::{
    AiBackend, ColorSpace, DecodeOptions, DecodedImage, EncodeOptions, FormatProbe, GpuBackend,
    GpuBuffer, GpuContext, GuiSchema, HardwareRequirement, ImageFormat, ImageInfo, Metadata,
    MetadataScope, MetadataTarget, MetadataWriteReport, PixelBuffer, PixelFormat, PluginCategory,
    PluginResult, PluginVersion, ProcessingStats, Tensor, ValidationIssue, VersionRequirement,
};

use crate::schema::{ParameterSchema, ParameterSet};
pub use photopipeline_core::PluginId;

// ---- Progress reporting ----
pub trait ProgressSink: Send + Sync {
    fn set_progress(&self, fraction: f32, message: &str);
    fn is_canceled(&self) -> bool;
}

// ---- Config ----
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub search_paths: Vec<String>,
    pub settings: std::collections::HashMap<String, String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            search_paths: vec![],
            settings: Default::default(),
        }
    }
}

// ---- Manifest ----
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub dependencies: std::collections::HashMap<String, String>,
}

// ---- Base Plugin Trait ----
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
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

// ---- Metadata Processor Trait ----
#[async_trait]
pub trait MetadataProcessor: Plugin {
    fn metadata_scope(&self) -> Vec<MetadataScope>;

    async fn read_metadata(
        &self,
        target: &MetadataTarget,
        params: &ParameterSet,
    ) -> PluginResult<Metadata>;

    async fn write_metadata(
        &self,
        target: &mut MetadataTarget,
        metadata: &Metadata,
        params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport>;
}

// ---- Pixel Processor Trait ----
#[async_trait]
pub trait PixelProcessor: Plugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat>;
    fn supported_output_formats(&self) -> Vec<PixelFormat>;
    fn supported_color_spaces(&self) -> Vec<ColorSpace>;
    fn required_gpu_backend(&self) -> Option<GpuBackend>;

    async fn process_pixels(
        &self,
        input: &PixelBuffer,
        output: &mut PixelBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}

// ---- Format Processor Trait ----
#[async_trait]
pub trait FormatProcessor: Plugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)>;
    fn format_id(&self) -> ImageFormat;

    fn can_decode(&self, data: &FormatProbe) -> bool;
    async fn decode(&self, data: &[u8], options: &DecodeOptions) -> PluginResult<DecodedImage>;

    fn can_encode(&self, format: &ImageFormat) -> bool;
    async fn encode(
        &self,
        image: &PixelBuffer,
        metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>>;
}

// ---- GPU Processor Trait ----
#[async_trait]
pub trait GpuProcessor: Plugin {
    fn supported_backends(&self) -> Vec<GpuBackend>;
    fn gpu_memory_required(&self, info: &ImageInfo, params: &ParameterSet) -> u64;

    async fn process_gpu(
        &self,
        ctx: &GpuContext,
        input: &GpuBuffer,
        output: &mut GpuBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats>;
}

// ---- AI Processor Trait ----
#[async_trait]
pub trait AiProcessor: Plugin {
    fn model_info(&self) -> &ModelInfo;

    fn supported_backends(&self) -> Vec<AiBackend>;
    async fn load_model(&mut self, backend: &AiBackend) -> PluginResult<()>;
    async fn unload_model(&mut self) -> PluginResult<()>;
    async fn infer(&self, input: &Tensor, params: &ParameterSet) -> PluginResult<Tensor>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub source: ModelSource,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub memory_mb: u64,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ModelSource {
    Bundled,
    ExternalFile(String),
    HuggingFace { repo: String, file: String },
    Url(String),
}

// ---- External Tool Processor Trait ----
#[async_trait]
pub trait ExternalToolProcessor: Plugin {
    fn tool_id(&self) -> &str;
    fn tool_version_requirement(&self) -> VersionRequirement;
    fn trusted(&self) -> bool;

    async fn check_available(&self) -> PluginResult<ToolAvailability>;
    async fn execute(
        &self,
        input_paths: &[std::path::PathBuf],
        output_path: &std::path::PathBuf,
        params: &ParameterSet,
    ) -> PluginResult<()>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub error: Option<String>,
}

// ---- Plugin Query ----
#[derive(Debug, Clone, Default)]
pub struct PluginQuery {
    pub category: Option<PluginCategory>,
    pub tags: Vec<String>,
    pub requires_pixel: Option<bool>,
    pub keyword: Option<String>,
    pub enabled_only: bool,
}

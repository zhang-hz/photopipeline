use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::image::PixelFormat;
use crate::color::ColorSpace;
use crate::error::PluginError;

pub type PluginId = String;
pub type NodeId = Uuid;
pub type ImageId = Uuid;
pub type BatchId = Uuid;
pub type PortId = Uuid;
pub type GroupId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl PluginVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch, pre: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionRequirement {
    pub min_version: PluginVersion,
    pub max_version: Option<PluginVersion>,
}

impl std::fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ">={}", self.min_version)?;
        if let Some(ref max) = self.max_version {
            write!(f, ",<{}", max)?;
        }
        Ok(())
    }
}

impl VersionRequirement {
    pub fn is_satisfied_by(&self, version: &PluginVersion) -> bool {
        version >= &self.min_version
            && self.max_version.as_ref().map_or(true, |max| version < max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
pub enum PluginCategory {
    Input,
    Metadata,
    Color,
    Transform,
    Enhance,
    Merge,
    Format,
    External,
    #[strum(default)]
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
pub enum GpuBackend {
    None,
    CUDA,
    Metal,
    Vulkan,
    DirectX,
    OpenCL,
    ROCm,
    OpenVINO,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
pub enum AiBackend {
    ONNX,
    TensorRT,
    CoreML,
    OpenVINO,
    Burn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuContext {
    pub backend: GpuBackend,
    pub device_name: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub compute_units: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_ram_mb: u64,
    pub gpus: Vec<GpuContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub elapsed_ms: u64,
    pub cpu_time_ms: u64,
    pub gpu_time_ms: Option<u64>,
    pub peak_memory_mb: u64,
    pub input_pixels: u64,
    pub output_pixels: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    HEIF,
    HEIC,
    AVIF,
    JXL,
    PNG,
    TIFF,
    JPEG,
    WEBP,
    OpenEXR,
    RAW,
    DNG,
    PPM,
    PGM,
    BMP,
    Unknown(String),
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ColorMode {
    RGB,
    RGBA,
    HSL,
    HSV,
    Lab,
}

impl Default for ColorMode {
    fn default() -> Self { Self::RGB }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FilePathKind {
    File,
    Directory,
    SaveFile,
}

impl Default for FilePathKind {
    fn default() -> Self { Self::File }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SliderOrientation {
    Horizontal,
    Vertical,
}

impl Default for SliderOrientation {
    fn default() -> Self { Self::Horizontal }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SliderStyle {
    Continuous,
    Discrete,
    Range,
    DualHandle,
}

impl Default for SliderStyle {
    fn default() -> Self { Self::Continuous }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatWidget {
    SpinBox,
    Slider,
    ComboSlider,
    DragInput,
}

impl Default for FloatWidget {
    fn default() -> Self { Self::SpinBox }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegerWidget {
    SpinBox,
    Slider,
    Combo,
}

impl Default for IntegerWidget {
    fn default() -> Self { Self::SpinBox }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnumDisplay {
    Dropdown,
    RadioGroup,
    ButtonGroup,
    SegmentedControl,
    Tabs,
    PopupCard,
}

impl Default for EnumDisplay {
    fn default() -> Self { Self::Dropdown }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LabelPosition {
    Top,
    Left,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RowHeight {
    Compact,
    Normal,
    Expanded,
    Custom(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SectionStyle {
    Default,
    Card,
    AccentCard,
    CollapsibleCard,
}

// Gui schema re-exports
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiSchema {
    pub layout: GuiLayout,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub preview: PreviewMode,
    pub aux_views: Vec<AuxView>,
    pub min_panel_width: u32,
}

impl Default for GuiSchema {
    fn default() -> Self {
        Self {
            layout: GuiLayout::default(),
            icon: None,
            color: None,
            preview: PreviewMode::None,
            aux_views: vec![],
            min_panel_width: 320,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuiLayout {
    Standard { sections: Vec<GuiSection> },
    Custom { rows: Vec<GuiRow> },
}

impl Default for GuiLayout {
    fn default() -> Self { Self::Standard { sections: vec![] } }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiSection {
    pub param_section_id: String,
    pub title_visible: bool,
    pub style: SectionStyle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiRow {
    pub height: RowHeight,
    pub cells: Vec<GuiCell>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiCell {
    pub param_field_id: String,
    pub width_fraction: f64,
    pub label_position: LabelPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMode {
    None,
    Live,
    ManualRefresh,
    BeforeAfter {
        default_split: f32,
        orientation: SplitOrientation,
        lock_zoom: bool,
    },
    Tiled { rows: u32, cols: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuxView {
    Histogram,
    Waveform,
    Vectorscope,
    GamutDiagram,
    Map,
    FocusPeaking,
    ClippingWarning,
    MetadataTable,
    ProgressBar,
    StatusText,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_version_comparison() {
        let v1 = PluginVersion::new(1, 0, 0);
        let v2 = PluginVersion::new(1, 1, 0);
        let v3 = PluginVersion::new(2, 0, 0);
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
        assert_eq!(v1, PluginVersion::new(1, 0, 0));
    }

    #[test]
    fn plugin_version_display() {
        let v = PluginVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn plugin_version_display_with_pre() {
        let v = PluginVersion { major: 1, minor: 2, patch: 3, pre: Some("alpha".into()) };
        assert_eq!(v.to_string(), "1.2.3-alpha");
    }

    #[test]
    fn version_requirement_satisfied_by_exact() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: None,
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(1, 0, 0)));
        assert!(req.is_satisfied_by(&PluginVersion::new(2, 0, 0)));
        assert!(!req.is_satisfied_by(&PluginVersion::new(0, 9, 0)));
    }

    #[test]
    fn version_requirement_satisfied_by_range() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(1, 0, 0)));
        assert!(req.is_satisfied_by(&PluginVersion::new(1, 5, 0)));
        assert!(!req.is_satisfied_by(&PluginVersion::new(2, 0, 0)));
        assert!(!req.is_satisfied_by(&PluginVersion::new(2, 1, 0)));
        assert!(!req.is_satisfied_by(&PluginVersion::new(0, 9, 0)));
    }

    #[test]
    fn version_requirement_display() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: None,
        };
        assert_eq!(req.to_string(), ">=1.0.0");

        let req2 = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        assert_eq!(req2.to_string(), ">=1.0.0,<2.0.0");
    }

    #[test]
    fn pixel_format_bytes_per_channel() {
        use crate::image::PixelFormat;
        assert_eq!(PixelFormat::U8.bytes_per_channel(), 1);
        assert_eq!(PixelFormat::U16.bytes_per_channel(), 2);
        assert_eq!(PixelFormat::F16.bytes_per_channel(), 2);
        assert_eq!(PixelFormat::U32.bytes_per_channel(), 4);
        assert_eq!(PixelFormat::F32.bytes_per_channel(), 4);
    }

    #[test]
    fn pixel_format_is_float() {
        use crate::image::PixelFormat;
        assert!(!PixelFormat::U8.is_float());
        assert!(!PixelFormat::U16.is_float());
        assert!(PixelFormat::F16.is_float());
        assert!(!PixelFormat::U32.is_float());
        assert!(PixelFormat::F32.is_float());
    }

    #[test]
    fn pixel_format_is_high_precision() {
        use crate::image::PixelFormat;
        assert!(!PixelFormat::U8.is_high_precision());
        assert!(PixelFormat::U16.is_high_precision());
        assert!(PixelFormat::F16.is_high_precision());
        assert!(PixelFormat::U32.is_high_precision());
        assert!(PixelFormat::F32.is_high_precision());
    }

    #[test]
    fn pixel_format_max_value_u16() {
        use crate::image::PixelFormat;
        assert_eq!(PixelFormat::U8.max_value_u16(), 255);
        assert_eq!(PixelFormat::U16.max_value_u16(), 65535);
        assert_eq!(PixelFormat::F16.max_value_u16(), 65535);
        assert_eq!(PixelFormat::U32.max_value_u16(), 65535);
        assert_eq!(PixelFormat::F32.max_value_u16(), 65535);
    }

    #[test]
    fn channel_layout_channel_count() {
        use crate::image::ChannelLayout;
        assert_eq!(ChannelLayout::Gray.channel_count(), 1);
        assert_eq!(ChannelLayout::GrayAlpha.channel_count(), 2);
        assert_eq!(ChannelLayout::RGB.channel_count(), 3);
        assert_eq!(ChannelLayout::RGBA.channel_count(), 4);
        assert_eq!(ChannelLayout::Planar(5).channel_count(), 5);
        assert_eq!(ChannelLayout::Custom(7).channel_count(), 7);
    }

    #[test]
    fn channel_layout_is_interleaved() {
        use crate::image::ChannelLayout;
        assert!(ChannelLayout::Gray.is_interleaved());
        assert!(ChannelLayout::GrayAlpha.is_interleaved());
        assert!(ChannelLayout::RGB.is_interleaved());
        assert!(ChannelLayout::RGBA.is_interleaved());
        assert!(!ChannelLayout::Planar(3).is_interleaved());
        assert!(!ChannelLayout::Custom(4).is_interleaved());
    }

    #[test]
    fn gpu_backend_display() {
        assert_eq!(GpuBackend::None.to_string(), "None");
        assert_eq!(GpuBackend::CUDA.to_string(), "CUDA");
        assert_eq!(GpuBackend::Vulkan.to_string(), "Vulkan");
        assert_eq!(GpuBackend::OpenCL.to_string(), "OpenCL");
        assert_eq!(GpuBackend::Auto.to_string(), "Auto");
    }

    #[test]
    fn ai_backend_display() {
        assert_eq!(AiBackend::ONNX.to_string(), "ONNX");
        assert_eq!(AiBackend::TensorRT.to_string(), "TensorRT");
        assert_eq!(AiBackend::CoreML.to_string(), "CoreML");
        assert_eq!(AiBackend::OpenVINO.to_string(), "OpenVINO");
        assert_eq!(AiBackend::Burn.to_string(), "Burn");
    }

    #[test]
    fn image_format_display() {
        assert_eq!(ImageFormat::JPEG.to_string(), "JPEG");
        assert_eq!(ImageFormat::PNG.to_string(), "PNG");
        assert_eq!(ImageFormat::HEIF.to_string(), "HEIF");
        assert_eq!(ImageFormat::AVIF.to_string(), "AVIF");
        assert_eq!(ImageFormat::JXL.to_string(), "JXL");
        assert_eq!(ImageFormat::Unknown("custom".into()).to_string(), "custom");
    }
}

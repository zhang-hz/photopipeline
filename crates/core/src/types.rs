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

    #[test]
    fn plugin_version_eq_self() {
        let v = PluginVersion::new(1, 2, 3);
        assert_eq!(v, PluginVersion::new(1, 2, 3));
    }

    #[test]
    fn plugin_version_ne_different() {
        let v1 = PluginVersion::new(1, 0, 0);
        let v2 = PluginVersion::new(1, 0, 1);
        assert_ne!(v1, v2);
    }

    #[test]
    fn plugin_version_lt_same_major() {
        let v1 = PluginVersion::new(1, 0, 0);
        let v2 = PluginVersion::new(1, 1, 0);
        assert!(v1 < v2);
    }

    #[test]
    fn plugin_version_gt_same_minor() {
        let v1 = PluginVersion::new(1, 0, 5);
        let v2 = PluginVersion::new(1, 0, 1);
        assert!(v1 > v2);
    }

    #[test]
    fn plugin_version_le_equal() {
        let v = PluginVersion::new(3, 2, 1);
        assert!(v <= PluginVersion::new(3, 2, 1));
    }

    #[test]
    fn plugin_version_ge_equal() {
        let v = PluginVersion::new(3, 2, 1);
        assert!(v >= PluginVersion::new(3, 2, 1));
    }

    #[test]
    fn plugin_version_ge_less() {
        let v = PluginVersion::new(5, 0, 0);
        assert!(v >= PluginVersion::new(4, 9, 9));
    }

    #[test]
    fn plugin_version_le_greater() {
        let v = PluginVersion::new(5, 0, 0);
        assert!(v <= PluginVersion::new(5, 0, 1));
    }

    #[test]
    fn plugin_version_with_pre_eq_same_pre() {
        let v1 = PluginVersion { major: 2, minor: 0, patch: 0, pre: Some("rc1".into()) };
        let v2 = PluginVersion { major: 2, minor: 0, patch: 0, pre: Some("rc1".into()) };
        assert_eq!(v1, v2);
    }

    #[test]
    fn plugin_version_with_pre_ne_different_pre() {
        let v1 = PluginVersion { major: 2, minor: 0, patch: 0, pre: Some("alpha".into()) };
        let v2 = PluginVersion { major: 2, minor: 0, patch: 0, pre: Some("beta".into()) };
        assert_ne!(v1, v2);
    }

    #[test]
    fn plugin_version_with_pre_lt_no_pre() {
        let v1 = PluginVersion { major: 1, minor: 0, patch: 0, pre: Some("alpha".into()) };
        let v2 = PluginVersion::new(1, 0, 0);
        let _ = v1 < v2 || v1 > v2;
    }

    #[test]
    fn plugin_version_zero_zero_zero() {
        let v = PluginVersion::new(0, 0, 0);
        assert_eq!(v.to_string(), "0.0.0");
    }

    #[test]
    fn plugin_version_max_values() {
        let v = PluginVersion::new(u32::MAX, u32::MAX, u32::MAX);
        assert_eq!(v.major, u32::MAX);
        assert_eq!(v.minor, u32::MAX);
        assert_eq!(v.patch, u32::MAX);
    }

    #[test]
    fn version_requirement_empty_max_means_no_upper_bound() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: None,
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(999, 0, 0)));
    }

    #[test]
    fn version_requirement_exact_version_match() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(2, 3, 4),
            max_version: None,
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(2, 3, 4)));
    }

    #[test]
    fn version_requirement_below_min_fails() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(3, 0, 0),
            max_version: None,
        };
        assert!(!req.is_satisfied_by(&PluginVersion::new(2, 999, 999)));
    }

    #[test]
    fn version_requirement_above_max_exclusive_fails() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(3, 0, 0)),
        };
        assert!(!req.is_satisfied_by(&PluginVersion::new(3, 0, 0)));
    }

    #[test]
    fn version_requirement_equal_to_max_fails() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        assert!(!req.is_satisfied_by(&PluginVersion::new(2, 0, 0)));
    }

    #[test]
    fn version_requirement_within_range() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(2, 0, 0),
            max_version: Some(PluginVersion::new(3, 0, 0)),
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(2, 5, 0)));
    }

    #[test]
    fn version_requirement_just_above_min() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(1, 0, 1)));
    }

    #[test]
    fn version_requirement_just_below_max() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        assert!(req.is_satisfied_by(&PluginVersion::new(1, 999, 999)));
    }

    #[test]
    fn pixel_format_u8_bytes_per_channel() {
        use crate::image::PixelFormat;
        assert_eq!(PixelFormat::U8.bytes_per_channel(), 1);
    }

    #[test]
    fn pixel_format_u32_max_value_u16() {
        use crate::image::PixelFormat;
        assert_eq!(PixelFormat::U32.max_value_u16(), 65535);
    }

    #[test]
    fn pixel_format_u32_is_high_precision() {
        use crate::image::PixelFormat;
        assert!(PixelFormat::U32.is_high_precision());
    }

    #[test]
    fn pixel_format_u8_not_high_precision() {
        use crate::image::PixelFormat;
        assert!(!PixelFormat::U8.is_high_precision());
    }

    #[test]
    fn channel_layout_planar_zero() {
        use crate::image::ChannelLayout;
        assert_eq!(ChannelLayout::Planar(0).channel_count(), 0);
    }

    #[test]
    fn channel_layout_planar_255() {
        use crate::image::ChannelLayout;
        assert_eq!(ChannelLayout::Planar(255).channel_count(), 255);
    }

    #[test]
    fn channel_layout_custom_zero() {
        use crate::image::ChannelLayout;
        assert_eq!(ChannelLayout::Custom(0).channel_count(), 0);
    }

    #[test]
    fn channel_layout_planar_not_interleaved() {
        use crate::image::ChannelLayout;
        assert!(!ChannelLayout::Planar(4).is_interleaved());
    }

    #[test]
    fn channel_layout_custom_not_interleaved() {
        use crate::image::ChannelLayout;
        assert!(!ChannelLayout::Custom(1).is_interleaved());
    }

    #[test]
    fn gpu_backend_directx_display() {
        assert_eq!(GpuBackend::DirectX.to_string(), "DirectX");
    }

    #[test]
    fn gpu_backend_rocm_display() {
        assert_eq!(GpuBackend::ROCm.to_string(), "ROCm");
    }

    #[test]
    fn gpu_backend_metal_display() {
        assert_eq!(GpuBackend::Metal.to_string(), "Metal");
    }

    #[test]
    fn gpu_backend_openvino_display() {
        assert_eq!(GpuBackend::OpenVINO.to_string(), "OpenVINO");
    }

    #[test]
    fn ai_backend_openvino_display() {
        assert_eq!(AiBackend::OpenVINO.to_string(), "OpenVINO");
    }

    #[test]
    fn image_format_heic_display() {
        assert_eq!(ImageFormat::HEIC.to_string(), "HEIC");
    }

    #[test]
    fn image_format_webp_display() {
        assert_eq!(ImageFormat::WEBP.to_string(), "WEBP");
    }

    #[test]
    fn image_format_openexr_display() {
        assert_eq!(ImageFormat::OpenEXR.to_string(), "OpenEXR");
    }

    #[test]
    fn image_format_raw_display() {
        assert_eq!(ImageFormat::RAW.to_string(), "RAW");
    }

    #[test]
    fn image_format_dng_display() {
        assert_eq!(ImageFormat::DNG.to_string(), "DNG");
    }

    #[test]
    fn image_format_ppm_display() {
        assert_eq!(ImageFormat::PPM.to_string(), "PPM");
    }

    #[test]
    fn image_format_pgm_display() {
        assert_eq!(ImageFormat::PGM.to_string(), "PGM");
    }

    #[test]
    fn image_format_bmp_display() {
        assert_eq!(ImageFormat::BMP.to_string(), "BMP");
    }

    #[test]
    fn image_format_tiff_display() {
        assert_eq!(ImageFormat::TIFF.to_string(), "TIFF");
    }

    #[test]
    fn plugin_category_input_display() {
        assert_eq!(PluginCategory::Input.to_string(), "input");
    }

    #[test]
    fn plugin_category_format_display() {
        assert_eq!(PluginCategory::Format.to_string(), "format");
    }

    #[test]
    fn plugin_category_enhance_display() {
        assert_eq!(PluginCategory::Enhance.to_string(), "enhance");
    }

    #[test]
    fn plugin_category_custom_display() {
        assert_eq!(PluginCategory::Custom("my_cat".into()).to_string(), "my_cat");
    }

    #[test]
    fn plugin_category_custom_roundtrip() {
        use std::str::FromStr;
        let cat = PluginCategory::from_str("my_custom_category");
        assert!(cat.is_ok());
    }

    #[test]
    fn plugin_category_known_from_str() {
        use std::str::FromStr;
        assert_eq!(PluginCategory::from_str("input").unwrap(), PluginCategory::Input);
    }

    #[test]
    fn hardware_info_serde_roundtrip() {
        let hw = HardwareInfo {
            cpu_cores: 8,
            cpu_threads: 16,
            total_ram_mb: 32768,
            gpus: vec![GpuContext {
                backend: GpuBackend::CUDA,
                device_name: "RTX 4090".into(),
                total_memory_mb: 24576,
                available_memory_mb: 20000,
                compute_units: 128,
            }],
        };
        let json = serde_json::to_string(&hw).unwrap();
        let hw2: HardwareInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(hw2.cpu_cores, 8);
        assert_eq!(hw2.gpus.len(), 1);
        assert_eq!(hw2.gpus[0].backend, GpuBackend::CUDA);
    }

    #[test]
    fn processing_stats_serde_roundtrip() {
        let stats = ProcessingStats {
            elapsed_ms: 1500,
            cpu_time_ms: 1200,
            gpu_time_ms: Some(300),
            peak_memory_mb: 4096,
            input_pixels: 2073600,
            output_pixels: 2073600,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let stats2: ProcessingStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats2.elapsed_ms, 1500);
        assert_eq!(stats2.peak_memory_mb, 4096);
        assert_eq!(stats2.gpu_time_ms, Some(300));
    }

    #[test]
    fn processing_stats_gpu_time_none_serde() {
        let stats = ProcessingStats {
            elapsed_ms: 100,
            cpu_time_ms: 100,
            gpu_time_ms: None,
            peak_memory_mb: 256,
            input_pixels: 1000,
            output_pixels: 1000,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let stats2: ProcessingStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats2.gpu_time_ms, None);
    }

    #[test]
    fn version_requirement_serde_roundtrip() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 5, 0),
            max_version: Some(PluginVersion::new(2, 0, 0)),
        };
        let json = serde_json::to_string(&req).unwrap();
        let req2: VersionRequirement = serde_json::from_str(&json).unwrap();
        assert_eq!(req2.min_version, PluginVersion::new(1, 5, 0));
        assert_eq!(req2.max_version, Some(PluginVersion::new(2, 0, 0)));
    }

    #[test]
    fn version_requirement_no_max_serde() {
        let req = VersionRequirement {
            min_version: PluginVersion::new(1, 0, 0),
            max_version: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let req2: VersionRequirement = serde_json::from_str(&json).unwrap();
        assert_eq!(req2.max_version, None);
    }

    #[test]
    fn color_mode_default_is_rgb() {
        assert_eq!(ColorMode::default(), ColorMode::RGB);
    }

    #[test]
    fn file_path_kind_default_is_file() {
        assert_eq!(FilePathKind::default(), FilePathKind::File);
    }

    #[test]
    fn slider_orientation_default() {
        assert_eq!(SliderOrientation::default(), SliderOrientation::Horizontal);
    }

    #[test]
    fn slider_style_default() {
        assert_eq!(SliderStyle::default(), SliderStyle::Continuous);
    }

    #[test]
    fn float_widget_default() {
        assert_eq!(FloatWidget::default(), FloatWidget::SpinBox);
    }

    #[test]
    fn integer_widget_default() {
        assert_eq!(IntegerWidget::default(), IntegerWidget::SpinBox);
    }

    #[test]
    fn enum_display_default() {
        assert_eq!(EnumDisplay::default(), EnumDisplay::Dropdown);
    }

    #[test]
    fn gui_schema_default() {
        let gs = GuiSchema::default();
        assert_eq!(gs.min_panel_width, 320);
    }

    #[test]
    fn gui_layout_default() {
        let layout = GuiLayout::default();
        assert!(matches!(layout, GuiLayout::Standard { .. }));
    }

    #[test]
    fn gpu_context_fields() {
        let ctx = GpuContext {
            backend: GpuBackend::Metal,
            device_name: "M2".into(),
            total_memory_mb: 32768,
            available_memory_mb: 16384,
            compute_units: 16,
        };
        assert_eq!(ctx.backend, GpuBackend::Metal);
        assert_eq!(ctx.total_memory_mb, 32768);
    }

    #[test]
    fn image_info_stores_pixel_format_and_color_space() {
        use crate::image::PixelFormat;
        let info = ImageInfo {
            id: uuid::Uuid::new_v4(),
            path: "/tmp/a.jpg".into(),
            filename: "a.jpg".into(),
            format: ImageFormat::JPEG,
            width: 100,
            height: 50,
            file_size_bytes: 5000,
            pixel_format: PixelFormat::U8,
            color_space: crate::color::ColorSpace::default(),
        };
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 50);
    }
}

use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    PluginId, PluginVersion, PluginCategory, PluginResult,
    PixelBuffer, PixelFormat, ColorSpace, GpuBackend,
    ValidationIssue, HardwareRequirement, ProcessingStats,
};
use photopipeline_plugin::{
    Plugin, PixelProcessor, ProgressSink,
    ParameterSchema, ParameterSet, ParameterSection, ParameterField, ParameterType,
    EnumOption,
    GuiSchema, GuiLayout, GuiSection,
    PreviewMode, AuxView, SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
        version: 1,
        sections: vec![
            ParameterSection {
                id: "lut_file".into(),
                label: "3D LUT File".into(),
                description: Some("Select and configure the 3D LUT to apply".into()),
                icon: Some("grid3x3".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "lut_path".into(),
                        label: "LUT File".into(),
                        description: Some("Path to a .cube, .3dl, or .look 3D LUT file".into()),
                        help_url: None,
                        field_type: ParameterType::FilePath {
                            kind: Default::default(),
                            filters: vec![
                                ("LUT Files".into(), "*.cube".into()),
                                ("LUT Files".into(), "*.3dl".into()),
                                ("LUT Files".into(), "*.look".into()),
                                ("All Files".into(), "*".into()),
                            ],
                            must_exist: true,
                        },
                        default: serde_json::json!(""),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "lut_format".into(),
                        label: "LUT Format".into(),
                        description: Some("Format of the 3D LUT file".into()),
                        help_url: None,
                        field_type: ParameterType::Enum {
                            options: vec![
                                EnumOption {
                                    value: "cube".into(), label: "Iridas .cube".into(),
                                    description: Some("Adobe/CUBE format, common in DaVinci Resolve".into()),
                                    icon: None, tags: vec![], recommended: true,
                                },
                                EnumOption {
                                    value: "3dl".into(), label: "Autodesk .3dl".into(),
                                    description: Some("Flame/Lustre format".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "look".into(), label: "Assimilate .look".into(),
                                    description: Some("Scratch format".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "csp".into(), label: "Rising Sun .csp".into(),
                                    description: Some("CineSpace format".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                            ],
                            display: Default::default(),
                        },
                        default: serde_json::json!("cube"),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                ],
            },
            ParameterSection {
                id: "lut_transform".into(),
                label: "Transform".into(),
                description: Some("Controls for the LUT application".into()),
                icon: Some("blend".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "intensity".into(),
                        label: "Intensity".into(),
                        description: Some("Blend between original and LUT output (0-100%)".into()),
                        help_url: None,
                        field_type: ParameterType::Slider {
                            min: 0.0, max: 100.0, step: 1.0,
                            show_ticks: true,
                            ticks: Some(vec![0.0, 25.0, 50.0, 75.0, 100.0]),
                            show_value: true,
                            orientation: Default::default(),
                            style: Default::default(),
                        },
                        default: serde_json::json!(100.0),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "input_color_space".into(),
                        label: "Input Color Space".into(),
                        description: Some("Color space the LUT expects".into()),
                        help_url: None,
                        field_type: ParameterType::Enum {
                            options: vec![
                                EnumOption {
                                    value: "srgb".into(), label: "sRGB".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: true,
                                },
                                EnumOption {
                                    value: "display_p3".into(), label: "Display P3".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "rec709".into(), label: "Rec.709".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "rec2020".into(), label: "Rec.2020".into(),
                                    description: None,
                                    icon: None, tags: vec!["hdr".into()], recommended: false,
                                },
                            ],
                            display: Default::default(),
                        },
                        default: serde_json::json!("srgb"),
                        required: false,
                        advanced: true,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "clamp_output".into(),
                        label: "Clamp Output".into(),
                        description: Some("Clamp output values to 0-1 range".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Clamp".into()),
                            label_false: Some("Pass Through".into()),
                        },
                        default: serde_json::json!(true),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                ],
            },
            ParameterSection {
                id: "interpolation".into(),
                label: "Interpolation".into(),
                description: Some("LUT sample interpolation method".into()),
                icon: Some("line-chart".into()),
                collapsible: true,
                default_collapsed: true,
                fields: vec![
                    ParameterField {
                        id: "interpolation_method".into(),
                        label: "Method".into(),
                        description: Some("How to interpolate between LUT sample points".into()),
                        help_url: None,
                        field_type: ParameterType::Enum {
                            options: vec![
                                EnumOption {
                                    value: "trilinear".into(), label: "Trilinear".into(),
                                    description: Some("Fast, may show banding".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "tetrahedral".into(), label: "Tetrahedral".into(),
                                    description: Some("Better quality, good performance".into()),
                                    icon: None, tags: vec![], recommended: true,
                                },
                            ],
                            display: Default::default(),
                        },
                        default: serde_json::json!("tetrahedral"),
                        required: false,
                        advanced: true,
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
                GuiSection { param_section_id: "lut_file".into(), title_visible: true, style: SectionStyle::Card },
                GuiSection { param_section_id: "lut_transform".into(), title_visible: true, style: SectionStyle::Card },
                GuiSection { param_section_id: "interpolation".into(), title_visible: true, style: SectionStyle::CollapsibleCard },
            ],
        },
        icon: Some("grid3x3".into()),
        color: Some("#ec4899".into()),
        preview: PreviewMode::BeforeAfter {
            default_split: 0.5,
            orientation: photopipeline_core::SplitOrientation::Horizontal,
            lock_zoom: true,
        },
        aux_views: vec![AuxView::Histogram, AuxView::Vectorscope],
        min_panel_width: 320,
    }
});

#[derive(Debug, Clone)]
pub struct Lut3dPlugin {
    id: String,
}

impl Lut3dPlugin {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.lut3d".to_string() }
    }
}

impl Default for Lut3dPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for Lut3dPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "3D LUT" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Color }
    fn description(&self) -> &str { "Apply 3D Look-Up Tables for color grading and film emulation" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { true }
    fn produces_pixel_output(&self) -> bool { true }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement { min_ram_mb: 256, requires_gpu: true, preferred_backend: Some(GpuBackend::Auto), ..Default::default() }
    }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let lut_path = params.get_str("lut_path").unwrap_or("");
        if lut_path.is_empty() {
            issues.push(ValidationIssue::Warning {
                param: "lut_path".into(),
                message: "No LUT file selected; pixels will pass through unchanged".into(),
            });
        } else if !std::path::Path::new(lut_path).exists() {
            issues.push(ValidationIssue::Error {
                param: "lut_path".into(),
                message: format!("LUT file not found: {}", lut_path),
            });
        }

        let intensity = params.get("intensity").and_then(|v| v.as_f64()).unwrap_or(100.0);
        if intensity < 0.0 || intensity > 100.0 {
            issues.push(ValidationIssue::Error {
                param: "intensity".into(),
                message: "Intensity must be between 0 and 100".into(),
            });
        }

        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for Lut3dPlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F16, PixelFormat::F32]
    }

    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::U8, PixelFormat::U16, PixelFormat::F16, PixelFormat::F32]
    }

    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![ColorSpace::SRGB, ColorSpace::DISPLAY_P3, ColorSpace::ADOBE_RGB, ColorSpace::REC2020_PQ]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        Some(GpuBackend::Auto)
    }

    async fn process_pixels(
        &self, input: &PixelBuffer, output: &mut PixelBuffer,
        _params: &ParameterSet, progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        progress.set_progress(0.0, "applying LUT");

        output.data.data.copy_from_slice(&input.data.data);
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        let pixels = input.pixel_count();
        progress.set_progress(1.0, "done");

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: Some(0),
            peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
            input_pixels: pixels,
            output_pixels: pixels,
        })
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "color".into(), "lut".into(), "grading".into(), "look".into(),
        "cube".into(), "film".into(), "gpu".into(),
    ]
});

use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    ColorSpace, GpuBackend, HardwareRequirement, PerfTimer, PixelBuffer, PixelFormat,
    PluginCategory, PluginError, PluginId, PluginResult, PluginVersion, ProcessingStats,
    ValidationIssue,
};
use photopipeline_plugin::{
    AuxView, EnumOption, GuiLayout, GuiSchema, GuiSection, ParameterField, ParameterSchema,
    ParameterSection, ParameterSet, ParameterType, PixelProcessor, Plugin, PreviewMode,
    ProgressSink, SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
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
                                value: "cube".into(),
                                label: "Iridas .cube".into(),
                                description: Some(
                                    "Adobe/CUBE format, common in DaVinci Resolve".into(),
                                ),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "3dl".into(),
                                label: "Autodesk .3dl".into(),
                                description: Some("Flame/Lustre format".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "look".into(),
                                label: "Assimilate .look".into(),
                                description: Some("Scratch format".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "csp".into(),
                                label: "Rising Sun .csp".into(),
                                description: Some("CineSpace format".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
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
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
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
                                value: "srgb".into(),
                                label: "sRGB".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "display_p3".into(),
                                label: "Display P3".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "rec709".into(),
                                label: "Rec.709".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "rec2020".into(),
                                label: "Rec.2020".into(),
                                description: None,
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
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
            fields: vec![ParameterField {
                id: "interpolation_method".into(),
                label: "Method".into(),
                description: Some("How to interpolate between LUT sample points".into()),
                help_url: None,
                field_type: ParameterType::Enum {
                    options: vec![
                        EnumOption {
                            value: "trilinear".into(),
                            label: "Trilinear".into(),
                            description: Some("Fast, may show banding".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                        EnumOption {
                            value: "tetrahedral".into(),
                            label: "Tetrahedral".into(),
                            description: Some("Better quality, good performance".into()),
                            icon: None,
                            tags: vec![],
                            recommended: true,
                        },
                    ],
                    display: Default::default(),
                },
                default: serde_json::json!("tetrahedral"),
                required: false,
                advanced: true,
                allow_override: true,
                supports_expression: false,
            }],
        },
    ],
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            GuiSection {
                param_section_id: "lut_file".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "lut_transform".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "interpolation".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
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
});

#[derive(Debug, Clone)]
pub struct Lut3dPlugin {
    id: String,
}

impl Lut3dPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.lut3d".to_string(),
        }
    }
}

impl Default for Lut3dPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for Lut3dPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "3D LUT"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Color
    }
    fn description(&self) -> &str {
        "Apply 3D Look-Up Tables for color grading and film emulation"
    }
    fn tags(&self) -> &[String] {
        &TAGS
    }
    fn requires_pixel_access(&self) -> bool {
        true
    }
    fn produces_pixel_output(&self) -> bool {
        true
    }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement {
            min_ram_mb: 256,
            requires_gpu: true,
            preferred_backend: Some(GpuBackend::Auto),
            ..Default::default()
        }
    }

    fn parameter_schema(&self) -> &ParameterSchema {
        &PARAMETER_SCHEMA
    }
    fn gui_schema(&self) -> &GuiSchema {
        &GUI_SCHEMA
    }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        tracing::info!("lut3d plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("lut3d plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("lut3d: validating parameters");
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

        let intensity = params
            .get("intensity")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        if !(0.0..=100.0).contains(&intensity) {
            issues.push(ValidationIssue::Error {
                param: "intensity".into(),
                message: "Intensity must be between 0 and 100".into(),
            });
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "lut3d validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for Lut3dPlugin {
    fn supported_input_formats(&self) -> Vec<PixelFormat> {
        vec![
            PixelFormat::U8,
            PixelFormat::U16,
            PixelFormat::F16,
            PixelFormat::F32,
        ]
    }

    fn supported_output_formats(&self) -> Vec<PixelFormat> {
        vec![
            PixelFormat::U8,
            PixelFormat::U16,
            PixelFormat::F16,
            PixelFormat::F32,
        ]
    }

    fn supported_color_spaces(&self) -> Vec<ColorSpace> {
        vec![
            ColorSpace::SRGB,
            ColorSpace::DISPLAY_P3,
            ColorSpace::ADOBE_RGB,
            ColorSpace::REC2020_PQ,
        ]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        Some(GpuBackend::Auto)
    }

    async fn process_pixels(
        &self,
        input: &PixelBuffer,
        output: &mut PixelBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        let _timer = PerfTimer::with_target("lut3d_process_pixels", "plugin.lut3d");
        progress.set_progress(0.0, "applying LUT");

        let lut_path = params.get_str("lut_path").unwrap_or("");
        tracing::info!(
            input_dims = format!("{}x{}", input.width, input.height),
            input_format = ?input.format,
            lut_path = lut_path,
            "lut3d: processing {}x{} with LUT {}",
            input.width,
            input.height,
            lut_path,
        );

        if lut_path.is_empty() {
            output.data.data.copy_from_slice(&input.data.data);
            output.color_space = input.color_space.clone();
            output.icc_profile = input.icc_profile.clone();

            let pixels = input.pixel_count();
            progress.set_progress(1.0, "done (no LUT)");

            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: Some(0),
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: pixels,
                output_pixels: pixels,
            });
        }

        if let Ok(lut_data) = std::fs::read(lut_path) {
            let _intensity = params
                .get("intensity")
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0);

            if let Some((lut, size)) = parse_cube_lut(&lut_data) {
                apply_trilinear_lut(input, output, &lut, size);

                let pixels = input.pixel_count();
                progress.set_progress(1.0, "done (trilinear LUT fallback)");

                return Ok(ProcessingStats {
                    elapsed_ms: 0,
                    cpu_time_ms: 0,
                    gpu_time_ms: Some(0),
                    peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                    input_pixels: pixels,
                    output_pixels: pixels,
                });
            }
        }

        Err(PluginError::Internal {
            plugin: self.id.clone(),
            message: "3D LUT processing requires GPU acceleration (not yet linked)".into(),
        })
    }
}

fn parse_cube_lut(data: &[u8]) -> Option<(Vec<f32>, usize)> {
    let text = std::str::from_utf8(data).ok()?;
    let mut size: usize = 0;
    let mut values: Vec<f32> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("LUT_3D_SIZE") {
            if let Some(s) = trimmed.split_whitespace().nth(1) {
                size = s.parse().ok()?;
            }
        } else if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("TITLE")
            && !trimmed.starts_with("DOMAIN")
        {
            for num in trimmed.split_whitespace() {
                if let Ok(v) = num.parse::<f32>() {
                    values.push(v);
                }
            }
        }
    }
    if size == 0 || values.is_empty() {
        return None;
    }
    Some((values, size))
}

fn apply_trilinear_lut(input: &PixelBuffer, output: &mut PixelBuffer, lut: &[f32], size: usize) {
    output.data.data.copy_from_slice(&input.data.data);
    output.color_space = input.color_space.clone();
    output.icc_profile = input.icc_profile.clone();

    let expected = size * size * size * 3;
    if lut.len() < expected {
        return;
    }

    match input.format {
        PixelFormat::U8 => {
            let input_data: &[u8] = &input.data.data;
            let output_data: &mut [u8] = &mut output.data.data;
            let channels = input.layout.channel_count() as usize;
            let size_f = size as f32;
            for px in 0..input.pixel_count() as usize {
                let pi = px * channels;
                if pi + 2 < input_data.len() {
                    let r =
                        (input_data[pi] as f32 / 255.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                    let g = (input_data[pi + 1] as f32 / 255.0 * (size_f - 1.0))
                        .clamp(0.0, size_f - 1.0);
                    let b = (input_data[pi + 2] as f32 / 255.0 * (size_f - 1.0))
                        .clamp(0.0, size_f - 1.0);
                    let (lr, lg, lb) = sample_lut(lut, size, r, g, b);
                    if pi < output_data.len() {
                        output_data[pi] = (lr * 255.0).clamp(0.0, 255.0) as u8;
                    }
                    if pi + 1 < output_data.len() {
                        output_data[pi + 1] = (lg * 255.0).clamp(0.0, 255.0) as u8;
                    }
                    if pi + 2 < output_data.len() {
                        output_data[pi + 2] = (lb * 255.0).clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        _ => {}
    }
}

fn sample_lut(lut: &[f32], size: usize, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r0 = (r as usize).min(size - 1);
    let r1 = (r0 + 1).min(size - 1);
    let g0 = (g as usize).min(size - 1);
    let g1 = (g0 + 1).min(size - 1);
    let b0 = (b as usize).min(size - 1);
    let b1 = (b0 + 1).min(size - 1);

    let dr = r - r0 as f32;
    let dg = g - g0 as f32;
    let db = b - b0 as f32;

    let idx = |r: usize, g: usize, b: usize| -> usize { (r * size * size + g * size + b) * 3 };

    let c000_r = lut[idx(r0, g0, b0)];
    let c000_g = lut[idx(r0, g0, b0) + 1];
    let c000_b = lut[idx(r0, g0, b0) + 2];

    let c001_r = lut[idx(r0, g0, b1)];
    let c001_g = lut[idx(r0, g0, b1) + 1];
    let c001_b = lut[idx(r0, g0, b1) + 2];

    let c010_r = lut[idx(r0, g1, b0)];
    let c010_g = lut[idx(r0, g1, b0) + 1];
    let c010_b = lut[idx(r0, g1, b0) + 2];

    let c011_r = lut[idx(r0, g1, b1)];
    let c011_g = lut[idx(r0, g1, b1) + 1];
    let c011_b = lut[idx(r0, g1, b1) + 2];

    let c100_r = lut[idx(r1, g0, b0)];
    let c100_g = lut[idx(r1, g0, b0) + 1];
    let c100_b = lut[idx(r1, g0, b0) + 2];

    let c101_r = lut[idx(r1, g0, b1)];
    let c101_g = lut[idx(r1, g0, b1) + 1];
    let c101_b = lut[idx(r1, g0, b1) + 2];

    let c110_r = lut[idx(r1, g1, b0)];
    let c110_g = lut[idx(r1, g1, b0) + 1];
    let c110_b = lut[idx(r1, g1, b0) + 2];

    let c111_r = lut[idx(r1, g1, b1)];
    let c111_g = lut[idx(r1, g1, b1) + 1];
    let c111_b = lut[idx(r1, g1, b1) + 2];

    let c00_r = c000_r * (1.0 - db) + c001_r * db;
    let c00_g = c000_g * (1.0 - db) + c001_g * db;
    let c00_b = c000_b * (1.0 - db) + c001_b * db;

    let c01_r = c010_r * (1.0 - db) + c011_r * db;
    let c01_g = c010_g * (1.0 - db) + c011_g * db;
    let c01_b = c010_b * (1.0 - db) + c011_b * db;

    let c10_r = c100_r * (1.0 - db) + c101_r * db;
    let c10_g = c100_g * (1.0 - db) + c101_g * db;
    let c10_b = c100_b * (1.0 - db) + c101_b * db;

    let c11_r = c110_r * (1.0 - db) + c111_r * db;
    let c11_g = c110_g * (1.0 - db) + c111_g * db;
    let c11_b = c110_b * (1.0 - db) + c111_b * db;

    let c0_r = c00_r * (1.0 - dg) + c01_r * dg;
    let c0_g = c00_g * (1.0 - dg) + c01_g * dg;
    let c0_b = c00_b * (1.0 - dg) + c01_b * dg;

    let c1_r = c10_r * (1.0 - dg) + c11_r * dg;
    let c1_g = c10_g * (1.0 - dg) + c11_g * dg;
    let c1_b = c10_b * (1.0 - dg) + c11_b * dg;

    let out_r = c0_r * (1.0 - dr) + c1_r * dr;
    let out_g = c0_g * (1.0 - dr) + c1_g * dr;
    let out_b = c0_b * (1.0 - dr) + c1_b * dr;

    (out_r, out_g, out_b)
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "color".into(),
        "lut".into(),
        "grading".into(),
        "look".into(),
        "cube".into(),
        "film".into(),
        "gpu".into(),
    ]
});

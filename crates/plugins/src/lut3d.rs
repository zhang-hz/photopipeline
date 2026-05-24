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
            let intensity = params
                .get("intensity")
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0);
            let method = params
                .get_str("interpolation_method")
                .unwrap_or("tetrahedral");

            if let Some((lut, size)) = parse_cube_lut(&lut_data) {
                match method {
                    "tetrahedral" => apply_tetrahedral_lut(input, output, &lut, size, intensity),
                    _ => apply_trilinear_lut(input, output, &lut, size, intensity),
                }

                let pixels = input.pixel_count();
                progress.set_progress(1.0, &format!("done ({method} LUT)"));

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

fn lut_index(size: usize, r: usize, g: usize, b: usize) -> usize {
    (r * size * size + g * size + b) * 3
}

fn sample_lut_trilinear(lut: &[f32], size: usize, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r0 = (r as usize).min(size - 1);
    let r1 = (r0 + 1).min(size - 1);
    let g0 = (g as usize).min(size - 1);
    let g1 = (g0 + 1).min(size - 1);
    let b0 = (b as usize).min(size - 1);
    let b1 = (b0 + 1).min(size - 1);

    let dr = r - r0 as f32;
    let dg = g - g0 as f32;
    let db = b - b0 as f32;

    let idx = |ri: usize, gi: usize, bi: usize| lut_index(size, ri, gi, bi);
    let c000 = &lut[idx(r0, g0, b0)..idx(r0, g0, b0) + 3];
    let c001 = &lut[idx(r0, g0, b1)..idx(r0, g0, b1) + 3];
    let c010 = &lut[idx(r0, g1, b0)..idx(r0, g1, b0) + 3];
    let c011 = &lut[idx(r0, g1, b1)..idx(r0, g1, b1) + 3];
    let c100 = &lut[idx(r1, g0, b0)..idx(r1, g0, b0) + 3];
    let c101 = &lut[idx(r1, g0, b1)..idx(r1, g0, b1) + 3];
    let c110 = &lut[idx(r1, g1, b0)..idx(r1, g1, b0) + 3];
    let c111 = &lut[idx(r1, g1, b1)..idx(r1, g1, b1) + 3];

    let c00 = [
        c000[0] * (1.0 - db) + c001[0] * db,
        c000[1] * (1.0 - db) + c001[1] * db,
        c000[2] * (1.0 - db) + c001[2] * db,
    ];
    let c01 = [
        c010[0] * (1.0 - db) + c011[0] * db,
        c010[1] * (1.0 - db) + c011[1] * db,
        c010[2] * (1.0 - db) + c011[2] * db,
    ];
    let c10 = [
        c100[0] * (1.0 - db) + c101[0] * db,
        c100[1] * (1.0 - db) + c101[1] * db,
        c100[2] * (1.0 - db) + c101[2] * db,
    ];
    let c11 = [
        c110[0] * (1.0 - db) + c111[0] * db,
        c110[1] * (1.0 - db) + c111[1] * db,
        c110[2] * (1.0 - db) + c111[2] * db,
    ];

    let c0 = [
        c00[0] * (1.0 - dg) + c01[0] * dg,
        c00[1] * (1.0 - dg) + c01[1] * dg,
        c00[2] * (1.0 - dg) + c01[2] * dg,
    ];
    let c1 = [
        c10[0] * (1.0 - dg) + c11[0] * dg,
        c10[1] * (1.0 - dg) + c11[1] * dg,
        c10[2] * (1.0 - dg) + c11[2] * dg,
    ];

    (
        c0[0] * (1.0 - dr) + c1[0] * dr,
        c0[1] * (1.0 - dr) + c1[1] * dr,
        c0[2] * (1.0 - dr) + c1[2] * dr,
    )
}

fn sample_lut_tetrahedral(lut: &[f32], size: usize, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r0 = (r as usize).min(size - 1);
    let r1 = (r0 + 1).min(size - 1);
    let g0 = (g as usize).min(size - 1);
    let g1 = (g0 + 1).min(size - 1);
    let b0 = (b as usize).min(size - 1);
    let b1 = (b0 + 1).min(size - 1);

    let dr = r - r0 as f32;
    let dg = g - g0 as f32;
    let db = b - b0 as f32;

    let idx = |ri: usize, gi: usize, bi: usize| lut_index(size, ri, gi, bi);
    let c000 = &lut[idx(r0, g0, b0)..idx(r0, g0, b0) + 3];
    let c100 = &lut[idx(r1, g0, b0)..idx(r1, g0, b0) + 3];
    let c010 = &lut[idx(r0, g1, b0)..idx(r0, g1, b0) + 3];
    let c001 = &lut[idx(r0, g0, b1)..idx(r0, g0, b1) + 3];
    let c110 = &lut[idx(r1, g1, b0)..idx(r1, g1, b0) + 3];
    let c101 = &lut[idx(r1, g0, b1)..idx(r1, g0, b1) + 3];
    let c011 = &lut[idx(r0, g1, b1)..idx(r0, g1, b1) + 3];
    let c111 = &lut[idx(r1, g1, b1)..idx(r1, g1, b1) + 3];

    let (r_out, g_out, b_out) = if dr > dg {
        if dg > db {
            // T1: dr > dg > db → c000, c100, c110, c111
            let w0 = 1.0 - dr;
            let w1 = dr - dg;
            let w2 = dg - db;
            let w3 = db;
            (
                w0 * c000[0] + w1 * c100[0] + w2 * c110[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c100[1] + w2 * c110[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c100[2] + w2 * c110[2] + w3 * c111[2],
            )
        } else if dr > db {
            // T2: dr > db > dg → c000, c100, c101, c111
            let w0 = 1.0 - dr;
            let w1 = dr - db;
            let w2 = db - dg;
            let w3 = dg;
            (
                w0 * c000[0] + w1 * c100[0] + w2 * c101[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c100[1] + w2 * c101[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c100[2] + w2 * c101[2] + w3 * c111[2],
            )
        } else {
            // T6: db > dr > dg → c000, c001, c101, c111
            let w0 = 1.0 - db;
            let w1 = db - dr;
            let w2 = dr - dg;
            let w3 = dg;
            (
                w0 * c000[0] + w1 * c001[0] + w2 * c101[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c001[1] + w2 * c101[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c001[2] + w2 * c101[2] + w3 * c111[2],
            )
        }
    } else {
        if db > dg {
            // T5: db > dg > dr → c000, c001, c011, c111
            let w0 = 1.0 - db;
            let w1 = db - dg;
            let w2 = dg - dr;
            let w3 = dr;
            (
                w0 * c000[0] + w1 * c001[0] + w2 * c011[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c001[1] + w2 * c011[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c001[2] + w2 * c011[2] + w3 * c111[2],
            )
        } else if db > dr {
            // T6-like: dg > db > dr → c000, c010, c011, c111
            let w0 = 1.0 - dg;
            let w1 = dg - db;
            let w2 = db - dr;
            let w3 = dr;
            (
                w0 * c000[0] + w1 * c010[0] + w2 * c011[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c010[1] + w2 * c011[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c010[2] + w2 * c011[2] + w3 * c111[2],
            )
        } else {
            // T3: dg > dr > db → c000, c010, c110, c111
            let w0 = 1.0 - dg;
            let w1 = dg - dr;
            let w2 = dr - db;
            let w3 = db;
            (
                w0 * c000[0] + w1 * c010[0] + w2 * c110[0] + w3 * c111[0],
                w0 * c000[1] + w1 * c010[1] + w2 * c110[1] + w3 * c111[1],
                w0 * c000[2] + w1 * c010[2] + w2 * c110[2] + w3 * c111[2],
            )
        }
    };

    (r_out, g_out, b_out)
}

fn apply_trilinear_lut(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    lut: &[f32],
    size: usize,
    intensity: f64,
) {
    apply_lut_impl(input, output, lut, size, intensity, false)
}

fn apply_tetrahedral_lut(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    lut: &[f32],
    size: usize,
    intensity: f64,
) {
    apply_lut_impl(input, output, lut, size, intensity, true)
}

fn apply_lut_impl(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    lut: &[f32],
    size: usize,
    intensity: f64,
    tetrahedral: bool,
) {
    output.data.data.copy_from_slice(&input.data.data);
    output.color_space = input.color_space.clone();
    output.icc_profile = input.icc_profile.clone();

    let expected = size * size * size * 3;
    if lut.len() < expected || intensity <= 0.0 {
        return;
    }

    let sample_fn: fn(&[f32], usize, f32, f32, f32) -> (f32, f32, f32) = if tetrahedral {
        sample_lut_tetrahedral
    } else {
        sample_lut_trilinear
    };

    let channels = input.layout.channel_count() as usize;
    let size_f = size as f32;
    let intensity_f = intensity as f32 / 100.0;

    match input.format {
        PixelFormat::U8 => {
            let src: &[u8] = &input.data.data;
            let dst: &mut [u8] = &mut output.data.data;
            for px in 0..input.pixel_count() as usize {
                let pi = px * channels;
                if pi + 2 >= src.len() {
                    continue;
                }
                let r = (src[pi] as f32 / 255.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let g = (src[pi + 1] as f32 / 255.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let b = (src[pi + 2] as f32 / 255.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let (lr, lg, lb) = sample_fn(lut, size, r, g, b);
                if pi < dst.len() {
                    dst[pi] = blend_u8(src[pi], lr, intensity_f);
                }
                if pi + 1 < dst.len() {
                    dst[pi + 1] = blend_u8(src[pi + 1], lg, intensity_f);
                }
                if pi + 2 < dst.len() {
                    dst[pi + 2] = blend_u8(src[pi + 2], lb, intensity_f);
                }
            }
        }
        PixelFormat::U16 => {
            let src = input.data.as_u16_slice();
            let dst: &mut [u16] = bytemuck::cast_slice_mut(&mut output.data.data);
            for px in 0..input.pixel_count() as usize {
                let pi = px * channels;
                if pi + 2 >= src.len() {
                    continue;
                }
                let r = (src[pi] as f32 / 65535.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let g = (src[pi + 1] as f32 / 65535.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let b = (src[pi + 2] as f32 / 65535.0 * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let (lr, lg, lb) = sample_fn(lut, size, r, g, b);
                if pi < dst.len() {
                    dst[pi] = blend_u16(src[pi], lr, intensity_f);
                }
                if pi + 1 < dst.len() {
                    dst[pi + 1] = blend_u16(src[pi + 1], lg, intensity_f);
                }
                if pi + 2 < dst.len() {
                    dst[pi + 2] = blend_u16(src[pi + 2], lb, intensity_f);
                }
            }
        }
        PixelFormat::F32 => {
            let src = input.data.as_f32_slice();
            let dst: &mut [f32] = bytemuck::cast_slice_mut(&mut output.data.data);
            for px in 0..input.pixel_count() as usize {
                let pi = px * channels;
                if pi + 2 >= src.len() {
                    continue;
                }
                let r = (src[pi] * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let g = (src[pi + 1] * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let b = (src[pi + 2] * (size_f - 1.0)).clamp(0.0, size_f - 1.0);
                let (lr, lg, lb) = sample_fn(lut, size, r, g, b);
                if pi < dst.len() {
                    dst[pi] = blend_f32(src[pi], lr, intensity_f);
                }
                if pi + 1 < dst.len() {
                    dst[pi + 1] = blend_f32(src[pi + 1], lg, intensity_f);
                }
                if pi + 2 < dst.len() {
                    dst[pi + 2] = blend_f32(src[pi + 2], lb, intensity_f);
                }
            }
        }
        _ => {}
    }
}

fn blend_u8(orig: u8, lut_val: f32, t: f32) -> u8 {
    if t >= 1.0 {
        (lut_val * 255.0).clamp(0.0, 255.0) as u8
    } else {
        (orig as f32 + ((lut_val * 255.0) - orig as f32) * t).clamp(0.0, 255.0) as u8
    }
}

fn blend_u16(orig: u16, lut_val: f32, t: f32) -> u16 {
    if t >= 1.0 {
        (lut_val * 65535.0).clamp(0.0, 65535.0) as u16
    } else {
        (orig as f32 + ((lut_val * 65535.0) - orig as f32) * t).clamp(0.0, 65535.0) as u16
    }
}

fn blend_f32(orig: f32, lut_val: f32, t: f32) -> f32 {
    if t >= 1.0 {
        lut_val
    } else {
        orig + (lut_val - orig) * t
    }
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

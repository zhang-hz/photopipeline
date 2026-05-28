use async_trait::async_trait;
use parking_lot::RwLock;
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
            id: "lens_detection".into(),
            label: "Lens Detection".into(),
            description: Some("Detect and configure lens parameters".into()),
            icon: Some("camera".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "correction_mode".into(),
                label: "Correction Mode".into(),
                description: Some("How to determine lens correction parameters".into()),
                help_url: None,
                field_type: ParameterType::Enum {
                    options: vec![
                        EnumOption {
                            value: "auto".into(),
                            label: "Auto (from metadata)".into(),
                            description: Some(
                                "Detect lens from EXIF and use LensFun database".into(),
                            ),
                            icon: None,
                            tags: vec![],
                            recommended: true,
                        },
                        EnumOption {
                            value: "manual".into(),
                            label: "Manual".into(),
                            description: Some("Specify lens parameters manually".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                        EnumOption {
                            value: "off".into(),
                            label: "Off".into(),
                            description: Some("Disable lens correction".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                    ],
                    display: Default::default(),
                },
                default: serde_json::json!("auto"),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
                ..Default::default()
            }],
        },
        ParameterSection {
            id: "corrections".into(),
            label: "Corrections".into(),
            description: Some("Types of lens correction to apply".into()),
            icon: Some("wrench".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "correct_distortion".into(),
                    label: "Distortion".into(),
                    description: Some("Correct barrel/pincushion distortion".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Correct".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "correct_tca".into(),
                    label: "TCA (Chromatic Aberration)".into(),
                    description: Some("Correct transverse chromatic aberration".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Correct".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "correct_vignetting".into(),
                    label: "Vignetting".into(),
                    description: Some("Correct light falloff towards corners".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Correct".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "correct_geometry".into(),
                    label: "Geometry".into(),
                    description: Some(
                        "Correct perspective/geometry distortion independently".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Correct".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
            ],
        },
        ParameterSection {
            id: "lensfun".into(),
            label: "LensFun".into(),
            description: Some("LensFun library configuration".into()),
            icon: Some("database".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "lensfun_db_path".into(),
                    label: "Database Path".into(),
                    description: Some("Custom path to LensFun XML database directory".into()),
                    help_url: None,
                    field_type: ParameterType::FilePath {
                        kind: photopipeline_core::FilePathKind::Directory,
                        filters: vec![],
                        must_exist: true,
                    },
                    default: serde_json::json!("/usr/share/lensfun"),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "camera_make".into(),
                    label: "Camera Make".into(),
                    description: Some("Override camera manufacturer for manual mode".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 128,
                        pattern: None,
                        placeholder: Some("Sony".into()),
                    },
                    default: serde_json::json!(""),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "camera_model".into(),
                    label: "Camera Model".into(),
                    description: Some("Override camera model for manual mode".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 128,
                        pattern: None,
                        placeholder: Some("ILCE-7RM5".into()),
                    },
                    default: serde_json::json!(""),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "lens_model".into(),
                    label: "Lens Model".into(),
                    description: Some("Override lens model for manual mode".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 256,
                        pattern: None,
                        placeholder: Some("FE 24-70mm F2.8 GM II".into()),
                    },
                    default: serde_json::json!(""),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
            ],
        },
    ],
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            GuiSection {
                param_section_id: "lens_detection".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "corrections".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "lensfun".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("aperture".into()),
    color: Some("#6366f1".into()),
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: photopipeline_core::SplitOrientation::Horizontal,
        lock_zoom: true,
    },
    aux_views: vec![AuxView::Histogram, AuxView::StatusText],
    min_panel_width: 340,
});

#[derive(Debug)]
pub struct LensCorrectPlugin {
    id: String,
    db: RwLock<Option<lensfun::Database>>,
}

impl LensCorrectPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.lens_correct".to_string(),
            db: RwLock::new(None),
        }
    }
}

impl Default for LensCorrectPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for LensCorrectPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "Lens Correction"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Enhance
    }
    fn description(&self) -> &str {
        "Correct lens distortion, chromatic aberration, and vignetting via LensFun"
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
        match lensfun::Database::load_bundled() {
            Ok(db) => {
                let cam_count = db.cameras.len();
                let lens_count = db.lenses.len();
                tracing::info!(
                    "LensCorrect initialized: LensFun database loaded ({} cameras, {} lenses)",
                    cam_count,
                    lens_count,
                );
                *self.db.write() = Some(db);
            }
            Err(e) => {
                tracing::warn!(
                    "LensCorrect: failed to load bundled LensFun database: {}. Lens correction will be unavailable.",
                    e,
                );
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("lens_correct plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("lens_correct: validating parameters");
        let mode = params.get_str("correction_mode").unwrap_or("auto");

        if mode == "manual" {
            let make = params.get_str("camera_make").unwrap_or("");
            let cam = params.get_str("camera_model").unwrap_or("");
            let lens = params.get_str("lens_model").unwrap_or("");

            if make.is_empty() && cam.is_empty() && lens.is_empty() {
                issues.push(ValidationIssue::Warning {
                    param: "correction_mode".into(),
                    message: "Manual mode selected but no camera/lens parameters provided".into(),
                });
            }
        }

        let db = params
            .get_str("lensfun_db_path")
            .unwrap_or("/usr/share/lensfun");
        if mode == "auto" && !db.is_empty() && !std::path::Path::new(db).exists() {
            issues.push(ValidationIssue::Warning {
                param: "lensfun_db_path".into(),
                message: format!("LensFun database directory not found: {}", db),
            });
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "lens_correct validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for LensCorrectPlugin {
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
            ColorSpace::LINEAR_SRGB,
            ColorSpace::DISPLAY_P3,
            ColorSpace::ADOBE_RGB,
        ]
    }

    fn required_gpu_backend(&self) -> Option<GpuBackend> {
        None
    }

    async fn process_pixels(
        &self,
        input: &PixelBuffer,
        output: &mut PixelBuffer,
        params: &ParameterSet,
        progress: Box<dyn ProgressSink>,
    ) -> PluginResult<ProcessingStats> {
        let _timer = PerfTimer::with_target("lens_correct_process_pixels", "plugin.lens_correct");
        progress.set_progress(0.0, "analyzing lens parameters");

        tracing::info!(
            input_dims = format!("{}x{}", input.width, input.height),
            input_format = ?input.format,
            "lens_correct: processing {}x{} {:?}",
            input.width,
            input.height,
            input.format,
        );

        let mode = params.get_str("correction_mode").unwrap_or("auto");
        let correct_dist = params
            .get("correct_distortion")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let correct_tca = params
            .get("correct_tca")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let correct_vig = params
            .get("correct_vignetting")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if mode == "off" {
            output.data.data.copy_from_slice(&input.data.data);
            output.width = input.width;
            output.height = input.height;
            output.layout = input.layout;
            output.format = input.format;
            output.color_space = input.color_space.clone();
            output.icc_profile = input.icc_profile.clone();

            progress.set_progress(1.0, "done");
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: input.pixel_count(),
                output_pixels: input.pixel_count(),
            });
        }

        progress.set_progress(0.3, "detecting lens from metadata");

        let corrections_active = [
            if correct_dist { "distortion" } else { "" },
            if correct_tca { "tca" } else { "" },
            if correct_vig { "vignetting" } else { "" },
        ];
        let active: Vec<&str> = corrections_active
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect();

        if active.is_empty() {
            output.data.data.copy_from_slice(&input.data.data);
            output.width = input.width;
            output.height = input.height;
            output.layout = input.layout;
            output.format = input.format;
            output.color_space = input.color_space.clone();
            output.icc_profile = input.icc_profile.clone();

            progress.set_progress(1.0, "done (no corrections)");
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: None,
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: input.pixel_count(),
                output_pixels: input.pixel_count(),
            });
        }

        // Try to find matching lens in database
        let db_guard = self.db.read();
        let db = match db_guard.as_ref() {
            Some(db) => db,
            None => {
                // LensFun database not loaded; pass-through with warning
                output.data.data.copy_from_slice(&input.data.data);
                output.width = input.width;
                output.height = input.height;
                output.layout = input.layout;
                output.format = input.format;
                output.color_space = input.color_space.clone();
                output.icc_profile = input.icc_profile.clone();
                tracing::warn!("LensCorrect: no LensFun database available, pass-through");
                progress.set_progress(1.0, "done (no database)");
                return Ok(ProcessingStats {
                    elapsed_ms: 0,
                    cpu_time_ms: 0,
                    gpu_time_ms: None,
                    peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                    input_pixels: input.pixel_count(),
                    output_pixels: input.pixel_count(),
                });
            }
        };

        let camera_make = params.get_str("camera_make").unwrap_or("");
        let camera_model = params.get_str("camera_model").unwrap_or("");
        let lens_name = params.get_str("lens_model").unwrap_or("");
        let focal_length: f32 = params
            .get("focal_length")
            .and_then(|v| v.as_f64())
            .unwrap_or(35.0) as f32;
        let aperture: f32 = params
            .get("aperture")
            .and_then(|v| v.as_f64())
            .unwrap_or(4.0) as f32;

        let cameras = db.find_cameras(
            if camera_make.is_empty() {
                None
            } else {
                Some(camera_make)
            },
            camera_model,
        );

        let mut modifier = None;
        for camera in &cameras {
            let lenses = db.find_lenses(
                Some(camera),
                if lens_name.is_empty() { "" } else { lens_name },
            );
            if let Some(lens) = lenses.first() {
                let mut m = lensfun::Modifier::new(
                    lens,
                    focal_length,
                    camera.crop_factor,
                    input.width,
                    input.height,
                    true,
                );
                if active.contains(&"distortion") {
                    m.enable_distortion_correction(lens);
                }
                if active.contains(&"tca") {
                    m.enable_tca_correction(lens);
                }
                if active.contains(&"vignetting") {
                    m.enable_vignetting_correction(lens, aperture, 5.0);
                }
                modifier = Some(m);
                tracing::info!(
                    "LensCorrect: matched camera {:?} lens {:?}",
                    camera.model,
                    lens.model,
                );
                break;
            }
        }

        match &modifier {
            Some(m) => {
                // Apply geometry distortion correction
                let h = input.height as usize;
                let w = input.width as usize;
                let c = input.layout.channel_count() as usize;

                output.width = input.width;
                output.height = input.height;
                output.layout = input.layout;
                output.format = input.format;
                output.color_space = input.color_space.clone();
                output.icc_profile = input.icc_profile.clone();
                output.data.data.resize(input.data.data.len(), 0);

                // Generate distortion coordinate map for each row
                let mut coords = vec![0.0f32; w * 2];
                for y in 0..h {
                    m.apply_geometry_distortion(0.0, y as f32, w, 1, &mut coords);
                    for x in 0..w {
                        let src_x = coords[x * 2];
                        let src_y = coords[x * 2 + 1];

                        // Bilinear interpolation from source
                        let sx = src_x.clamp(0.0, (w - 1) as f32);
                        let sy = src_y.clamp(0.0, (h - 1) as f32);
                        let ix = sx as usize;
                        let iy = sy as usize;
                        let fx = sx - ix as f32;
                        let fy = sy - iy as f32;

                        let ixp1 = (ix + 1).min(w - 1);
                        let iyp1 = (iy + 1).min(h - 1);

                        let dst_off = (y * w + x) * c * input.format.bytes_per_channel();
                        let s00 = (iy * w + ix) * c;
                        let s10 = (iy * w + ixp1) * c;
                        let s01 = (iyp1 * w + ix) * c;
                        let s11 = (iyp1 * w + ixp1) * c;

                        for ch in 0..c {
                            let v00 = input.data.data[s00 + ch] as f32;
                            let v10 = input.data.data[s10 + ch] as f32;
                            let v01 = input.data.data[s01 + ch] as f32;
                            let v11 = input.data.data[s11 + ch] as f32;

                            let v0 = v00 + (v10 - v00) * fx;
                            let v1 = v01 + (v11 - v01) * fx;
                            let v = (v0 + (v1 - v0) * fy).clamp(0.0, 255.0) as u8;

                            output.data.data[dst_off + ch] = v;
                        }
                    }
                    progress.set_progress(
                        (y + 1) as f32 / h as f32 * 0.9 + 0.1,
                        "correcting distortion",
                    );
                }

                progress.set_progress(1.0, "done");
                Ok(ProcessingStats {
                    elapsed_ms: 0,
                    cpu_time_ms: 0,
                    gpu_time_ms: None,
                    peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                    input_pixels: input.pixel_count(),
                    output_pixels: input.pixel_count(),
                })
            }
            None => {
                // No matching lens found; pass-through
                output.data.data.copy_from_slice(&input.data.data);
                output.width = input.width;
                output.height = input.height;
                output.layout = input.layout;
                output.format = input.format;
                output.color_space = input.color_space.clone();
                output.icc_profile = input.icc_profile.clone();
                tracing::info!(
                    "LensCorrect: no matching lens found for make={}, model={}, lens={}; pass-through",
                    camera_make,
                    camera_model,
                    lens_name,
                );
                progress.set_progress(1.0, "done (no lens match)");
                Ok(ProcessingStats {
                    elapsed_ms: 0,
                    cpu_time_ms: 0,
                    gpu_time_ms: None,
                    peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                    input_pixels: input.pixel_count(),
                    output_pixels: input.pixel_count(),
                })
            }
        }
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "lens".into(),
        "correction".into(),
        "distortion".into(),
        "chromatic".into(),
        "vignetting".into(),
        "lensfun".into(),
        "optics".into(),
        "enhance".into(),
    ]
});

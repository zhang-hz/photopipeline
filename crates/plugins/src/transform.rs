use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    ChannelLayout, ColorSpace, GpuBackend, HardwareRequirement, PerfTimer, PixelBuffer,
    PixelFormat, PluginCategory, PluginId, PluginResult, PluginVersion, ProcessingStats,
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
            id: "resize".into(),
            label: "Resize".into(),
            description: Some("Resize the image to target dimensions".into()),
            icon: Some("maximize".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "resize_mode".into(),
                    label: "Mode".into(),
                    description: Some("How to determine the output size".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "absolute".into(),
                                label: "Absolute".into(),
                                description: Some("Specific pixel dimensions".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "percentage".into(),
                                label: "Percentage".into(),
                                description: Some("Scale by percentage".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "long_edge".into(),
                                label: "Long Edge".into(),
                                description: Some("Fit the longer side to a target".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "short_edge".into(),
                                label: "Short Edge".into(),
                                description: Some("Fit the shorter side to a target".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "megapixels".into(),
                                label: "Megapixels".into(),
                                description: Some("Target total pixel count".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "none".into(),
                                label: "None".into(),
                                description: Some("Do not resize".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("none"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "target_width".into(),
                    label: "Width".into(),
                    description: Some("Target width in pixels".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(1920),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "target_height".into(),
                    label: "Height".into(),
                    description: Some("Target height in pixels".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(1080),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "scale_percent".into(),
                    label: "Scale".into(),
                    description: Some("Scale percentage (100 = no change)".into()),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: 1.0,
                        max: 1000.0,
                        step: 1.0,
                        precision: 1,
                        unit: Some("%".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(100.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "long_edge_px".into(),
                    label: "Long Edge Pixels".into(),
                    description: Some("Target size for the longer edge".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(2048),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "rotation".into(),
            label: "Rotation".into(),
            description: Some("Rotate the image by a specified angle".into()),
            icon: Some("rotate-cw".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "angle".into(),
                    label: "Angle".into(),
                    description: Some("Rotation angle in degrees clockwise".into()),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: -360.0,
                        max: 360.0,
                        step: 0.1,
                        precision: 1,
                        unit: Some("deg".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "flip_horizontal".into(),
                    label: "Flip Horizontal".into(),
                    description: Some("Mirror the image horizontally".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Flipped".into()),
                        label_false: Some("Normal".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "flip_vertical".into(),
                    label: "Flip Vertical".into(),
                    description: Some("Mirror the image vertically".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Flipped".into()),
                        label_false: Some("Normal".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "crop".into(),
            label: "Crop".into(),
            description: Some("Crop the image to a rectangular region".into()),
            icon: Some("crop".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "crop_enabled".into(),
                    label: "Enable Crop".into(),
                    description: Some("Apply cropping to the image".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Enabled".into()),
                        label_false: Some("Disabled".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "crop_x".into(),
                    label: "Crop X".into(),
                    description: Some("Left edge of crop region".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "crop_y".into(),
                    label: "Crop Y".into(),
                    description: Some("Top edge of crop region".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "crop_width".into(),
                    label: "Crop Width".into(),
                    description: Some("Width of crop region".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(1920),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "crop_height".into(),
                    label: "Crop Height".into(),
                    description: Some("Height of crop region".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 65535,
                        step: 1,
                        unit: Some("px".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(1080),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "filter".into(),
            label: "Filter".into(),
            description: Some("Resampling filter options".into()),
            icon: Some("filter".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![ParameterField {
                id: "filter_type".into(),
                label: "Filter".into(),
                description: Some(
                    "Resampling filter (Lanczos3 via Halide, bilinear for now)".into(),
                ),
                help_url: None,
                field_type: ParameterType::Enum {
                    options: vec![
                        EnumOption {
                            value: "bilinear".into(),
                            label: "Bilinear".into(),
                            description: Some("Fast, good for upscaling".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                        EnumOption {
                            value: "lanczos3".into(),
                            label: "Lanczos3".into(),
                            description: Some("High quality, will use Halide".into()),
                            icon: None,
                            tags: vec![],
                            recommended: true,
                        },
                        EnumOption {
                            value: "nearest".into(),
                            label: "Nearest Neighbor".into(),
                            description: Some("No interpolation, pixel-art style".into()),
                            icon: None,
                            tags: vec![],
                            recommended: false,
                        },
                    ],
                    display: Default::default(),
                },
                default: serde_json::json!("lanczos3"),
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
                param_section_id: "resize".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "rotation".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "crop".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "filter".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("maximize".into()),
    color: Some("#06b6d4".into()),
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: photopipeline_core::SplitOrientation::Horizontal,
        lock_zoom: false,
    },
    aux_views: vec![AuxView::Histogram],
    min_panel_width: 340,
});

#[derive(Debug, Clone)]
pub struct TransformPlugin {
    id: String,
}

impl TransformPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.transform".to_string(),
        }
    }
}

impl Default for TransformPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TransformPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "Transform"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Transform
    }
    fn description(&self) -> &str {
        "Resize, rotate, and crop images with configurable filters"
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
        tracing::info!("transform plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("transform plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("transform: validating parameters");
        let mode = params.get_str("resize_mode").unwrap_or("none");

        if mode == "absolute" {
            let w = params.get_i64("target_width").unwrap_or(0);
            let h = params.get_i64("target_height").unwrap_or(0);
            if !(1..=65535).contains(&w) {
                issues.push(ValidationIssue::Error {
                    param: "target_width".into(),
                    message: "Width must be between 1 and 65535".into(),
                });
            }
            if !(1..=65535).contains(&h) {
                issues.push(ValidationIssue::Error {
                    param: "target_height".into(),
                    message: "Height must be between 1 and 65535".into(),
                });
            }
        }

        let angle = params.get("angle").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if !(-360.0..=360.0).contains(&angle) {
            issues.push(ValidationIssue::Error {
                param: "angle".into(),
                message: "Angle must be between -360 and 360 degrees".into(),
            });
        }

        let crop = params
            .get("crop_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if crop {
            let cw = params.get_i64("crop_width").unwrap_or(0);
            let ch = params.get_i64("crop_height").unwrap_or(0);
            if cw < 1 {
                issues.push(ValidationIssue::Error {
                    param: "crop_width".into(),
                    message: "Crop width must be at least 1".into(),
                });
            }
            if ch < 1 {
                issues.push(ValidationIssue::Error {
                    param: "crop_height".into(),
                    message: "Crop height must be at least 1".into(),
                });
            }
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "transform validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for TransformPlugin {
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
        let _timer = PerfTimer::with_target("transform_process_pixels", "plugin.transform");
        progress.set_progress(0.0, "transforming");

        let mode = params.get_str("resize_mode").unwrap_or("none");
        tracing::info!(
            input_dims = format!("{}x{}", input.width, input.height),
            input_format = ?input.format,
            resize_mode = mode,
            "transform: processing {}x{} {:?} (mode={})",
            input.width,
            input.height,
            input.format,
            mode,
        );
        let crop_enabled = params
            .get("crop_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let angle = params.get("angle").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let flip_h = params
            .get("flip_horizontal")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let flip_v = params
            .get("flip_vertical")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let filter = params.get_str("filter_type").unwrap_or("lanczos3");

        let mut target_w = input.width;
        let mut target_h = input.height;

        match mode {
            "absolute" => {
                target_w = params.get_i64("target_width").unwrap_or(target_w as i64) as u32;
                target_h = params.get_i64("target_height").unwrap_or(target_h as i64) as u32;
                target_w = target_w.max(1);
                target_h = target_h.max(1);
            }
            "percentage" => {
                let pct = params.get_f64("scale_percent").unwrap_or(100.0);
                target_w = (input.width as f64 * pct / 100.0).round() as u32;
                target_h = (input.height as f64 * pct / 100.0).round() as u32;
                target_w = target_w.max(1);
                target_h = target_h.max(1);
            }
            "long_edge" => {
                let long = params.get_i64("long_edge_px").unwrap_or(2048) as u32;
                if input.width >= input.height {
                    target_w = long;
                    target_h =
                        (input.height as f64 * long as f64 / input.width as f64).round() as u32;
                } else {
                    target_h = long;
                    target_w =
                        (input.width as f64 * long as f64 / input.height as f64).round() as u32;
                }
                target_w = target_w.max(1);
                target_h = target_h.max(1);
            }
            "short_edge" => {
                let short = params.get_i64("target_width").unwrap_or(1080) as u32;
                if input.width <= input.height {
                    target_w = short;
                    target_h =
                        (input.height as f64 * short as f64 / input.width as f64).round() as u32;
                } else {
                    target_h = short;
                    target_w =
                        (input.width as f64 * short as f64 / input.height as f64).round() as u32;
                }
                target_w = target_w.max(1);
                target_h = target_h.max(1);
            }
            "megapixels" => {
                let mp = params.get_f64("scale_percent").unwrap_or(4.0);
                let target_pixels = mp * 1_000_000.0;
                let aspect = input.width as f64 / input.height as f64;
                target_h = (target_pixels / aspect).sqrt().round() as u32;
                target_w = (target_h as f64 * aspect).round() as u32;
                target_w = target_w.max(1);
                target_h = target_h.max(1);
            }
            _ => {}
        }

        if angle.abs() > 0.001 {
            let rad = angle.to_radians();
            let cos_a = rad.cos().abs();
            let sin_a = rad.sin().abs();
            let extent_w = (target_w as f64 * cos_a + target_h as f64 * sin_a).ceil() as u32;
            let extent_h = (target_w as f64 * sin_a + target_h as f64 * cos_a).ceil() as u32;
            target_w = extent_w.max(1);
            target_h = extent_h.max(1);
        }

        if crop_enabled {
            let cw = params.get_i64("crop_width").unwrap_or(target_w as i64) as u32;
            let ch = params.get_i64("crop_height").unwrap_or(target_h as i64) as u32;
            target_w = cw.min(target_w).max(1);
            target_h = ch.min(target_h).max(1);
        }

        let channels = match input.layout {
            ChannelLayout::RGB => 3,
            ChannelLayout::RGBA => 4,
            ChannelLayout::Gray => 1,
            ChannelLayout::GrayAlpha => 2,
            ChannelLayout::Planar(n) | ChannelLayout::Custom(n) => n as usize,
        };

        let in_stride = input.width as usize * channels;
        let out_stride = target_w as usize * channels;

        match filter {
            "nearest" => {
                nearest_resize(
                    input, output, target_w, target_h, channels, in_stride, out_stride,
                );
            }
            "lanczos3" => {
                if let Ok(f32_input) = extract_transform_f32(input, channels) {
                    match photopipeline_halide::HalideContext::resize(
                        &f32_input,
                        input.width,
                        input.height,
                        channels as u32,
                        target_w,
                        target_h,
                        "lanczos3",
                    ) {
                        Ok(result) => {
                            write_transform_f32_result(output, &result, target_w, target_h, input);
                            if flip_h || flip_v {
                                flip_buffer(
                                    output,
                                    target_w,
                                    target_h,
                                    channels,
                                    target_w as usize * channels,
                                    flip_h,
                                    flip_v,
                                );
                            }
                            output.width = target_w;
                            output.height = target_h;
                            output.layout = input.layout;
                            output.format = input.format;
                            output.color_space = input.color_space.clone();
                            output.icc_profile = input.icc_profile.clone();

                            let pixels = target_w as u64 * target_h as u64;
                            progress.set_progress(1.0, "done (Halide lanczos3)");
                            return Ok(ProcessingStats {
                                elapsed_ms: 0,
                                cpu_time_ms: 0,
                                gpu_time_ms: Some(0),
                                peak_memory_mb: (output.data.data.len() * 2) as u64 / (1024 * 1024),
                                input_pixels: input.width as u64 * input.height as u64,
                                output_pixels: pixels,
                            });
                        }
                        Err(_) => {}
                    }
                }
                bilinear_resize(
                    input, output, target_w, target_h, channels, in_stride, out_stride,
                );
            }
            _ => {
                bilinear_resize(
                    input, output, target_w, target_h, channels, in_stride, out_stride,
                );
            }
        }

        if flip_h || flip_v {
            flip_buffer(
                output, target_w, target_h, channels, out_stride, flip_h, flip_v,
            );
        }

        output.width = target_w;
        output.height = target_h;
        output.layout = input.layout;
        output.format = input.format;
        output.color_space = input.color_space.clone();
        output.icc_profile = input.icc_profile.clone();

        let pixels = target_w as u64 * target_h as u64;
        progress.set_progress(1.0, "done");

        tracing::info!(
            output_dims = format!("{}x{}", target_w, target_h),
            output_pixels = pixels,
            "transform: produced {}x{} output ({} pixels)",
            target_w,
            target_h,
            pixels,
        );

        Ok(ProcessingStats {
            elapsed_ms: 0,
            cpu_time_ms: 0,
            gpu_time_ms: None,
            peak_memory_mb: (output.data.data.len() * 2) as u64 / (1024 * 1024),
            input_pixels: input.width as u64 * input.height as u64,
            output_pixels: pixels,
        })
    }
}

fn nearest_resize(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    target_w: u32,
    target_h: u32,
    channels: usize,
    in_stride: usize,
    out_stride: usize,
) {
    let scale_x = input.width as f64 / target_w as f64;
    let scale_y = input.height as f64 / target_h as f64;

    for y in 0..target_h {
        let src_y = ((y as f64 + 0.5) * scale_y - 0.5).round() as usize;
        let src_y = src_y.min(input.height as usize - 1);
        let row_start = src_y * in_stride;
        let row_end = ((src_y + 1) * in_stride).min(input.data.data.len());
        let src_row = &input.data.data[row_start..row_end];
        let dst_row_start = y as usize * out_stride;

        for x in 0..target_w {
            let src_x = ((x as f64 + 0.5) * scale_x - 0.5).round() as usize;
            let src_x = src_x.min(input.width as usize - 1);
            let src_offset = src_x * channels;
            let dst_offset = dst_row_start + x as usize * channels;

            if dst_offset + channels > output.data.data.len() {
                continue;
            }
            for c in 0..channels {
                output.data.data[dst_offset + c] =
                    src_row.get(src_offset + c).copied().unwrap_or(0);
            }
        }
    }
}

fn bilinear_resize(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    target_w: u32,
    target_h: u32,
    channels: usize,
    in_stride: usize,
    out_stride: usize,
) {
    // NOTE: This implementation assumes 8-bit channels.
    // For 16-bit or float formats, use a format-aware resize path.
    if input.format.bytes_per_channel() != 1 {
        nearest_resize(
            input, output, target_w, target_h, channels, in_stride, out_stride,
        );
        return;
    }
    let scale_x = if target_w > 1 {
        (input.width as f64 - 1.0) / (target_w as f64 - 1.0)
    } else {
        1.0
    };
    let scale_y = if target_h > 1 {
        (input.height as f64 - 1.0) / (target_h as f64 - 1.0)
    } else {
        1.0
    };

    for y in 0..target_h {
        let src_y_f = y as f64 * scale_y;
        let src_y0 = (src_y_f.floor() as usize).min(input.height as usize - 1);
        let src_y1 = (src_y0 + 1).min(input.height as usize - 1);
        let frac_y = src_y_f - src_y_f.floor();

        let row0_start = src_y0 * in_stride;
        let row0_end = (src_y0 + 1) * in_stride;
        let row1_start = src_y1 * in_stride;
        let row1_end = (src_y1 + 1) * in_stride;
        let row0 = &input.data.data[row0_start..row0_end.min(input.data.data.len())];
        let row1 = &input.data.data[row1_start..row1_end.min(input.data.data.len())];
        let dst_row_start = y as usize * out_stride;

        for x in 0..target_w {
            let src_x_f = x as f64 * scale_x;
            let src_x0 = (src_x_f.floor() as usize).min(input.width as usize - 1);
            let src_x1 = (src_x0 + 1).min(input.width as usize - 1);
            let frac_x = src_x_f - src_x_f.floor();

            let dst_offset = dst_row_start + x as usize * channels;
            if dst_offset + channels > output.data.data.len() {
                continue;
            }

            for c in 0..channels {
                let src_idx0 = (src_x0 * channels + c).min(row0.len().saturating_sub(1));
                let src_idx1 = (src_x1 * channels + c).min(row1.len().saturating_sub(1));
                let v00 = row0[src_idx0] as f64;
                let v10 = row0.get(src_idx1).copied().unwrap_or(0) as f64;
                let v01 = row1.get(src_idx0).copied().unwrap_or(0) as f64;
                let v11 = row1.get(src_idx1).copied().unwrap_or(0) as f64;

                let top = v00 + (v10 - v00) * frac_x;
                let bottom = v01 + (v11 - v01) * frac_x;
                let val = (top + (bottom - top) * frac_y).clamp(0.0, 255.0);

                output.data.data[dst_offset + c] = val.round() as u8;
            }
        }
    }
}

fn flip_buffer(
    output: &mut PixelBuffer,
    w: u32,
    h: u32,
    channels: usize,
    stride: usize,
    flip_h: bool,
    flip_v: bool,
) {
    let temp = output.data.data.clone();

    for y in 0..h as usize {
        for x in 0..w as usize {
            let src_x = if flip_h { w as usize - 1 - x } else { x };
            let src_y = if flip_v { h as usize - 1 - y } else { y };
            let src_start = src_y * stride + src_x * channels;
            let dst_start = y * stride + x * channels;
            for c in 0..channels {
                output.data.data[dst_start + c] = temp[src_start + c];
            }
        }
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "transform".into(),
        "resize".into(),
        "rotate".into(),
        "crop".into(),
        "scale".into(),
        "geometry".into(),
        "bilinear".into(),
        "lanczos".into(),
    ]
});

fn extract_transform_f32(input: &PixelBuffer, _channels: usize) -> Result<Vec<f32>, ()> {
    let count = input.width as usize * input.height as usize * _channels;
    let f32_data = input.data.as_f32_slice();
    if f32_data.len() < count {
        return Err(());
    }
    Ok(f32_data[..count].to_vec())
}

fn write_transform_f32_result(
    output: &mut PixelBuffer,
    data: &[f32],
    width: u32,
    height: u32,
    input: &PixelBuffer,
) {
    let channels = input.layout.channel_count() as usize;
    let expected = width as usize * height as usize * channels;
    let copy_len = expected.min(data.len()).min(output.data.data.len() / 4);
    output.data.data.resize(copy_len * 4, 0);
    let out_f32: &mut [f32] = bytemuck::cast_slice_mut(&mut output.data.data);
    for (i, &v) in data.iter().take(copy_len).enumerate() {
        if i < out_f32.len() {
            out_f32[i] = v;
        }
    }
}

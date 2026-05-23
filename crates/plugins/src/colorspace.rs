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
            id: "conversion".into(),
            label: "Color Space Conversion".into(),
            description: Some("Select source and target color spaces".into()),
            icon: Some("palette".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "source_color_space".into(),
                    label: "Source".into(),
                    description: Some("Input color space (overrides detected)".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "auto".into(),
                                label: "Auto-detect".into(),
                                description: Some("Detect from embedded profile".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "srgb".into(),
                                label: "sRGB".into(),
                                description: Some("Standard sRGB IEC61966-2.1".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "display_p3".into(),
                                label: "Display P3".into(),
                                description: Some("Wide gamut P3 D65".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "adobe_rgb".into(),
                                label: "Adobe RGB".into(),
                                description: Some("Adobe RGB (1998)".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "pro_photo".into(),
                                label: "ProPhoto RGB".into(),
                                description: Some("Kodak ProPhoto RGB".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "bt2020".into(),
                                label: "BT.2020".into(),
                                description: Some("Rec. 2020 UHDTV".into()),
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "aces_cg".into(),
                                label: "ACEScg".into(),
                                description: Some("ACES CG linear".into()),
                                icon: None,
                                tags: vec!["cinema".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "linear_srgb".into(),
                                label: "Linear sRGB".into(),
                                description: Some("Linear-light sRGB".into()),
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
                },
                ParameterField {
                    id: "target_color_space".into(),
                    label: "Target".into(),
                    description: Some("Output color space".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "srgb".into(),
                                label: "sRGB".into(),
                                description: Some("Standard sRGB".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "display_p3".into(),
                                label: "Display P3".into(),
                                description: Some("Wide gamut P3".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "adobe_rgb".into(),
                                label: "Adobe RGB".into(),
                                description: Some("Adobe RGB (1998)".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "pro_photo".into(),
                                label: "ProPhoto RGB".into(),
                                description: Some("Kodak ProPhoto".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "bt2020_pq".into(),
                                label: "BT.2020 PQ (HDR)".into(),
                                description: Some(
                                    "Rec. 2020 with PQ transfer, HDR 1000 nits".into(),
                                ),
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "linear_srgb".into(),
                                label: "Linear sRGB".into(),
                                description: Some("Linear-light working space".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("srgb"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "rendering".into(),
            label: "Rendering".into(),
            description: Some("Rendering intent and gamut mapping options".into()),
            icon: Some("sliders".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "rendering_intent".into(),
                    label: "Rendering Intent".into(),
                    description: Some("How out-of-gamut colors are handled".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "relative_colorimetric".into(),
                                label: "Relative Colorimetric".into(),
                                description: Some("Clip out-of-gamut, preserve white point".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "perceptual".into(),
                                label: "Perceptual".into(),
                                description: Some("Compress gamut, preserve relationships".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "saturation".into(),
                                label: "Saturation".into(),
                                description: Some("Preserve saturation at cost of accuracy".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "absolute_colorimetric".into(),
                                label: "Absolute Colorimetric".into(),
                                description: Some("Preserve exact colors, clip".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("relative_colorimetric"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "black_point_compensation".into(),
                    label: "Black Point Compensation".into(),
                    description: Some(
                        "Adjust for different black levels between color spaces".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("On".into()),
                        label_false: Some("Off".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "gamut_mapping".into(),
                    label: "Gamut Mapping".into(),
                    description: Some("Algorithm for mapping out-of-gamut colors".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "clip".into(),
                                label: "Clip".into(),
                                description: Some("Hard clip to target gamut".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "compress".into(),
                                label: "Compress".into(),
                                description: Some("Smoothly compress into gamut".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "luminance_preserve".into(),
                                label: "Luminance Preserve".into(),
                                description: Some("Preserve luminance at cost of chroma".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("compress"),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "icc_profile".into(),
            label: "ICC Profile".into(),
            description: Some("ICC profile embedding and external profile usage".into()),
            icon: Some("file".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "embed_icc".into(),
                    label: "Embed ICC Profile".into(),
                    description: Some("Embed target ICC profile in output image".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Embed".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(true),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "icc_profile_path".into(),
                    label: "Custom ICC Profile".into(),
                    description: Some(
                        "Path to a custom ICC profile file for the target space".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::FilePath {
                        kind: Default::default(),
                        filters: vec![
                            ("ICC Profiles".into(), "*.icc".into()),
                            ("ICM Profiles".into(), "*.icm".into()),
                            ("All Files".into(), "*".into()),
                        ],
                        must_exist: true,
                    },
                    default: serde_json::json!(""),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
    ],
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| GuiSchema {
    layout: GuiLayout::Standard {
        sections: vec![
            GuiSection {
                param_section_id: "conversion".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "rendering".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "icc_profile".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("palette".into()),
    color: Some("#8b5cf6".into()),
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: photopipeline_core::SplitOrientation::Horizontal,
        lock_zoom: true,
    },
    aux_views: vec![AuxView::Histogram, AuxView::GamutDiagram],
    min_panel_width: 340,
});

#[derive(Debug, Clone)]
pub struct ColorSpacePlugin {
    id: String,
}

impl ColorSpacePlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.colorspace".to_string(),
        }
    }
}

impl Default for ColorSpacePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ColorSpacePlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "Color Space"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Color
    }
    fn description(&self) -> &str {
        "Convert between color spaces with ICC profile support and rendering intents"
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
        tracing::info!("colorspace plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("colorspace plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("colorspace: validating parameters");
        let source = params.get_str("source_color_space").unwrap_or("auto");
        let target = params.get_str("target_color_space").unwrap_or("srgb");

        if source != "auto" && source == target {
            issues.push(ValidationIssue::Warning {
                param: "target_color_space".into(),
                message: "Source and target color spaces are identical; no conversion needed"
                    .into(),
            });
        }

        let icc_path = params.get_str("icc_profile_path").unwrap_or("");
        if params
            .get("embed_icc")
            .map(|v| v.as_bool().unwrap_or(true))
            .unwrap_or(true)
            && !icc_path.is_empty()
            && !std::path::Path::new(icc_path).exists()
        {
            issues.push(ValidationIssue::Error {
                param: "icc_profile_path".into(),
                message: format!("ICC profile not found: {}", icc_path),
            });
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "colorspace validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl PixelProcessor for ColorSpacePlugin {
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
            ColorSpace::LINEAR_SRGB,
            ColorSpace::REC2020_PQ,
            ColorSpace::ACES_CG,
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
        let _timer = PerfTimer::with_target("colorspace_process_pixels", "plugin.colorspace");
        progress.set_progress(0.0, "converting color space");

        let source_str = params.get_str("source_color_space").unwrap_or("auto");
        let target_str = params.get_str("target_color_space").unwrap_or("srgb");

        tracing::info!(
            input_dims = format!("{}x{}", input.width, input.height),
            input_format = ?input.format,
            source_cs = source_str,
            target_cs = target_str,
            "colorspace: converting {}x{} {:?} from {} to {}",
            input.width,
            input.height,
            input.format,
            source_str,
            target_str,
        );
        let embed_icc = params
            .get("embed_icc")
            .map(|v| v.as_bool().unwrap_or(true))
            .unwrap_or(true);

        let target_cs = resolve_color_space(target_str);
        let source_cs = if source_str == "auto" {
            input.color_space.clone()
        } else {
            resolve_color_space(source_str)
        };

        if source_cs.primaries == target_cs.primaries && source_cs.transfer == target_cs.transfer {
            let copy_len = input.data.data.len().min(output.data.data.len());
            output.data.data[..copy_len].copy_from_slice(&input.data.data[..copy_len]);
            output.color_space = target_cs.clone();
            output.width = input.width;
            output.height = input.height;

            if embed_icc {
                output.icc_profile = Some(generate_icc_profile(&source_cs, &target_cs));
            }

            let pixels = input.pixel_count();
            progress.set_progress(1.0, "done (passthrough - same color space)");

            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: Some(0),
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: pixels,
                output_pixels: pixels,
            });
        }

        let channels = match input.layout {
            photopipeline_core::ChannelLayout::RGB => 3,
            photopipeline_core::ChannelLayout::RGBA => 4,
            photopipeline_core::ChannelLayout::Gray => 1,
            photopipeline_core::ChannelLayout::GrayAlpha => 2,
            photopipeline_core::ChannelLayout::Planar(n)
            | photopipeline_core::ChannelLayout::Custom(n) => n as u32,
        };

        if let Ok(f32_input) = extract_f32_pixels(input) {
            match photopipeline_halide::HalideContext::colorspace_convert(
                &f32_input,
                input.width,
                input.height,
                channels,
                &source_cs,
                &target_cs,
            ) {
                Ok(result) => {
                    write_f32_result(output, &result, input.width, input.height, input.layout);
                    output.color_space = target_cs.clone();
                    if embed_icc {
                        output.icc_profile = Some(generate_icc_profile(&source_cs, &target_cs));
                    }

                    let pixels = output.pixel_count();
                    progress.set_progress(1.0, "done (Halide)");
                    return Ok(ProcessingStats {
                        elapsed_ms: 0,
                        cpu_time_ms: 0,
                        gpu_time_ms: Some(0),
                        peak_memory_mb: (output.data.data.len() * 2) as u64 / (1024 * 1024),
                        input_pixels: input.pixel_count(),
                        output_pixels: pixels,
                    });
                }
                Err(_) => {}
            }
        }

        let copy_len = input.data.data.len().min(output.data.data.len());
        output.data.data[..copy_len].copy_from_slice(&input.data.data[..copy_len]);
        output.color_space = target_cs.clone();
        output.width = input.width;
        output.height = input.height;

        let pixels = input.pixel_count();
        progress.set_progress(1.0, "done (passthrough - no conversion engine)");
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

fn resolve_color_space(name: &str) -> ColorSpace {
    match name {
        "srgb" => ColorSpace::SRGB,
        "display_p3" => ColorSpace::DISPLAY_P3,
        "adobe_rgb" => ColorSpace::ADOBE_RGB,
        "pro_photo" => ColorSpace {
            primaries: photopipeline_core::ColorPrimaries::ProPhoto,
            transfer: photopipeline_core::TransferFunction::Gamma22,
            white_point: Default::default(),
            hdr_nits: None,
        },
        "bt2020_pq" => ColorSpace::REC2020_PQ,
        "linear_srgb" => ColorSpace::LINEAR_SRGB,
        "aces_cg" => ColorSpace::ACES_CG,
        _ => ColorSpace::SRGB,
    }
}

fn generate_icc_profile(_source: &ColorSpace, target: &ColorSpace) -> Vec<u8> {
    let desc = match target.primaries {
        photopipeline_core::ColorPrimaries::SRGB => "sRGB IEC61966-2.1",
        photopipeline_core::ColorPrimaries::DisplayP3 => "Display P3",
        photopipeline_core::ColorPrimaries::AdobeRGB => "Adobe RGB (1998)",
        photopipeline_core::ColorPrimaries::BT2020 => "ITU-R BT.2020",
        _ => "Custom Color Space",
    };

    format!(
        "ICC_PROFILE_PLACEHOLDER:{}:{}:{:?}:{:?}\n",
        desc,
        match target.transfer {
            photopipeline_core::TransferFunction::SRGB => "srgb-trc",
            photopipeline_core::TransferFunction::Linear => "linear",
            photopipeline_core::TransferFunction::Gamma22 => "gamma2.2",
            photopipeline_core::TransferFunction::PQ => "pq",
            _ => "custom",
        },
        target.white_point,
        target.hdr_nits,
    )
    .into_bytes()
}

fn extract_f32_pixels(input: &PixelBuffer) -> Result<Vec<f32>, ()> {
    if input.format != PixelFormat::F32 {
        return Err(());
    }
    let expected =
        input.width as usize * input.height as usize * input.layout.channel_count() as usize;
    let f32_data = input.data.as_f32_slice();
    if f32_data.len() < expected {
        return Err(());
    }
    Ok(f32_data[..expected].to_vec())
}

fn write_f32_result(
    output: &mut PixelBuffer,
    data: &[f32],
    width: u32,
    height: u32,
    layout: ChannelLayout,
) {
    let expected = width as usize * height as usize * layout.channel_count() as usize;
    let copy_len = expected.min(data.len()).min(output.data.data.len() / 4);
    output.data.data.resize(copy_len * 4, 0);
    let out_f32: &mut [f32] = bytemuck::cast_slice_mut(&mut output.data.data);
    for (i, &v) in data.iter().take(copy_len).enumerate() {
        if i < out_f32.len() {
            out_f32[i] = v;
        }
    }
    output.width = width;
    output.height = height;
    output.layout = layout;
    output.format = PixelFormat::F32;
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "color".into(),
        "colorspace".into(),
        "icc".into(),
        "profile".into(),
        "srgb".into(),
        "p3".into(),
        "hdr".into(),
        "display".into(),
        "gpu".into(),
    ]
});

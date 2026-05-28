use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    ChannelLayout, ColorSpace, GpuBackend, HardwareRequirement, PerfTimer, PixelBuffer,
    PixelFormat, PluginCategory, PluginError, PluginId, PluginResult, PluginVersion,
    ProcessingStats, RenderingIntent, ValidationIssue,
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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

        let intent = match params
            .get_str("rendering_intent")
            .unwrap_or("relative_colorimetric")
        {
            "perceptual" => RenderingIntent::Perceptual,
            "saturation" => RenderingIntent::Saturation,
            "absolute_colorimetric" => RenderingIntent::AbsoluteColorimetric,
            _ => RenderingIntent::RelativeColorimetric,
        };

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

        if convert_via_lcms2(input, output, &source_cs, &target_cs, intent).is_ok() {
            if embed_icc {
                output.icc_profile = Some(generate_icc_profile(&source_cs, &target_cs));
            }
            let pixels = output.pixel_count();
            progress.set_progress(1.0, "done (lcms2)");
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: Some(0),
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: input.pixel_count(),
                output_pixels: pixels,
            });
        }

        if convert_via_matrix(input, output, &source_cs, &target_cs).is_ok() {
            tracing::warn!("Halide and lcms2 unavailable, using pure Rust matrix conversion");
            if embed_icc {
                output.icc_profile = Some(generate_icc_profile(&source_cs, &target_cs));
            }
            let pixels = output.pixel_count();
            progress.set_progress(1.0, "done (matrix)");
            return Ok(ProcessingStats {
                elapsed_ms: 0,
                cpu_time_ms: 0,
                gpu_time_ms: Some(0),
                peak_memory_mb: (input.data.data.len() * 2) as u64 / (1024 * 1024),
                input_pixels: input.pixel_count(),
                output_pixels: pixels,
            });
        }

        Err(PluginError::Internal {
            plugin: self.id.clone(),
            message: format!(
                "Color space conversion from {:?}/{:?} to {:?}/{:?} requires Halide runtime. \
                 Install Halide 14.0+ or enable lcms2-native feature.",
                source_cs.primaries, source_cs.transfer, target_cs.primaries, target_cs.transfer,
            ),
        })
    }
}

#[cfg(feature = "lcms2-native")]
mod lcms2_ffi {
    use std::ffi::{c_uint, c_void};

    pub type CmsHPROFILE = *mut c_void;
    pub type CmsHTRANSFORM = *mut c_void;

    pub const TYPE_RGB_8: c_uint = 0x1906;
    pub const TYPE_RGB_16: c_uint = 0x1A06;
    pub const TYPE_RGBA_8: c_uint = 0x1908;
    pub const TYPE_RGBA_16: c_uint = 0x1A08;
    pub const TYPE_GRAY_8: c_uint = 0x1901;
    pub const TYPE_GRAY_16: c_uint = 0x1A01;
    pub const TYPE_RGB_FLT: c_uint = 0x1606;
    pub const TYPE_RGBA_FLT: c_uint = 0x1608;
    pub const TYPE_GRAY_FLT: c_uint = 0x1601;
    pub const TYPE_BGR_FLT: c_uint = 0x1607;

    pub const INTENT_PERCEPTUAL: c_uint = 0;
    pub const INTENT_RELATIVE_COLORIMETRIC: c_uint = 1;
    pub const INTENT_SATURATION: c_uint = 2;
    pub const INTENT_ABSOLUTE_COLORIMETRIC: c_uint = 3;

    // Link directives provided by lcms2-sys build.rs
    unsafe extern "C" {
        pub fn cmsCreate_sRGBProfile() -> CmsHPROFILE;
        pub fn cmsOpenProfileFromMem(data: *const c_void, size: c_uint) -> CmsHPROFILE;
        pub fn cmsCloseProfile(h: CmsHPROFILE);
        pub fn cmsCreateTransform(
            input: CmsHPROFILE,
            input_format: c_uint,
            output: CmsHPROFILE,
            output_format: c_uint,
            intent: c_uint,
            flags: c_uint,
        ) -> CmsHTRANSFORM;
        pub fn cmsDeleteTransform(h: CmsHTRANSFORM);
        pub fn cmsDoTransform(
            transform: CmsHTRANSFORM,
            input: *const c_void,
            output: *mut c_void,
            size: c_uint,
        );
    }
}

fn convert_via_lcms2(
    _input: &PixelBuffer,
    _output: &mut PixelBuffer,
    _source_space: &ColorSpace,
    _target_space: &ColorSpace,
    _intent: RenderingIntent,
) -> PluginResult<()> {
    #[cfg(not(feature = "lcms2-native"))]
    {
        Err(PluginError::Internal {
            plugin: "colorspace".into(),
            message: "lcms2 native not compiled".into(),
        })
    }

    #[cfg(feature = "lcms2-native")]
    unsafe {
        use lcms2_ffi::*;
        use std::ffi::{c_uint, c_void};

        let src_icc =
            _source_space
                .generate_icc_profile()
                .ok_or_else(|| PluginError::Internal {
                    plugin: "colorspace".into(),
                    message: format!(
                        "unsupported source color space: {:?} + {:?}",
                        _source_space.primaries, _source_space.transfer
                    ),
                })?;

        let dst_icc =
            _target_space
                .generate_icc_profile()
                .ok_or_else(|| PluginError::Internal {
                    plugin: "colorspace".into(),
                    message: format!(
                        "unsupported target color space: {:?} + {:?}",
                        _target_space.primaries, _target_space.transfer
                    ),
                })?;

        let src_profile =
            cmsOpenProfileFromMem(src_icc.as_ptr() as *const c_void, src_icc.len() as u32);
        let dst_profile =
            cmsOpenProfileFromMem(dst_icc.as_ptr() as *const c_void, dst_icc.len() as u32);

        if src_profile.is_null() || dst_profile.is_null() {
            if !src_profile.is_null() {
                cmsCloseProfile(src_profile);
            }
            if !dst_profile.is_null() {
                cmsCloseProfile(dst_profile);
            }
            return Err(PluginError::Internal {
                plugin: "colorspace".into(),
                message: "failed to open ICC profiles".into(),
            });
        }

        let (pixel_format, _channels) = match (_input.format, _input.layout) {
            (PixelFormat::U8, ChannelLayout::RGB) => (TYPE_RGB_8, 3usize),
            (PixelFormat::U16, ChannelLayout::RGB) => (TYPE_RGB_16, 3),
            (PixelFormat::U8, ChannelLayout::RGBA) => (TYPE_RGBA_8, 4),
            (PixelFormat::U16, ChannelLayout::RGBA) => (TYPE_RGBA_16, 4),
            (PixelFormat::U8, ChannelLayout::Gray) => (TYPE_GRAY_8, 1),
            (PixelFormat::U16, ChannelLayout::Gray) => (TYPE_GRAY_16, 1),
            (PixelFormat::F32, ChannelLayout::RGB) => (TYPE_RGB_FLT, 3),
            (PixelFormat::F32, ChannelLayout::RGBA) => (TYPE_RGBA_FLT, 4),
            (PixelFormat::F32, ChannelLayout::Gray) => (TYPE_GRAY_FLT, 1),
            (PixelFormat::F32, ChannelLayout::BGR) => (TYPE_BGR_FLT, 3),
            _ => (TYPE_RGB_8, 3),
        };

        let intent_val = match _intent {
            RenderingIntent::Perceptual => INTENT_PERCEPTUAL,
            RenderingIntent::RelativeColorimetric => INTENT_RELATIVE_COLORIMETRIC,
            RenderingIntent::Saturation => INTENT_SATURATION,
            RenderingIntent::AbsoluteColorimetric => INTENT_ABSOLUTE_COLORIMETRIC,
        };

        let transform = cmsCreateTransform(
            src_profile,
            pixel_format,
            dst_profile,
            pixel_format,
            intent_val,
            0,
        );
        if transform.is_null() {
            cmsCloseProfile(src_profile);
            cmsCloseProfile(dst_profile);
            return Err(PluginError::Internal {
                plugin: "colorspace".into(),
                message: "failed to create transform".into(),
            });
        }

        let pixel_count = (_input.width as usize * _input.height as usize) as c_uint;
        cmsDoTransform(
            transform,
            _input.data.data.as_ptr() as *const c_void,
            _output.data.data.as_mut_ptr() as *mut c_void,
            pixel_count,
        );

        cmsDeleteTransform(transform);
        cmsCloseProfile(src_profile);
        cmsCloseProfile(dst_profile);

        _output.width = _input.width;
        _output.height = _input.height;
        _output.layout = _input.layout;
        _output.format = _input.format;
        _output.color_space = _target_space.clone();

        Ok(())
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
    target.generate_icc_profile().unwrap_or_else(|| {
        format!(
            "ICC_PROFILE:{}:{:?}\n",
            target.primaries.description(),
            target.white_point
        )
        .into_bytes()
    })
}

fn convert_via_matrix(
    input: &PixelBuffer,
    output: &mut PixelBuffer,
    source_space: &ColorSpace,
    target_space: &ColorSpace,
) -> PluginResult<()> {
    let matrix = source_space
        .conversion_matrix_to(target_space)
        .ok_or_else(|| PluginError::Internal {
            plugin: "colorspace".into(),
            message: format!(
                "Cannot compute conversion matrix from {:?}/{:?} to {:?}/{:?}",
                source_space.primaries,
                source_space.transfer,
                target_space.primaries,
                target_space.transfer,
            ),
        })?;

    let channels = input.layout.channel_count() as usize;
    let pixel_count = input.width as usize * input.height as usize;
    let src_tf = source_space.transfer;
    let dst_tf = target_space.transfer;

    match input.format {
        PixelFormat::U8 => {
            let src: &[u8] = &input.data.data;
            let dst: &mut [u8] = bytemuck::cast_slice_mut(&mut output.data.data);
            for i in 0..pixel_count {
                let base = i * channels;
                let r = src_tf.decode_to_linear(src[base] as f32 / 255.0);
                let g = src_tf.decode_to_linear(src[base + 1] as f32 / 255.0);
                let b = src_tf.decode_to_linear(src[base + 2] as f32 / 255.0);

                let r2 =
                    (matrix[0][0] as f32 * r + matrix[0][1] as f32 * g + matrix[0][2] as f32 * b)
                        .clamp(0.0, 1.0);
                let g2 =
                    (matrix[1][0] as f32 * r + matrix[1][1] as f32 * g + matrix[1][2] as f32 * b)
                        .clamp(0.0, 1.0);
                let b2 =
                    (matrix[2][0] as f32 * r + matrix[2][1] as f32 * g + matrix[2][2] as f32 * b)
                        .clamp(0.0, 1.0);

                let enc_r = dst_tf.encode_from_linear(r2);
                let enc_g = dst_tf.encode_from_linear(g2);
                let enc_b = dst_tf.encode_from_linear(b2);

                dst[base] = (enc_r * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[base + 1] = (enc_g * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[base + 2] = (enc_b * 255.0).round().clamp(0.0, 255.0) as u8;
                if channels >= 4 {
                    dst[base + 3] = src[base + 3];
                }
            }
        }
        PixelFormat::U16 => {
            let src = input.data.as_u16_slice();
            let dst: &mut [u16] = bytemuck::cast_slice_mut(&mut output.data.data);
            for i in 0..pixel_count {
                let base = i * channels;
                let r = src_tf.decode_to_linear(src[base] as f32 / 65535.0);
                let g = src_tf.decode_to_linear(src[base + 1] as f32 / 65535.0);
                let b = src_tf.decode_to_linear(src[base + 2] as f32 / 65535.0);

                let r2 =
                    (matrix[0][0] as f32 * r + matrix[0][1] as f32 * g + matrix[0][2] as f32 * b)
                        .clamp(0.0, 1.0);
                let g2 =
                    (matrix[1][0] as f32 * r + matrix[1][1] as f32 * g + matrix[1][2] as f32 * b)
                        .clamp(0.0, 1.0);
                let b2 =
                    (matrix[2][0] as f32 * r + matrix[2][1] as f32 * g + matrix[2][2] as f32 * b)
                        .clamp(0.0, 1.0);

                let enc_r = dst_tf.encode_from_linear(r2);
                let enc_g = dst_tf.encode_from_linear(g2);
                let enc_b = dst_tf.encode_from_linear(b2);

                dst[base] = (enc_r * 65535.0).round().clamp(0.0, 65535.0) as u16;
                dst[base + 1] = (enc_g * 65535.0).round().clamp(0.0, 65535.0) as u16;
                dst[base + 2] = (enc_b * 65535.0).round().clamp(0.0, 65535.0) as u16;
                if channels >= 4 {
                    dst[base + 3] = src[base + 3];
                }
            }
        }
        PixelFormat::U32 => {
            return Err(PluginError::Internal {
                plugin: "colorspace".into(),
                message: "U32 matrix conversion not supported in pure Rust path".into(),
            });
        }
        PixelFormat::F16 => {
            return Err(PluginError::Internal {
                plugin: "colorspace".into(),
                message: "F16 matrix conversion not supported in pure Rust path".into(),
            });
        }
        PixelFormat::F32 => {
            let src = input.data.as_f32_slice();
            let dst: &mut [f32] = bytemuck::cast_slice_mut(&mut output.data.data);
            for i in 0..pixel_count {
                let base = i * channels;
                let r = src_tf.decode_to_linear(src[base]);
                let g = src_tf.decode_to_linear(src[base + 1]);
                let b = src_tf.decode_to_linear(src[base + 2]);

                let r2 =
                    matrix[0][0] as f32 * r + matrix[0][1] as f32 * g + matrix[0][2] as f32 * b;
                let g2 =
                    matrix[1][0] as f32 * r + matrix[1][1] as f32 * g + matrix[1][2] as f32 * b;
                let b2 =
                    matrix[2][0] as f32 * r + matrix[2][1] as f32 * g + matrix[2][2] as f32 * b;

                dst[base] = dst_tf.encode_from_linear(r2.max(0.0));
                dst[base + 1] = dst_tf.encode_from_linear(g2.max(0.0));
                dst[base + 2] = dst_tf.encode_from_linear(b2.max(0.0));
                if channels >= 4 {
                    dst[base + 3] = src[base + 3];
                }
            }
        }
    }

    output.width = input.width;
    output.height = input.height;
    output.layout = input.layout;
    output.format = input.format;
    output.color_space = target_space.clone();

    Ok(())
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

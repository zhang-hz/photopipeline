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

    pub const INTENT_PERCEPTUAL: c_uint = 0;
    pub const INTENT_RELATIVE_COLORIMETRIC: c_uint = 1;
    pub const INTENT_SATURATION: c_uint = 2;
    pub const INTENT_ABSOLUTE_COLORIMETRIC: c_uint = 3;

    #[link(name = "lcms2")]
    extern "C" {
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

#[cfg(feature = "lcms2-native")]
mod lcms2_profile_builder {
    use photopipeline_core::{ColorPrimaries, ColorSpace, TransferFunction, WhitePoint};

    pub fn primaries_chromaticities(p: &ColorPrimaries) -> ((f64, f64), (f64, f64), (f64, f64)) {
        match p {
            ColorPrimaries::SRGB | ColorPrimaries::BT709 => {
                ((0.6400, 0.3300), (0.3000, 0.6000), (0.1500, 0.0600))
            }
            ColorPrimaries::DisplayP3 | ColorPrimaries::DCIP3 => {
                ((0.6800, 0.3200), (0.2650, 0.6900), (0.1500, 0.0600))
            }
            ColorPrimaries::AdobeRGB => ((0.6400, 0.3300), (0.2100, 0.7100), (0.1500, 0.0600)),
            ColorPrimaries::BT2020 | ColorPrimaries::Rec2100 => {
                ((0.7080, 0.2920), (0.1700, 0.7970), (0.1310, 0.0460))
            }
            ColorPrimaries::ACEScg | ColorPrimaries::ACES => {
                ((0.7130, 0.2930), (0.1650, 0.8300), (0.1280, 0.0440))
            }
            ColorPrimaries::ProPhoto => ((0.7347, 0.2653), (0.1596, 0.8404), (0.0366, 0.0001)),
            ColorPrimaries::CIEXYZ => ((1.0, 0.0), (0.0, 1.0), (0.0, 0.0)),
        }
    }

    pub fn white_point_chromaticity(wp: &WhitePoint) -> (f64, f64) {
        match wp {
            WhitePoint::D50 => (0.34567, 0.35850),
            WhitePoint::D55 => (0.33242, 0.34743),
            WhitePoint::D60 => (0.32168, 0.33767),
            WhitePoint::D65 => (0.31270, 0.32900),
            WhitePoint::D75 => (0.29900, 0.31490),
            WhitePoint::DCI => (0.31400, 0.35100),
            WhitePoint::E => (1.0 / 3.0, 1.0 / 3.0),
            WhitePoint::Custom(x, _) => (*x as f64, 0.32900),
        }
    }

    pub fn white_point_to_xyz(wp: &WhitePoint) -> (f64, f64, f64) {
        match wp {
            WhitePoint::D50 => (0.96422, 1.0, 0.82521),
            WhitePoint::D55 => (0.95682, 1.0, 0.92149),
            WhitePoint::D60 => (0.952646, 1.0, 1.008825),
            WhitePoint::D65 => (0.95047, 1.0, 1.08883),
            WhitePoint::D75 => (0.94972, 1.0, 1.22638),
            WhitePoint::DCI => (0.98000, 1.0, 1.18000),
            WhitePoint::E => (1.0, 1.0, 1.0),
            WhitePoint::Custom(_, _) => (0.95047, 1.0, 1.08883),
        }
    }

    fn mat3_inverse(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        let inv_det = 1.0 / det;
        [
            [
                (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
                (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
                (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
            ],
            [
                (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
                (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
                (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
            ],
            [
                (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
                (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
                (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
            ],
        ]
    }

    fn mat3_mul_vec3(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
        [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ]
    }

    fn mat3_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            }
        }
        result
    }

    fn bradford_cat(src_xyz: &[f64; 3], dst_xyz: &[f64; 3]) -> [[f64; 3]; 3] {
        let bfd = [
            [0.8951000, 0.2664000, -0.1614000],
            [-0.7502000, 1.7135000, 0.0367000],
            [0.0389000, -0.0685000, 1.0296000],
        ];
        let bfd_inv = [
            [0.9869929, -0.1470543, 0.1599627],
            [0.4323053, 0.5183603, 0.0492912],
            [-0.0085287, 0.0400428, 0.9684867],
        ];

        let src_lms = mat3_mul_vec3(&bfd, src_xyz);
        let dst_lms = mat3_mul_vec3(&bfd, dst_xyz);

        let d = [
            dst_lms[0] / src_lms[0],
            dst_lms[1] / src_lms[1],
            dst_lms[2] / src_lms[2],
        ];

        let mut m1 = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                m1[i][j] = d[i] * bfd[i][j];
            }
        }
        mat3_mul(&bfd_inv, &m1)
    }

    pub fn compute_color_space_matrix(
        primaries: &ColorPrimaries,
        white_point: &WhitePoint,
    ) -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
        let ((xr, yr), (xg, yg), (xb, yb)) = primaries_chromaticities(primaries);
        let (xw, yw) = white_point_chromaticity(white_point);

        let xr_xyz = xr / yr;
        let zr_xyz = (1.0 - xr - yr) / yr;
        let xg_xyz = xg / yg;
        let zg_xyz = (1.0 - xg - yg) / yg;
        let xb_xyz = xb / yb;
        let zb_xyz = (1.0 - xb - yb) / yb;

        let xw_xyz = xw / yw;
        let zw_xyz = (1.0 - xw - yw) / yw;

        let m_prim = [
            [xr_xyz, xg_xyz, xb_xyz],
            [1.0, 1.0, 1.0],
            [zr_xyz, zg_xyz, zb_xyz],
        ];
        let m_inv = mat3_inverse(&m_prim);
        let w_vec = [xw_xyz, 1.0, zw_xyz];
        let s = mat3_mul_vec3(&m_inv, &w_vec);

        let rxyz = [s[0] * xr_xyz, s[0] * 1.0, s[0] * zr_xyz];
        let gxyz = [s[1] * xg_xyz, s[1] * 1.0, s[1] * zg_xyz];
        let bxyz = [s[2] * xb_xyz, s[2] * 1.0, s[2] * zb_xyz];
        let wtpt_native = [xw_xyz, 1.0, zw_xyz];

        let wp_src = white_point_to_xyz(white_point);
        let wp_d50 = white_point_to_xyz(&WhitePoint::D50);

        if (wp_src.0 - wp_d50.0).abs() < 0.001
            && (wp_src.1 - wp_d50.1).abs() < 0.001
            && (wp_src.2 - wp_d50.2).abs() < 0.001
        {
            return (rxyz, gxyz, bxyz, wtpt_native);
        }

        let cat = bradford_cat(
            &[wp_src.0, wp_src.1, wp_src.2],
            &[wp_d50.0, wp_d50.1, wp_d50.2],
        );
        let rxyz_d50 = mat3_mul_vec3(&cat, &rxyz);
        let gxyz_d50 = mat3_mul_vec3(&cat, &gxyz);
        let bxyz_d50 = mat3_mul_vec3(&cat, &bxyz);
        let wtpt_d50 = mat3_mul_vec3(&cat, &wtpt_native);

        (rxyz_d50, gxyz_d50, bxyz_d50, wtpt_d50)
    }

    fn generate_trc_curve(tf: &TransferFunction) -> Vec<u8> {
        const CURVE_ENTRIES: usize = 256;
        let mut data = Vec::with_capacity(12 + CURVE_ENTRIES * 2);

        data.extend_from_slice(b"curv");
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&(CURVE_ENTRIES as u32).to_be_bytes());

        for i in 0..CURVE_ENTRIES {
            let x = i as f64 / (CURVE_ENTRIES - 1) as f64;
            let y = match tf {
                TransferFunction::Linear => x,
                TransferFunction::SRGB => srgb_to_linear_f64(x),
                TransferFunction::Gamma22 => x.powf(2.2),
                TransferFunction::Gamma24 => x.powf(2.4),
                TransferFunction::Gamma26 => x.powf(2.6),
                TransferFunction::Gamma28 => x.powf(2.8),
                TransferFunction::PQ => pq_eotf_normalized(x),
                TransferFunction::HLG => hlg_oetf_inverse(x),
                TransferFunction::SLog3 => x,
                TransferFunction::LogC => x,
                TransferFunction::Custom(g) => x.powf(*g),
            };
            let y16 = (y.clamp(0.0, 1.0) * 65535.0).round() as u16;
            data.extend_from_slice(&y16.to_be_bytes());
        }

        data
    }

    fn srgb_to_linear_f64(v: f64) -> f64 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn pq_eotf_normalized(v: f64) -> f64 {
        let m1 = 2610.0 / 16384.0;
        let m2 = 2523.0 / 32.0;
        let c1 = 3424.0 / 4096.0;
        let c2 = 2413.0 / 128.0;
        let c3 = 2392.0 / 128.0;

        let v_pow = v.powf(1.0 / m2);
        let num = (v_pow - c1).max(0.0);
        let den = c2 - c3 * v_pow;
        let linear = (num / den.max(1e-10)).powf(1.0 / m1);
        linear / 10000.0
    }

    fn hlg_oetf_inverse(v: f64) -> f64 {
        let a = 0.17883277;
        let b = 1.0 - 4.0 * a;
        let c = 0.5 - a * (4.0f64).ln();

        if v <= 0.5 {
            v * v / 3.0
        } else {
            (((v - c) / a).exp() + b) / 12.0
        }
    }

    pub fn build_icc_v2_profile(
        rxyz: &[f64; 3],
        gxyz: &[f64; 3],
        bxyz: &[f64; 3],
        wtpt: &[f64; 3],
        tf: &TransferFunction,
        primaries: &ColorPrimaries,
    ) -> Vec<u8> {
        let desc = match primaries {
            ColorPrimaries::SRGB => "sRGB IEC61966-2.1",
            ColorPrimaries::DisplayP3 => "Display P3",
            ColorPrimaries::AdobeRGB => "Adobe RGB (1998)",
            ColorPrimaries::BT2020 => "ITU-R BT.2020",
            ColorPrimaries::ProPhoto => "ProPhoto RGB",
            ColorPrimaries::ACEScg => "ACEScg",
            ColorPrimaries::ACES => "ACES",
            ColorPrimaries::DCIP3 => "DCI-P3",
            ColorPrimaries::Rec2100 => "ITU-R BT.2100",
            ColorPrimaries::BT709 => "ITU-R BT.709",
            ColorPrimaries::CIEXYZ => "CIE XYZ",
        };

        fn s15f16(v: f64) -> u32 {
            (v * 65536.0).round() as i32 as u32
        }

        fn write_xyz_tag(data: &mut Vec<u8>, _sig: &[u8; 4], xyz: &[f64; 3]) {
            let _offset = data.len();
            data.extend_from_slice(b"XYZ ");
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&s15f16(xyz[0]).to_be_bytes());
            data.extend_from_slice(&s15f16(xyz[1]).to_be_bytes());
            data.extend_from_slice(&s15f16(xyz[2]).to_be_bytes());
        }

        let mut tag_data: Vec<u8> = Vec::new();
        let mut tag_table: Vec<(u32, usize, usize)> = Vec::new();

        let rxyz_offset = tag_data.len();
        write_xyz_tag(&mut tag_data, b"rXYZ", rxyz);
        tag_table.push((0x7258595A, rxyz_offset, 20));

        let gxyz_offset = tag_data.len();
        write_xyz_tag(&mut tag_data, b"gXYZ", gxyz);
        tag_table.push((0x6758595A, gxyz_offset, 20));

        let bxyz_offset = tag_data.len();
        write_xyz_tag(&mut tag_data, b"bXYZ", bxyz);
        tag_table.push((0x6258595A, bxyz_offset, 20));

        let wtpt_offset = tag_data.len();
        write_xyz_tag(&mut tag_data, b"wtpt", wtpt);
        tag_table.push((0x77747074, wtpt_offset, 20));

        let trc = generate_trc_curve(tf);
        let trc_size = trc.len();

        let rtrc_offset = tag_data.len();
        tag_data.extend_from_slice(&trc);
        tag_table.push((0x72545243, rtrc_offset, trc_size));

        let gtrc_offset = tag_data.len();
        tag_data.extend_from_slice(&trc);
        tag_table.push((0x67545243, gtrc_offset, trc_size));

        let btrc_offset = tag_data.len();
        tag_data.extend_from_slice(&trc);
        tag_table.push((0x62545243, btrc_offset, trc_size));

        let desc_ascii = format!("{}\0", desc);
        let desc_payload_len = desc_ascii.len();
        let desc_padded = (desc_payload_len + 3) & !3;

        let mut desc_tag = Vec::with_capacity(12 + desc_padded + 16);
        desc_tag.extend_from_slice(b"desc");
        desc_tag.extend_from_slice(&0u32.to_be_bytes());
        desc_tag.extend_from_slice(&(desc_payload_len as u32).to_be_bytes());
        desc_tag.extend_from_slice(desc_ascii.as_bytes());
        while desc_tag.len() < 12 + desc_padded {
            desc_tag.push(0);
        }
        desc_tag.extend_from_slice(&0u32.to_be_bytes());
        desc_tag.extend_from_slice(&0u32.to_be_bytes());
        desc_tag.extend_from_slice(&0u32.to_be_bytes());
        desc_tag.extend_from_slice(&0u32.to_be_bytes());

        let desc_offset = tag_data.len();
        let desc_size = desc_tag.len();
        tag_data.extend_from_slice(&desc_tag);
        tag_table.push((0x64657363, desc_offset, desc_size));

        let header_size = 128;
        let tag_table_size = 4 + tag_table.len() * 12;
        let tag_data_offset = header_size + tag_table_size;
        let total_size = tag_data_offset + tag_data.len();

        let mut profile = Vec::with_capacity(total_size);
        profile.resize(total_size, 0);

        profile[0..4].copy_from_slice(&(total_size as u32).to_be_bytes());
        profile[4..8].copy_from_slice(b"lcms");
        profile[8..12].copy_from_slice(&0x04200000u32.to_be_bytes());
        profile[12..16].copy_from_slice(b"mntr");
        profile[16..20].copy_from_slice(b"RGB ");
        profile[20..24].copy_from_slice(b"XYZ ");
        profile[24..36].fill(0);
        profile[36..40].copy_from_slice(b"acsp");
        profile[40..44].fill(0);
        profile[44..48].fill(0);
        profile[48..52].fill(0);
        profile[52..56].fill(0);
        profile[56..64].fill(0);
        profile[64..68].copy_from_slice(&0u32.to_be_bytes());
        profile[68..72].copy_from_slice(&s15f16(0.9642).to_be_bytes());
        profile[72..76].copy_from_slice(&s15f16(1.0).to_be_bytes());
        profile[76..80].copy_from_slice(&s15f16(0.8249).to_be_bytes());
        profile[80..84].fill(0);
        profile[84..128].fill(0);

        let table_start = 128;
        profile[table_start..table_start + 4]
            .copy_from_slice(&(tag_table.len() as u32).to_be_bytes());

        for (i, (sig, data_off, size)) in tag_table.iter().enumerate() {
            let entry_offset = table_start + 4 + i * 12;
            let actual_offset = (tag_data_offset + data_off) as u32;
            profile[entry_offset..entry_offset + 4].copy_from_slice(&sig.to_be_bytes());
            profile[entry_offset + 4..entry_offset + 8]
                .copy_from_slice(&actual_offset.to_be_bytes());
            profile[entry_offset + 8..entry_offset + 12]
                .copy_from_slice(&(*size as u32).to_be_bytes());
        }

        profile[tag_data_offset..tag_data_offset + tag_data.len()].copy_from_slice(&tag_data);

        profile
    }

    pub fn color_space_to_icc_profile(cs: &ColorSpace) -> Option<Vec<u8>> {
        if matches!((&cs.primaries, &cs.transfer), (ColorPrimaries::CIEXYZ, _)) {
            return None;
        }

        let (rxyz, gxyz, bxyz, wtpt) = compute_color_space_matrix(&cs.primaries, &cs.white_point);
        Some(build_icc_v2_profile(
            &rxyz,
            &gxyz,
            &bxyz,
            &wtpt,
            &cs.transfer,
            &cs.primaries,
        ))
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
        use lcms2_profile_builder::color_space_to_icc_profile;

        let src_icc =
            color_space_to_icc_profile(_source_space).ok_or_else(|| PluginError::Internal {
                plugin: "colorspace".into(),
                message: format!(
                    "unsupported source color space: {:?} + {:?}",
                    _source_space.primaries, _source_space.transfer
                ),
            })?;

        let dst_icc =
            color_space_to_icc_profile(_target_space).ok_or_else(|| PluginError::Internal {
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
            (PixelFormat::F32, _) => (TYPE_RGB_FLT, 3),
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

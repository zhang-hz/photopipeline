use async_trait::async_trait;
use std::fmt;
use std::sync::LazyLock;

#[cfg(feature = "libheif-native")]
use libheif_rs;

use photopipeline_core::{
    DecodeOptions, DecodedImage, EncodeOptions, FormatProbe, HardwareRequirement, ImageFormat,
    Metadata, PixelBuffer, PluginCategory, PluginError, PluginId, PluginResult, PluginVersion,
    ValidationIssue,
};
use photopipeline_plugin::{
    EnumOption, FormatProcessor, GuiLayout, GuiSchema, GuiSection, ParameterField, ParameterSchema,
    ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode, SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "quality".into(),
            label: "Quality".into(),
            description: Some("HEIF encoding quality settings".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "quality".into(),
                    label: "Quality".into(),
                    description: Some("Encoding quality (0-100)".into()),
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
                    default: serde_json::json!(95.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "lossless".into(),
                    label: "Lossless".into(),
                    description: Some("Use lossless compression".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Lossless".into()),
                        label_false: Some("Lossy".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "bit_depth".into(),
                    label: "Bit Depth".into(),
                    description: Some("Output bit depth (8 or 10 bit)".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "8".into(),
                                label: "8-bit".into(),
                                description: Some("Standard 8-bit output".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "10".into(),
                                label: "10-bit".into(),
                                description: Some("High bit depth 10-bit output".into()),
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: true,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("10"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
            ],
        },
        ParameterSection {
            id: "advanced".into(),
            label: "Advanced".into(),
            description: Some("Advanced HEIF encoder options".into()),
            icon: None,
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "chroma_subsampling".into(),
                    label: "Chroma Subsampling".into(),
                    description: Some("Chroma subsampling mode".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "444".into(),
                                label: "4:4:4".into(),
                                description: Some("No subsampling, best quality".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "422".into(),
                                label: "4:2:2".into(),
                                description: Some("Horizontal subsampling".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "420".into(),
                                label: "4:2:0".into(),
                                description: Some("2x subsampling, best compression".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("444"),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
                },
                ParameterField {
                    id: "encoder_effort".into(),
                    label: "Effort".into(),
                    description: Some("Encoder effort (0 = fast, 10 = best compression)".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 10,
                        step: 1,
                        unit: None,
                        style: Default::default(),
                    },
                    default: serde_json::json!(4),
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
                param_section_id: "quality".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "advanced".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("image".into()),
    color: Some("#14b8a6".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

pub struct HeifEncoderPlugin {
    id: String,
    lib_version: LazyLock<String>,
    #[cfg(feature = "libheif-native")]
    lib_heif: libheif_rs::LibHeif,
}

impl fmt::Debug for HeifEncoderPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeifEncoderPlugin")
            .field("id", &self.id)
            .finish()
    }
}

impl HeifEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.heif_encoder".to_string(),
            lib_version: LazyLock::new(detect_heif_encoder),
            #[cfg(feature = "libheif-native")]
            lib_heif: libheif_rs::LibHeif::new(),
        }
    }

    pub fn library_version(&self) -> &str {
        &self.lib_version
    }
}

impl Default for HeifEncoderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for HeifEncoderPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "HEIF Encoder"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Format
    }
    fn description(&self) -> &str {
        "Encode images in HEIF/HEIC 10-bit format using libheif-rs"
    }
    fn tags(&self) -> &[String] {
        &TAGS
    }
    fn requires_pixel_access(&self) -> bool {
        false
    }
    fn produces_pixel_output(&self) -> bool {
        false
    }
    fn supported_hardware(&self) -> HardwareRequirement {
        HardwareRequirement {
            min_ram_mb: 512,
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
        let version = detect_heif_encoder();
        tracing::info!("HEIF encoder detected: {}", version);
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("heif_encoder plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let quality = params
            .get("quality")
            .and_then(|v| v.as_f64())
            .unwrap_or(95.0);
        if !(0.0..=100.0).contains(&quality) {
            issues.push(ValidationIssue::Error {
                param: "quality".into(),
                message: "Quality must be between 0 and 100".into(),
            });
        }

        if !issues.is_empty() {
            tracing::debug!(
                issue_count = issues.len(),
                "heif_encoder validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for HeifEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![
            ("heif", "image/heif"),
            ("heic", "image/heic"),
            ("hif", "image/heif"),
        ]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::HEIF
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension {
            let ext_low = ext.to_lowercase();
            if ext_low == "heif" || ext_low == "heic" || ext_low == "hif" {
                tracing::trace!(extension = %ext, "heif_encoder: can_decode matched extension");
                return true;
            }
        }
        if let Some(ref magic) = probe.magic_bytes
            && magic.len() >= 12
            && &magic[4..8] == b"ftyp"
            && (&magic[8..12] == b"heic" || &magic[8..12] == b"heix" || &magic[8..12] == b"mif1")
        {
            tracing::trace!("heif_encoder: can_decode matched magic bytes");
            return true;
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat(
            "HEIF decoding not supported by encoder plugin".into(),
        ))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::HEIF | ImageFormat::HEIC)
    }

    async fn encode(
        &self,
        image: &PixelBuffer,
        metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let _timer =
            photopipeline_core::PerfTimer::with_target("heif_encode", "plugin.heif_encoder");
        let quality = options.quality.unwrap_or(95.0);
        let lossless = options.lossless;

        tracing::info!(
            input_dims = format!("{}x{}", image.width, image.height),
            format = ?image.format,
            quality = quality,
            lossless = lossless,
            "heif_encoder: encoding {}x{} HEIF (q={}, lossless={})",
            image.width,
            image.height,
            quality,
            lossless,
        );

        if photopipeline_oiio::OiioContext::available() {
            let tmp_path = std::env::temp_dir()
                .join(format!("pp_oiio_out_{}.heic", std::process::id()));
            let tmp_str = tmp_path.to_string_lossy().to_string();
            if let Ok(()) = photopipeline_oiio::OiioContext::write_image(
                &tmp_str, image, metadata,
            ) {
                if let Ok(data) = std::fs::read(&tmp_str) {
                    let _ = std::fs::remove_file(&tmp_str);
                    return Ok(data);
                }
            }
        }

        let _ = metadata;

        let chroma_str_for_ffi = options
            .chroma_subsampling
            .as_ref()
            .map_or("444", |cs| match cs {
                photopipeline_core::ChromaSubsampling::Yuv444 => "444",
                photopipeline_core::ChromaSubsampling::Yuv422 => "422",
                photopipeline_core::ChromaSubsampling::Yuv420 => "420",
            });
        let effort_val = options.effort.unwrap_or(4);
        self.encode_via_libheif(
            image, quality, lossless, options.bit_depth,
            chroma_str_for_ffi, effort_val, &self.id,
        )
    }
}

impl HeifEncoderPlugin {
    fn internal(&self, e: impl std::fmt::Display) -> PluginError {
        PluginError::Internal {
            plugin: self.id.clone(),
            message: e.to_string(),
        }
    }

    fn internal_msg(&self, msg: &str) -> PluginError {
        PluginError::Internal {
            plugin: self.id.clone(),
            message: msg.into(),
        }
    }

    #[cfg(not(feature = "libheif-native"))]
    fn encode_via_libheif(
        &self,
        image: &PixelBuffer, quality: f32, lossless: bool,
        _bit_depth: u8, chroma_str: &str, effort: u8,
        plugin_id: &PluginId,
    ) -> PluginResult<Vec<u8>> {
        let _ = (image, quality, lossless, _bit_depth, chroma_str, effort, plugin_id);
        Err(PluginError::Internal {
            plugin: plugin_id.clone(),
            message: "libheif native not compiled (use heif-enc fallback)".into(),
        })
    }

    #[cfg(feature = "libheif-native")]
    fn encode_via_libheif(
        &self,
        image: &PixelBuffer, quality: f32, lossless: bool,
        _bit_depth: u8, chroma_str: &str, effort: u8,
        plugin_id: &PluginId,
    ) -> PluginResult<Vec<u8>> {
        use libheif_rs::{
            Channel, ColorSpace, CompressionFormat, EncoderParameterValue,
            EncoderQuality, HeifContext, Image, RgbChroma,
        };

        let w = image.width as u32;
        let h = image.height as u32;
        let bpc = image.format.bytes_per_channel();

        // interleaved RGB: RgbChroma::Rgb (8-bit) or HdrRgbLe (10-bit LE)
        let chroma = match bpc {
            1 => RgbChroma::Rgb,
            _ => RgbChroma::HdrRgbLe,
        };
        let bit_depth = match bpc { 1 => 8, _ => 10 };

        let mut img = Image::new(w, h, ColorSpace::Rgb(chroma))
            .map_err(|e| self.internal(e))?;
        img.create_plane(Channel::Interleaved, w, h, bit_depth)
            .map_err(|e| self.internal(e))?;

        // copy interleaved pixel data — single plane, no de-interleave
        {
            let mut planes = img.planes_mut();
            let plane = planes.interleaved.as_mut()
                .ok_or_else(|| self.internal_msg("no interleaved plane"))?;
            let src = &image.data.data;
            let src_row = w as usize * 3 * bpc as usize;
            for y in 0..h as usize {
                let dst_off = y * plane.stride;
                let src_off = y * src_row;
                plane.data[dst_off..dst_off + src_row]
                    .copy_from_slice(&src[src_off..src_off + src_row]);
            }
        }

        let mut ctx = HeifContext::new()
            .map_err(|e| self.internal(e))?;
        let mut encoder = self.lib_heif.encoder_for_format(CompressionFormat::Hevc)
            .map_err(|e| self.internal(e))?;

        if lossless {
            encoder.set_quality(EncoderQuality::LossLess)
                .map_err(|e| self.internal(e))?;
        } else {
            encoder.set_quality(EncoderQuality::Lossy(quality as u8))
                .map_err(|e| self.internal(e))?;
        }

        encoder.set_parameter_value("effort",
            EncoderParameterValue::String(effort.to_string()))
            .map_err(|e| self.internal(e))?;
        encoder.set_parameter_value("chroma",
            EncoderParameterValue::String(chroma_str.to_string()))
            .map_err(|e| self.internal(e))?;

        ctx.encode_image(&img, &mut encoder, None)
            .map_err(|e| self.internal(e))?;

        let bytes = ctx.write_to_bytes()
            .map_err(|e| self.internal(e))?;

        if bytes.is_empty() {
            return Err(PluginError::Internal {
                plugin: plugin_id.clone(),
                message: "encode produced no output".into(),
            });
        }
        Ok(bytes)
    }
}

fn detect_heif_encoder() -> String {
    #[cfg(feature = "libheif-native")]
    {
        let v = libheif_rs::LibHeif::new().version();
        format!("libheif-rs v{}.{}.{}", v[0], v[1], v[2])
    }
    #[cfg(not(feature = "libheif-native"))]
    {
        "libheif-native feature not enabled".to_string()
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(),
        "heif".into(),
        "heic".into(),
        "encode".into(),
        "10bit".into(),
        "hdr".into(),
        "output".into(),
    ]
});

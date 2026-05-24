use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    ChannelLayout, DecodeOptions, DecodedImage, EncodeOptions, FormatProbe, HardwareRequirement,
    ImageFormat, Metadata, PerfTimer, PixelBuffer, PixelFormat, PluginCategory, PluginError,
    PluginId, PluginResult, PluginVersion, ValidationIssue,
};
use photopipeline_plugin::{
    EnumOption, FormatProcessor, GuiLayout, GuiSchema, GuiSection, ParameterField, ParameterSchema,
    ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode, SectionStyle,
};
use ravif::{Encoder as AvifEncoder, Img};
use rgb::FromSlice;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "quality".into(),
            label: "Quality".into(),
            description: Some("AVIF encoding quality settings".into()),
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
                    default: serde_json::json!(85.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "speed".into(),
                    label: "Speed".into(),
                    description: Some("Encoder speed preset (0=slow/best, 10=fast)".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 10,
                        step: 1,
                        unit: None,
                        style: Default::default(),
                    },
                    default: serde_json::json!(6),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "format".into(),
            label: "Format Options".into(),
            description: Some("AVIF format-specific settings".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "bit_depth".into(),
                    label: "Bit Depth".into(),
                    description: Some("Output bit depth".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "8".into(),
                                label: "8-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "10".into(),
                                label: "10-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "12".into(),
                                label: "12-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("10"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "chroma_subsampling".into(),
                    label: "Chroma".into(),
                    description: Some("Chroma subsampling mode".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "444".into(),
                                label: "4:4:4".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "422".into(),
                                label: "4:2:2".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "420".into(),
                                label: "4:2:0".into(),
                                description: None,
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
                },
            ],
        },
        ParameterSection {
            id: "advanced".into(),
            label: "Advanced".into(),
            description: Some("Advanced AVIF options".into()),
            icon: None,
            collapsible: true,
            default_collapsed: true,
            fields: vec![ParameterField {
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
                param_section_id: "quality".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "format".into(),
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
    color: Some("#22c55e".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug)]
pub struct AvifEncoderPlugin {
    id: String,
}

impl Default for AvifEncoderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AvifEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.avif_encoder".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for AvifEncoderPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "AVIF Encoder"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Format
    }
    fn description(&self) -> &str {
        "Encode images in AVIF format using ravif (pure-Rust AV1)"
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
        tracing::info!("avif_encoder plugin initialized (ravif)");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("avif_encoder plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("avif_encoder: validating parameters");
        if let Some(q) = params.get("quality").and_then(|v| v.as_f64())
            && (!(0.0..=100.0).contains(&q))
        {
            issues.push(ValidationIssue::Error {
                param: "quality".into(),
                message: "Quality must be between 0 and 100".into(),
            });
        }
        if let Some(s) = params.get_i64("speed")
            && (!(0..=10).contains(&s))
        {
            issues.push(ValidationIssue::Error {
                param: "speed".into(),
                message: "Speed must be between 0 and 10".into(),
            });
        }
        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "avif_encoder validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for AvifEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("avif", "image/avif")]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::AVIF
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension
            && ext.to_lowercase() == "avif"
        {
            tracing::trace!(extension = %ext, "avif_encoder: can_decode matched extension");
            return true;
        }
        if let Some(ref magic) = probe.magic_bytes
            && magic.len() >= 12
            && &magic[4..8] == b"ftyp"
            && &magic[8..12] == b"avif"
        {
            tracing::trace!("avif_encoder: can_decode matched magic bytes");
            return true;
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat(
            "AVIF decoding not supported yet".into(),
        ))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::AVIF)
    }

    async fn encode(
        &self,
        image: &PixelBuffer,
        _metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let _timer = PerfTimer::with_target("encode_avif", &format!("plugins.{}", self.id()));

        let quality = options.quality.unwrap_or(80.0);
        let speed = options
            .effort
            .map(|e| (10 - e.min(9)).max(1) as u8)
            .unwrap_or(5);

        tracing::info!(
            format = ?image.format,
            layout = ?image.layout,
            width = image.width,
            height = image.height,
            quality = quality,
            speed = speed,
            "AVIF encoding {}x{} {:?}/{:?}",
            image.width, image.height, image.layout, image.format,
        );

        let result = match (image.layout, image.format) {
            (ChannelLayout::RGB, PixelFormat::U8) => {
                let pixels = image.data.data.as_rgb();
                let img = Img::new(pixels, image.width as usize, image.height as usize);
                AvifEncoder::new()
                    .with_quality(quality)
                    .with_speed(speed)
                    .encode_rgb(img)
            }
            (ChannelLayout::RGBA, PixelFormat::U8) => {
                let pixels = image.data.data.as_rgba();
                let img = Img::new(pixels, image.width as usize, image.height as usize);
                AvifEncoder::new()
                    .with_quality(quality)
                    .with_alpha_quality(quality)
                    .with_speed(speed)
                    .encode_rgba(img)
            }
            (ChannelLayout::RGB, PixelFormat::U16) => {
                tracing::warn!(
                    "ravif 0.11 only supports 8-bit; downconverting U16 → U8 for AVIF encoding"
                );
                let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                let data_u8: Vec<u8> = data_u16.iter().map(|&v| (v >> 8) as u8).collect();
                let pixels = data_u8.as_rgb();
                let img = Img::new(pixels, image.width as usize, image.height as usize);
                AvifEncoder::new()
                    .with_quality(quality)
                    .with_speed(speed)
                    .encode_rgb(img)
            }
            (ChannelLayout::RGBA, PixelFormat::U16) => {
                tracing::warn!(
                    "ravif 0.11 only supports 8-bit; downconverting U16 → U8 for AVIF encoding"
                );
                let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                let data_u8: Vec<u8> = data_u16.iter().map(|&v| (v >> 8) as u8).collect();
                let pixels = data_u8.as_rgba();
                let img = Img::new(pixels, image.width as usize, image.height as usize);
                AvifEncoder::new()
                    .with_quality(quality)
                    .with_alpha_quality(quality)
                    .with_speed(speed)
                    .encode_rgba(img)
            }
            _ => {
                return Err(PluginError::EncodingFailed(
                    "ravif only supports RGB or RGBA U8/U16 input".into(),
                ));
            }
        }
        .map_err(|e| PluginError::EncodingFailed(format!("ravif encode: {}", e)))?;

        tracing::info!(
            target = format!("plugins.{}", self.id()),
            width = image.width,
            height = image.height,
            output_bytes = result.avif_file.len(),
            quality = quality,
            "AVIF encoded via ravif"
        );

        Ok(result.avif_file)
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(),
        "avif".into(),
        "av1".into(),
        "encode".into(),
        "10bit".into(),
        "hdr".into(),
        "output".into(),
    ]
});

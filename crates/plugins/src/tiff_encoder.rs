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
use tiff::encoder::compression::{Deflate, Lzw, Packbits, Uncompressed};
use tiff::encoder::{TiffEncoder, colortype};
use tiff::tags::Tag;

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "encoding".into(),
            label: "Encoding".into(),
            description: Some("TIFF encoding options".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "compression".into(),
                    label: "Compression".into(),
                    description: Some("TIFF compression algorithm".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "none".into(),
                                label: "None".into(),
                                description: Some("Uncompressed data".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "lzw".into(),
                                label: "LZW".into(),
                                description: Some("LZW lossless compression".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "deflate".into(),
                                label: "Deflate".into(),
                                description: Some("ZIP/Deflate compression".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "packbits".into(),
                                label: "PackBits".into(),
                                description: Some("Simple RLE compression".into()),
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
                    id: "bigtiff".into(),
                    label: "BigTIFF".into(),
                    description: Some("Use BigTIFF format for files larger than 4GB".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("BigTIFF".into()),
                        label_false: Some("Classic TIFF".into()),
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
            id: "metadata".into(),
            label: "Metadata".into(),
            description: Some("TIFF metadata embedding".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "embed_icc".into(),
                    label: "Embed ICC Profile".into(),
                    description: Some("Embed color profile in TIFF tags".into()),
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
                    id: "pixel_format".into(),
                    label: "Pixel Format".into(),
                    description: Some("Output pixel format".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "u8".into(),
                                label: "8-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "u16".into(),
                                label: "16-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "f32".into(),
                                label: "32-bit float".into(),
                                description: None,
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("u16"),
                    required: false,
                    advanced: false,
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
                param_section_id: "encoding".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "metadata".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
        ],
    },
    icon: Some("file".into()),
    color: Some("#64748b".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug)]
pub struct TiffEncoderPlugin {
    id: String,
}

impl Default for TiffEncoderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TiffEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.tiff_encoder".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for TiffEncoderPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "TIFF Encoder"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Format
    }
    fn description(&self) -> &str {
        "Encode images as TIFF files with configurable compression"
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
        tracing::info!("tiff_encoder plugin initialized (builtin)");
        Ok(())
    }
    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("tiff_encoder plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        tracing::debug!("tiff_encoder: validating parameters");
        let mut issues = Vec::new();
        if let Some(bd) = params.get("bit_depth").and_then(|v| v.as_u64()) {
            if bd != 8 && bd != 16 {
                issues.push(ValidationIssue::Warning {
                    param: "bit_depth".into(),
                    message: format!("bit_depth {} not supported (8 or 16), will use 8", bd),
                });
            }
        }
        if let Some(comp) = params.get("compression").and_then(|v| v.as_str()) {
            let valid = matches!(
                comp.to_lowercase().as_str(),
                "none" | "lzw" | "deflate" | "packbits"
            );
            if !valid {
                issues.push(ValidationIssue::Warning {
                    param: "compression".into(),
                    message: format!("unknown compression '{}', using lzw", comp),
                });
            }
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for TiffEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("tiff", "image/tiff"), ("tif", "image/tiff")]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::TIFF
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension {
            let lower = ext.to_lowercase();
            if lower == "tiff" || lower == "tif" {
                tracing::trace!(extension = %ext, "tiff_encoder: can_decode matched extension");
                return true;
            }
        }
        if let Some(ref magic) = probe.magic_bytes {
            if magic.len() >= 4 && (&magic[0..4] == b"II\x2A\x00" || &magic[0..4] == b"MM\x00\x2A")
            {
                tracing::trace!("tiff_encoder: can_decode matched TIFF magic bytes");
                return true;
            }
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat(
            "TIFF decoding not supported by encoder plugin".into(),
        ))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::TIFF)
    }

    async fn encode(
        &self,
        image: &PixelBuffer,
        metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let _timer = PerfTimer::with_target("encode_tiff", &format!("plugins.{}", self.id()));

        let compression = options.compression.as_deref().unwrap_or("deflate");
        let embed_icc = options.embed_profile.unwrap_or(true);

        tracing::info!(
            format = ?image.format,
            layout = ?image.layout,
            width = image.width,
            height = image.height,
            compression = compression,
            embed_icc = embed_icc,
            "TIFF encoding {}x{} {:?}/{:?}",
            image.width, image.height, image.layout, image.format,
        );

        let mut output_buf = Vec::new();
        let use_bigtiff = image.width as u64
            * image.height as u64
            * image.format.bytes_per_channel() as u64
            * image.layout.channel_count() as u64
            > 4_000_000_000;

        if use_bigtiff {
            tracing::info!("TIFF output exceeds 4GB, using BigTIFF format");
        }

        macro_rules! encode_tiff {
            ($color:ty, $comp:ty, $comp_expr:expr, $data:expr) => {{
                let writer = std::io::Cursor::new(&mut output_buf);
                if use_bigtiff {
                    let mut tiff = TiffEncoder::new_big(writer)
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                    let mut img = tiff
                        .new_image_with_compression::<$color, $comp>(
                            image.width as u32,
                            image.height as u32,
                            $comp_expr,
                        )
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                    if embed_icc {
                        if let Some(ref icc) = image.icc_profile {
                            let _ = img.encoder().write_tag(Tag::Unknown(34675), icc.as_slice());
                        }
                    }
                    img.write_data($data)
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                } else {
                    let mut tiff = TiffEncoder::new(writer)
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                    let mut img = tiff
                        .new_image_with_compression::<$color, $comp>(
                            image.width as u32,
                            image.height as u32,
                            $comp_expr,
                        )
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                    if embed_icc {
                        if let Some(ref icc) = image.icc_profile {
                            let _ = img.encoder().write_tag(Tag::Unknown(34675), icc.as_slice());
                        }
                    }
                    img.write_data($data)
                        .map_err(|e| PluginError::EncodingFailed(e.to_string()))?;
                }
            }};
        }

        match compression {
            "deflate" | "zip" => match (image.layout, image.format) {
                (ChannelLayout::RGB, PixelFormat::U8) => encode_tiff!(
                    colortype::RGB8,
                    Deflate,
                    Deflate::default(),
                    &image.data.data
                ),
                (ChannelLayout::RGB, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::RGB16, Deflate, Deflate::default(), data_u16)
                }
                (ChannelLayout::RGB, PixelFormat::F32) => {
                    let data_f32 = image.data.as_f32_slice();
                    encode_tiff!(colortype::RGB32Float, Deflate, Deflate::default(), data_f32)
                }
                (ChannelLayout::RGBA, PixelFormat::U8) => encode_tiff!(
                    colortype::RGBA8,
                    Deflate,
                    Deflate::default(),
                    &image.data.data
                ),
                (ChannelLayout::Gray, PixelFormat::U8) => encode_tiff!(
                    colortype::Gray8,
                    Deflate,
                    Deflate::default(),
                    &image.data.data
                ),
                (ChannelLayout::Gray, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::Gray16, Deflate, Deflate::default(), data_u16)
                }
                _ => {
                    return Err(PluginError::EncodingFailed(format!(
                        "unsupported pixel format {:?}/{:?} for TIFF",
                        image.layout, image.format
                    )));
                }
            },
            "lzw" => match (image.layout, image.format) {
                (ChannelLayout::RGB, PixelFormat::U8) => {
                    encode_tiff!(colortype::RGB8, Lzw, Lzw, &image.data.data)
                }
                (ChannelLayout::RGB, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::RGB16, Lzw, Lzw, data_u16)
                }
                (ChannelLayout::RGB, PixelFormat::F32) => {
                    let data_f32 = image.data.as_f32_slice();
                    encode_tiff!(colortype::RGB32Float, Lzw, Lzw, data_f32)
                }
                (ChannelLayout::RGBA, PixelFormat::U8) => {
                    encode_tiff!(colortype::RGBA8, Lzw, Lzw, &image.data.data)
                }
                (ChannelLayout::Gray, PixelFormat::U8) => {
                    encode_tiff!(colortype::Gray8, Lzw, Lzw, &image.data.data)
                }
                (ChannelLayout::Gray, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::Gray16, Lzw, Lzw, data_u16)
                }
                _ => {
                    return Err(PluginError::EncodingFailed(format!(
                        "unsupported pixel format {:?}/{:?} for TIFF",
                        image.layout, image.format
                    )));
                }
            },
            "packbits" => match (image.layout, image.format) {
                (ChannelLayout::RGB, PixelFormat::U8) => {
                    encode_tiff!(colortype::RGB8, Packbits, Packbits, &image.data.data)
                }
                (ChannelLayout::RGB, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::RGB16, Packbits, Packbits, data_u16)
                }
                (ChannelLayout::RGB, PixelFormat::F32) => {
                    let data_f32 = image.data.as_f32_slice();
                    encode_tiff!(colortype::RGB32Float, Packbits, Packbits, data_f32)
                }
                (ChannelLayout::RGBA, PixelFormat::U8) => {
                    encode_tiff!(colortype::RGBA8, Packbits, Packbits, &image.data.data)
                }
                (ChannelLayout::Gray, PixelFormat::U8) => {
                    encode_tiff!(colortype::Gray8, Packbits, Packbits, &image.data.data)
                }
                (ChannelLayout::Gray, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::Gray16, Packbits, Packbits, data_u16)
                }
                _ => {
                    return Err(PluginError::EncodingFailed(format!(
                        "unsupported pixel format {:?}/{:?} for TIFF",
                        image.layout, image.format
                    )));
                }
            },
            _ => match (image.layout, image.format) {
                (ChannelLayout::RGB, PixelFormat::U8) => encode_tiff!(
                    colortype::RGB8,
                    Uncompressed,
                    Uncompressed,
                    &image.data.data
                ),
                (ChannelLayout::RGB, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::RGB16, Uncompressed, Uncompressed, data_u16)
                }
                (ChannelLayout::RGB, PixelFormat::F32) => {
                    let data_f32 = image.data.as_f32_slice();
                    encode_tiff!(colortype::RGB32Float, Uncompressed, Uncompressed, data_f32)
                }
                (ChannelLayout::RGBA, PixelFormat::U8) => encode_tiff!(
                    colortype::RGBA8,
                    Uncompressed,
                    Uncompressed,
                    &image.data.data
                ),
                (ChannelLayout::Gray, PixelFormat::U8) => encode_tiff!(
                    colortype::Gray8,
                    Uncompressed,
                    Uncompressed,
                    &image.data.data
                ),
                (ChannelLayout::Gray, PixelFormat::U16) => {
                    let data_u16: &[u16] = bytemuck::cast_slice(&image.data.data);
                    encode_tiff!(colortype::Gray16, Uncompressed, Uncompressed, data_u16)
                }
                _ => {
                    return Err(PluginError::EncodingFailed(format!(
                        "unsupported pixel format {:?}/{:?} for TIFF",
                        image.layout, image.format
                    )));
                }
            },
        }

        tracing::info!(
            width = image.width,
            height = image.height,
            output_bytes = output_buf.len(),
            bigtiff = use_bigtiff,
            "TIFF encoded ({} bytes)",
            output_buf.len(),
        );

        let _ = metadata;
        Ok(output_buf)
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(),
        "tiff".into(),
        "encode".into(),
        "output".into(),
        "lossless".into(),
        "16bit".into(),
    ]
});

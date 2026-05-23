use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    ChannelLayout, DecodeOptions, DecodedImage, EncodeOptions, FormatProbe, HardwareRequirement,
    ImageFormat, Metadata, PixelBuffer, PixelFormat as CorePixelFormat, PluginCategory,
    PluginError, PluginId, PluginResult, PluginVersion, ValidationIssue,
};
use photopipeline_plugin::{
    EnumOption, FormatProcessor, GuiLayout, GuiSchema, GuiSection, ParameterField, ParameterSchema,
    ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode, SectionStyle,
};

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
        Ok(())
    }
    async fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }

    async fn validate(&self, _params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        Ok(vec![])
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
            let e = ext.to_lowercase();
            if e == "tiff" || e == "tif" {
                return true;
            }
        }
        if let Some(ref magic) = probe.magic_bytes
            && magic.len() >= 4
            && (&magic[0..4] == b"II\x2A\x00" || &magic[0..4] == b"MM\x00\x2A")
        {
            return true;
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
        _options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let width = image.width;
        let height = image.height;
        let channels = match image.layout {
            ChannelLayout::RGB => 3u16,
            ChannelLayout::RGBA => 4u16,
            ChannelLayout::Gray => 1u16,
            ChannelLayout::GrayAlpha => 2u16,
            _ => 3u16,
        };

        let bps = match image.format {
            CorePixelFormat::U8 => 8u16,
            CorePixelFormat::U16 => 16u16,
            CorePixelFormat::F16 => 16u16,
            CorePixelFormat::U32 => 32u16,
            CorePixelFormat::F32 => 32u16,
        };

        let bytes_per_sample = bps as u32 / 8;
        let samples_per_pixel = channels;

        let has_extra_ifd = metadata.exif.is_some();
        let extra_tags = if has_extra_ifd { 4u16 } else { 0u16 };

        let _rows_per_strip = 1u32;
        let strip_count = height;
        let strip_byte_count = width * samples_per_pixel as u32 * bytes_per_sample;

        let ifd_entry_count: u16 = 12 + extra_tags;
        let ifd_offset: u32 = 8;
        let ifd_size: u32 = 2 + ifd_entry_count as u32 * 12 + 4;
        let strip_offsets_start: u32 = ifd_offset + ifd_size;
        let exif_ifd_offset: u32 = strip_offsets_start + strip_count * 4;
        let strip_data_start: u32 = if has_extra_ifd {
            exif_ifd_offset + 4
        } else {
            strip_offsets_start + strip_count * 4
        };

        let total_size = strip_data_start + (strip_byte_count * strip_count);
        let mut buf = Vec::with_capacity(total_size as usize);

        buf.extend_from_slice(&[0x49, 0x49]);
        buf.extend_from_slice(&42u16.to_le_bytes());
        buf.extend_from_slice(&ifd_offset.to_le_bytes());

        buf.extend_from_slice(&ifd_entry_count.to_le_bytes());

        write_ifd_short(&mut buf, 256, width as u16);
        write_ifd_short(&mut buf, 257, height as u16);
        write_ifd_long(&mut buf, 258, &[bps as u32]);
        write_ifd_short(&mut buf, 259, 1);
        write_ifd_short(&mut buf, 262, 2);
        write_ifd_long(&mut buf, 273, &[0u32]);
        write_ifd_short(&mut buf, 277, samples_per_pixel);
        write_ifd_long(&mut buf, 278, &[height]);
        write_ifd_long(&mut buf, 279, &[strip_byte_count]);
        write_ifd_long(&mut buf, 282, &[300, 1]);
        write_ifd_long(&mut buf, 283, &[300, 1]);
        write_ifd_short(&mut buf, 296, 2);

        if has_extra_ifd {
            write_ifd_long(&mut buf, 34665, &[exif_ifd_offset]);
        }

        buf.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);

        for i in 0..strip_count {
            let offset = strip_data_start + i * strip_byte_count;
            buf.extend_from_slice(&offset.to_le_bytes());
        }

        if has_extra_ifd {
            write_exif_ifd(&mut buf, metadata, exif_ifd_offset, strip_data_start);
        }

        let pixel_data = &image.data.data;
        let data_size = width as usize
            * height as usize
            * samples_per_pixel as usize
            * bytes_per_sample as usize;
        let copy_len = data_size.min(pixel_data.len());
        buf.resize(strip_data_start as usize + copy_len, 0);
        buf[strip_data_start as usize..strip_data_start as usize + copy_len]
            .copy_from_slice(&pixel_data[..copy_len]);

        if buf.len() < strip_data_start as usize + data_size {
            buf.resize(strip_data_start as usize + data_size, 0);
        }

        Ok(buf)
    }
}

fn write_ifd_short(buf: &mut Vec<u8>, tag: u16, value: u16) {
    buf.extend_from_slice(&tag.to_le_bytes());
    buf.extend_from_slice(&3u16.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&value.to_le_bytes());
    buf.extend_from_slice(&[0, 0]);
}

fn write_ifd_long(buf: &mut Vec<u8>, tag: u16, values: &[u32]) {
    let count = values.len() as u32;
    buf.extend_from_slice(&tag.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&count.to_le_bytes());
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    if bytes.len() <= 4 {
        buf.extend_from_slice(&bytes);
        let padding = 4 - bytes.len();
        buf.resize(buf.len() + padding, 0);
    } else {
        buf.extend_from_slice(&[0; 4]);
    }
}

fn write_exif_ifd(
    buf: &mut Vec<u8>,
    metadata: &Metadata,
    exif_offset: u32,
    _strip_data_start: u32,
) {
    let pos = exif_offset as usize;
    if buf.len() <= pos + 6 {
        buf.resize(pos + 6, 0);
    }
    if let Some(ref _exif) = metadata.exif {
        buf[pos] = 1;
        buf[pos + 1] = 0;
        write_ifd_long_at(buf, pos + 2, 0x8827, &[0]);
        buf[pos + 14] = 0;
        buf[pos + 15] = 0;
        buf[pos + 16] = 0;
        buf[pos + 17] = 0;
    } else {
        buf[pos] = 0;
        buf[pos + 1] = 0;
        buf[pos + 2] = 0;
        buf[pos + 3] = 0;
        buf[pos + 4] = 0;
        buf[pos + 5] = 0;
    }
}

fn write_ifd_long_at(buf: &mut Vec<u8>, offset: usize, tag: u16, values: &[u32]) {
    let count = values.len() as u32;
    let pos = offset;
    buf[pos..pos + 2].copy_from_slice(&tag.to_le_bytes());
    buf[pos + 2..pos + 4].copy_from_slice(&4u16.to_le_bytes());
    buf[pos + 4..pos + 8].copy_from_slice(&count.to_le_bytes());
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let write_len = bytes.len().min(4);
    buf[pos + 8..pos + 8 + write_len].copy_from_slice(&bytes[..write_len]);
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

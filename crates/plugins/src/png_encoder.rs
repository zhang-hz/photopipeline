use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    AlignedBuffer, ChannelLayout, ColorSpace, DecodeOptions, DecodedImage, EncodeOptions,
    FormatProbe, HardwareRequirement, ImageFormat, Metadata, PerfTimer, PixelBuffer,
    PixelFormat as CorePixelFormat, PluginCategory, PluginError, PluginId, PluginResult,
    PluginVersion, ValidationIssue,
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
            description: Some("PNG encoding options".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "compression_level".into(),
                    label: "Compression Level".into(),
                    description: Some("Deflate compression level (0=store, 9=best)".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 9,
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
                                value: "16".into(),
                                label: "16-bit".into(),
                                description: Some("High precision 16-bit PNG".into()),
                                icon: None,
                                tags: vec!["hdr".into()],
                                recommended: true,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("16"),
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
            description: Some("PNG metadata chunks".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "embed_icc".into(),
                    label: "Embed ICC Profile".into(),
                    description: Some("Include iCCP chunk with color profile".into()),
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
                    id: "include_exif".into(),
                    label: "Include EXIF".into(),
                    description: Some("Include eXIf chunk with EXIF data".into()),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Include".into()),
                        label_false: Some("Skip".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "color_type".into(),
                    label: "Color Type".into(),
                    description: Some("PNG color type".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "rgb".into(),
                                label: "RGB (Truecolor)".into(),
                                description: Some("Type 2: RGB triple".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "rgba".into(),
                                label: "RGBA (Truecolor+Alpha)".into(),
                                description: Some("Type 6: RGBA quad".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "gray".into(),
                                label: "Grayscale".into(),
                                description: Some("Type 0: Single channel".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "graya".into(),
                                label: "Grayscale+Alpha".into(),
                                description: Some("Type 4: Two channels".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("rgb"),
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
    icon: Some("image".into()),
    color: Some("#0ea5e9".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug)]
pub struct PngEncoderPlugin {
    id: String,
}

impl Default for PngEncoderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PngEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.png_encoder".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for PngEncoderPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "PNG Encoder"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Format
    }
    fn description(&self) -> &str {
        "Encode images as PNG files with 16-bit and ICC profile support"
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
            min_ram_mb: 128,
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
        tracing::info!("png_encoder plugin initialized (builtin)");
        Ok(())
    }
    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("png_encoder plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        tracing::debug!("png_encoder: validating parameters (always ok)");
        let _ = params;
        Ok(vec![])
    }
}

#[async_trait]
impl FormatProcessor for PngEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("png", "image/png")]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::PNG
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension
            && ext.to_lowercase() == "png"
        {
            tracing::trace!(extension = %ext, "png_encoder: can_decode matched extension");
            return true;
        }
        if let Some(ref magic) = probe.magic_bytes
            && magic.len() >= 8
            && &magic[0..8] == b"\x89PNG\r\n\x1a\n"
        {
            tracing::trace!("png_encoder: can_decode matched PNG signature");
            return true;
        }
        false
    }

    async fn decode(&self, data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        let _timer = PerfTimer::with_target("png_decode", "plugin.png_encoder");
        tracing::info!(
            data_len = data.len(),
            "png_encoder: decoding {} bytes of PNG data",
            data.len(),
        );
        decode_png(data)
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::PNG)
    }

    async fn encode(
        &self,
        image: &PixelBuffer,
        metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let _timer = PerfTimer::with_target("png_encode", "plugin.png_encoder");

        tracing::info!(
            input_dims = format!("{}x{}", image.width, image.height),
            format = ?image.format,
            "png_encoder: encoding {}x{} PNG",
            image.width,
            image.height,
        );
        if photopipeline_oiio::OiioContext::available() {
            let tmp_out =
                std::env::temp_dir().join(format!("pp_oiio_out_{}.png", std::process::id()));
            if let Ok(()) = photopipeline_oiio::OiioContext::write_image(
                &tmp_out.to_string_lossy(),
                image,
                metadata,
            ) {
                if let Ok(data) = std::fs::read(&tmp_out) {
                    let _ = std::fs::remove_file(&tmp_out);
                    return Ok(data);
                }
                let _ = std::fs::remove_file(&tmp_out);
            }
        }

        let _ = options;
        let width = image.width;
        let height = image.height;
        let (color_type, channels) = match image.layout {
            ChannelLayout::RGB => (2u8, 3u8),
            ChannelLayout::RGBA => (6u8, 4u8),
            ChannelLayout::Gray => (0u8, 1u8),
            ChannelLayout::GrayAlpha => (4u8, 2u8),
            _ => (2u8, 3u8),
        };

        let bit_depth = match image.format {
            CorePixelFormat::U8 => 8u8,
            CorePixelFormat::U16 | CorePixelFormat::F16 => 16u8,
            _ => 8u8,
        };

        let bytes_per_sample = bit_depth as usize / 8;
        let row_bytes_raw = width as usize * channels as usize * bytes_per_sample;
        let row_bytes_filtered = row_bytes_raw + 1;

        let mut buf: Vec<u8> = Vec::new();

        buf.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        let mut ihdr_data = Vec::new();
        ihdr_data.extend_from_slice(&width.to_be_bytes());
        ihdr_data.extend_from_slice(&height.to_be_bytes());
        ihdr_data.push(bit_depth);
        ihdr_data.push(color_type);
        ihdr_data.extend_from_slice(&[0, 0, 0]);
        write_png_chunk(&mut buf, b"IHDR", &ihdr_data);

        if let Some(ref exif) = metadata.exif {
            let mut exif_text = String::new();
            if let Some(ref make) = exif.make {
                exif_text.push_str(&format!("Make: {}\n", make));
            }
            if let Some(ref model) = exif.model {
                exif_text.push_str(&format!("Model: {}\n", model));
            }
            if let Some(iso) = exif.iso {
                exif_text.push_str(&format!("ISO: {}\n", iso));
            }
            if let Some(ref exp) = exif.exposure_time {
                exif_text.push_str(&format!("ExposureTime: {}\n", exp));
            }
            if let Some(ref fnum) = exif.f_number {
                exif_text.push_str(&format!("FNumber: {}\n", fnum));
            }
            if let Some(ref fl) = exif.focal_length {
                exif_text.push_str(&format!("FocalLength: {}\n", fl));
            }
            if !exif_text.is_empty() {
                let mut kw = b"EXIF".to_vec();
                kw.push(0);
                kw.extend_from_slice(exif_text.as_bytes());
                write_png_chunk(&mut buf, b"tEXt", &kw);
            }
        }

        let pixel_data = &image.data.data;
        let mut raw_filtered = Vec::with_capacity(row_bytes_filtered * height as usize);
        for y in 0..height as usize {
            raw_filtered.push(0u8);
            let row_start = y * row_bytes_raw;
            let copy_len = row_bytes_raw.min(pixel_data.len().saturating_sub(row_start));
            if copy_len > 0 {
                raw_filtered.extend_from_slice(&pixel_data[row_start..row_start + copy_len]);
                for _ in copy_len..row_bytes_raw {
                    raw_filtered.push(0);
                }
            } else {
                raw_filtered.resize(raw_filtered.len() + row_bytes_raw, 0);
            }
        }

        let mut zlib_data = Vec::new();
        zlib_data.push(0x78);
        zlib_data.push(0x01);

        let mut pos: usize = 0;
        while pos < raw_filtered.len() {
            let remaining = raw_filtered.len() - pos;
            let chunk_size = remaining.min(65535);
            let is_final = pos + chunk_size >= raw_filtered.len();

            if is_final {
                zlib_data.push(0x01);
            } else {
                zlib_data.push(0x00);
            }

            let len_le = (chunk_size as u16).to_le_bytes();
            let nlen_le = (!(chunk_size as u16)).to_le_bytes();
            zlib_data.extend_from_slice(&len_le);
            zlib_data.extend_from_slice(&nlen_le);
            zlib_data.extend_from_slice(&raw_filtered[pos..pos + chunk_size]);
            pos += chunk_size;
        }

        let adler = adler32(&raw_filtered);
        zlib_data.extend_from_slice(&adler.to_be_bytes());

        write_png_chunk(&mut buf, b"IDAT", &zlib_data);

        if image.icc_profile.is_some() {
            let mut iccp_data = Vec::new();
            iccp_data.extend_from_slice(b"ICC Profile\0\0");
            if let Some(ref prof) = image.icc_profile {
                iccp_data.extend_from_slice(prof);
            }
            write_png_chunk(&mut buf, b"iCCP", &iccp_data);
        }

        write_png_chunk(&mut buf, b"IEND", &[]);

        let output_size = buf.len();
        tracing::info!(
            output_bytes = output_size,
            "png_encoder: encoded {} bytes",
            output_size,
        );

        Ok(buf)
    }
}

fn write_png_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let crc = crc32(&crc_input);
    buf.extend_from_slice(chunk_type);
    buf.extend_from_slice(data);
    buf.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    let table = crc32_table();
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = table[idx] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256 {
        let mut c = i as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        table[i] = c;
    }
    table
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn decode_png(data: &[u8]) -> PluginResult<DecodedImage> {
    let sig: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data.len() < 8 || data[..8] != sig {
        return Err(PluginError::DecodingFailed("not a PNG file".into()));
    }

    let mut pos: usize = 8;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut bit_depth: u8 = 0;
    let mut color_type: u8 = 0;
    let mut idat_data: Vec<u8> = Vec::new();

    while pos + 12 <= data.len() {
        let len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        pos += 8;
        if pos + len + 4 > data.len() {
            return Err(PluginError::DecodingFailed("truncated PNG chunk".into()));
        }
        let chunk_data = &data[pos..pos + len];

        match chunk_type {
            b"IHDR" if len >= 13 => {
                width = u32::from_be_bytes([
                    chunk_data[0],
                    chunk_data[1],
                    chunk_data[2],
                    chunk_data[3],
                ]);
                height = u32::from_be_bytes([
                    chunk_data[4],
                    chunk_data[5],
                    chunk_data[6],
                    chunk_data[7],
                ]);
                bit_depth = chunk_data[8];
                color_type = chunk_data[9];
            }
            b"IDAT" => {
                idat_data.extend_from_slice(chunk_data);
            }
            b"IEND" => break,
            _ => {}
        }

        pos += len + 4;
    }

    if width == 0 || height == 0 {
        return Err(PluginError::DecodingFailed(
            "missing or invalid IHDR".into(),
        ));
    }
    if idat_data.is_empty() {
        return Err(PluginError::DecodingFailed("no IDAT data found".into()));
    }
    if !matches!(color_type, 2 | 6 | 0 | 4) {
        return Err(PluginError::DecodingFailed(format!(
            "unsupported color type {}",
            color_type
        )));
    }
    if !matches!(bit_depth, 8 | 16) {
        return Err(PluginError::DecodingFailed(format!(
            "unsupported bit depth {}",
            bit_depth
        )));
    }

    let channels: usize = match color_type {
        2 => 3,
        6 => 4,
        0 => 1,
        4 => 2,
        _ => unreachable!(),
    };

    let raw_filtered = inflate_zlib(&idat_data).map_err(|e| PluginError::DecodingFailed(e))?;

    let bytes_per_sample = bit_depth as usize / 8;
    let row_bytes_raw = width as usize * channels * bytes_per_sample;
    let expected_raw = row_bytes_raw * height as usize + height as usize;
    if raw_filtered.len() < expected_raw {
        return Err(PluginError::DecodingFailed(
            "decompressed data too short".into(),
        ));
    }

    let mut pixel_data = vec![0u8; row_bytes_raw * height as usize];
    for y in 0..height as usize {
        let row_start = y * (row_bytes_raw + 1);
        let filter = raw_filtered[row_start];
        let filtered = &raw_filtered[row_start + 1..row_start + 1 + row_bytes_raw];
        let out_start = y * row_bytes_raw;
        match filter {
            0 => {
                pixel_data[out_start..out_start + row_bytes_raw].copy_from_slice(filtered);
            }
            1 => {
                for x in 0..row_bytes_raw {
                    let left = if x >= bytes_per_sample {
                        pixel_data[out_start + x - bytes_per_sample] as u32
                    } else {
                        0
                    };
                    pixel_data[out_start + x] = (filtered[x] as u32 + left) as u8 & 0xFF;
                }
            }
            2 => {
                let prev = if y > 0 { out_start - row_bytes_raw } else { 0 };
                if y > 0 {
                    for x in 0..row_bytes_raw {
                        let above = pixel_data[prev + x] as u32;
                        pixel_data[out_start + x] = (filtered[x] as u32 + above) as u8 & 0xFF;
                    }
                } else {
                    pixel_data[out_start..out_start + row_bytes_raw].copy_from_slice(filtered);
                }
            }
            3 => {
                let prev = if y > 0 { out_start - row_bytes_raw } else { 0 };
                for x in 0..row_bytes_raw {
                    let left = if x >= bytes_per_sample {
                        pixel_data[out_start + x - bytes_per_sample] as u32
                    } else {
                        0
                    };
                    let above = if y > 0 {
                        pixel_data[prev + x] as u32
                    } else {
                        0
                    };
                    pixel_data[out_start + x] =
                        (filtered[x] as u32 + ((left + above) / 2)) as u8 & 0xFF;
                }
            }
            4 => {
                let prev = if y > 0 { out_start - row_bytes_raw } else { 0 };
                for x in 0..row_bytes_raw {
                    let left = if x >= bytes_per_sample {
                        pixel_data[out_start + x - bytes_per_sample] as i32
                    } else {
                        0
                    };
                    let above = if y > 0 {
                        pixel_data[prev + x] as i32
                    } else {
                        0
                    };
                    let above_left = if y > 0 && x >= bytes_per_sample {
                        pixel_data[prev + x - bytes_per_sample] as i32
                    } else {
                        0
                    };
                    let p = left + above - above_left;
                    let pa = (p - left).abs();
                    let pb = (p - above).abs();
                    let pc = (p - above_left).abs();
                    let pr = if pa <= pb && pa <= pc {
                        left
                    } else if pb <= pc {
                        above
                    } else {
                        above_left
                    };
                    pixel_data[out_start + x] = (filtered[x] as i32 + pr) as u8 & 0xFF;
                }
            }
            _ => {
                return Err(PluginError::DecodingFailed(format!(
                    "unknown filter type {}",
                    filter
                )));
            }
        }
    }

    let layout = match color_type {
        2 => ChannelLayout::RGB,
        6 => ChannelLayout::RGBA,
        0 => ChannelLayout::Gray,
        4 => ChannelLayout::GrayAlpha,
        _ => ChannelLayout::RGB,
    };
    let format = match bit_depth {
        16 => CorePixelFormat::U16,
        _ => CorePixelFormat::U8,
    };
    let actual_row_bytes = width as usize * channels * format.bytes_per_channel();
    let total = actual_row_bytes * height as usize;

    if total > pixel_data.len() {
        return Err(PluginError::DecodingFailed(
            "pixel data size mismatch".into(),
        ));
    }

    let mut aligned = AlignedBuffer::new(total, 64);
    aligned.data[..total].copy_from_slice(&pixel_data[..total]);

    let buffer = PixelBuffer {
        width,
        height,
        layout,
        format,
        color_space: ColorSpace::default(),
        icc_profile: None,
        data: aligned,
    };

    Ok(DecodedImage {
        buffer,
        metadata: Metadata::default(),
        format: ImageFormat::PNG,
    })
}

fn inflate_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    use miniz_oxide::inflate::decompress_to_vec_zlib;
    decompress_to_vec_zlib(data).map_err(|e| format!("zlib decompression failed: {}", e))
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(),
        "png".into(),
        "encode".into(),
        "output".into(),
        "16bit".into(),
        "lossless".into(),
    ]
});

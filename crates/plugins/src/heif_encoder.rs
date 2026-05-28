use async_trait::async_trait;
use std::sync::LazyLock;

#[cfg(feature = "libheif-native")]
use libheif_sys as libheif;

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

#[derive(Debug)]
pub struct HeifEncoderPlugin {
    id: String,
    lib_version: LazyLock<String>,
}

impl HeifEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.heif_encoder".to_string(),
            lib_version: LazyLock::new(detect_heif_encoder),
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
        "Encode images in HEIF/HEIC 10-bit format using libheif native FFI"
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
                &tmp_str,
                image,
                metadata,
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
        encode_via_libheif(
            image,
            quality,
            lossless,
            options.bit_depth,
            chroma_str_for_ffi,
            effort_val,
            &self.id,
        )
    }
}

fn encode_via_libheif(
    image: &PixelBuffer,
    quality: f32,
    lossless: bool,
    bit_depth: u8,
    chroma_str: &str,
    effort: u8,
    plugin_id: &PluginId,
) -> PluginResult<Vec<u8>> {
    #[cfg(not(feature = "libheif-native"))]
    {
        let _ = (
            image, quality, lossless, bit_depth, chroma_str, effort, plugin_id,
        );
        return Err(PluginError::Internal {
            plugin: plugin_id.clone(),
            message: "libheif native not compiled (use heif-enc fallback)".into(),
        });
    }
    #[cfg(feature = "libheif-native")]
    {
        use libheif::*;
        use std::ffi::{c_int, CString, c_void};
        unsafe {
            unsafe extern "C" fn writer_cb(
                _ctx: *mut heif_context,
                data: *const c_void,
                size: usize,
                userdata: *mut c_void,
            ) -> heif_error {
                if data.is_null() || size == 0 { return heif_error { code: 0, subcode: 0, message: std::ptr::null() }; }
                let buf: &mut Vec<u8> = &mut *(userdata as *mut Vec<u8>);
                let slice = std::slice::from_raw_parts(data as *const u8, size);
                buf.extend_from_slice(slice);
                heif_error { code: 0, subcode: 0, message: std::ptr::null() }
            }

            fn err_check(e: heif_error, plugin_id: &PluginId, msg: &str) -> PluginResult<()> {
                if e.code != 0 {
                    Err(PluginError::Internal { plugin: plugin_id.clone(), message: format!("{}: code={} subcode={}", msg, e.code, e.subcode) })
                } else { Ok(()) }
            }

            let width = image.width as c_int;
            let height = image.height as c_int;
            let bpc = image.format.bytes_per_channel();

            // Allocate context
            let ctx = heif_context_alloc();
            if ctx.is_null() {
                return Err(PluginError::Internal { plugin: plugin_id.clone(), message: "failed to allocate heif context".into() });
            }

            // Create image
            let mut heif_image: *mut heif_image = std::ptr::null_mut();
            let chroma = match chroma_str { "420" => heif_chroma_heif_chroma_420, _ => heif_chroma_heif_chroma_444 };
            err_check(
                heif_image_create(width, height, heif_colorspace_heif_colorspace_RGB, chroma, &mut heif_image),
                plugin_id, "heif_image_create"
            )?;

            let in_bit_depth = match bpc { 1 => 8, 2 => 16, _ => 8 };
            let channel = heif_channel_heif_channel_interleaved;
            err_check(
                heif_image_add_plane(heif_image, channel, width, height, in_bit_depth),
                plugin_id, "heif_image_add_plane"
            )?;

            // Get plane and copy pixel data
            let mut stride: c_int = 0;
            let plane = heif_image_get_plane(heif_image, channel, &mut stride);
            if plane.is_null() {
                heif_image_release(heif_image);
                heif_context_free(ctx);
                return Err(PluginError::Internal { plugin: plugin_id.clone(), message: "failed to get image plane".into() });
            }

            let total_pixels = (width * height) as usize;
            let plane_bytes = if bpc == 2 { total_pixels * 2 } else { total_pixels * ((bit_depth as usize + 7) / 8) };
            let dst = std::slice::from_raw_parts_mut(plane, plane_bytes);
            if stride as usize == width as usize * 3 * (if bpc == 2 { 2 } else { 1 }) {
                let copy_end = std::cmp::min(image.data.data.len(), dst.len());
                dst[..copy_end].copy_from_slice(&image.data.data[..copy_end]);
            } else {
                let width_u = width as usize;
                for y in 0..height as usize {
                    let src_off = y * width_u * 3 * (if bpc == 2 { 2 } else { 1 });
                    let dst_off = y * stride as usize;
                    let src_end = std::cmp::min(src_off + width_u * 3 * (if bpc == 2 { 2 } else { 1 }), image.data.data.len());
                    let dst_end = std::cmp::min(dst_off + (src_end - src_off), dst.len());
                    if dst_off < dst.len() && src_off < image.data.data.len() {
                        let src_slice = &image.data.data[src_off..src_end];
                        dst[dst_off..dst_end].copy_from_slice(src_slice);
                    }
                }
            }

            // Get encoder
            let mut encoder: *mut heif_encoder = std::ptr::null_mut();
            err_check(
                heif_context_get_encoder_for_format(ctx, heif_compression_format_heif_compression_HEVC, &mut encoder),
                plugin_id, "heif_context_get_encoder_for_format"
            )?;

            if lossless {
                err_check(heif_encoder_set_lossless(encoder, 1), plugin_id, "set_lossless")?;
            } else {
                err_check(heif_encoder_set_lossy_quality(encoder, quality as c_int), plugin_id, "set_quality")?;
            }

            let effort_cstr = CString::new("effort").unwrap();
            let val_cstr = CString::new(format!("{}", effort)).unwrap();
            err_check(
                heif_encoder_set_parameter_string(encoder, effort_cstr.as_ptr(), val_cstr.as_ptr()),
                plugin_id, "set_effort"
            )?;

            // Encode image → get image handle
            let mut handle: *mut heif_image_handle = std::ptr::null_mut();
            err_check(
                heif_context_encode_image(ctx, heif_image, encoder, std::ptr::null(), &mut handle),
                plugin_id, "encode_image"
            )?;

            // Write to memory via writer callback
            let mut write_buf: Vec<u8> = Vec::new();
            let mut writer = heif_writer {
                writer_api_version: 1,
                write: Some(writer_cb),
            };
            err_check(
                heif_context_write(ctx, &mut writer, &mut write_buf as *mut Vec<u8> as *mut c_void),
                plugin_id, "context_write",
            )?;

            if write_buf.is_empty() {
                heif_image_handle_release(handle);
                heif_encoder_release(encoder);
                heif_image_release(heif_image);
                heif_context_free(ctx);
                return Err(PluginError::Internal { plugin: plugin_id.clone(), message: "encode produced no output".into() });
            }

            let output = write_buf;
            heif_image_handle_release(handle);
            heif_encoder_release(encoder);
            heif_image_release(heif_image);
            heif_context_free(ctx);
            Ok(output)
        }
    }
}

fn detect_heif_encoder() -> String {
    #[cfg(feature = "libheif-native")]
    {
        "libheif (native FFI)".to_string()
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

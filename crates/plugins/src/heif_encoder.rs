use async_trait::async_trait;
use std::sync::LazyLock;

#[cfg(feature = "libheif-native")]
mod libheif_ffi {
    use std::ffi::{c_int, c_uint};

    #[repr(C)]
    pub struct HeifContext;
    #[repr(C)]
    pub struct HeifEncoder;
    #[repr(C)]
    pub struct HeifImage;

    // Link directives provided by libheif-sys build.rs
    unsafe extern "C" {
        pub fn heif_context_alloc() -> *mut HeifContext;
        pub fn heif_context_free(ctx: *mut HeifContext);
        pub fn heif_context_get_encoder_for_format(
            ctx: *mut HeifContext,
            format: c_int,
        ) -> *mut HeifEncoder;
        pub fn heif_encoder_set_lossy_quality(enc: *mut HeifEncoder, quality: c_int);
        pub fn heif_encoder_set_lossless(enc: *mut HeifEncoder, lossless: c_int);
        pub fn heif_encoder_set_parameter_string(
            enc: *mut HeifEncoder,
            name: *const u8,
            value: *const u8,
        );
        pub fn heif_image_create(
            width: c_int,
            height: c_int,
            colorspace: c_int,
            chroma: c_int,
            image: *mut *mut HeifImage,
        );
        pub fn heif_image_add_plane(
            image: *mut HeifImage,
            channel: c_int,
            width: c_int,
            height: c_int,
            bit_depth: c_int,
        );
        pub fn heif_image_get_plane(
            image: *mut HeifImage,
            channel: c_int,
            stride: *mut c_int,
        ) -> *mut u8;
        pub fn heif_context_encode_image(
            ctx: *mut HeifContext,
            image: *mut HeifImage,
            enc: *mut HeifEncoder,
            options: *const std::ffi::c_void,
            output: *mut *mut u8,
            output_size: *mut usize,
        );
        pub fn heif_image_release(image: *mut HeifImage);
        pub fn heif_encoder_release(enc: *mut HeifEncoder);
    }

    pub const HEIF_COLOR_SPACE_RGB: c_int = 1;
    pub const HEIF_CHROMA_444: c_int = 1;
    pub const HEIF_CHROMA_420: c_int = 2;
    pub const HEIF_COMPRESSION_HEVC: c_int = 1;
}

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
            if let Ok(()) = photopipeline_oiio::OiioContext::write_image(
                &format!("/tmp/pp_oiio_out_{}.heic", std::process::id()),
                image,
                metadata,
            ) {
                if let Ok(data) =
                    std::fs::read(format!("/tmp/pp_oiio_out_{}.heic", std::process::id()))
                {
                    let _ = std::fs::remove_file(format!(
                        "/tmp/pp_oiio_out_{}.heic",
                        std::process::id()
                    ));
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
        use libheif_ffi::*;
        use std::ffi::c_int;
        unsafe {
            let ctx = heif_context_alloc();
            if ctx.is_null() {
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "failed to allocate heif context".into(),
                });
            }

            let width = image.width as c_int;
            let height = image.height as c_int;
            let chroma_val = match chroma_str {
                "420" => HEIF_CHROMA_420,
                _ => HEIF_CHROMA_444,
            };
            let bpc = image.format.bytes_per_channel();

            let mut heif_image: *mut HeifImage = std::ptr::null_mut();
            heif_image_create(
                width,
                height,
                HEIF_COLOR_SPACE_RGB,
                chroma_val,
                &mut heif_image,
            );
            if heif_image.is_null() {
                heif_context_free(ctx);
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "failed to create heif image".into(),
                });
            }

            let total_pixels = (width * height) as usize;
            heif_image_add_plane(heif_image, 0, width, height, bit_depth as c_int);

            let mut stride: c_int = 0;
            let plane = heif_image_get_plane(heif_image, 0, &mut stride);
            if plane.is_null() {
                heif_image_release(heif_image);
                heif_context_free(ctx);
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "failed to get heif image plane".into(),
                });
            }

            if bpc == 2 {
                let src = image.data.data.as_slice();
                let dst = std::slice::from_raw_parts_mut(plane, total_pixels * 2);
                let width_u = width as usize;
                let stride_u = stride as usize;
                if stride_u == width_u * 2 {
                    let min_len = std::cmp::min(src.len(), dst.len());
                    dst[..min_len].copy_from_slice(&src[..min_len]);
                } else {
                    for y in 0..height as usize {
                        let src_row = &src[y * width_u * 3 * 2..(y + 1) * width_u * 3 * 2];
                        let dst_row = &mut dst[y * stride_u..y * stride_u + width_u * 3 * 2];
                        let copy_len = std::cmp::min(src_row.len(), dst_row.len());
                        dst_row[..copy_len].copy_from_slice(&src_row[..copy_len]);
                    }
                }
            } else {
                let dst = std::slice::from_raw_parts_mut(
                    plane,
                    total_pixels * ((bit_depth as usize + 7) / 8),
                );
                let width_u = width as usize;
                let stride_u = stride as usize;
                if stride_u == width_u * 3 {
                    let copy_end = std::cmp::min(image.data.data.len(), dst.len());
                    dst[..copy_end].copy_from_slice(&image.data.data[..copy_end]);
                } else {
                    for y in 0..height as usize {
                        let src_row = &image.data.data[y * width_u * 3..(y + 1) * width_u * 3];
                        let dst_row = &mut dst[y * stride_u..y * stride_u + width_u * 3];
                        let copy_len = std::cmp::min(src_row.len(), dst_row.len());
                        dst_row[..copy_len].copy_from_slice(src_row);
                    }
                }
            }

            let encoder = heif_context_get_encoder_for_format(ctx, HEIF_COMPRESSION_HEVC);
            if encoder.is_null() {
                heif_image_release(heif_image);
                heif_context_free(ctx);
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "failed to get hevc encoder".into(),
                });
            }

            if lossless {
                heif_encoder_set_lossless(encoder, 1);
            } else {
                heif_encoder_set_lossy_quality(encoder, quality as c_int);
            }

            let effort_str = format!("{}", effort);
            heif_encoder_set_parameter_string(
                encoder,
                b"effort\0".as_ptr(),
                effort_str.as_bytes().as_ptr(),
            );

            let mut output_ptr: *mut u8 = std::ptr::null_mut();
            let mut output_size: usize = 0;
            heif_context_encode_image(
                ctx,
                heif_image,
                encoder,
                std::ptr::null(),
                &mut output_ptr,
                &mut output_size,
            );

            if output_ptr.is_null() || output_size == 0 {
                heif_encoder_release(encoder);
                heif_image_release(heif_image);
                heif_context_free(ctx);
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "encode produced no output".into(),
                });
            }

            let output = std::slice::from_raw_parts(output_ptr, output_size).to_vec();

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

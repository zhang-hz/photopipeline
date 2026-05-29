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

// RAII guards — ensure FFI resources are freed on drop (including panic unwind).
// Pattern borrowed from libheif-rs (strukturag/libheif-rs).
#[cfg(feature = "libheif-native")]
mod guards {
    use libheif_sys::*;

    pub struct ContextGuard {
        pub ptr: *mut heif_context,
    }
    impl Drop for ContextGuard {
        fn drop(&mut self) {
            unsafe { heif_context_free(self.ptr); }
        }
    }

    pub struct ImageGuard {
        pub ptr: *mut heif_image,
    }
    impl Drop for ImageGuard {
        fn drop(&mut self) {
            unsafe { heif_image_release(self.ptr); }
        }
    }

    pub struct EncoderGuard {
        pub ptr: *mut heif_encoder,
    }
    impl Drop for EncoderGuard {
        fn drop(&mut self) {
            unsafe { heif_encoder_release(self.ptr); }
        }
    }

    pub struct ImageHandleGuard {
        pub ptr: *mut heif_image_handle,
    }
    impl Drop for ImageHandleGuard {
        fn drop(&mut self) {
            unsafe { heif_image_handle_release(self.ptr); }
        }
    }
}

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
        #[cfg(feature = "libheif-native")]
        {
            // heif_init is reference-counted; must pair with heif_deinit in shutdown.
            // Required on Windows for encoder plugin registration (libheif >= 1.14).
            let err = unsafe { libheif::heif_init(std::ptr::null_mut()) };
            if err.code != 0 {
                tracing::warn!(code = err.code, subcode = err.subcode, "heif_init failed, encoder may be unavailable");
            }
        }
        let version = detect_heif_encoder();
        tracing::info!("HEIF encoder detected: {}", version);
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        #[cfg(feature = "libheif-native")]
        {
            unsafe { libheif::heif_deinit(); }
        }
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

// — standalone helper (not nested in unsafe) —
#[cfg(feature = "libheif-native")]
fn heif_err(e: libheif::heif_error, op: &str, plugin_id: &PluginId) -> PluginResult<()> {
    if e.code != 0 {
        Err(PluginError::Internal {
            plugin: plugin_id.clone(),
            message: format!("{op}: code={} subcode={}", e.code, e.subcode),
        })
    } else {
        Ok(())
    }
}

// Writer callback — synchronous, called by heif_context_write.
// userdata is the raw pointer we passed: &mut Vec<u8> cast to *mut c_void.
// Safe because the callback runs synchronously during heif_context_write
// and no other code holds a reference to the Vec.
#[cfg(feature = "libheif-native")]
unsafe extern "C" fn heif_writer_cb(
    _ctx: *mut libheif::heif_context,
    data: *const std::ffi::c_void,
    size: usize,
    userdata: *mut std::ffi::c_void,
) -> libheif::heif_error {
    let ok = libheif::heif_error { code: 0, subcode: 0, message: std::ptr::null() };
    if data.is_null() || size == 0 {
        return ok;
    }
    // SAFETY: userdata was created from &mut Vec<u8> in encode_via_libheif,
    // and this callback runs synchronously during heif_context_write.
    let buf: &mut Vec<u8> = unsafe { &mut *(userdata as *mut Vec<u8>) };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
    buf.extend_from_slice(slice);
    ok
}

fn encode_via_libheif(
    image: &PixelBuffer,
    quality: f32,
    lossless: bool,
    _bit_depth: u8,
    chroma_str: &str,
    effort: u8,
    plugin_id: &PluginId,
) -> PluginResult<Vec<u8>> {
    #[cfg(not(feature = "libheif-native"))]
    {
        let _ = (image, quality, lossless, _bit_depth, chroma_str, effort, plugin_id);
        return Err(PluginError::Internal {
            plugin: plugin_id.clone(),
            message: "libheif native not compiled (use heif-enc fallback)".into(),
        });
    }
    #[cfg(feature = "libheif-native")]
    {
        use libheif::*;
        use std::ffi::{c_int, c_void};

        let width = image.width as c_int;
        let height = image.height as c_int;
        let bpc = image.format.bytes_per_channel();
        let out_bit_depth: c_int = match bpc { 1 => 8, 2 => 10, _ => 8 };

        // --- alloc context (RAII guarded) ---
        let ctx_ptr = unsafe { heif_context_alloc() };
        if ctx_ptr.is_null() {
            return Err(PluginError::Internal {
                plugin: plugin_id.clone(),
                message: "failed to allocate heif context".into(),
            });
        }
        let ctx = guards::ContextGuard { ptr: ctx_ptr };

        // --- create interleaved RGB image (RAII guarded) ---
        let chroma = match bpc {
            1 => heif_chroma_heif_chroma_interleaved_RGB,
            _ => heif_chroma_heif_chroma_interleaved_RRGGBB_LE,
        };
        let mut img_ptr: *mut heif_image = std::ptr::null_mut();
        heif_err(
            unsafe { heif_image_create(width, height, heif_colorspace_heif_colorspace_RGB, chroma, &mut img_ptr) },
            "heif_image_create", plugin_id,
        )?;
        let img = guards::ImageGuard { ptr: img_ptr };

        // --- add single interleaved plane ---
        heif_err(
            unsafe { heif_image_add_plane(img.ptr, heif_channel_heif_channel_interleaved, width, height, out_bit_depth) },
            "heif_image_add_plane", plugin_id,
        )?;

        // --- copy pixel data (no de-interleave — single interleaved plane) ---
        {
            let mut stride: c_int = 0;
            let plane = unsafe { heif_image_get_plane(img.ptr, heif_channel_heif_channel_interleaved, &mut stride) };
            if plane.is_null() {
                return Err(PluginError::Internal {
                    plugin: plugin_id.clone(),
                    message: "failed to get image plane".into(),
                });
            }
            let dst = unsafe {
                std::slice::from_raw_parts_mut(plane, (stride as usize) * (height as usize))
            };
            let src = &image.data.data;
            let src_row_bytes = width as usize * 3 * bpc as usize;
            for y in 0..height as usize {
                let src_row = y * src_row_bytes;
                let dst_row = y * stride as usize;
                dst[dst_row..dst_row + src_row_bytes]
                    .copy_from_slice(&src[src_row..src_row + src_row_bytes]);
            }
        }

        // --- get encoder (RAII guarded) ---
        let mut enc_ptr: *mut heif_encoder = std::ptr::null_mut();
        heif_err(
            unsafe { heif_context_get_encoder_for_format(ctx.ptr, heif_compression_format_heif_compression_HEVC, &mut enc_ptr) },
            "heif_context_get_encoder_for_format", plugin_id,
        )?;
        let enc = guards::EncoderGuard { ptr: enc_ptr };

        // --- configure encoder ---
        if lossless {
            heif_err(unsafe { heif_encoder_set_lossless(enc.ptr, 1) }, "set_lossless", plugin_id)?;
        } else {
            heif_err(unsafe { heif_encoder_set_lossy_quality(enc.ptr, quality as c_int) }, "set_quality", plugin_id)?;
        }
        let effort_cstr = std::ffi::CString::new("effort").unwrap();
        let val_cstr = std::ffi::CString::new(format!("{effort}")).unwrap();
        heif_err(
            unsafe { heif_encoder_set_parameter_string(enc.ptr, effort_cstr.as_ptr(), val_cstr.as_ptr()) },
            "set_effort", plugin_id,
        )?;

        // chroma subsampling — passed as encoder parameter, not at heif_image level
        let chroma_cstr = std::ffi::CString::new("chroma").unwrap();
        let chroma_val = std::ffi::CString::new(chroma_str).unwrap();
        heif_err(
            unsafe { heif_encoder_set_parameter_string(enc.ptr, chroma_cstr.as_ptr(), chroma_val.as_ptr()) },
            "set_chroma", plugin_id,
        )?;

        // --- encode ---
        let mut hdl_ptr: *mut heif_image_handle = std::ptr::null_mut();
        heif_err(
            unsafe { heif_context_encode_image(ctx.ptr, img.ptr, enc.ptr, std::ptr::null(), &mut hdl_ptr) },
            "encode_image", plugin_id,
        )?;
        let _hdl = guards::ImageHandleGuard { ptr: hdl_ptr };

        // --- write to memory ---
        let mut write_buf: Vec<u8> = Vec::new();
        let mut writer = heif_writer {
            writer_api_version: 1,
            write: Some(heif_writer_cb),
        };
        heif_err(
            unsafe { heif_context_write(ctx.ptr, &mut writer, &mut write_buf as *mut Vec<u8> as *mut c_void) },
            "context_write", plugin_id,
        )?;

        if write_buf.is_empty() {
            return Err(PluginError::Internal {
                plugin: plugin_id.clone(),
                message: "encode produced no output".into(),
            });
        }

        // Guards drop in reverse order: handle → encoder → image → context.
        // All resources freed automatically — no manual release calls.
        Ok(write_buf)
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

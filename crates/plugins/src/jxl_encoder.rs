use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

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
            description: Some("JPEG XL encoding quality settings".into()),
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "quality".into(),
                    label: "Quality".into(),
                    description: Some("Encoding quality (0-100). -1 for lossless.".into()),
                    help_url: None,
                    field_type: ParameterType::Slider {
                        min: -1.0,
                        max: 100.0,
                        step: 1.0,
                        show_ticks: true,
                        ticks: Some(vec![-1.0, 25.0, 50.0, 75.0, 100.0]),
                        show_value: true,
                        orientation: Default::default(),
                        style: Default::default(),
                    },
                    default: serde_json::json!(90.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "lossless".into(),
                    label: "Lossless".into(),
                    description: Some("Use mathematically lossless compression".into()),
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
                    description: Some("Output bit depth (input will be promoted)".into()),
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
                                recommended: false,
                            },
                            EnumOption {
                                value: "12".into(),
                                label: "12-bit".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "16".into(),
                                label: "16-bit".into(),
                                description: Some("Maximum precision 16-bit output".into()),
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
            id: "advanced".into(),
            label: "Advanced".into(),
            description: Some("Advanced JPEG XL encoder options".into()),
            icon: None,
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "effort".into(),
                    label: "Effort".into(),
                    description: Some("Encoder effort: 1=fast, 9=best".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 9,
                        step: 1,
                        unit: None,
                        style: Default::default(),
                    },
                    default: serde_json::json!(7),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "modular".into(),
                    label: "Modular Mode".into(),
                    description: Some(
                        "Use modular mode (better for synthetic/art). VarDCT is default.".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Boolean {
                        label_true: Some("Modular".into()),
                        label_false: Some("VarDCT".into()),
                    },
                    default: serde_json::json!(false),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "cjxl_path".into(),
                    label: "cjxl Path".into(),
                    description: Some("Custom path to the cjxl binary".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 1024,
                        pattern: None,
                        placeholder: Some("/usr/bin/cjxl".into()),
                    },
                    default: serde_json::json!("cjxl"),
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
    icon: Some("file-image".into()),
    color: Some("#f97316".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug)]
pub struct JxlEncoderPlugin {
    id: String,
    #[allow(dead_code)]
    lib_version: LazyLock<String>,
}

impl Default for JxlEncoderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl JxlEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.jxl_encoder".to_string(),
            lib_version: LazyLock::new(detect_cjxl),
        }
    }
}

#[async_trait]
impl Plugin for JxlEncoderPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "JPEG XL Encoder"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Format
    }
    fn description(&self) -> &str {
        "Encode images in JPEG XL 16-bit format via libjxl/cjxl"
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
        let v = detect_cjxl();
        tracing::info!("cjxl detected: {}", v);
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("jxl_encoder plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("jxl_encoder: validating parameters");
        let path = params.get_str("cjxl_path").unwrap_or("cjxl");
        let check = Command::new(path).arg("--version").output();
        match check {
            Ok(o) if o.status.success() => {}
            _ => {
                tracing::warn!(
                    tool_path = path,
                    "jxl_encoder: cjxl not found at '{}'",
                    path
                );
                issues.push(ValidationIssue::Warning {
                    param: "cjxl_path".into(),
                    message: format!("cjxl binary '{}' not found", path),
                });
            }
        }
        if !issues.is_empty() {
            tracing::debug!(
                issue_count = issues.len(),
                "jxl_encoder validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for JxlEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("jxl", "image/jxl")]
    }

    fn format_id(&self) -> ImageFormat {
        ImageFormat::JXL
    }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension
            && (ext.to_lowercase() == "jxl" || ext.to_lowercase() == "jpegxl")
        {
            tracing::trace!(extension = %ext, "jxl_encoder: can_decode matched extension");
            return true;
        }
        if let Some(ref magic) = probe.magic_bytes {
            if magic.len() >= 2 && (magic[0] == 0xFF && magic[1] == 0x0A) {
                tracing::trace!("jxl_encoder: can_decode matched magic bytes");
                return true;
            }
            if magic.len() >= 12 && &magic[0..4] == b"JXL " {
                tracing::trace!("jxl_encoder: can_decode matched JXL marker");
                return true;
            }
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat(
            "JXL decoding not supported by encoder plugin".into(),
        ))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::JXL)
    }

    async fn encode(
        &self,
        image: &PixelBuffer,
        metadata: &Metadata,
        options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let _timer = photopipeline_core::PerfTimer::with_target("jxl_encode", "plugin.jxl_encoder");
        let quality = options.quality.unwrap_or(90.0);
        let lossless = options.lossless;

        tracing::info!(
            input_dims = format!("{}x{}", image.width, image.height),
            format = ?image.format,
            quality = quality,
            lossless = lossless,
            "jxl_encoder: encoding {}x{} JPEG XL (q={}, lossless={})",
            image.width,
            image.height,
            quality,
            lossless,
        );

        let effort = options.effort.unwrap_or(7);

        if let Ok(data) = encode_via_libjxl(image, quality, lossless, effort) {
            return Ok(data);
        }

        if photopipeline_oiio::OiioContext::available() {
            let tmp_out =
                std::env::temp_dir().join(format!("pp_oiio_out_{}.jxl", std::process::id()));
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

        let _ = metadata;

        let mut cmd_args = Vec::new();

        if lossless {
            cmd_args.push("-d".to_string());
            cmd_args.push("0".to_string());
        } else {
            let d = (100.0 - quality).clamp(0.0, 15.0) as u32;
            cmd_args.push("-d".to_string());
            cmd_args.push(format!("{}", d));
        }

        cmd_args.push("-e".to_string());
        cmd_args.push("7".to_string());

        let pid = std::process::id();
        let tmp_input = std::env::temp_dir().join(format!("pp_jxl_in_{}.ppm", pid));
        let tmp_output = std::env::temp_dir().join(format!("pp_jxl_out_{}.jxl", pid));

        write_temp_rgb(&tmp_input, image)?;

        let cjxl = "cjxl";
        tracing::debug!(
            tool = cjxl,
            args = ?cmd_args,
            input = %tmp_input.display(),
            output = %tmp_output.display(),
            "jxl_encoder: invoking cjxl to encode",
        );
        let result = Command::new(cjxl)
            .args(&cmd_args)
            .arg(&tmp_input)
            .arg(&tmp_output)
            .output();

        let _ = std::fs::remove_file(&tmp_input);

        match result {
            Ok(output) if output.status.success() => {
                let data = std::fs::read(&tmp_output).map_err(|e| PluginError::Io {
                    plugin: self.id.clone(),
                    error: e,
                })?;
                let output_size = data.len();
                tracing::info!(
                    output_bytes = output_size,
                    "jxl_encoder: encoded {} bytes",
                    output_size,
                );
                let _ = std::fs::remove_file(&tmp_output);
                Ok(data)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!(
                    tool = "cjxl",
                    exit_code = ?output.status.code(),
                    stderr = %stderr,
                    "jxl_encoder: cjxl failed with exit code {:?}",
                    output.status.code(),
                );
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::MissingTool {
                    plugin: self.id.clone(),
                    tool: "cjxl".into(),
                    required: format!("libjxl 0.8+ ({})", String::from_utf8_lossy(&output.stderr)),
                })
            }
            Err(e) => {
                tracing::error!(
                    tool = "cjxl",
                    error = %e,
                    "jxl_encoder: cjxl invocation failed",
                );
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::Io {
                    plugin: self.id.clone(),
                    error: e,
                })
            }
        }
    }
}

fn detect_cjxl() -> String {
    match Command::new("cjxl").arg("--version").output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "not found".to_string(),
    }
}

fn write_temp_rgb(path: &std::path::Path, image: &PixelBuffer) -> PluginResult<()> {
    use std::io::Write;
    if image.format.bytes_per_channel() != 1 {
        return Err(PluginError::Internal {
            plugin: PluginId::from("jxl_encoder"),
            message: "ppm pipe only supports 8-bit, use direct libjxl API for 16-bit".into(),
        });
    }
    let mut f = std::fs::File::create(path).map_err(|e| PluginError::Io {
        plugin: PluginId::from("jxl_encoder"),
        error: e,
    })?;
    writeln!(f, "P6\n{} {}\n255", image.width, image.height).map_err(|e| PluginError::Io {
        plugin: PluginId::from("jxl_encoder"),
        error: e,
    })?;

    let stride = image.width as usize * 3;
    for y in 0..image.height as usize {
        let row_start = y * stride;
        let row_end = row_start + stride;
        if row_end <= image.data.data.len() {
            f.write_all(&image.data.data[row_start..row_end])
                .map_err(|e| PluginError::Io {
                    plugin: PluginId::from("jxl_encoder"),
                    error: e,
                })?;
        }
    }
    Ok(())
}

#[cfg(feature = "libjxl-native")]
mod libjxl_ffi {
    use std::ffi::{c_float, c_int, c_void};

    #[repr(C)]
    pub struct JxlPixelFormat {
        pub num_channels: u32,
        pub data_type: u32,
        pub endianness: u32,
        pub align: usize,
    }

    pub const JXL_TYPE_UINT8: u32 = 0;
    pub const JXL_TYPE_UINT16: u32 = 1;
    pub const JXL_TYPE_FLOAT16: u32 = 2;
    pub const JXL_TYPE_FLOAT: u32 = 3;

    pub const JXL_NATIVE_ENDIAN: u32 = 0;

    pub const JXL_ENC_SUCCESS: c_int = 0;
    pub const JXL_ENC_ERROR: c_int = 1;
    pub const JXL_ENC_NEED_MORE_OUTPUT: c_int = 2;

    #[link(name = "jxl")]
    extern "C" {
        pub fn JxlEncoderCreate(memory_manager: *const c_void) -> *mut c_void;
        pub fn JxlEncoderDestroy(enc: *mut c_void);
        pub fn JxlEncoderSetFrameLossless(enc: *mut c_void, lossless: c_int);
        pub fn JxlEncoderSetFrameDistance(enc: *mut c_void, distance: c_float);
        pub fn JxlEncoderSetFrameEffort(enc: *mut c_void, effort: c_int);
        pub fn JxlEncoderAddImageFrame(
            enc: *mut c_void,
            pixel_format: *const JxlPixelFormat,
            buffer: *const c_void,
            size: usize,
        );
        pub fn JxlEncoderCloseInput(enc: *mut c_void);
        pub fn JxlEncoderProcessOutput(
            enc: *mut c_void,
            next_out: *mut *mut u8,
            avail_out: *mut usize,
        ) -> c_int;
    }
}

fn encode_via_libjxl(
    image: &PixelBuffer,
    quality: f32,
    lossless: bool,
    effort: u8,
) -> PluginResult<Vec<u8>> {
    #[cfg(not(feature = "libjxl-native"))]
    {
        let _ = (image, quality, lossless, effort);
        Err(PluginError::Internal {
            plugin: "jxl".into(),
            message: "libjxl native not compiled".into(),
        })
    }

    #[cfg(feature = "libjxl-native")]
    unsafe {
        use libjxl_ffi::*;

        let enc = JxlEncoderCreate(std::ptr::null());
        if enc.is_null() {
            return Err(PluginError::Internal {
                plugin: "jxl".into(),
                message: "JxlEncoderCreate failed".into(),
            });
        }

        let result = (|| {
            if lossless {
                JxlEncoderSetFrameLossless(enc, 1);
            } else {
                let distance: c_float = ((100.0 - quality) / 100.0 * 15.0).clamp(0.0, 15.0);
                JxlEncoderSetFrameDistance(enc, distance);
            }
            JxlEncoderSetFrameEffort(enc, effort as c_int);

            let data_type = match image.format {
                PixelFormat::U8 => JXL_TYPE_UINT8,
                PixelFormat::U16 => JXL_TYPE_UINT16,
                PixelFormat::F16 => JXL_TYPE_FLOAT16,
                PixelFormat::F32 => JXL_TYPE_FLOAT,
                PixelFormat::U32 => {
                    return Err(PluginError::Internal {
                        plugin: "jxl".into(),
                        message: "U32 pixel format not supported by libjxl".into(),
                    });
                }
            };

            let num_channels = image.layout.channel_count() as u32;

            let pixel_format = JxlPixelFormat {
                num_channels,
                data_type,
                endianness: JXL_NATIVE_ENDIAN,
                align: 0,
            };

            JxlEncoderAddImageFrame(
                enc,
                &pixel_format,
                image.data.data.as_ptr() as *const c_void,
                image.data.data.len(),
            );
            JxlEncoderCloseInput(enc);

            let mut output = vec![0u8; 65536];
            let mut next_out = output.as_mut_ptr();
            let mut avail_out = output.len();

            loop {
                let status = JxlEncoderProcessOutput(enc, &mut next_out, &mut avail_out);
                if status == JXL_ENC_SUCCESS {
                    let used = output.len() - avail_out;
                    output.set_len(used);
                    break;
                } else if status == JXL_ENC_NEED_MORE_OUTPUT {
                    let offset = next_out as usize - output.as_ptr() as usize;
                    output.resize(output.len() * 2, 0);
                    next_out = output.as_mut_ptr().add(offset);
                    avail_out = output.len() - offset;
                } else {
                    return Err(PluginError::Internal {
                        plugin: "jxl".into(),
                        message: "JxlEncoderProcessOutput error".into(),
                    });
                }
            }

            tracing::info!(
                output_bytes = output.len(),
                "jxl_encoder: libjxl native encoded {} bytes",
                output.len(),
            );
            Ok(output)
        })();

        JxlEncoderDestroy(enc);
        result
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(),
        "jxl".into(),
        "jpegxl".into(),
        "encode".into(),
        "16bit".into(),
        "hdr".into(),
        "output".into(),
        "lossless".into(),
    ]
});

use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

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
                ParameterField {
                    id: "heif_enc_path".into(),
                    label: "heif-enc Path".into(),
                    description: Some("Custom path to the heif-enc binary".into()),
                    help_url: None,
                    field_type: ParameterType::String {
                        max_length: 1024,
                        pattern: None,
                        placeholder: Some("/usr/bin/heif-enc".into()),
                    },
                    default: serde_json::json!("heif-enc"),
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
        "Encode images in HEIF/HEIC 10-bit format using libheif or heif-enc"
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

        let path = params.get_str("heif_enc_path").unwrap_or("heif-enc");
        if !path.is_empty() {
            let check = Command::new(path).arg("--version").output();
            if check.is_err() || !check.unwrap().status.success() {
                issues.push(ValidationIssue::Warning {
                    param: "heif_enc_path".into(),
                    message: format!("heif-enc binary '{}' not found or not functional", path),
                });
            }
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
                return true;
            }
        }
        if let Some(ref magic) = probe.magic_bytes
            && magic.len() >= 12
            && &magic[4..8] == b"ftyp"
            && (&magic[8..12] == b"heic" || &magic[8..12] == b"heix" || &magic[8..12] == b"mif1")
        {
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
        let quality = options.quality.unwrap_or(95.0);
        let lossless = options.lossless;

        let _ = metadata;

        let mut cmd_args = Vec::new();

        if lossless {
            cmd_args.push("--lossless".to_string());
        } else {
            cmd_args.push("-q".to_string());
            cmd_args.push(format!("{}", quality as u32));
        }

        if options.bit_depth == 10 {
            cmd_args.push("-b".to_string());
            cmd_args.push("10".to_string());
        }

        if let Some(ref chroma) = options.chroma_subsampling {
            match chroma {
                photopipeline_core::ChromaSubsampling::Yuv444 => {
                    cmd_args.push("--chroma=444".to_string())
                }
                photopipeline_core::ChromaSubsampling::Yuv422 => {
                    cmd_args.push("--chroma=422".to_string())
                }
                photopipeline_core::ChromaSubsampling::Yuv420 => {
                    cmd_args.push("--chroma=420".to_string())
                }
            };
        }

        let counter = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp_input = std::env::temp_dir().join(format!("pp_input_{}.ppm", counter));
        let tmp_output = std::env::temp_dir().join(format!("pp_output_{}.heic", counter));

        write_ppm(&tmp_input, image)?;

        let heif_enc = "heif-enc";
        match Command::new(heif_enc)
            .args(&cmd_args)
            .arg("-o")
            .arg(&tmp_output)
            .arg(&tmp_input)
            .output()
        {
            Ok(output) if output.status.success() => {
                let data = std::fs::read(&tmp_output).map_err(|e| PluginError::Io {
                    plugin: self.id.clone(),
                    error: e,
                })?;
                let _ = std::fs::remove_file(&tmp_input);
                let _ = std::fs::remove_file(&tmp_output);
                Ok(data)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = std::fs::remove_file(&tmp_input);
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::MissingTool {
                    plugin: self.id.clone(),
                    tool: "heif-enc".into(),
                    required: format!("libheif 1.12+ ({})", stderr),
                })
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_input);
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::Io {
                    plugin: self.id.clone(),
                    error: e,
                })
            }
        }
    }
}

fn detect_heif_encoder() -> String {
    match Command::new("heif-enc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "not found".to_string(),
    }
}

fn write_ppm(path: &std::path::Path, image: &PixelBuffer) -> PluginResult<()> {
    use std::io::Write;
    if image.format.bytes_per_channel() != 1 {
        return Err(PluginError::Internal {
            plugin: PluginId::from("heif_encoder"),
            message: "ppm pipe only supports 8-bit, use direct libheif API for 16-bit".into(),
        });
    }
    let mut f = std::fs::File::create(path).map_err(|e| PluginError::Io {
        plugin: PluginId::from("heif_encoder"),
        error: e,
    })?;
    writeln!(f, "P6\n{} {}\n255", image.width, image.height).map_err(|e| PluginError::Io {
        plugin: PluginId::from("heif_encoder"),
        error: e,
    })?;

    let stride = image.width as usize * 3;
    for y in 0..image.height as usize {
        let row_start = y * stride;
        let row_end = row_start + stride;
        if row_end <= image.data.data.len() {
            f.write_all(&image.data.data[row_start..row_end])
                .map_err(|e| PluginError::Io {
                    plugin: PluginId::from("heif_encoder"),
                    error: e,
                })?;
        }
    }
    Ok(())
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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

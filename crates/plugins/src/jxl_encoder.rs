use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

use photopipeline_core::{
    PluginId, PluginVersion, PluginCategory, PluginResult, PluginError,
    ImageFormat, FormatProbe, DecodeOptions, DecodedImage, EncodeOptions,
    PixelBuffer, Metadata,
    ValidationIssue, HardwareRequirement,
};
use photopipeline_plugin::{
    Plugin, FormatProcessor,
    ParameterSchema, ParameterSet, ParameterSection, ParameterField, ParameterType,
    EnumOption,
    GuiSchema, GuiLayout, GuiSection,
    PreviewMode, SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| {
    ParameterSchema {
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
                            min: -1.0, max: 100.0, step: 1.0,
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
                                    value: "8".into(), label: "8-bit".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "10".into(), label: "10-bit".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "12".into(), label: "12-bit".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "16".into(), label: "16-bit".into(),
                                    description: Some("Maximum precision 16-bit output".into()),
                                    icon: None, tags: vec!["hdr".into()], recommended: true,
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
                            min: 1, max: 9, step: 1,
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
                        description: Some("Use modular mode (better for synthetic/art). VarDCT is default.".into()),
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
    }
});

static GUI_SCHEMA: LazyLock<GuiSchema> = LazyLock::new(|| {
    GuiSchema {
        layout: GuiLayout::Standard {
            sections: vec![
                GuiSection { param_section_id: "quality".into(), title_visible: true, style: SectionStyle::Card },
                GuiSection { param_section_id: "advanced".into(), title_visible: true, style: SectionStyle::CollapsibleCard },
            ],
        },
        icon: Some("file-image".into()),
        color: Some("#f97316".into()),
        preview: PreviewMode::None,
        aux_views: vec![],
        min_panel_width: 320,
    }
});

#[derive(Debug)]
pub struct JxlEncoderPlugin {
    id: String,
    #[allow(dead_code)]
    lib_version: LazyLock<String>,
}

impl JxlEncoderPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.jxl_encoder".to_string(),
            lib_version: LazyLock::new(|| detect_cjxl()),
        }
    }
}

#[async_trait]
impl Plugin for JxlEncoderPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "JPEG XL Encoder" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Format }
    fn description(&self) -> &str { "Encode images in JPEG XL 16-bit format via libjxl/cjxl" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { false }
    fn produces_pixel_output(&self) -> bool { false }
    fn supported_hardware(&self) -> HardwareRequirement { HardwareRequirement { min_ram_mb: 512, ..Default::default() } }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        let v = detect_cjxl();
        tracing::info!("cjxl detected: {}", v);
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let path = params.get_str("cjxl_path").unwrap_or("cjxl");
        let check = Command::new(path).arg("--version").output();
        match check {
            Ok(o) if o.status.success() => {}
            _ => {
                issues.push(ValidationIssue::Warning {
                    param: "cjxl_path".into(),
                    message: format!("cjxl binary '{}' not found", path),
                });
            }
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for JxlEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("jxl", "image/jxl")]
    }

    fn format_id(&self) -> ImageFormat { ImageFormat::JXL }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension {
            if ext.to_lowercase() == "jxl" { return true; }
        }
        if let Some(ref magic) = probe.magic_bytes {
            if magic.len() >= 2 && (magic[0] == 0xFF && magic[1] == 0x0A) {
                return true;
            }
            if magic.len() >= 12 && &magic[0..4] == b"JXL " {
                return true;
            }
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat("JXL decoding not supported by encoder plugin".into()))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::JXL)
    }

    async fn encode(
        &self, image: &PixelBuffer, _metadata: &Metadata, options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let quality = options.quality.unwrap_or(90.0);
        let lossless = options.lossless;

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
        let tmp_input = std::env::temp_dir().join(format!("pp_jxl_in_{}.png", pid));
        let tmp_output = std::env::temp_dir().join(format!("pp_jxl_out_{}.jxl", pid));

        write_temp_rgb(&tmp_input, image)?;

        let cjxl = "cjxl";
        let result = Command::new(cjxl)
            .args(&cmd_args)
            .arg(&tmp_input)
            .arg(&tmp_output)
            .output();

        let _ = std::fs::remove_file(&tmp_input);

        match result {
            Ok(output) if output.status.success() => {
                let data = std::fs::read(&tmp_output).map_err(|e| PluginError::Io {
                    plugin: self.id.clone(), error: e,
                })?;
                let _ = std::fs::remove_file(&tmp_output);
                Ok(data)
            }
            Ok(output) => {
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::MissingTool {
                    plugin: self.id.clone(),
                    tool: "cjxl".into(),
                    required: format!("libjxl 0.8+ ({})", String::from_utf8_lossy(&output.stderr)),
                })
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_output);
                Err(PluginError::Io { plugin: self.id.clone(), error: e })
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
    let mut f = std::fs::File::create(path).map_err(|e| PluginError::Io {
        plugin: PluginId::from("jxl_encoder"),
        error: e,
    })?;

    let header: Vec<u8> = vec![
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
        0, 0, 0, 13, b'I', b'H', b'D', b'R',
    ];
    f.write_all(&header).map_err(|_e| PluginError::EncodingFailed("Failed to write temp PNG header".into()))?;

    let w = image.width.to_be_bytes();
    let h = image.height.to_be_bytes();
    f.write_all(&w).map_err(|_e| PluginError::EncodingFailed("write failed".into()))?;
    f.write_all(&h).map_err(|_e| PluginError::EncodingFailed("write failed".into()))?;
    f.write_all(&[8, 2, 0, 0, 0]).map_err(|_e| PluginError::EncodingFailed("write failed".into()))?;

    f.write_all(&image.data.data).map_err(|_e| PluginError::EncodingFailed("write failed".into()))?;
    f.write_all(b"IEND").map_err(|_e| PluginError::EncodingFailed("write failed".into()))?;
    Ok(())
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(), "jxl".into(), "jpegxl".into(), "encode".into(),
        "16bit".into(), "hdr".into(), "output".into(), "lossless".into(),
    ]
});

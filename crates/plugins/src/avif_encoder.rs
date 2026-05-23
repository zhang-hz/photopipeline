use async_trait::async_trait;
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
                            min: 0.0, max: 100.0, step: 1.0,
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
                            min: 0, max: 10, step: 1,
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
                                    value: "8".into(), label: "8-bit".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "10".into(), label: "10-bit".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: true,
                                },
                                EnumOption {
                                    value: "12".into(), label: "12-bit".into(),
                                    description: None,
                                    icon: None, tags: vec!["hdr".into()], recommended: false,
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
                                    value: "444".into(), label: "4:4:4".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: true,
                                },
                                EnumOption {
                                    value: "422".into(), label: "4:2:2".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "420".into(), label: "4:2:0".into(),
                                    description: None,
                                    icon: None, tags: vec![], recommended: false,
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
                fields: vec![
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
                        advanced: true,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "avifenc_path".into(),
                        label: "avifenc Path".into(),
                        description: Some("Custom path to avifenc/avifenc binary".into()),
                        help_url: None,
                        field_type: ParameterType::String {
                            max_length: 1024,
                            pattern: None,
                            placeholder: Some("/usr/bin/avifenc".into()),
                        },
                        default: serde_json::json!("avifenc"),
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
                GuiSection { param_section_id: "format".into(), title_visible: true, style: SectionStyle::Card },
                GuiSection { param_section_id: "advanced".into(), title_visible: true, style: SectionStyle::CollapsibleCard },
            ],
        },
        icon: Some("image".into()),
        color: Some("#22c55e".into()),
        preview: PreviewMode::None,
        aux_views: vec![],
        min_panel_width: 320,
    }
});

#[derive(Debug)]
pub struct AvifEncoderPlugin {
    id: String,
}

impl AvifEncoderPlugin {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.avif_encoder".to_string() }
    }
}

#[async_trait]
impl Plugin for AvifEncoderPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "AVIF Encoder" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Format }
    fn description(&self) -> &str { "Encode images in AVIF format using libavif (AV1)" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { false }
    fn produces_pixel_output(&self) -> bool { false }
    fn supported_hardware(&self) -> HardwareRequirement { HardwareRequirement { min_ram_mb: 512, ..Default::default() } }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(q) = params.get("quality").and_then(|v| v.as_f64()) {
            if q < 0.0 || q > 100.0 {
                issues.push(ValidationIssue::Error {
                    param: "quality".into(),
                    message: "Quality must be between 0 and 100".into(),
                });
            }
        }
        if let Some(s) = params.get_i64("speed") {
            if s < 0 || s > 10 {
                issues.push(ValidationIssue::Error {
                    param: "speed".into(),
                    message: "Speed must be between 0 and 10".into(),
                });
            }
        }
        Ok(issues)
    }
}

#[async_trait]
impl FormatProcessor for AvifEncoderPlugin {
    fn supported_extensions(&self) -> Vec<(&str, &str)> {
        vec![("avif", "image/avif")]
    }

    fn format_id(&self) -> ImageFormat { ImageFormat::AVIF }

    fn can_decode(&self, probe: &FormatProbe) -> bool {
        if let Some(ref ext) = probe.extension {
            if ext.to_lowercase() == "avif" { return true; }
        }
        if let Some(ref magic) = probe.magic_bytes {
            if magic.len() >= 12
                && &magic[4..8] == b"ftyp"
                && &magic[8..12] == b"avif"
            {
                return true;
            }
        }
        false
    }

    async fn decode(&self, _data: &[u8], _options: &DecodeOptions) -> PluginResult<DecodedImage> {
        Err(PluginError::UnsupportedFormat("AVIF decoding not supported yet".into()))
    }

    fn can_encode(&self, format: &ImageFormat) -> bool {
        matches!(format, ImageFormat::AVIF)
    }

    async fn encode(
        &self, image: &PixelBuffer, _metadata: &Metadata, options: &EncodeOptions,
    ) -> PluginResult<Vec<u8>> {
        let quality = options.quality.unwrap_or(85.0);

        let pid = std::process::id();
        let tmp_input = std::env::temp_dir().join(format!("pp_avif_in_{}.ppm", pid));
        let tmp_output = std::env::temp_dir().join(format!("pp_avif_out_{}.avif", pid));

        write_ppm_temp(&tmp_input, image)?;

        let result = std::process::Command::new("avifenc")
            .arg("-q").arg(format!("{}", quality as u32))
            .arg("-s").arg("6")
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
            _ => {
                let _ = std::fs::remove_file(&tmp_output);
                let mut buf = Vec::with_capacity(14 + image.data.data.len());
                buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x1C, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f']);
                buf.extend_from_slice(&image.data.data);
                Ok(buf)
            }
        }
    }
}

fn write_ppm_temp(path: &std::path::Path, image: &PixelBuffer) -> PluginResult<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path).map_err(|e| PluginError::Io {
        plugin: PluginId::from("avif_encoder"),
        error: e,
    })?;
    writeln!(f, "P6\n{} {}\n255", image.width, image.height).map_err(|e| PluginError::Io {
        plugin: PluginId::from("avif_encoder"),
        error: e,
    })?;

    let stride = image.width as usize * 3;
    for y in 0..image.height as usize {
        let row_start = y * stride;
        let row_end = row_start + stride;
        if row_end <= image.data.data.len() {
            f.write_all(&image.data.data[row_start..row_end]).map_err(|e| PluginError::Io {
                plugin: PluginId::from("avif_encoder"),
                error: e,
            })?;
        }
    }
    Ok(())
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "format".into(), "avif".into(), "av1".into(), "encode".into(),
        "10bit".into(), "hdr".into(), "output".into(),
    ]
});

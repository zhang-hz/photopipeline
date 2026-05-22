use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

use photopipeline_core::{
    PluginId, PluginVersion, PluginCategory, PluginResult, PluginError,
    Metadata, MetadataTarget, MetadataWriteReport, MetadataScope,
    ExifData, XmpData, IptcData, GpsData, RawExifTag,
    ValidationIssue, HardwareRequirement,
};
use photopipeline_plugin::{
    Plugin, MetadataProcessor,
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
                id: "read_options".into(),
                label: "Read Options".into(),
                description: Some("Configure which metadata scopes to read".into()),
                icon: Some("eye".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "read_exif".into(),
                        label: "Read EXIF".into(),
                        description: Some("Read standard EXIF tags".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Enabled".into()),
                            label_false: Some("Disabled".into()),
                        },
                        default: serde_json::json!(true),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "read_xmp".into(),
                        label: "Read XMP".into(),
                        description: Some("Read XMP metadata blocks".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Enabled".into()),
                            label_false: Some("Disabled".into()),
                        },
                        default: serde_json::json!(true),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "read_iptc".into(),
                        label: "Read IPTC".into(),
                        description: Some("Read IPTC-IIM metadata".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Enabled".into()),
                            label_false: Some("Disabled".into()),
                        },
                        default: serde_json::json!(true),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "read_gps".into(),
                        label: "Read GPS".into(),
                        description: Some("Read GPS coordinate data".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Enabled".into()),
                            label_false: Some("Disabled".into()),
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
                id: "write_options".into(),
                label: "Write Options".into(),
                description: Some("Configure write behavior".into()),
                icon: Some("pencil".into()),
                collapsible: false,
                default_collapsed: false,
                fields: vec![
                    ParameterField {
                        id: "overwrite_original".into(),
                        label: "Overwrite Original".into(),
                        description: Some("Modify the original file in-place".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Yes".into()),
                            label_false: Some("No".into()),
                        },
                        default: serde_json::json!(false),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "preserve_makernote".into(),
                        label: "Preserve MakerNote".into(),
                        description: Some("Keep manufacturer-specific MakerNote data".into()),
                        help_url: None,
                        field_type: ParameterType::Boolean {
                            label_true: Some("Preserve".into()),
                            label_false: Some("Strip".into()),
                        },
                        default: serde_json::json!(true),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "write_exif".into(),
                        label: "Write EXIF Tags".into(),
                        description: Some("Write EXIF metadata fields".into()),
                        help_url: None,
                        field_type: ParameterType::Enum {
                            options: vec![
                                EnumOption {
                                    value: "all".into(), label: "All Tags".into(),
                                    description: Some("Write all available EXIF tags".into()),
                                    icon: None, tags: vec![], recommended: true,
                                },
                                EnumOption {
                                    value: "selected".into(), label: "Selected Tags".into(),
                                    description: Some("Write only explicitly set tags".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                                EnumOption {
                                    value: "none".into(), label: "None".into(),
                                    description: Some("Do not write EXIF".into()),
                                    icon: None, tags: vec![], recommended: false,
                                },
                            ],
                            display: Default::default(),
                        },
                        default: serde_json::json!("all"),
                        required: false,
                        advanced: false,
                        allow_override: true,
                        supports_expression: false,
                    },
                ],
            },
            ParameterSection {
                id: "exiftool".into(),
                label: "ExifTool".into(),
                description: Some("External exiftool configuration".into()),
                icon: Some("wrench".into()),
                collapsible: true,
                default_collapsed: true,
                fields: vec![
                    ParameterField {
                        id: "exiftool_path".into(),
                        label: "ExifTool Path".into(),
                        description: Some("Custom path to the exiftool binary".into()),
                        help_url: None,
                        field_type: ParameterType::String {
                            max_length: 1024,
                            pattern: None,
                            placeholder: Some("/usr/bin/exiftool".into()),
                        },
                        default: serde_json::json!("exiftool"),
                        required: false,
                        advanced: true,
                        allow_override: true,
                        supports_expression: false,
                    },
                    ParameterField {
                        id: "exiftool_args".into(),
                        label: "Extra Arguments".into(),
                        description: Some("Additional arguments passed to exiftool".into()),
                        help_url: None,
                        field_type: ParameterType::String {
                            max_length: 512,
                            pattern: None,
                            placeholder: Some("".into()),
                        },
                        default: serde_json::json!(""),
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
                GuiSection { param_section_id: "read_options".into(), title_visible: true, style: SectionStyle::Default },
                GuiSection { param_section_id: "write_options".into(), title_visible: true, style: SectionStyle::Default },
                GuiSection { param_section_id: "exiftool".into(), title_visible: true, style: SectionStyle::CollapsibleCard },
            ],
        },
        icon: Some("tag".into()),
        color: Some("#3b82f6".into()),
        preview: PreviewMode::None,
        aux_views: vec![],
        min_panel_width: 320,
    }
});

#[derive(Debug, Clone)]
pub struct ExifRwPlugin {
    id: String,
}

impl ExifRwPlugin {
    pub fn new() -> Self {
        Self { id: "photopipeline.plugins.exif_rw".to_string() }
    }
}

impl Default for ExifRwPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExifRwPlugin {
    fn id(&self) -> &PluginId { &self.id }
    fn name(&self) -> &str { "EXIF Reader/Writer" }
    fn version(&self) -> PluginVersion { PluginVersion::new(1, 0, 0) }
    fn category(&self) -> PluginCategory { PluginCategory::Metadata }
    fn description(&self) -> &str { "Read and write EXIF, XMP, IPTC, and GPS metadata via exiftool" }
    fn tags(&self) -> &[String] { &TAGS }
    fn requires_pixel_access(&self) -> bool { false }
    fn produces_pixel_output(&self) -> bool { false }
    fn supported_hardware(&self) -> HardwareRequirement { HardwareRequirement { min_ram_mb: 128, ..Default::default() } }

    fn parameter_schema(&self) -> &ParameterSchema { &PARAMETER_SCHEMA }
    fn gui_schema(&self) -> &GuiSchema { &GUI_SCHEMA }

    async fn initialize(&mut self, _cfg: &photopipeline_plugin::PluginConfig) -> PluginResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if let Some(v) = params.get_str("read_exif") {
            if v != "true" && v != "false" {
                issues.push(ValidationIssue::Error {
                    param: "read_exif".into(),
                    message: "Must be true or false".into(),
                });
            }
        }
        if let Some(v) = params.get_str("exiftool_path") {
            if v.is_empty() {
                issues.push(ValidationIssue::Error {
                    param: "exiftool_path".into(),
                    message: "ExifTool path cannot be empty".into(),
                });
            }
        }
        Ok(issues)
    }
}

#[async_trait]
impl MetadataProcessor for ExifRwPlugin {
    fn metadata_scope(&self) -> Vec<MetadataScope> {
        vec![MetadataScope::EXIF, MetadataScope::XMP, MetadataScope::IPTC, MetadataScope::GPS]
    }

    async fn read_metadata(
        &self, target: &MetadataTarget, params: &ParameterSet,
    ) -> PluginResult<Metadata> {
        let exiftool = params.get_str("exiftool_path").unwrap_or("exiftool");
        let mut cmd = Command::new(exiftool);
        cmd.arg("-json").arg("-G").arg(&target.path);

        let output = cmd.output().map_err(|e| PluginError::Io {
            plugin: self.id.clone(),
            error: e,
        })?;

        if !output.status.success() {
            return Err(PluginError::MissingTool {
                plugin: self.id.clone(),
                tool: exiftool.to_string(),
                required: "exiftool 12.00+".into(),
            });
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .map_err(|e| PluginError::Internal {
                plugin: self.id.clone(),
                message: format!("Failed to parse exiftool JSON output: {}", e),
            })?;

        if parsed.is_empty() {
            return Ok(Metadata::default());
        }

        let first = &parsed[0];
        let read_exif = params.get("read_exif").map(|v| v.as_bool().unwrap_or(true)).unwrap_or(true);
        let _read_xmp = params.get("read_xmp").map(|v| v.as_bool().unwrap_or(true)).unwrap_or(true);
        let _read_iptc = params.get("read_iptc").map(|v| v.as_bool().unwrap_or(true)).unwrap_or(true);
        let _read_gps = params.get("read_gps").map(|v| v.as_bool().unwrap_or(true)).unwrap_or(true);

        let mut metadata = Metadata::default();

        if read_exif {
            let mut exif = ExifData::default();
            if let Some(v) = first.get("Make") {
                exif.make = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("Model") {
                exif.model = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("LensModel") {
                exif.lens_model = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("SerialNumber") {
                exif.serial_number = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("Software") {
                exif.software = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("Artist") {
                exif.artist = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("Copyright") {
                exif.copyright = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("ImageDescription") {
                exif.image_description = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("Orientation") {
                exif.orientation = v.as_u64().map(|n| n as u16);
            }
            if let Some(v) = first.get("ISO") {
                exif.iso = v.as_u64().map(|n| n as u32);
            }
            if let Some(v) = first.get("ExposureTime") {
                exif.exposure_time = v.as_str().map(|s| s.to_string())
                    .or_else(|| v.as_f64().map(|f| f.to_string()));
            }
            if let Some(v) = first.get("FNumber") {
                exif.f_number = v.as_str().map(|s| s.to_string())
                    .or_else(|| v.as_f64().map(|f| f.to_string()));
            }
            if let Some(v) = first.get("FocalLength") {
                exif.focal_length = v.as_str().map(|s| s.to_string())
                    .or_else(|| v.as_f64().map(|f| format!("{}mm", f)));
            }
            if let Some(v) = first.get("ImageWidth") {
                exif.image_width = v.as_u64().map(|n| n as u32);
            }
            if let Some(v) = first.get("ImageHeight") {
                exif.image_height = v.as_u64().map(|n| n as u32);
            }
            if let Some(v) = first.get("Compression") {
                exif.compression = v.as_u64().map(|n| n as u16);
            }

            let mut raw_tags = Vec::new();
            for (key, val) in first.as_object().into_iter().flat_map(|o| o.iter()) {
                if key.contains(":") {
                    let mut parts = key.splitn(2, ':');
                    let group = parts.next().unwrap_or("").to_string();
                    let tag = parts.next().unwrap_or(key).to_string();
                    raw_tags.push(RawExifTag {
                        tag,
                        group,
                        value: val.to_string(),
                    });
                }
            }
            if !raw_tags.is_empty() {
                exif.raw_tags = raw_tags;
            }
            metadata.exif = Some(exif);
        }

        if _read_xmp {
            let mut xmp = XmpData::default();
            for (key, val) in first.as_object().into_iter().flat_map(|o| o.iter()) {
                let key_lower = key.to_lowercase();
                if key_lower.contains("xmp") {
                    if key_lower.contains("creator") || key_lower.contains("artist") {
                        if xmp.creator.is_none() {
                            xmp.creator = val.as_str().map(|s| s.to_string())
                                .or_else(|| val.as_f64().map(|f| f.to_string()));
                        }
                    }
                    if key_lower.contains("rights") || key_lower.contains("copyright") {
                        if xmp.rights.is_none() {
                            xmp.rights = val.as_str().map(|s| s.to_string());
                        }
                    }
                    if key_lower.contains("title") {
                        if xmp.title.is_none() {
                            xmp.title = val.as_str().map(|s| s.to_string());
                        }
                    }
                    if key_lower.contains("description") {
                        if xmp.description.is_none() {
                            xmp.description = val.as_str().map(|s| s.to_string());
                        }
                    }
                }
            }
            metadata.xmp = Some(xmp);
        }

        if _read_iptc {
            let mut iptc = IptcData::default();
            for (key, val) in first.as_object().into_iter().flat_map(|o| o.iter()) {
                let key_lower = key.to_lowercase();
                if key_lower.contains("iptc") {
                    if key_lower.contains("keywords") {
                        if let Some(arr) = val.as_array() {
                            iptc.keywords = arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                        } else if let Some(s) = val.as_str() {
                            iptc.keywords = s.split(',').map(|s| s.trim().to_string()).collect();
                        }
                    }
                    if key_lower.contains("city") && iptc.city.is_none() {
                        iptc.city = val.as_str().map(|s| s.to_string());
                    }
                    if key_lower.contains("country") && iptc.country.is_none() {
                        iptc.country = val.as_str().map(|s| s.to_string());
                    }
                    if key_lower.contains("state") && iptc.state.is_none() {
                        iptc.state = val.as_str().map(|s| s.to_string());
                    }
                    if key_lower.contains("caption") && iptc.caption.is_none() {
                        iptc.caption = val.as_str().map(|s| s.to_string());
                    }
                    if key_lower.contains("headline") && iptc.headline.is_none() {
                        iptc.headline = val.as_str().map(|s| s.to_string());
                    }
                }
            }
            metadata.iptc = Some(iptc);
        }

        if _read_gps {
            let mut gps = GpsData::default();
            if let Some(v) = first.get("GPSLatitude") {
                gps.latitude = v.as_str().and_then(parse_dms).or_else(|| v.as_f64());
            }
            if let Some(v) = first.get("GPSLongitude") {
                gps.longitude = v.as_str().and_then(parse_dms).or_else(|| v.as_f64());
            }
            if let Some(v) = first.get("GPSAltitude") {
                gps.altitude = v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64());
            }
            if let Some(v) = first.get("GPSLatitudeRef") {
                gps.latitude_ref = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("GPSLongitudeRef") {
                gps.longitude_ref = v.as_str().map(|s| s.to_string());
            }
            if let Some(v) = first.get("GPSImgDirection") {
                gps.img_direction = v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64());
            }
            metadata.gps = Some(gps);
        }

        Ok(metadata)
    }

    async fn write_metadata(
        &self, target: &mut MetadataTarget, metadata: &Metadata, params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport> {
        let exiftool = params.get_str("exiftool_path").unwrap_or("exiftool");
        let overwrite = params.get("overwrite_original")
            .map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);

        let mut tags_written: u32 = 0;
        let mut tags_skipped: u32 = 0;
        let mut warnings: Vec<String> = Vec::new();

        let which_exif = params.get_str("write_exif").unwrap_or("all");

        if which_exif != "none" {
            if let Some(ref exif) = metadata.exif {
                let mut cmd = Command::new(exiftool);
                if overwrite {
                    cmd.arg("-overwrite_original");
                }
                if let Some(ref make) = exif.make {
                    cmd.arg(format!("-Make={}", make)); tags_written += 1;
                }
                if let Some(ref model) = exif.model {
                    cmd.arg(format!("-Model={}", model)); tags_written += 1;
                }
                if let Some(ref lens) = exif.lens_model {
                    cmd.arg(format!("-LensModel={}", lens)); tags_written += 1;
                }
                if let Some(ref artist) = exif.artist {
                    cmd.arg(format!("-Artist={}", artist)); tags_written += 1;
                }
                if let Some(ref copyright) = exif.copyright {
                    cmd.arg(format!("-Copyright={}", copyright)); tags_written += 1;
                }
                if let Some(ref desc) = exif.image_description {
                    cmd.arg(format!("-ImageDescription={}", desc)); tags_written += 1;
                }
                if let Some(iso) = exif.iso {
                    cmd.arg(format!("-ISO={}", iso)); tags_written += 1;
                }
                if let Some(orientation) = exif.orientation {
                    cmd.arg(format!("-Orientation={}", orientation)); tags_written += 1;
                }
                cmd.arg(&target.path);

                let output = cmd.output().map_err(|e| PluginError::Io {
                    plugin: self.id.clone(),
                    error: e,
                })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warnings.push(format!("exiftool write error: {}", stderr));
                    tags_skipped = tags_written;
                    tags_written = 0;
                }
            }
        } else {
            tags_skipped += 1;
        }

        if let Some(ref xmp) = metadata.xmp {
            if let Some(ref creator) = xmp.creator {
                let mut cmd = Command::new(exiftool);
                if overwrite { cmd.arg("-overwrite_original"); }
                cmd.arg(format!("-XMP:Creator={}", creator));
                cmd.arg(&target.path);
                let result = cmd.output();
                match result {
                    Ok(o) if o.status.success() => { tags_written += 1; }
                    _ => { tags_skipped += 1; warnings.push("XMP writer failed".into()); }
                }
            }
        }

        if let Some(ref iptc) = metadata.iptc {
            if !iptc.keywords.is_empty() {
                let mut cmd = Command::new(exiftool);
                if overwrite { cmd.arg("-overwrite_original"); }
                for kw in &iptc.keywords {
                    cmd.arg(format!("-IPTC:Keywords+={}", kw));
                }
                cmd.arg(&target.path);
                let result = cmd.output();
                match result {
                    Ok(o) if o.status.success() => { tags_written += iptc.keywords.len() as u32; }
                    _ => { tags_skipped += iptc.keywords.len() as u32; }
                }
            }
        }

        Ok(MetadataWriteReport { tags_written, tags_skipped, warnings })
    }
}

fn parse_dms(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Ok(val) = s.parse::<f64>() {
        return Some(val);
    }
    let re = regex::Regex::new(r##"(\d+)°?\s*(\d+)'?\s*([\d.]+)""##).ok()?;
    let caps = re.captures(s)?;
    let deg: f64 = caps.get(1)?.as_str().parse().ok()?;
    let min: f64 = caps.get(2)?.as_str().parse().ok()?;
    let sec: f64 = caps.get(3)?.as_str().parse().ok()?;
    Some(deg + min / 60.0 + sec / 3600.0)
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "metadata".into(), "exif".into(), "xmp".into(), "iptc".into(),
        "gps".into(), "reader".into(), "writer".into(), "exiftool".into(),
    ]
});

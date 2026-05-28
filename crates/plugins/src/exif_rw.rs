use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

use exif::{Reader, Tag, Value};

use photopipeline_core::{
    ExifData, GpsData, HardwareRequirement, IptcData, Metadata, MetadataScope, MetadataTarget,
    MetadataWriteReport, PerfTimer, PluginCategory, PluginError, PluginId, PluginResult,
    PluginVersion, RawExifTag, ValidationIssue, XmpData,
};
use photopipeline_plugin::{
    EnumOption, GuiLayout, GuiSchema, GuiSection, MetadataProcessor, ParameterField,
    ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode,
    SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
                },
                ParameterField {
                    id: "write_exif".into(),
                    label: "Write EXIF Tags".into(),
                    description: Some("Write EXIF metadata fields".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "all".into(),
                                label: "All Tags".into(),
                                description: Some("Write all available EXIF tags".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "selected".into(),
                                label: "Selected Tags".into(),
                                description: Some("Write only explicitly set tags".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "none".into(),
                                label: "None".into(),
                                description: Some("Do not write EXIF".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("all"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                    ..Default::default()
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
                    ..Default::default()
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
                param_section_id: "read_options".into(),
                title_visible: true,
                style: SectionStyle::Default,
            },
            GuiSection {
                param_section_id: "write_options".into(),
                title_visible: true,
                style: SectionStyle::Default,
            },
            GuiSection {
                param_section_id: "exiftool".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("tag".into()),
    color: Some("#3b82f6".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug, Clone)]
pub struct ExifRwPlugin {
    id: String,
}

impl ExifRwPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.exif_rw".to_string(),
        }
    }
}

impl Default for ExifRwPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExifRwPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "EXIF Reader/Writer"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Metadata
    }
    fn description(&self) -> &str {
        "Read and write EXIF, XMP, IPTC, and GPS metadata via exiftool"
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
        tracing::info!(version = %PluginVersion::new(1,0,0), "exif_rw plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("exif_rw plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        tracing::debug!("exif_rw: validating parameters");
        if let Some(v) = params.get_str("read_exif")
            && v != "true"
            && v != "false"
        {
            issues.push(ValidationIssue::Error {
                param: "read_exif".into(),
                message: "Must be true or false".into(),
            });
        }
        if let Some(v) = params.get_str("exiftool_path")
            && v.is_empty()
        {
            issues.push(ValidationIssue::Error {
                param: "exiftool_path".into(),
                message: "ExifTool path cannot be empty".into(),
            });
        }
        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "exif_rw validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

/// Cached exiftool path resolution, shared across all plugins.
pub fn find_exiftool_path() -> Option<String> {
    use std::sync::OnceLock;
    static EXIFTOOL_PATH: OnceLock<Option<String>> = OnceLock::new();
    EXIFTOOL_PATH
        .get_or_init(|| find_exiftool_path_inner())
        .clone()
}

fn find_exiftool_path_inner() -> Option<String> {
    let exe_name: &str = if cfg!(windows) {
        "exiftool.exe"
    } else {
        "exiftool"
    };

    // 1. Check PHOTOPIPELINE_EXIFTOOL env var (set by build.rs at compile time)
    if let Some(path) = option_env!("PHOTOPIPELINE_EXIFTOOL") {
        let p = std::path::Path::new(path);
        if p.exists() {
            tracing::debug!(path = %path, "exiftool found via compile-time PHOTOPIPELINE_EXIFTOOL");
            return Some(path.to_string());
        }
    }

    // 2. Check PHOTOPIPELINE_EXIFTOOL env var at runtime (overrides compile-time)
    if let Ok(path) = std::env::var("PHOTOPIPELINE_EXIFTOOL") {
        let p = std::path::Path::new(&path);
        if p.exists() {
            tracing::debug!(path = %path, "exiftool found via runtime PHOTOPIPELINE_EXIFTOOL");
            return Some(path);
        }
    }

    // 3. Check next to the current executable (embedded)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let embedded = exe_dir.join(exe_name);
            if embedded.exists() {
                let path_str = embedded.to_string_lossy().to_string();
                tracing::debug!(path = %path_str, "exiftool found embedded next to binary");
                return Some(path_str);
            }
        }
    }

    // 4. Check vendor directory (development, including versioned subdirs)
    let vendor_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("vendor").join("exiftool"));

    if let Some(ref vd) = vendor_dir {
        // Direct file first
        let direct = vd.join(exe_name);
        if direct.exists() {
            let path_str = direct.to_string_lossy().to_string();
            tracing::debug!(path = %path_str, "exiftool found in vendor directory");
            return Some(path_str);
        }
        // Scan versioned subdirectories
        if let Ok(entries) = std::fs::read_dir(vd) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let exe = path.join(exe_name);
                    if exe.exists() {
                        let path_str = exe.to_string_lossy().to_string();
                        tracing::debug!(path = %path_str, "exiftool found in vendor subdirectory");
                        return Some(path_str);
                    }
                }
            }
        }
    }

    // 5. Fall back to $PATH
    let path_in_path = if cfg!(windows) {
        "exiftool.exe"
    } else {
        "exiftool"
    };
    if std::process::Command::new(path_in_path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        tracing::debug!("exiftool found in PATH");
        return Some(path_in_path.to_string());
    }

    None
}

fn resolve_exiftool(params: &ParameterSet) -> Option<String> {
    // Custom path from parameters takes priority
    if let Some(custom) = params.get_str("exiftool_path")
        && !custom.is_empty()
        && custom != "exiftool"
    {
        let p = std::path::Path::new(custom);
        if p.exists() {
            tracing::debug!(path = %custom, "exiftool via custom exiftool_path parameter");
            return Some(custom.to_string());
        }
        tracing::warn!(path = %custom, "exiftool_path specified but binary not found");
    }

    find_exiftool_path()
}

async fn read_metadata_via_kamadak(
    id: &PluginId,
    target: &MetadataTarget,
    params: &ParameterSet,
) -> PluginResult<Metadata> {
    let _timer = PerfTimer::with_target("read_metadata_exif", &format!("plugins.{}", id));

    let file_data = std::fs::read(&target.path).map_err(|e| PluginError::Io {
        plugin: id.clone(),
        error: e,
    })?;

    let mut cursor = std::io::Cursor::new(&file_data);
    let reader = Reader::new();
    let exif = reader
        .read_from_container(&mut cursor)
        .map_err(|e| PluginError::Internal {
            plugin: id.clone(),
            message: format!("EXIF parse: {}", e),
        })?;

    let read_exif = params
        .get("read_exif")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_xmp = params
        .get("read_xmp")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_iptc = params
        .get("read_iptc")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_gps = params
        .get("read_gps")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);

    let mut metadata = Metadata::default();

    if read_exif {
        let mut exif_data = ExifData::default();
        let mut gps = GpsData::default();
        let field_count = exif.fields().len();

        for field in exif.fields() {
            match field.tag {
                Tag::Make => exif_data.make = Some(field.display_value().to_string()),
                Tag::Model => exif_data.model = Some(field.display_value().to_string()),
                Tag::ISOSpeed => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.iso = Some(v);
                    }
                }
                Tag::ExposureTime => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(et) = v.first() {
                            exif_data.exposure_time = Some(format!("{}/{}", et.num, et.denom));
                        }
                    }
                }
                Tag::FNumber => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(fn_val) = v.first() {
                            exif_data.f_number =
                                Some(format!("{:.1}", fn_val.num as f64 / fn_val.denom as f64));
                        }
                    }
                }
                Tag::FocalLength => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(fl) = v.first() {
                            exif_data.focal_length =
                                Some(format!("{:.1}", fl.num as f64 / fl.denom as f64));
                        }
                    }
                }
                Tag::LensModel => exif_data.lens_model = Some(field.display_value().to_string()),
                Tag::Artist => exif_data.artist = Some(field.display_value().to_string()),
                Tag::Copyright => exif_data.copyright = Some(field.display_value().to_string()),
                Tag::ImageDescription => {
                    exif_data.image_description = Some(field.display_value().to_string())
                }
                Tag::Orientation => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.orientation = Some(v as u16);
                    }
                }
                Tag::Software => exif_data.software = Some(field.display_value().to_string()),
                Tag::BodySerialNumber => {
                    exif_data.serial_number = Some(field.display_value().to_string())
                }
                Tag::ImageWidth => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.image_width = Some(v);
                    }
                }
                Tag::ImageLength => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.image_height = Some(v);
                    }
                }
                Tag::Compression => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.compression = Some(v as u16);
                    }
                }
                Tag::DateTimeOriginal => {
                    exif_data.date_time_original =
                        parse_exif_date_time(&field.display_value().to_string());
                }
                Tag::DateTimeDigitized => {
                    exif_data.date_time_digitized =
                        parse_exif_date_time(&field.display_value().to_string());
                }
                Tag::SubSecTimeOriginal => {
                    exif_data.sub_sec_time_original = Some(field.display_value().to_string());
                }
                Tag::OffsetTimeOriginal => {
                    exif_data.offset_time_original = Some(field.display_value().to_string());
                }
                Tag::ExposureBiasValue => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(eb) = v.first() {
                            exif_data.exposure_bias = Some(format!("{}/{}", eb.num, eb.denom));
                        }
                    }
                }
                Tag::MeteringMode => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.metering_mode = Some(v as u16);
                    }
                }
                Tag::Flash => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.flash = Some(v as u16);
                    }
                }
                Tag::ExposureProgram => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.exposure_program = Some(v as u16);
                    }
                }
                Tag::WhiteBalance => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.white_balance = Some(v as u16);
                    }
                }
                Tag::ColorSpace => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.color_space = Some(v as u16);
                    }
                }
                Tag::ApertureValue => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(av) = v.first() {
                            exif_data.aperture_value =
                                Some(format!("{:.1}", av.num as f64 / av.denom as f64));
                        }
                    }
                }
                Tag::ShutterSpeedValue => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(sv) = v.first() {
                            exif_data.shutter_speed_value =
                                Some(format!("{}/{}", sv.num, sv.denom));
                        }
                    }
                }
                Tag::FocalLengthIn35mmFilm => {
                    if let Some(v) = field.value.get_uint(0) {
                        exif_data.focal_length_35mm = Some(v as u16);
                    }
                }
                Tag::BitsPerSample => {
                    let mut bps = Vec::new();
                    let mut i = 0;
                    while let Some(v) = field.value.get_uint(i) {
                        bps.push(v as u16);
                        i += 1;
                    }
                    if !bps.is_empty() {
                        exif_data.bits_per_sample = Some(bps);
                    }
                }
                Tag::MakerNote => {
                    exif_data.maker_note = Some(field.display_value().to_string().into_bytes());
                }
                // GPS tags
                Tag::GPSLatitude => {
                    if let Value::Rational(ref values) = field.value {
                        if values.len() == 3 {
                            let deg = values[0].num as f64 / values[0].denom as f64;
                            let min = values[1].num as f64 / values[1].denom as f64;
                            let sec = values[2].num as f64 / values[2].denom as f64;
                            gps.latitude = Some(deg + min / 60.0 + sec / 3600.0);
                        }
                    }
                }
                Tag::GPSLatitudeRef => {
                    gps.latitude_ref = Some(field.display_value().to_string());
                }
                Tag::GPSLongitude => {
                    if let Value::Rational(ref values) = field.value {
                        if values.len() == 3 {
                            let deg = values[0].num as f64 / values[0].denom as f64;
                            let min = values[1].num as f64 / values[1].denom as f64;
                            let sec = values[2].num as f64 / values[2].denom as f64;
                            gps.longitude = Some(deg + min / 60.0 + sec / 3600.0);
                        }
                    }
                }
                Tag::GPSLongitudeRef => {
                    gps.longitude_ref = Some(field.display_value().to_string());
                }
                Tag::GPSAltitude => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(alt) = v.first() {
                            gps.altitude = Some(alt.num as f64 / alt.denom as f64);
                        }
                    }
                }
                Tag::GPSAltitudeRef => {
                    if let Some(v) = field.value.get_uint(0) {
                        gps.altitude_ref = Some(v as i8);
                    }
                }
                Tag::GPSImgDirection => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(dir) = v.first() {
                            gps.img_direction = Some(dir.num as f64 / dir.denom as f64);
                        }
                    }
                }
                Tag::GPSImgDirectionRef => {
                    gps.img_direction_ref = Some(field.display_value().to_string());
                }
                Tag::GPSSpeed => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(spd) = v.first() {
                            gps.speed = Some(spd.num as f64 / spd.denom as f64);
                        }
                    }
                }
                Tag::GPSSpeedRef => {
                    gps.speed_ref = Some(field.display_value().to_string());
                }
                Tag::GPSMapDatum => {
                    gps.map_datum = Some(field.display_value().to_string());
                }
                Tag::GPSDateStamp => {
                    gps.date_stamp = Some(field.display_value().to_string());
                }
                Tag::GPSSatellites => {
                    gps.satellites = Some(field.display_value().to_string());
                }
                Tag::GPSStatus => {
                    gps.status = Some(field.display_value().to_string());
                }
                Tag::GPSMeasureMode => {
                    gps.measure_mode = Some(field.display_value().to_string());
                }
                Tag::GPSDOP => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(dop) = v.first() {
                            gps.dop = Some(dop.num as f64 / dop.denom as f64);
                        }
                    }
                }
                Tag::GPSTrack => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(trk) = v.first() {
                            gps.track = Some(trk.num as f64 / trk.denom as f64);
                        }
                    }
                }
                Tag::GPSTrackRef => {
                    gps.track_ref = Some(field.display_value().to_string());
                }
                Tag::GPSDestBearing => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(brg) = v.first() {
                            gps.dest_bearing = Some(brg.num as f64 / brg.denom as f64);
                        }
                    }
                }
                Tag::GPSDestBearingRef => {
                    gps.dest_bearing_ref = Some(field.display_value().to_string());
                }
                Tag::GPSDestDistance => {
                    if let Value::Rational(ref v) = field.value {
                        if let Some(dist) = v.first() {
                            gps.dest_distance = Some(dist.num as f64 / dist.denom as f64);
                        }
                    }
                }
                Tag::GPSDestLatitude => {
                    if let Value::Rational(ref values) = field.value {
                        if values.len() == 3 {
                            let deg = values[0].num as f64 / values[0].denom as f64;
                            let min = values[1].num as f64 / values[1].denom as f64;
                            let sec = values[2].num as f64 / values[2].denom as f64;
                            gps.dest_latitude = Some(deg + min / 60.0 + sec / 3600.0);
                        }
                    }
                }
                Tag::GPSDestLongitude => {
                    if let Value::Rational(ref values) = field.value {
                        if values.len() == 3 {
                            let deg = values[0].num as f64 / values[0].denom as f64;
                            let min = values[1].num as f64 / values[1].denom as f64;
                            let sec = values[2].num as f64 / values[2].denom as f64;
                            gps.dest_longitude = Some(deg + min / 60.0 + sec / 3600.0);
                        }
                    }
                }
                Tag::GPSProcessingMethod => {
                    gps.processing_method = Some(field.display_value().to_string());
                }
                Tag::GPSAreaInformation => {
                    gps.area_information = Some(field.display_value().to_string());
                }
                _ => {
                    exif_data.raw_tags.push(RawExifTag {
                        tag: format!("{:?}", field.tag),
                        group: format!("{:?}", field.ifd_num),
                        value: field.display_value().to_string(),
                    });
                }
            }
        }

        metadata.exif = Some(exif_data);
        if _read_gps && gps.latitude.is_some() {
            metadata.gps = Some(gps);
        }

        tracing::debug!(
            target = format!("plugins.{}", id),
            "EXIF read (kamadak): {} fields",
            field_count,
        );
    }

    Ok(metadata)
}

pub(crate) fn parse_exif_date_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = s.trim();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
        return Some(chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&chrono::Utc));
    }
    None
}

async fn read_metadata_via_exiftool(
    id: &PluginId,
    target: &MetadataTarget,
    params: &ParameterSet,
) -> PluginResult<Metadata> {
    let _timer = PerfTimer::with_target("exif_rw_read_metadata", "plugin.exif_rw");
    let exiftool = resolve_exiftool(params).ok_or_else(|| PluginError::MissingTool {
        plugin: id.clone(),
        tool: "exiftool".into(),
        required: "exiftool 12.00+".into(),
    })?;

    tracing::info!(target_path = %target.path, exiftool = %exiftool, "exif_rw: reading metadata from {}", target.path);
    tracing::debug!(
        exiftool_cmd = %exiftool,
        args = "-json -G",
        "exif_rw: running exiftool to read metadata",
    );

    let mut cmd = Command::new(&exiftool);
    cmd.arg("-json").arg("-G").arg(&target.path);

    let output = cmd.output().map_err(|e| PluginError::Io {
        plugin: id.clone(),
        error: e,
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(
            tool = exiftool,
            exit_code = ?output.status.code(),
            stderr = %stderr,
            "exif_rw: exiftool read failed",
        );
        return Err(PluginError::MissingTool {
            plugin: id.clone(),
            tool: exiftool.to_string(),
            required: "exiftool 12.00+".into(),
        });
    }

    tracing::trace!(
        stdout_len = output.stdout.len(),
        "exif_rw: exiftool produced {} bytes of JSON output",
        output.stdout.len()
    );

    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).map_err(|e| PluginError::Internal {
            plugin: id.clone(),
            message: format!("Failed to parse exiftool JSON output: {}", e),
        })?;

    if parsed.is_empty() {
        return Ok(Metadata::default());
    }

    let first = &parsed[0];
    let read_exif = params
        .get("read_exif")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_xmp = params
        .get("read_xmp")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_iptc = params
        .get("read_iptc")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);
    let _read_gps = params
        .get("read_gps")
        .map(|v| v.as_bool().unwrap_or(true))
        .unwrap_or(true);

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
            exif.exposure_time = v
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| v.as_f64().map(|f| f.to_string()));
        }
        if let Some(v) = first.get("FNumber") {
            exif.f_number = v
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| v.as_f64().map(|f| f.to_string()));
        }
        if let Some(v) = first.get("FocalLength") {
            exif.focal_length = v
                .as_str()
                .map(|s| s.to_string())
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
                if (key_lower.contains("creator") || key_lower.contains("artist"))
                    && xmp.creator.is_none()
                {
                    xmp.creator = val
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| val.as_f64().map(|f| f.to_string()));
                }
                if (key_lower.contains("rights") || key_lower.contains("copyright"))
                    && xmp.rights.is_none()
                {
                    xmp.rights = val.as_str().map(|s| s.to_string());
                }
                if key_lower.contains("title") && xmp.title.is_none() {
                    xmp.title = val.as_str().map(|s| s.to_string());
                }
                if key_lower.contains("description") && xmp.description.is_none() {
                    xmp.description = val.as_str().map(|s| s.to_string());
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
                        iptc.keywords = arr
                            .iter()
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
            gps.altitude = v
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64());
        }
        if let Some(v) = first.get("GPSLatitudeRef") {
            gps.latitude_ref = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = first.get("GPSLongitudeRef") {
            gps.longitude_ref = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = first.get("GPSImgDirection") {
            gps.img_direction = v
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64());
        }
        metadata.gps = Some(gps);
    }

    Ok(metadata)
}

async fn write_metadata_via_exiftool(
    id: &PluginId,
    target: &mut MetadataTarget,
    metadata: &Metadata,
    params: &ParameterSet,
) -> PluginResult<MetadataWriteReport> {
    let _timer = PerfTimer::with_target("exif_rw_write_metadata", "plugin.exif_rw");
    let exiftool = resolve_exiftool(params).ok_or_else(|| PluginError::MissingTool {
        plugin: id.clone(),
        tool: "exiftool".into(),
        required: "exiftool 12.00+".into(),
    })?;
    let overwrite = params
        .get("overwrite_original")
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false);

    let mut tags_written: u32 = 0;
    let mut tags_skipped: u32 = 0;
    let mut warnings: Vec<String> = Vec::new();

    let which_exif = params.get_str("write_exif").unwrap_or("all");

    if which_exif != "none" {
        if let Some(ref exif) = metadata.exif {
            let mut cmd = Command::new(&exiftool);
            if overwrite {
                cmd.arg("-overwrite_original");
            }
            if let Some(ref make) = exif.make {
                cmd.arg(format!("-Make={}", make));
                tags_written += 1;
            }
            if let Some(ref model) = exif.model {
                cmd.arg(format!("-Model={}", model));
                tags_written += 1;
            }
            if let Some(ref lens) = exif.lens_model {
                cmd.arg(format!("-LensModel={}", lens));
                tags_written += 1;
            }
            if let Some(ref artist) = exif.artist {
                cmd.arg(format!("-Artist={}", artist));
                tags_written += 1;
            }
            if let Some(ref copyright) = exif.copyright {
                cmd.arg(format!("-Copyright={}", copyright));
                tags_written += 1;
            }
            if let Some(ref desc) = exif.image_description {
                cmd.arg(format!("-ImageDescription={}", desc));
                tags_written += 1;
            }
            if let Some(iso) = exif.iso {
                cmd.arg(format!("-ISO={}", iso));
                tags_written += 1;
            }
            if let Some(orientation) = exif.orientation {
                cmd.arg(format!("-Orientation={}", orientation));
                tags_written += 1;
            }
            cmd.arg(&target.path);

            let output = cmd.output().map_err(|e| PluginError::Io {
                plugin: id.clone(),
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

    if let Some(ref xmp) = metadata.xmp
        && let Some(ref creator) = xmp.creator
    {
        let mut cmd = Command::new(&exiftool);
        if overwrite {
            cmd.arg("-overwrite_original");
        }
        cmd.arg(format!("-XMP:Creator={}", creator));
        cmd.arg(&target.path);
        let result = cmd.output();
        match result {
            Ok(o) if o.status.success() => {
                tags_written += 1;
            }
            _ => {
                tags_skipped += 1;
                warnings.push("XMP writer failed".into());
            }
        }
    }

    if let Some(ref iptc) = metadata.iptc
        && !iptc.keywords.is_empty()
    {
        let mut cmd = Command::new(&exiftool);
        if overwrite {
            cmd.arg("-overwrite_original");
        }
        for kw in &iptc.keywords {
            cmd.arg(format!("-IPTC:Keywords+={}", kw));
        }
        cmd.arg(&target.path);
        let result = cmd.output();
        match result {
            Ok(o) if o.status.success() => {
                tags_written += iptc.keywords.len() as u32;
            }
            _ => {
                tags_skipped += iptc.keywords.len() as u32;
            }
        }
    }

    tracing::info!(
        tags_written = tags_written,
        tags_skipped = tags_skipped,
        "exif_rw: wrote {} tags, skipped {} tags",
        tags_written,
        tags_skipped,
    );

    Ok(MetadataWriteReport {
        tags_written,
        tags_skipped,
        warnings,
    })
}

#[async_trait]
impl MetadataProcessor for ExifRwPlugin {
    fn metadata_scope(&self) -> Vec<MetadataScope> {
        vec![
            MetadataScope::EXIF,
            MetadataScope::XMP,
            MetadataScope::IPTC,
            MetadataScope::GPS,
        ]
    }

    async fn read_metadata(
        &self,
        target: &MetadataTarget,
        params: &ParameterSet,
    ) -> PluginResult<Metadata> {
        match read_metadata_via_kamadak(&self.id, target, params).await {
            Ok(metadata) => Ok(metadata),
            Err(e) => {
                tracing::warn!(
                    target = format!("plugins.{}", self.id()),
                    error = %e,
                    "kamadak-exif read failed, falling back to exiftool",
                );
                if resolve_exiftool(params).is_some() {
                    read_metadata_via_exiftool(&self.id, target, params).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn write_metadata(
        &self,
        target: &mut MetadataTarget,
        metadata: &Metadata,
        params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport> {
        if let Some(_exiftool) = resolve_exiftool(params) {
            write_metadata_via_exiftool(&self.id, target, metadata, params).await
        } else {
            tracing::warn!(
                target = format!("plugins.{}", self.id()),
                "exiftool not available and kamadak-exif write is not yet fully implemented",
            );
            Ok(MetadataWriteReport {
                tags_written: 0,
                tags_skipped: 1,
                warnings: vec!["exiftool not available for writing".into()],
            })
        }
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
        "metadata".into(),
        "exif".into(),
        "xmp".into(),
        "iptc".into(),
        "gps".into(),
        "reader".into(),
        "writer".into(),
        "exiftool".into(),
    ]
});

use async_trait::async_trait;
use std::sync::LazyLock;

use photopipeline_core::{
    GpsData, GpxTrack, HardwareRequirement, Metadata, MetadataScope, MetadataTarget,
    MetadataWriteReport, PerfTimer, PluginCategory, PluginError, PluginId, PluginResult,
    PluginVersion, ValidationIssue,
};
use photopipeline_plugin::{
    AuxView, EnumOption, GuiLayout, GuiSchema, GuiSection, MetadataProcessor, ParameterField,
    ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin, PreviewMode,
    SectionStyle,
};

static PARAMETER_SCHEMA: LazyLock<ParameterSchema> = LazyLock::new(|| ParameterSchema {
    version: 1,
    sections: vec![
        ParameterSection {
            id: "source".into(),
            label: "GPS Source".into(),
            description: Some("Choose how GPS coordinates are determined".into()),
            icon: Some("crosshair".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "gps_mode".into(),
                    label: "Mode".into(),
                    description: Some("Method for assigning GPS coordinates".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "manual".into(),
                                label: "Manual Entry".into(),
                                description: Some("Enter coordinates manually".into()),
                                icon: Some("pencil".into()),
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "gpx_track".into(),
                                label: "GPX Track".into(),
                                description: Some("Interpolate from GPX track log".into()),
                                icon: Some("route".into()),
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "clear".into(),
                                label: "Clear GPS".into(),
                                description: Some("Remove all GPS data".into()),
                                icon: Some("trash".into()),
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("manual"),
                    required: true,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "gpx_file".into(),
                    label: "GPX File".into(),
                    description: Some("Path to a GPX track file for interpolation".into()),
                    help_url: None,
                    field_type: ParameterType::FilePath {
                        kind: Default::default(),
                        filters: vec![
                            ("GPX Files".into(), "*.gpx".into()),
                            ("All Files".into(), "*".into()),
                        ],
                        must_exist: true,
                    },
                    default: serde_json::json!(""),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "manual_coords".into(),
            label: "Manual Coordinates".into(),
            description: Some("Enter GPS coordinates directly".into()),
            icon: Some("map-pin".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "latitude".into(),
                    label: "Latitude".into(),
                    description: Some("Latitude in decimal degrees (-90 to 90)".into()),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: -90.0,
                        max: 90.0,
                        step: 0.000001,
                        precision: 6,
                        unit: Some("deg".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "longitude".into(),
                    label: "Longitude".into(),
                    description: Some("Longitude in decimal degrees (-180 to 180)".into()),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: -180.0,
                        max: 180.0,
                        step: 0.000001,
                        precision: 6,
                        unit: Some("deg".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "altitude".into(),
                    label: "Altitude".into(),
                    description: Some("Altitude in meters above sea level".into()),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: -500.0,
                        max: 9000.0,
                        step: 0.1,
                        precision: 1,
                        unit: Some("m".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0.0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "gpx_options".into(),
            label: "GPX Options".into(),
            description: Some("Configure GPX track interpolation behavior".into()),
            icon: Some("sliders".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "time_offset_seconds".into(),
                    label: "Time Offset".into(),
                    description: Some("Offset between camera clock and GPS time in seconds".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: -86400,
                        max: 86400,
                        step: 1,
                        unit: Some("s".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "max_interpolation_gap".into(),
                    label: "Max Gap".into(),
                    description: Some(
                        "Maximum seconds between GPX points to allow interpolation".into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 1,
                        max: 3600,
                        step: 1,
                        unit: Some("s".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(300),
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
                param_section_id: "source".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "manual_coords".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "gpx_options".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("map-pin".into()),
    color: Some("#10b981".into()),
    preview: PreviewMode::None,
    aux_views: vec![AuxView::Map],
    min_panel_width: 340,
});

#[derive(Debug, Clone)]
pub struct GpsSetPlugin {
    id: String,
}

impl GpsSetPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.gps_set".to_string(),
        }
    }
}

impl Default for GpsSetPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GpsSetPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "GPS Coordinate Manager"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Metadata
    }
    fn description(&self) -> &str {
        "Set GPS coordinates manually or interpolated from GPX track logs"
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
            min_ram_mb: 64,
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
        tracing::info!("gps_set plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("gps_set plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let mode = params.get_str("gps_mode").unwrap_or("manual");
        tracing::debug!("gps_set: validating parameters (mode={})", mode);

        if mode == "manual" {
            if let Some(lat) = params.get("latitude")
                && let Some(lat_val) = lat.as_f64()
                && (!(-90.0..=90.0).contains(&lat_val))
            {
                issues.push(ValidationIssue::Error {
                    param: "latitude".into(),
                    message: "Latitude must be between -90 and 90".into(),
                });
            }
            if let Some(lon) = params.get("longitude")
                && let Some(lon_val) = lon.as_f64()
                && (!(-180.0..=180.0).contains(&lon_val))
            {
                issues.push(ValidationIssue::Error {
                    param: "longitude".into(),
                    message: "Longitude must be between -180 and 180".into(),
                });
            }
        }

        if mode == "gpx_track"
            && let Some(gpx) = params.get_str("gpx_file")
            && gpx.is_empty()
        {
            issues.push(ValidationIssue::Error {
                param: "gpx_file".into(),
                message: "GPX file path is required for GPX track mode".into(),
            });
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "gps_set validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl MetadataProcessor for GpsSetPlugin {
    fn metadata_scope(&self) -> Vec<MetadataScope> {
        vec![MetadataScope::GPS]
    }

    async fn read_metadata(
        &self,
        _target: &MetadataTarget,
        _params: &ParameterSet,
    ) -> PluginResult<Metadata> {
        tracing::trace!("gps_set: read_metadata (no-op)");
        Ok(Metadata::default())
    }

    async fn write_metadata(
        &self,
        target: &mut MetadataTarget,
        metadata: &Metadata,
        params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport> {
        let _timer = PerfTimer::with_target("gps_set_write_metadata", "plugin.gps_set");
        let mode = params.get_str("gps_mode").unwrap_or("manual");
        tracing::info!(target_path = %target.path, mode = mode, "gps_set: writing GPS metadata to {} (mode={})", target.path, mode);
        let mut gps = GpsData::default();

        match mode {
            "manual" => {
                let lat = params.get("latitude").and_then(|v| v.as_f64());
                let lon = params.get("longitude").and_then(|v| v.as_f64());
                let alt = params.get("altitude").and_then(|v| v.as_f64());

                if lat.is_some() && lon.is_some() {
                    gps.latitude = lat;
                    gps.longitude = lon;
                    gps.latitude_ref = lat.map(|l| if l >= 0.0 { "N".into() } else { "S".into() });
                    gps.longitude_ref = lon.map(|l| if l >= 0.0 { "E".into() } else { "W".into() });
                    gps.altitude = alt;
                    gps.timestamp = Some(chrono::Utc::now());
                    gps.map_datum = Some("WGS-84".into());
                }
            }
            "gpx_track" => {
                let gpx_path = params.get_str("gpx_file").unwrap_or("");
                if gpx_path.is_empty() {
                    return Err(PluginError::InvalidParameter {
                        plugin: self.id.clone(),
                        field: "gpx_file".into(),
                        message: "GPX file path is required".into(),
                    });
                }

                let max_gap = params.get_i64("max_interpolation_gap").unwrap_or(300) as f64;

                let gpx_content =
                    std::fs::read_to_string(gpx_path).map_err(|e| PluginError::Io {
                        plugin: self.id.clone(),
                        error: e,
                    })?;
                let track = parse_gpx(&gpx_content).unwrap_or(GpxTrack {
                    name: None,
                    points: vec![],
                    duration_seconds: None,
                    distance_meters: None,
                });
                let time_offset = params.get_i64("time_offset_seconds").unwrap_or(0);

                if let Some(ref existing_gps) = metadata.gps {
                    let ts = existing_gps.timestamp.unwrap_or_else(chrono::Utc::now);
                    let adjusted = ts + chrono::Duration::seconds(time_offset);

                    if let Some(point) = track.interpolate_at(&adjusted) {
                        let gap_ok = check_interpolation_gap(&track, &adjusted, max_gap);
                        if !gap_ok {
                            tracing::warn!(
                                "GPS interpolation gap exceeds max_interpolation_gap of {}s for image at {}",
                                max_gap,
                                adjusted
                            );
                        }
                        gps.latitude = Some(point.latitude);
                        gps.longitude = Some(point.longitude);
                        gps.altitude = point.elevation;
                        gps.latitude_ref = Some(if point.latitude >= 0.0 {
                            "N".into()
                        } else {
                            "S".into()
                        });
                        gps.longitude_ref = Some(if point.longitude >= 0.0 {
                            "E".into()
                        } else {
                            "W".into()
                        });
                        gps.timestamp = Some(adjusted);
                        gps.img_direction = point.bearing;
                        gps.speed = point.speed;
                    }
                } else if !track.points.is_empty() {
                    let first = &track.points[0];
                    gps.latitude = Some(first.latitude);
                    gps.longitude = Some(first.longitude);
                    gps.altitude = first.elevation;
                    gps.latitude_ref = Some(if first.latitude >= 0.0 {
                        "N".into()
                    } else {
                        "S".into()
                    });
                    gps.longitude_ref = Some(if first.longitude >= 0.0 {
                        "E".into()
                    } else {
                        "W".into()
                    });
                    gps.timestamp = Some(chrono::Utc::now());
                }
            }
            "clear" => {}
            _ => {}
        }

        tracing::debug!(
            tool = "exiftool",
            "gps_set: invoking exiftool to write GPS data"
        );
        let mut cmd = std::process::Command::new("exiftool");
        cmd.arg("-overwrite_original");

        if let Some(lat) = gps.latitude {
            cmd.arg(format!("-GPSLatitude={}", lat));
        }
        if let Some(lon) = gps.longitude {
            cmd.arg(format!("-GPSLongitude={}", lon));
        }
        if let Some(alt) = gps.altitude {
            cmd.arg(format!("-GPSAltitude={}", alt));
        }
        if let Some(ref lat_ref) = gps.latitude_ref {
            cmd.arg(format!("-GPSLatitudeRef={}", lat_ref));
        }
        if let Some(ref lon_ref) = gps.longitude_ref {
            cmd.arg(format!("-GPSLongitudeRef={}", lon_ref));
        }
        if let Some(dir) = gps.img_direction {
            cmd.arg(format!("-GPSImgDirection={}", dir));
        }
        if let Some(speed) = gps.speed {
            cmd.arg(format!("-GPSSpeed={}", speed));
        }
        if let Some(ts) = gps.timestamp {
            cmd.arg(format!("-GPSDateStamp={}", ts.format("%Y:%m:%d")));
            cmd.arg(format!("-GPSTimeStamp={}", ts.format("%H:%M:%S")));
        }

        cmd.arg(&target.path);

        let output = cmd.output().map_err(|e| PluginError::Io {
            plugin: self.id.clone(),
            error: e,
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(
                tool = "exiftool",
                exit_code = ?output.status.code(),
                stderr = %stderr,
                "gps_set: exiftool write failed",
            );
        } else {
            tracing::trace!(
                stdout_len = output.stdout.len(),
                "gps_set: exiftool write succeeded"
            );
        }

        let tags_written = if output.status.success() { 8 } else { 0 };
        let warnings = if !output.status.success() {
            vec![String::from_utf8_lossy(&output.stderr).to_string()]
        } else {
            vec![]
        };

        tracing::info!(
            tags_written = tags_written,
            "gps_set: wrote {} GPS tags",
            tags_written,
        );

        Ok(MetadataWriteReport {
            tags_written,
            tags_skipped: 0,
            warnings,
        })
    }
}

fn check_interpolation_gap(
    track: &GpxTrack,
    ts: &chrono::DateTime<chrono::Utc>,
    max_gap: f64,
) -> bool {
    let target_ms = ts.timestamp_millis();
    let timed_points: Vec<_> = track
        .points
        .iter()
        .filter(|p| p.timestamp.is_some())
        .collect();

    if timed_points.len() < 2 {
        return true;
    }

    let mut prev_ts: Option<i64> = None;
    for pt in &timed_points {
        let pt_ms = pt.timestamp.unwrap().timestamp_millis();
        if pt_ms <= target_ms {
            prev_ts = Some(pt_ms);
        }
        if pt_ms >= target_ms {
            if let Some(prev) = prev_ts
                && (pt_ms - prev) as f64 / 1000.0 > max_gap
            {
                return false;
            }
            break;
        }
    }
    true
}

fn parse_gpx(content: &str) -> Option<GpxTrack> {
    let re_trkpt =
        regex::Regex::new(r#"<trkpt\s+lat="([^"]+)"\s+lon="([^"]+)"[^>]*>(.*?)</trkpt>"#).ok()?;
    let re_ele = regex::Regex::new(r"<ele>([^<]+)</ele>").ok()?;
    let re_time = regex::Regex::new(r"<time>([^<]+)</time>").ok()?;
    let re_name = regex::Regex::new(r"<name>([^<]+)</name>").ok()?;

    let mut points = Vec::new();
    for caps in re_trkpt.captures_iter(content) {
        let lat: f64 = caps.get(1)?.as_str().parse().ok()?;
        let lon: f64 = caps.get(2)?.as_str().parse().ok()?;
        let body = caps.get(3)?.as_str();

        let ele = re_ele
            .captures(body)
            .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok());
        let ts = re_time.captures(body).and_then(|c| {
            let s = c.get(1)?.as_str();
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .or_else(|| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                        .ok()
                        .map(|naive| {
                            chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc)
                        })
                })
        });

        points.push(photopipeline_core::GpxPoint {
            latitude: lat,
            longitude: lon,
            elevation: ele,
            timestamp: ts,
            speed: None,
            bearing: None,
        });
    }

    if points.is_empty() {
        return None;
    }

    let name = re_name
        .captures(content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let duration = if points.len() >= 2 {
        points
            .last()
            .and_then(|last| last.timestamp)
            .zip(points.first().and_then(|first| first.timestamp))
            .map(|(end, start)| (end - start).num_milliseconds() as f64 / 1000.0)
    } else {
        None
    };

    Some(GpxTrack {
        name,
        points,
        duration_seconds: duration,
        distance_meters: None,
    })
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "metadata".into(),
        "gps".into(),
        "geo".into(),
        "location".into(),
        "gpx".into(),
        "coordinates".into(),
        "track".into(),
    ]
});

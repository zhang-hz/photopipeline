use async_trait::async_trait;
use std::process::Command;
use std::sync::LazyLock;

use photopipeline_core::{
    HardwareRequirement, Metadata, MetadataScope, MetadataTarget, MetadataWriteReport, PerfTimer,
    PluginCategory, PluginError, PluginId, PluginResult, PluginVersion, ValidationIssue,
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
            id: "time_shift".into(),
            label: "Time Adjustment".into(),
            description: Some("Shift the capture timestamp by a specified offset".into()),
            icon: Some("clock".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "shift_hours".into(),
                    label: "Hours".into(),
                    description: Some("Hours to shift (negative for earlier)".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: -23,
                        max: 23,
                        step: 1,
                        unit: Some("h".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "shift_minutes".into(),
                    label: "Minutes".into(),
                    description: Some("Minutes to shift".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: -59,
                        max: 59,
                        step: 1,
                        unit: Some("min".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "shift_seconds".into(),
                    label: "Seconds".into(),
                    description: Some("Seconds to shift".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: -59,
                        max: 59,
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
            ],
        },
        ParameterSection {
            id: "timezone".into(),
            label: "Timezone".into(),
            description: Some("Convert timezone of the capture timestamp".into()),
            icon: Some("globe".into()),
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "source_timezone".into(),
                    label: "Source Timezone".into(),
                    description: Some("Timezone that the image timestamp is currently in".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "UTC".into(),
                                label: "UTC".into(),
                                description: Some("Coordinated Universal Time".into()),
                                icon: None,
                                tags: vec!["utc".into()],
                                recommended: true,
                            },
                            EnumOption {
                                value: "local".into(),
                                label: "Local".into(),
                                description: Some("System local timezone".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "America/New_York".into(),
                                label: "America/New_York".into(),
                                description: Some("Eastern Time (US)".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Europe/London".into(),
                                label: "Europe/London".into(),
                                description: Some("Greenwich Mean / British Summer Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Europe/Paris".into(),
                                label: "Europe/Paris".into(),
                                description: Some("Central European Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Asia/Tokyo".into(),
                                label: "Asia/Tokyo".into(),
                                description: Some("Japan Standard Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Asia/Shanghai".into(),
                                label: "Asia/Shanghai".into(),
                                description: Some("China Standard Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Australia/Sydney".into(),
                                label: "Australia/Sydney".into(),
                                description: Some("Australian Eastern Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Pacific/Auckland".into(),
                                label: "Pacific/Auckland".into(),
                                description: Some("New Zealand Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("UTC"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "target_timezone".into(),
                    label: "Target Timezone".into(),
                    description: Some("Timezone to convert the timestamp to".into()),
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            EnumOption {
                                value: "UTC".into(),
                                label: "UTC".into(),
                                description: Some("Coordinated Universal Time".into()),
                                icon: None,
                                tags: vec!["utc".into()],
                                recommended: false,
                            },
                            EnumOption {
                                value: "local".into(),
                                label: "Local".into(),
                                description: Some("System local timezone".into()),
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            EnumOption {
                                value: "America/New_York".into(),
                                label: "America/New_York".into(),
                                description: Some("Eastern Time (US)".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Europe/London".into(),
                                label: "Europe/London".into(),
                                description: Some("Greenwich Mean / British Summer Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Europe/Paris".into(),
                                label: "Europe/Paris".into(),
                                description: Some("Central European Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Asia/Tokyo".into(),
                                label: "Asia/Tokyo".into(),
                                description: Some("Japan Standard Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Asia/Shanghai".into(),
                                label: "Asia/Shanghai".into(),
                                description: Some("China Standard Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Australia/Sydney".into(),
                                label: "Australia/Sydney".into(),
                                description: Some("Australian Eastern Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                            EnumOption {
                                value: "Pacific/Auckland".into(),
                                label: "Pacific/Auckland".into(),
                                description: Some("New Zealand Time".into()),
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("local"),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        },
        ParameterSection {
            id: "batch".into(),
            label: "Batch Options".into(),
            description: Some("Options for processing multiple images".into()),
            icon: Some("layers".into()),
            collapsible: true,
            default_collapsed: true,
            fields: vec![
                ParameterField {
                    id: "increment_per_image".into(),
                    label: "Increment Per Image".into(),
                    description: Some(
                        "Add additional seconds per image in batch (for sequence correction)"
                            .into(),
                    ),
                    help_url: None,
                    field_type: ParameterType::Float {
                        min: 0.0,
                        max: 3600.0,
                        step: 0.1,
                        precision: 1,
                        unit: Some("s".into()),
                        logarithmic: false,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0.0),
                    required: false,
                    advanced: true,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "batch_image_index".into(),
                    label: "Image Index".into(),
                    description: Some("Zero-based index of this image in the batch".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 99999,
                        step: 1,
                        unit: None,
                        style: Default::default(),
                    },
                    default: serde_json::json!(0),
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
                param_section_id: "time_shift".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "timezone".into(),
                title_visible: true,
                style: SectionStyle::Card,
            },
            GuiSection {
                param_section_id: "batch".into(),
                title_visible: true,
                style: SectionStyle::CollapsibleCard,
            },
        ],
    },
    icon: Some("clock".into()),
    color: Some("#f59e0b".into()),
    preview: PreviewMode::None,
    aux_views: vec![],
    min_panel_width: 320,
});

#[derive(Debug, Clone)]
pub struct TimeShiftPlugin {
    id: String,
}

impl TimeShiftPlugin {
    pub fn new() -> Self {
        Self {
            id: "photopipeline.plugins.time_shift".to_string(),
        }
    }
}

impl Default for TimeShiftPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TimeShiftPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn name(&self) -> &str {
        "Time Shift"
    }
    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Metadata
    }
    fn description(&self) -> &str {
        "Adjust DateTimeOriginal by hours, minutes, and seconds with timezone support"
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
        tracing::info!("time_shift plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("time_shift plugin shutdown");
        Ok(())
    }

    async fn validate(&self, params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        let hours = params.get_i64("shift_hours").unwrap_or(0);
        let minutes = params.get_i64("shift_minutes").unwrap_or(0);
        let seconds = params.get_i64("shift_seconds").unwrap_or(0);
        tracing::debug!(
            "time_shift: validating (hours={}, minutes={}, seconds={})",
            hours,
            minutes,
            seconds
        );

        if hours == 0 && minutes == 0 && seconds == 0 {
            let src_tz = params.get_str("source_timezone").unwrap_or("UTC");
            let tgt_tz = params.get_str("target_timezone").unwrap_or("local");
            if src_tz == tgt_tz {
                issues.push(ValidationIssue::Warning {
                    param: "shift_hours".into(),
                    message: "No time shift and source/target timezones are identical".into(),
                });
            }
        }

        if !issues.is_empty() {
            tracing::warn!(
                issue_count = issues.len(),
                "time_shift validation found {} issues",
                issues.len()
            );
        }
        Ok(issues)
    }
}

#[async_trait]
impl MetadataProcessor for TimeShiftPlugin {
    fn metadata_scope(&self) -> Vec<MetadataScope> {
        vec![MetadataScope::EXIF]
    }

    async fn read_metadata(
        &self,
        _target: &MetadataTarget,
        _params: &ParameterSet,
    ) -> PluginResult<Metadata> {
        tracing::trace!("time_shift: read_metadata (no-op)");
        Ok(Metadata::default())
    }

    async fn write_metadata(
        &self,
        target: &mut MetadataTarget,
        metadata: &Metadata,
        params: &ParameterSet,
    ) -> PluginResult<MetadataWriteReport> {
        let _timer = PerfTimer::with_target("time_shift_write_metadata", "plugin.time_shift");
        let hours = params.get_i64("shift_hours").unwrap_or(0);
        let minutes = params.get_i64("shift_minutes").unwrap_or(0);
        let seconds = params.get_i64("shift_seconds").unwrap_or(0);
        let increment = params.get_f64("increment_per_image").unwrap_or(0.0);
        let batch_index = params.get_i64("batch_image_index").unwrap_or(0);

        tracing::info!(
            target_path = %target.path,
            shift_h = hours,
            shift_m = minutes,
            shift_s = seconds,
            "time_shift: applying time shift to {} ({}h {}m {}s)",
            target.path,
            hours,
            minutes,
            seconds,
        );

        let total_shift = hours as f64 * 3600.0
            + minutes as f64 * 60.0
            + seconds as f64
            + increment * batch_index as f64;

        let mut exiftool_args = Vec::new();
        exiftool_args.push("-overwrite_original".to_string());

        let exif_ts = metadata.exif.as_ref().and_then(|e| e.date_time_original);
        if let Some(orig_ts) = exif_ts {
            let shifted = orig_ts + chrono::Duration::seconds(total_shift as i64);
            let formatted = shifted.format("%Y:%m:%d %H:%M:%S").to_string();
            exiftool_args.push(format!("-DateTimeOriginal={}", formatted));
            exiftool_args.push(format!("-CreateDate={}", formatted));
        } else {
            let tag_op = if total_shift >= 0.0 {
                format!("-DateTimeOriginal+={}:{}:{}", hours, minutes, seconds)
            } else {
                format!(
                    "-DateTimeOriginal-={}:{}:{}",
                    (-hours).abs(),
                    (-minutes).abs(),
                    (-seconds).abs()
                )
            };
            exiftool_args.push(tag_op);
        }

        exiftool_args.push(target.path.clone());

        tracing::debug!(
            tool = "exiftool",
            args = ?exiftool_args,
            "time_shift: invoking exiftool to shift timestamp",
        );

        let exiftool_path = crate::exif_rw::find_exiftool_path().unwrap_or_else(|| "exiftool".to_string());
        let mut cmd = Command::new(&exiftool_path);
        for arg in &exiftool_args {
            cmd.arg(arg);
        }

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
                "time_shift: exiftool write failed",
            );
        } else {
            tracing::trace!("time_shift: exiftool write succeeded");
        }

        let tags_written = if output.status.success() { 2 } else { 0 };
        let warnings = if !output.status.success() {
            vec![String::from_utf8_lossy(&output.stderr).to_string()]
        } else {
            vec![]
        };

        tracing::info!(
            tags_written = tags_written,
            "time_shift: wrote {} timestamp tags",
            tags_written,
        );

        Ok(MetadataWriteReport {
            tags_written,
            tags_skipped: 0,
            warnings,
        })
    }
}

static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "metadata".into(),
        "time".into(),
        "exif".into(),
        "datetime".into(),
        "timezone".into(),
        "shift".into(),
        "batch".into(),
    ]
});

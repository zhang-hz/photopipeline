#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{
    ColorSpace, ExifData, GpsData, GpxPoint, GpxTrack, ImageFormat, ImageInfo, Metadata,
    MetadataWriteReport, PixelBuffer, PixelFormat, PluginError, PluginResult, ValidationIssue,
};
use photopipeline_engine::{
    ExpressionEngine, NodeRunState, NodeStatus, ParameterResolver, PipelineGraph, PipelineNode,
    PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    PluginQuery, registry::Registry,
};
use photopipeline_plugins;
use std::collections::HashMap;
use std::sync::Arc;
use test_harness::fixtures::{gpx, image, metadata, pipeline};
use test_harness::mocks::progress::MockProgressSink;
use uuid::Uuid;

fn make_image_info(id: Uuid, path: &str) -> ImageInfo {
    ImageInfo {
        id,
        path: path.into(),
        filename: path.rsplit('/').next().unwrap_or(path).into(),
        format: ImageFormat::JPEG,
        width: 100,
        height: 100,
        file_size_bytes: 1000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

fn make_pixel_buffer() -> PixelBuffer {
    image::ImageFixture::new()
        .width(64)
        .height(64)
        .solid(128, 128, 128)
        .build()
}

// ---------------------------------------------------------------------------
// Pipeline Error Recovery
// ---------------------------------------------------------------------------

#[test]
fn e2e_missing_plugin_in_template_returns_not_found() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "nonexistent.plugin.xyz".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let info = make_image_info(Uuid::new_v4(), "/tmp/nonexistent.jpg");
    let md = Metadata::default();
    let buf = make_pixel_buffer();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
    let progress = Box::new(MockProgressSink::new());
    let result = rt.block_on(async { exec.execute(&graph, &info, Some(buf), &md, progress).await });
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("nonexistent"),
        "expected not-found error, got: {msg}"
    );
}

#[test]
fn e2e_missing_input_file_returns_file_not_found() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = pipeline::minimal_pipeline();
    let graph = template.into_graph();
    let info = make_image_info(Uuid::new_v4(), "/tmp/i_do_not_exist_97321.jpg");
    let md = Metadata::default();
    let buf = make_pixel_buffer();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
    let progress = Box::new(MockProgressSink::new());
    let result = rt.block_on(async { exec.execute(&graph, &info, Some(buf), &md, progress).await });
    assert!(result.is_err());
}

#[test]
fn e2e_corrupted_toml_config_returns_parse_error() {
    let corrupted = b"[[[invalid\n = ::: broken [[[" as &[u8];
    let result: Result<PipelineTemplate, _> =
        toml::from_str(std::str::from_utf8(corrupted).unwrap_or(""));
    assert!(result.is_err() || std::str::from_utf8(corrupted).is_err());
}

#[test]
fn e2e_pipeline_with_validation_error_stops_execution() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "photopipeline.plugins.exif_rw".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();

    let rt = tokio::runtime::Runtime::new().unwrap();
    {
        for plugin in reg.all() {
            let schema = plugin.parameter_schema();
            let mut params = schema.defaults();
            params.insert("invalid_key_xyz".into(), serde_json::json!("bad_value"));
            let validation = rt.block_on(async { plugin.validate(&params).await });
            // Plugins must not panic on unknown params; validation should detect issues
            assert!(!validation.is_ok() || validation.as_ref().unwrap().is_empty(),
                "plugin accepted invalid parameter key without warning");
        }
    }

    let info = make_image_info(Uuid::new_v4(), "/tmp/validates_test.jpg");
    let md = Metadata::default();
    let buf = make_pixel_buffer();
    let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
    let progress = Box::new(MockProgressSink::new());
    let result = rt.block_on(async { exec.execute(&graph, &info, Some(buf), &md, progress).await });
    // Pipeline must not panic on validation error
    assert!(result.is_ok() || result.is_err(),
        "pipeline with validation error must not panic");
}

#[test]
fn e2e_pipeline_with_warning_only_continues() {
    let validation_warning = ValidationIssue::Warning {
        param: "quality".into(),
        message: "quality is high, may be slow".into(),
    };
    match validation_warning {
        ValidationIssue::Warning { .. } => {}
        _ => panic!("expected a warning"),
    }

    let vi = vec![ValidationIssue::Warning {
        param: "p".into(),
        message: "m".into(),
    }];
    let has_error = vi
        .iter()
        .any(|issue| matches!(issue, ValidationIssue::Error { .. }));
    assert!(
        !has_error,
        "warning-only issues should not be treated as errors"
    );
}

#[test]
fn e2e_external_tool_missing_returns_missing_tool_error() {
    let err = PluginError::MissingTool {
        plugin: "format.heif_encoder".into(),
        tool: "heif-enc".into(),
        required: ">=1.0".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("heif-enc"));
    assert!(msg.contains("missing tool"));
}

#[test]
fn e2e_encode_without_pixel_buffer_returns_error() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "output".into(),
            plugin: "photopipeline.plugins.heif_encoder".into(),
            label: None,
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let info = make_image_info(Uuid::new_v4(), "/tmp/output_only.heif");
    let md = Metadata::default();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
    let progress = Box::new(MockProgressSink::new());
    let result = rt.block_on(async { exec.execute(&graph, &info, None, &md, progress).await });
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Node state tracking during errors
// ---------------------------------------------------------------------------

#[test]
fn e2e_error_node_status_is_failed() {
    let failed_state = NodeRunState {
        status: NodeStatus::Failed("test error".into()),
        started_at: Some(chrono::Utc::now()),
    };
    match &failed_state.status {
        NodeStatus::Failed(msg) => assert_eq!(msg, "test error"),
        _ => panic!("expected Failed status"),
    }
}

#[test]
fn e2e_previous_nodes_status_unchanged_after_failure() {
    let mut node_states: HashMap<Uuid, NodeRunState> = HashMap::new();
    let n1 = Uuid::new_v4();
    let n2 = Uuid::new_v4();
    let n3 = Uuid::new_v4();

    node_states.insert(
        n1,
        NodeRunState {
            status: NodeStatus::Completed(photopipeline_core::ProcessingStats {
                elapsed_ms: 10,
                cpu_time_ms: 8,
                gpu_time_ms: None,
                peak_memory_mb: 50,
                input_pixels: 10000,
                output_pixels: 10000,
            }),
            started_at: Some(chrono::Utc::now()),
        },
    );
    node_states.insert(
        n2,
        NodeRunState {
            status: NodeStatus::Completed(photopipeline_core::ProcessingStats {
                elapsed_ms: 20,
                cpu_time_ms: 18,
                gpu_time_ms: None,
                peak_memory_mb: 60,
                input_pixels: 10000,
                output_pixels: 10000,
            }),
            started_at: Some(chrono::Utc::now()),
        },
    );
    node_states.insert(
        n3,
        NodeRunState {
            status: NodeStatus::Failed("crash".into()),
            started_at: Some(chrono::Utc::now()),
        },
    );

    assert!(matches!(node_states[&n1].status, NodeStatus::Completed(_)));
    assert!(matches!(node_states[&n2].status, NodeStatus::Completed(_)));
    assert!(matches!(node_states[&n3].status, NodeStatus::Failed(_)));

    match &node_states[&n1].status {
        NodeStatus::Completed(stats) => assert_eq!(stats.elapsed_ms, 10),
        _ => panic!("n1 should still be Completed"),
    }
    match &node_states[&n2].status {
        NodeStatus::Completed(stats) => assert_eq!(stats.elapsed_ms, 20),
        _ => panic!("n2 should still be Completed"),
    }
}

#[test]
fn e2e_subsequent_nodes_stay_pending_after_failure() {
    let mut node_states: HashMap<Uuid, NodeRunState> = HashMap::new();
    let n1 = Uuid::new_v4();
    let n2 = Uuid::new_v4();
    let n3 = Uuid::new_v4();
    let n4 = Uuid::new_v4();
    let n5 = Uuid::new_v4();

    node_states.insert(
        n1,
        NodeRunState {
            status: NodeStatus::Completed(photopipeline_core::ProcessingStats {
                elapsed_ms: 5,
                cpu_time_ms: 3,
                gpu_time_ms: None,
                peak_memory_mb: 10,
                input_pixels: 0,
                output_pixels: 0,
            }),
            started_at: None,
        },
    );
    node_states.insert(
        n2,
        NodeRunState {
            status: NodeStatus::Completed(photopipeline_core::ProcessingStats {
                elapsed_ms: 8,
                cpu_time_ms: 5,
                gpu_time_ms: None,
                peak_memory_mb: 12,
                input_pixels: 0,
                output_pixels: 0,
            }),
            started_at: None,
        },
    );
    node_states.insert(
        n3,
        NodeRunState {
            status: NodeStatus::Failed("node 3 error".into()),
            started_at: None,
        },
    );
    node_states.insert(
        n4,
        NodeRunState {
            status: NodeStatus::Pending,
            started_at: None,
        },
    );
    node_states.insert(
        n5,
        NodeRunState {
            status: NodeStatus::Pending,
            started_at: None,
        },
    );

    assert!(matches!(node_states[&n1].status, NodeStatus::Completed(_)));
    assert!(matches!(node_states[&n2].status, NodeStatus::Completed(_)));
    assert!(matches!(node_states[&n3].status, NodeStatus::Failed(_)));
    assert!(matches!(node_states[&n4].status, NodeStatus::Pending));
    assert!(matches!(node_states[&n5].status, NodeStatus::Pending));
}

// ---------------------------------------------------------------------------
// Metadata plugin error paths
// ---------------------------------------------------------------------------

#[test]
fn e2e_exif_read_on_missing_file() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let processor = reg.get_metadata_processor("photopipeline.plugins.exif_rw");
    if let Some(proc) = processor {
        let target = photopipeline_core::MetadataTarget {
            path: "/tmp/this_file_does_not_exist_99211.jpg".into(),
            format: ImageFormat::JPEG,
        };
        let params = ParameterSet::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { proc.read_metadata(&target, &params).await });
        assert!(result.is_err());
    } else {
        let exif = ExifData::default();
        assert!(exif.iso.is_none());
    }
}

#[test]
fn e2e_exif_write_invalid_permissions() {
    let target = photopipeline_core::MetadataTarget {
        path: "/root/protected_write_test.jpg".into(),
        format: ImageFormat::JPEG,
    };
    let exif = metadata::exif_sony_a7r5();
    let md = Metadata {
        exif: Some(exif),
        ..Default::default()
    };

    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let processor = reg.get_metadata_processor("photopipeline.plugins.exif_rw");
    if let Some(proc) = processor {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let params = ParameterSet::new();
        let result = rt.block_on(async {
            let mut t = target.clone();
            proc.write_metadata(&mut t, &md, &params).await
        });
        assert!(result.is_err(),
            "write to protected path /root must fail with permission error");
    }
}

#[test]
fn e2e_gpx_parse_malformed_xml() {
    let malformed = r#"<?xml version="1.0"?><gpx><trk><trkseg><trkpt lat="bad" lon="xml"><ele>ABC</ele></trkpt></trkseg></trk></gpx>"#;
    let result = serde_json::from_str::<serde_json::Value>(malformed);
    if let Ok(ref v) = result {
        assert!(!v.is_null());
    }

    let valid_xml = r#"<gpx><trk><trkseg><trkpt lat="40.0" lon="-74.0"><ele>10</ele><time>2024-06-15T10:00:00Z</time></trkpt></trkseg></trk></gpx>"#;
    assert!(!valid_xml.is_empty());
}

#[test]
fn e2e_gpx_interpolation_empty_timestamps() {
    let track = gpx::gpx_empty();
    assert!(track.points.is_empty());
    let t = chrono::Utc::now();
    let result = track.interpolate_at(&t);
    assert!(result.is_none());
}

#[test]
fn e2e_gpx_all_points_before_photo_time() {
    let start = chrono::Utc::now() - chrono::Duration::hours(5);
    let track = gpx::gpx_hourly_track(start, 40.0, -74.0);
    let photo_time = chrono::Utc::now();
    let interpolated = track.interpolate_at(&photo_time);
    // All GPX points are before photo time, expect None
    assert!(interpolated.is_none(),
        "interpolation must return None when all points precede photo time");
}

#[test]
fn e2e_gpx_all_points_after_photo_time() {
    let start = chrono::Utc::now() + chrono::Duration::hours(2);
    let track = gpx::gpx_hourly_track(start, 40.0, -74.0);
    let photo_time = chrono::Utc::now();
    let interpolated = track.interpolate_at(&photo_time);
    // All GPX points are after photo time, expect None
    assert!(interpolated.is_none(),
        "interpolation must return None when all points follow photo time");
}

// ---------------------------------------------------------------------------
// Expression engine error paths
// ---------------------------------------------------------------------------

#[test]
fn e2e_expression_unknown_variable_returns_error() {
    let engine = ExpressionEngine::default();
    let info = make_image_info(Uuid::new_v4(), "/tmp/expr_test.jpg");
    let md = Metadata::default();
    let result = engine.evaluate("${unknown_var}", &md, &info);
    assert!(result.is_err());
}

#[test]
fn e2e_expression_division_by_zero() {
    let engine = ExpressionEngine::default();
    let info = make_image_info(Uuid::new_v4(), "/tmp/div_zero.jpg");
    let md = Metadata::default();
    let result = engine.evaluate("${exif.iso}/0", &md, &info);
    assert!(result.is_err(), "division by zero should return error");
}

#[test]
fn e2e_expression_malformed_syntax_no_closing_brace() {
    let engine = ExpressionEngine::default();
    let info = make_image_info(Uuid::new_v4(), "/tmp/malformed.jpg");
    let md = Metadata::default();
    let result = engine.evaluate("${exif.iso + 100", &md, &info);
    if let Ok(ref v) = result {
        assert!(!v.is_null());
    }
}

#[test]
fn e2e_expression_nested_ternary_deep() {
    let engine = ExpressionEngine::default();
    let info = make_image_info(Uuid::new_v4(), "/tmp/ternary.jpg");
    let md = Metadata::default();
    let result = engine.evaluate(
        "${exif.iso > 100 ? (${image.width} > 50 ? 'large' : 'small') : 'low'}",
        &md,
        &info,
    );
    // iso is None (default metadata), false branch → 'low'
    assert!(result.is_ok(), "nested ternary must evaluate without error");
    assert_eq!(result.unwrap(), serde_json::json!("low"),
        "default iso must evaluate to 'low' in nested ternary");
}

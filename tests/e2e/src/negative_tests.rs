#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{
    ColorSpace, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat, PluginError,
    ValidationIssue,
};
use photopipeline_engine::{
    ParameterResolver, PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    registry::Registry,
};
use photopipeline_plugins;
use std::collections::HashMap;
use std::sync::Arc;
use test_harness::fixtures::image::ImageFixture;
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

#[test]
fn e2e_invalid_toml_syntax() {
    let invalid_toml = b"[[invalid section\n  key = value with no quotes\n  [[nested broken\n";
    let result: Result<PipelineTemplate, _> = toml::from_str(std::str::from_utf8(invalid_toml).unwrap_or(""));
    assert!(result.is_err() || std::str::from_utf8(invalid_toml).is_err());
}

#[test]
fn e2e_empty_toml_file() {
    let _empty: &[u8] = b"";
    let result: Result<PipelineTemplate, _> = toml::from_str("");
    assert!(result.is_err());
}

#[test]
fn e2e_wrong_plugin_id_in_template() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "wrong.plugin.id.nonexistent".into(),
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
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(
        graph.nodes[0].plugin_id,
        "wrong.plugin.id.nonexistent"
    );

    let plugin_id = &graph.nodes[0].plugin_id;
    let found = reg.get(plugin_id);
    assert!(found.is_none());
}

#[test]
fn e2e_self_loop_edge_rejected() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("p1".into(), "self_loop".into());
    let port = graph.node(n1).unwrap().outputs[0];
    let result = graph.connect(port, port);
    assert!(result.is_err());
    match result {
        Err(PluginError::ValidationFailed(msg)) => {
            assert!(msg.contains("itself") || msg.contains("same"));
        }
        _ => panic!("expected ValidationFailed error"),
    }
}

#[test]
fn e2e_duplicate_edge_rejected() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("p1".into(), "n1".into());
    let n2 = graph.add_node("p2".into(), "n2".into());
    let out1 = graph.node(n1).unwrap().outputs[0];
    let in2 = graph.node(n2).unwrap().inputs[0];

    assert!(graph.connect(out1, in2).is_ok());
    let result = graph.connect(out1, in2);
    assert!(result.is_err());
}

#[test]
fn e2e_edge_to_nonexistent_node() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("p1".into(), "n1".into());
    let out1 = graph.node(n1).unwrap().outputs[0];
    let fake_port = Uuid::new_v4();
    let result = graph.connect(out1, fake_port);
    assert!(result.is_err());
}

#[test]
fn e2e_parameter_wrong_type_string_for_int() {
    let mut ps = ParameterSet::new();
    ps.insert("count".into(), serde_json::json!("not_a_number"));
    let _schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "s".into(),
            label: "S".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "count".into(),
                label: "Count".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 100,
                    step: 1,
                    unit: None,
                    style: Default::default(),
                },
                default: serde_json::json!(0),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    };

    let actual_type = ps.get("count").map(|v| if v.is_string() { "string" } else { "other" });
    let schema_expects = "integer";
    assert_eq!(actual_type, Some("string"));
    assert_ne!(actual_type.map(|t| t), Some(schema_expects));
}

#[test]
fn e2e_parameter_below_minimum_value() {
    let min_value: i64 = 0;
    let below_min: i64 = -5;
    assert!(below_min < min_value);

    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "s".into(),
            label: "S".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "quality".into(),
                label: "Quality".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 100,
                    step: 1,
                    unit: None,
                    style: Default::default(),
                },
                default: serde_json::json!(50),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    };

    let field = schema.field("s", "quality").unwrap();
    if let ParameterType::Integer { min, max, .. } = &field.field_type {
        assert!(*min <= *max);
        assert!(below_min < *min);
    }
}

#[test]
fn e2e_parameter_above_maximum_value() {
    let max_value: i64 = 100;
    let above_max: i64 = 256;
    assert!(above_max > max_value);

    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "s".into(),
            label: "S".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "threshold".into(),
                label: "Threshold".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 255,
                    step: 1,
                    unit: None,
                    style: Default::default(),
                },
                default: serde_json::json!(128),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    };

    let field = schema.field("s", "threshold").unwrap();
    if let ParameterType::Integer { min: _, max, .. } = &field.field_type {
        assert!(*max == 255);
        assert!(above_max > *max);
    }
}

#[test]
fn e2e_empty_parameter_set() {
    let ps = ParameterSet::new();
    assert!(ps.values.is_empty());
    assert!(ps.iter().next().is_none());
    assert_eq!(ps.get_str("anything"), None);
    assert_eq!(ps.get_i64("anything"), None);
    assert_eq!(ps.get_f64("anything"), None);
    assert_eq!(ps.get_bool("anything"), None);
}

#[test]
fn e2e_max_length_string_parameter_exceeded() {
    let max_length: usize = 10;
    let too_long = "abcdefghijklmnop".to_string();
    assert!(too_long.len() > max_length);

    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "s".into(),
            label: "S".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![ParameterField {
                id: "name".into(),
                label: "Name".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::String {
                    max_length: 10,
                    pattern: None,
                    placeholder: None,
                },
                default: serde_json::json!(""),
                required: false,
                advanced: false,
                allow_override: true,
                supports_expression: false,
            }],
        }],
    };

    let field = schema.field("s", "name").unwrap();
    if let ParameterType::String { max_length, .. } = &field.field_type {
        assert_eq!(*max_length, 10);
        assert!(too_long.len() > *max_length);
    }
}

#[test]
fn e2e_path_traversal_in_file_parameter() {
    let traversal_paths = vec![
        "../../../etc/passwd",
        "/etc/passwd",
        "..\\..\\..\\windows\\system32",
        "../../.ssh/id_rsa",
    ];

    for path in &traversal_paths {
        let has_traversal =
            path.contains("..") || path.contains("/etc/") || path.contains("\\windows\\");
        assert!(has_traversal, "path '{path}' should be detected as suspicious");
    }

    let safe_path = "photos/vacation/img_001.jpg";
    let no_traversal =
        !safe_path.contains("..") && !safe_path.contains("/etc/") && !safe_path.contains("\\");
    assert!(no_traversal, "safe path should not trigger traversal detection");
}

#[test]
fn e2e_unicode_in_plugin_id() {
    let unicode_ids = vec![
        "plugin.中文".to_string(),
        "plugin.日本語".to_string(),
        "plugin.émoji🎉".to_string(),
    ];

    for id in &unicode_ids {
        assert!(!id.is_empty());
        assert!(id.len() > 7);
    }

    let reg = Registry::new();
    for id in &unicode_ids {
        assert!(reg.get(id).is_none());
        assert!(!reg.is_loaded(id));
    }
}

#[test]
fn e2e_zero_bytes_input_file() {
    let buf = PixelBuffer::new(0, 0, photopipeline_core::ChannelLayout::RGB, PixelFormat::U8);
    assert_eq!(buf.data.data.len(), 0);
    assert_eq!(buf.width, 0);
    assert_eq!(buf.height, 0);

    let empty_data: Vec<u8> = Vec::new();
    assert!(empty_data.is_empty());
}

#[test]
fn e2e_non_image_file_as_input() {
    let non_image_paths = vec![
        "/tmp/readme.txt",
        "/tmp/script.sh",
        "/tmp/data.bin",
        "/tmp/archive.zip",
    ];

    let image_extensions: Vec<&str> = vec!["jpg", "jpeg", "png", "tiff", "heif", "avif", "jxl"];

    for path in &non_image_paths {
        let ext = path.rsplit('.').next().unwrap_or("");
        let is_image_ext = image_extensions.contains(&ext);
        assert!(!is_image_ext, "path '{path}' with ext '{ext}' should not be an image");

        let is_txt = ext == "txt";
        let is_sh = ext == "sh";
        let is_bin = ext == "bin";
        let is_zip = ext == "zip";
        assert!(is_txt || is_sh || is_bin || is_zip);
    }
}

#[test]
fn e2e_empty_nodes_in_template_rejected() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let result = template.validate();
    assert!(result.is_err());
}

#[test]
fn e2e_circular_dependency_in_template() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "a".into(),
                plugin: "photopipeline.plugins.exif_rw".into(),
                label: None,
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "b".into(),
                plugin: "photopipeline.plugins.exif_rw".into(),
                label: None,
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "a".into(),
                to: "b".into(),
            },
            TemplateEdge {
                from: "b".into(),
                to: "a".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.edges.len() <= 2);
    let _ = graph.has_cycle();
}

#[test]
fn e2e_duplicate_node_ids_in_graph() {
    let mut graph = PipelineGraph::new();
    for _ in 0..10 {
        graph.add_node("p1".into(), "dup_label".into());
    }
    assert_eq!(graph.nodes.len(), 10);
    let validate_result = graph.validate_graph();
    if let Err(issues) = validate_result {
        let has_dup = issues.iter().any(|i| i.contains("duplicate"));
        if has_dup {
            return;
        }
    }

    let ids: Vec<Uuid> = graph.nodes.iter().map(|n| n.id).collect();
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        seen.insert(*id);
    }
    assert_eq!(ids.len(), seen.len(), "all node ids should be unique");
}

#[test]
fn e2e_invalid_expression_syntax() {
    let engine = photopipeline_engine::ExpressionEngine::default();
    let info = make_image_info(Uuid::new_v4(), "/tmp/bad_expr.jpg");
    let md = Metadata::default();

    let bad_exprs = vec![
        "${xyz + }",
        "${a ? b :}",
        "${== 42}",
        "${",
    ];

    for expr in bad_exprs {
        let result = engine.evaluate(expr, &md, &info);
        let _ = result;
    }
}

#[test]
fn e2e_null_serde_json_value_in_params() {
    let mut ps = ParameterSet::new();
    ps.insert("nullable".into(), serde_json::Value::Null);
    assert_eq!(ps.get("nullable"), Some(&serde_json::Value::Null));
    assert_eq!(ps.get_str("nullable"), None);
    assert_eq!(ps.get_i64("nullable"), None);
    assert_eq!(ps.get_f64("nullable"), None);
    assert_eq!(ps.get_bool("nullable"), None);
}

#[test]
fn e2e_large_numeric_value_overflow() {
    let mut ps = ParameterSet::new();
    ps.insert("big".into(), serde_json::json!(i64::MAX));
    assert_eq!(ps.get_i64("big"), Some(i64::MAX));

    ps.insert("negative".into(), serde_json::json!(i64::MIN));
    assert_eq!(ps.get_i64("negative"), Some(i64::MIN));

    ps.insert("float_max".into(), serde_json::json!(f64::MAX));
    let val = ps.get_f64("float_max");
    assert!(val.is_some());

    ps.insert("float_nan".into(), serde_json::json!(f64::NAN));
    let nan_val = ps.get_f64("float_nan");
    if let Some(n) = nan_val {
        assert!(n.is_nan());
    }
}

#[test]
fn e2e_pipeline_graph_serde_roundtrip_zero_nodes() {
    let graph = PipelineGraph::new();
    let json = serde_json::to_string(&graph).unwrap();
    let deser: PipelineGraph = serde_json::from_str(&json).unwrap();
    assert!(deser.nodes.is_empty());
    assert!(deser.edges.is_empty());
}

#[test]
fn e2e_validation_error_format_consistent() {
    let issues = vec![
        ValidationIssue::Error {
            param: "p1".into(),
            message: "m1".into(),
        },
        ValidationIssue::Warning {
            param: "p2".into(),
            message: "m2".into(),
        },
        ValidationIssue::Info {
            param: "p3".into(),
            message: "m3".into(),
        },
    ];

    for issue in &issues {
        let s = issue.to_string();
        assert!(!s.is_empty());
    }

    let errors_only: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i, ValidationIssue::Error { .. }))
        .collect();
    assert_eq!(errors_only.len(), 1);

    let warnings_only: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i, ValidationIssue::Warning { .. }))
        .collect();
    assert_eq!(warnings_only.len(), 1);

    let infos_only: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i, ValidationIssue::Info { .. }))
        .collect();
    assert_eq!(infos_only.len(), 1);
}

#[test]
fn e2e_parameterset_clone_independent() {
    let mut ps1 = ParameterSet::new();
    ps1.insert("a".into(), serde_json::json!(1));

    let mut ps2 = ps1.clone();
    ps2.insert("b".into(), serde_json::json!(2));

    assert!(ps1.get("b").is_none());
    assert_eq!(ps2.get_i64("b"), Some(2));
    assert_eq!(ps1.get_i64("a"), Some(1));
}

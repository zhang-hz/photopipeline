use photopipeline_core::{AiBackend, GpuBackend, ImageFormat, PluginCategory, PluginVersion};
use photopipeline_engine::{PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode};
use photopipeline_plugin::{PluginQuery, registry::Registry};
use uuid::Uuid;

#[test]
fn test_register_all_plugins() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let all = registry.all();
    assert!(all.len() >= 14, "expected at least 14 plugins, got {}", all.len());
}

#[test]
fn test_registry_categories_after_register_all() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let cats = registry.categories();
    assert!(!cats.is_empty());
    assert!(cats.contains(&PluginCategory::Input));
}

#[test]
fn test_build_pipeline_graph_from_template() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "input".into(),
                plugin: "raw_input".into(),
                label: Some("RAW Input".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "transform".into(),
                plugin: "transform".into(),
                label: Some("Transform".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "output".into(),
                plugin: "png_encoder".into(),
                label: Some("PNG Output".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "input".into(),
                to: "transform".into(),
            },
            TemplateEdge {
                from: "transform".into(),
                to: "output".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    assert!(template.validate().is_ok());
    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
}

#[test]
fn test_pipeline_execute_validation() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("exif_rw".into(), "EXIF".into());
    let n2 = graph.add_node("colorspace".into(), "ColorSpace".into());

    let out1 = graph.node(n1).unwrap().outputs[0];
    let in2 = graph.node(n2).unwrap().inputs[0];
    graph.connect(out1, in2).unwrap();

    assert!(graph.validate_graph().is_ok());

    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], n1);
    assert_eq!(order[1], n2);
}

#[test]
fn test_pipeline_graph_serialization_roundtrip() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("exif_rw".into(), "Read EXIF".into());
    let n2 = graph.add_node("time_shift".into(), "Time Shift".into());
    let n3 = graph.add_node("png_encoder".into(), "PNG Output".into());

    let out1 = graph.node(n1).unwrap().outputs[0];
    let in2 = graph.node(n2).unwrap().inputs[0];
    let out2 = graph.node(n2).unwrap().outputs[0];
    let in3 = graph.node(n3).unwrap().inputs[0];

    graph.connect(out1, in2).unwrap();
    graph.connect(out2, in3).unwrap();

    let json = serde_json::to_string_pretty(&graph).unwrap();
    let deserialized: PipelineGraph = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.nodes.len(), 3);
    assert_eq!(deserialized.edges.len(), 2);
    assert!(deserialized.validate_graph().is_ok());
}

#[test]
fn test_pipeline_graph_empty_validation() {
    let graph = PipelineGraph::new();
    assert!(graph.validate_graph().is_ok());
}

#[test]
fn test_pipeline_graph_disconnected_is_ok() {
    let mut graph = PipelineGraph::new();
    graph.add_node("raw_input".into(), "RAW".into());
    graph.add_node("png_encoder".into(), "PNG".into());
    assert!(graph.validate_graph().is_ok());
}

#[test]
fn test_registry_query_after_register_all() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let q = PluginQuery {
        category: Some(PluginCategory::Format),
        ..Default::default()
    };
    let results = registry.query(&q);
    assert!(!results.is_empty());
    for plugin in &results {
        assert_eq!(plugin.category(), PluginCategory::Format);
    }
}

#[test]
fn test_registry_manifest_integrity() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let manifests = registry.manifests();
    assert_eq!(manifests.len(), 14);

    for m in &manifests {
        assert!(!m.id.is_empty());
        assert!(!m.name.is_empty());
        assert!(!m.description.is_empty());
    }
}

#[test]
fn test_version_requirement_boundary() {
    let req = photopipeline_core::VersionRequirement {
        min_version: PluginVersion::new(2, 0, 0),
        max_version: Some(PluginVersion::new(3, 0, 0)),
    };
    assert!(req.is_satisfied_by(&PluginVersion::new(2, 0, 0)));
    assert!(req.is_satisfied_by(&PluginVersion::new(2, 5, 0)));
    assert!(!req.is_satisfied_by(&PluginVersion::new(3, 0, 0)));
    assert!(!req.is_satisfied_by(&PluginVersion::new(1, 9, 0)));
}

#[test]
fn test_gpu_and_ai_backend_display() {
    assert_eq!(GpuBackend::CUDA.to_string(), "CUDA");
    assert_eq!(GpuBackend::Metal.to_string(), "Metal");
    assert_eq!(GpuBackend::Vulkan.to_string(), "Vulkan");
    assert_eq!(GpuBackend::None.to_string(), "None");
    assert_eq!(GpuBackend::Auto.to_string(), "Auto");
    assert_eq!(AiBackend::ONNX.to_string(), "ONNX");
    assert_eq!(AiBackend::TensorRT.to_string(), "TensorRT");
    assert_eq!(AiBackend::CoreML.to_string(), "CoreML");
    assert_eq!(AiBackend::OpenVINO.to_string(), "OpenVINO");
    assert_eq!(AiBackend::Burn.to_string(), "Burn");
}

#[test]
fn test_image_format_display_all() {
    assert_eq!(ImageFormat::HEIF.to_string(), "HEIF");
    assert_eq!(ImageFormat::HEIC.to_string(), "HEIC");
    assert_eq!(ImageFormat::AVIF.to_string(), "AVIF");
    assert_eq!(ImageFormat::JXL.to_string(), "JXL");
    assert_eq!(ImageFormat::PNG.to_string(), "PNG");
    assert_eq!(ImageFormat::TIFF.to_string(), "TIFF");
    assert_eq!(ImageFormat::JPEG.to_string(), "JPEG");
    assert_eq!(ImageFormat::WEBP.to_string(), "WEBP");
    assert_eq!(ImageFormat::OpenEXR.to_string(), "OpenEXR");
    assert_eq!(ImageFormat::RAW.to_string(), "RAW");
    assert_eq!(ImageFormat::DNG.to_string(), "DNG");
    assert_eq!(ImageFormat::PPM.to_string(), "PPM");
    assert_eq!(ImageFormat::PGM.to_string(), "PGM");
    assert_eq!(ImageFormat::BMP.to_string(), "BMP");
    assert_eq!(ImageFormat::Unknown("custom".into()).to_string(), "custom");
}

#[test]
fn test_registry_query_by_tags() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let q = PluginQuery {
        tags: vec!["format".into()],
        ..Default::default()
    };
    let results = registry.query(&q);
    assert!(!results.is_empty());
    for plugin in &results {
        assert!(plugin.tags().iter().any(|t| t == "format"));
    }
}

#[test]
fn test_registry_query_enabled_only() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let q = PluginQuery {
        enabled_only: true,
        ..Default::default()
    };
    let results = registry.query(&q);
    assert_eq!(results.len(), 14);
}

#[test]
fn test_registry_query_keyword_not_found() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let q = PluginQuery {
        keyword: Some("zzz_not_exist_zzz".into()),
        ..Default::default()
    };
    let results = registry.query(&q);
    assert!(results.is_empty());
}

#[test]
fn test_registry_all_returns_all() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let all = registry.all();
    assert!(all.len() >= 14, "expected at least 14 plugins, got {}", all.len());
}

#[test]
fn test_pipeline_template_node_params_non_empty() {
    let mut params_map = std::collections::HashMap::new();
    params_map.insert("quality".into(), serde_json::json!(80.0));
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "heif_encoder".into(),
            label: None,
            enabled: true,
            params: Some(params_map),
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    assert!(template.validate().is_ok());
    let graph = template.into_graph();
    let node = &graph.nodes[0];
    assert!(node.parameter_overrides.is_some());
}

#[test]
fn test_pipeline_template_node_disabled() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "exif_rw".into(),
            label: None,
            enabled: false,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    assert!(template.validate().is_ok());
}

#[test]
fn test_pipeline_template_node_label_override() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "n1".into(),
            plugin: "exif_rw".into(),
            label: Some("Custom Label".into()),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    assert_eq!(graph.nodes[0].label, "Custom Label");
}

#[test]
fn test_pipeline_graph_serialize_empty_graph() {
    let graph = PipelineGraph::new();
    let json = serde_json::to_string(&graph).unwrap();
    let deserialized: PipelineGraph = serde_json::from_str(&json).unwrap();
    assert!(deserialized.nodes.is_empty());
    assert!(deserialized.edges.is_empty());
}

#[test]
fn test_pipeline_graph_same_node_connect_fails() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("p1".into(), "n1".into());
    let in1 = graph.node(n1).unwrap().inputs[0];
    let out1 = graph.node(n1).unwrap().outputs[0];
    assert!(graph.connect(out1, in1).is_err());
}

#[test]
fn test_pipeline_graph_has_cycle_on_dag() {
    let mut graph = PipelineGraph::new();
    let n1 = graph.add_node("a".into(), "A".into());
    let n2 = graph.add_node("b".into(), "B".into());
    let out1 = graph.node(n1).unwrap().outputs[0];
    let in2 = graph.node(n2).unwrap().inputs[0];
    graph.connect(out1, in2).unwrap();
    assert!(!graph.has_cycle());
}

#[test]
fn test_pipeline_graph_node_not_found_by_id() {
    let mut graph = PipelineGraph::new();
    graph.add_node("p1".into(), "n1".into());
    assert!(graph.node(Uuid::new_v4()).is_none());
}

#[test]
fn test_registry_manifests_after_register_all() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let manifests = registry.manifests();
    assert_eq!(manifests.len(), 14);
    for m in &manifests {
        assert!(m.version.major > 0 || m.version.minor > 0 || m.version.patch > 0);
        assert!(!m.tags.is_empty());
    }
}

#[test]
fn test_parameter_set_from_schema_defaults() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    for plugin in &registry.all() {
        let defaults = plugin.parameter_schema().defaults();
        let fields = plugin.parameter_schema().all_fields();
        assert_eq!(defaults.iter().count(), fields.len());
    }
}

#[test]
fn test_all_plugins_are_loaded_in_registry() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let all = registry.all();
    assert!(all.len() >= 14, "expected at least 14 plugins, got {}", all.len());
    for plugin in &all {
        assert!(!plugin.id().is_empty(), "plugin has empty id");
        assert!(!plugin.name().is_empty(), "plugin has empty name");
    }
}

#[test]
fn test_registry_register_get_remove_cycle() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);

    let count_before = registry.all().len();
    assert!(count_before >= 14);

    let manifests_before = registry.manifests().len();
    let first_id = registry.all()[0].id().clone();
    let removed = registry.unregister(&first_id);
    assert!(
        removed.is_some(),
        "unregister returned None for {}",
        first_id
    );
    let manifests_after = registry.manifests().len();
    assert_eq!(manifests_after, manifests_before - 1);
}

use photopipeline_core::{PluginCategory, PluginVersion, ImageFormat, GpuBackend, AiBackend};
use photopipeline_plugin::{
    registry::Registry, PluginQuery,
};
use photopipeline_engine::{PipelineTemplate, TemplateNode, TemplateEdge, PipelineGraph};

#[test]
fn test_register_all_plugins() {
    let registry = Registry::new();
    photopipeline_plugins::register_all(&registry);
    let all = registry.all();
    assert_eq!(all.len(), 14);
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
            TemplateEdge { from: "input".into(), to: "transform".into() },
            TemplateEdge { from: "transform".into(), to: "output".into() },
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
    assert_eq!(GpuBackend::OpenCL.to_string(), "OpenCL");
    assert_eq!(GpuBackend::ROCm.to_string(), "ROCm");
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

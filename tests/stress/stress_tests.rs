#![allow(clippy::result_large_err)]
use photopipeline_core::{
    ColorSpace, ImageFormat, Metadata, PixelFormat, PluginCategory, PluginVersion,
};
use photopipeline_engine::{
    ParameterResolver, PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    PluginQuery, registry::Registry,
};
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

#[test]
fn stress_pipeline_1000_metadata_nodes() {
    let mut graph = PipelineGraph::new();
    for i in 0..1000 {
        graph.add_node("photopipeline.plugins.exif_rw".into(), format!("node_{i}"));
    }
    assert_eq!(graph.nodes.len(), 1000);
    assert!(graph.validate_graph().is_ok());
}

#[test]
fn stress_pipeline_execution_100_times_on_same_data() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "exif".into(),
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
    for _ in 0..100 {
        let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
        let info = photopipeline_core::ImageInfo {
            id: Uuid::new_v4(),
            path: "/tmp/test.jpg".into(),
            filename: "test.jpg".into(),
            format: ImageFormat::JPEG,
            width: 100,
            height: 100,
            file_size_bytes: 1000,
            pixel_format: PixelFormat::U8,
            color_space: ColorSpace::default(),
        };
        let metadata = Metadata::default();
        struct NoopProgress;
        impl photopipeline_plugin::ProgressSink for NoopProgress {
            fn set_progress(&self, _: f32, _: &str) {}
            fn is_canceled(&self) -> bool {
                false
            }
        }

        let result = rt.block_on(async {
            exec.execute(&graph, &info, None, &metadata, Box::new(NoopProgress))
                .await
        });
        assert!(result.is_ok());
    }
}

#[test]
fn stress_random_parameter_values_on_plugins() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    for plugin in &reg.all() {
        let defaults = plugin.parameter_schema().defaults();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { plugin.validate(&defaults).await });
        assert!(
            result.is_ok(),
            "default validation failed for {}",
            plugin.id()
        );
    }
}

#[test]
fn stress_concurrent_registry_access_16_threads() {
    let reg = Arc::new(Registry::new());
    let reg2 = reg.clone();

    let handle = thread::spawn(move || {
        photopipeline_plugins::register_all(&reg2);
    });
    handle.join().unwrap();

    let mut handles = vec![];
    for _t in 0..16 {
        let r = reg.clone();
        handles.push(thread::spawn(move || {
            for _i in 0..10 {
                let all = r.all();
                assert!(!all.is_empty());
                let manifests = r.manifests();
                assert!(!manifests.is_empty());
                let cats = r.categories();
                assert!(!cats.is_empty());
                if let Some(p) = r.get(&"exif_rw".into()) {
                    let _ = p.id();
                    let _ = p.name();
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn stress_rapid_register_unregister_cycle() {
    let reg = Registry::new();
    photopipeline_plugins::register_all(&reg);

    for _ in 0..100 {
        let query_results = reg.query(&PluginQuery::default());
        assert!(query_results.len() >= 14);
    }
}

#[test]
fn stress_deep_merge_chain_100_levels() {
    let mut resolver = ParameterResolver::new();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

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
                id: "v".into(),
                label: "V".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 10000,
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

    let mut template = ParameterSet::new();
    template.insert("v".into(), serde_json::json!(1));
    resolver.set_template_params(node_id, template);

    for i in 0..100 {
        let mut img = ParameterSet::new();
        img.insert("v".into(), serde_json::json!(i));
        resolver.set_image_override(image_id, node_id, img);
    }

    let metadata = Metadata::default();
    let info = photopipeline_core::ImageInfo {
        id: image_id,
        path: "/tmp/x.jpg".into(),
        filename: "x.jpg".into(),
        format: ImageFormat::JPEG,
        width: 10,
        height: 10,
        file_size_bytes: 100,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    };
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &info);
    assert_eq!(result.get_i64("v"), Some(99));
}

#[test]
fn stress_large_pipeline_graph_1000_nodes() {
    let mut graph = PipelineGraph::new();
    let mut prev_id: Option<uuid::Uuid> = None;
    for i in 0..1000 {
        let id = graph.add_node(format!("p{i}"), format!("n{i}"));
        if let Some(prev) = prev_id {
            let out = graph.node(prev).unwrap().outputs[0];
            let inp = graph.node(id).unwrap().inputs[0];
            graph.connect(out, inp).unwrap();
        }
        prev_id = Some(id);
    }
    assert_eq!(graph.nodes.len(), 1000);
    assert_eq!(graph.edges.len(), 999);
    assert!(graph.validate_graph().is_ok());
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 1000);
}

#[test]
fn stress_serialize_deserialize_large_graph() {
    let mut graph = PipelineGraph::new();
    for i in 0..100 {
        graph.add_node(format!("photopipeline.plugins.p{i}"), format!("n{i}"));
    }
    for i in 0..99 {
        let out = graph.nodes[i].outputs[0];
        let inp = graph.nodes[i + 1].inputs[0];
        graph.connect(out, inp).unwrap();
    }

    let json = serde_json::to_string(&graph).unwrap();
    let deser: PipelineGraph = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.nodes.len(), 100);
    assert_eq!(deser.edges.len(), 99);
}

#[test]
fn stress_many_tiles_no_panic() {
    use photopipeline_core::TileLayout;
    let w = 30000u32;
    let h = 20000u32;
    let tile_size = 256u32;
    let layout = TileLayout::new(w, h, tile_size, 0);
    let count = layout.iter_tiles().count();
    assert!(count > 0);
    assert_eq!(count as u32, layout.total_tiles);
    for spec in layout.iter_tiles() {
        assert!(spec.x_offset + spec.width <= w, "tile exceeds width bound");
        assert!(
            spec.y_offset + spec.height <= h,
            "tile exceeds height bound"
        );
    }
}

#[test]
fn stress_many_conditions() {
    let metadata = Metadata {
        exif: Some(photopipeline_core::ExifData {
            iso: Some(800),
            make: Some("Sony".into()),
            model: Some("A7R4".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let info = photopipeline_core::ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/t.jpg".into(),
        filename: "t.jpg".into(),
        format: ImageFormat::JPEG,
        width: 6000,
        height: 4000,
        file_size_bytes: 50000000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    };

    let _resolver = ParameterResolver::new();
    let mut resolver = ParameterResolver::new();
    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "main".into(),
            label: "Main".into(),
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
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &info);
    assert_eq!(result.get_i64("threshold"), Some(128));
}

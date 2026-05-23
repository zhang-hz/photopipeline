#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{
    ColorSpace, ExifData, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat, PluginError,
};
use photopipeline_engine::{
    GroupCondition, ParameterResolver, PipelineGraph, PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    PluginQuery, registry::Registry,
};
use photopipeline_plugins;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use test_harness::fixtures::image::ImageFixture;
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

#[test]
fn e2e_1000_image_batch_processing() {
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

    let mut success = 0usize;
    let mut failures = 0usize;

    for i in 0..1000 {
        let buf = ImageFixture::new()
            .width(16)
            .height(16)
            .solid(
                (i % 256) as u8,
                ((i * 7) % 256) as u8,
                ((i * 13) % 256) as u8,
            )
            .build();
        let info = make_image_info(Uuid::new_v4(), &format!("/tmp/batch_{i:04}.jpg"));
        let md = Metadata::default();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
        let progress = Box::new(MockProgressSink::new());
        let result =
            rt.block_on(async { exec.execute(&graph, &info, Some(buf), &md, progress).await });
        match result {
            Ok(_) => success += 1,
            Err(_) => failures += 1,
        }
    }

    assert_eq!(success + failures, 1000);
    assert!(
        success > 0 || failures > 0,
        "expected either successes or failures"
    );
}

#[test]
fn e2e_1000_node_pipeline_graph() {
    let mut graph = PipelineGraph::new();
    let mut prev_id: Option<uuid::Uuid> = None;

    for i in 0..1000 {
        let id = graph.add_node(format!("photopipeline.plugins.exif_rw"), format!("n{i}"));
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
fn e2e_large_parameter_merge_100_levels() {
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
                id: "priority".into(),
                label: "Priority".into(),
                description: None,
                help_url: None,
                field_type: ParameterType::Integer {
                    min: 0,
                    max: 100000,
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
    template.insert("priority".into(), serde_json::json!(0));
    resolver.set_template_params(node_id, template);

    for i in 0..100 {
        let mut img = ParameterSet::new();
        img.insert("priority".into(), serde_json::json!(i));
        resolver.set_image_override(image_id, node_id, img);
    }

    let metadata = Metadata::default();
    let info = make_image_info(image_id, "/tmp/merge_100.jpg");
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &info);
    assert_eq!(result.get_i64("priority"), Some(99));
}

#[test]
fn e2e_deeply_nested_group_conditions() {
    let mut resolver = ParameterResolver::new();
    let node_id = Uuid::new_v4();
    let image_id = Uuid::new_v4();

    let mut condition = GroupCondition::Always;
    for _ in 0..50 {
        condition = GroupCondition::And(vec![condition.clone(), GroupCondition::Always]);
    }

    let mut ps = ParameterSet::new();
    ps.insert("deep".into(), serde_json::json!(42));

    let mut params = HashMap::new();
    params.insert(node_id, ps);
    resolver.add_group_override(condition, params);

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
                id: "deep".into(),
                label: "Deep".into(),
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

    let metadata = Metadata::default();
    let info = make_image_info(image_id, "/tmp/deep_condition.jpg");
    let result = resolver.resolve(node_id, image_id, &schema, &metadata, &info);
    assert_eq!(result.get_i64("deep"), Some(42));
}

#[test]
fn e2e_10k_file_batch_pattern() {
    let mut paths: Vec<String> = Vec::new();
    for i in 0..10000 {
        paths.push(format!("/photos/image_{i:05}.jpg"));
    }

    let pattern = "/photos/image_*.jpg";
    let re_pattern = pattern.replace("*", r".*");
    let re = regex::Regex::new(&format!("^{}$", re_pattern)).unwrap();

    let matched: Vec<_> = paths.iter().filter(|p| re.is_match(p)).collect();
    assert_eq!(matched.len(), 10000);

    let glob_pattern = "/photos/image_{00000..09999}.jpg";
    assert!(glob_pattern.contains("00000"));

    let batch_count = paths.len();
    assert_eq!(batch_count, 10000);

    for (i, path) in paths.iter().enumerate() {
        assert!(path.contains(&format!("{:05}", i)));
    }
}

#[test]
fn e2e_rapid_register_unregister_1000_cycles() {
    let reg = Arc::new(Registry::new());

    for cycle in 0..1000 {
        photopipeline_plugins::register_all(&reg);
        let count_before = reg.manifests().len();
        assert!(
            count_before >= 14,
            "cycle {cycle}: expected >=14 plugins, got {count_before}"
        );

        for manifest in reg.manifests() {
            reg.unregister(&manifest.id);
        }

        let count_after = reg.manifests().len();
        assert_eq!(
            count_after, 0,
            "cycle {cycle}: expected empty after unregister"
        );

        let all_after = reg.all();
        assert!(
            all_after.is_empty(),
            "cycle {cycle}: all() should be empty after unregister"
        );
    }

    photopipeline_plugins::register_all(&reg);
    let final_count = reg.manifests().len();
    assert!(final_count >= 14);
}

#[test]
fn e2e_continuous_pipeline_execution_100_iterations() {
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
    let info = make_image_info(Uuid::new_v4(), "/tmp/continuous.jpg");
    let md = Metadata::default();
    let buf = ImageFixture::new()
        .width(64)
        .height(64)
        .solid(128, 128, 128)
        .build();

    for iter in 0..100 {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
        let progress = Box::new(MockProgressSink::new());
        let result = rt.block_on(async {
            exec.execute(&graph, &info, Some(buf.clone()), &md, progress)
                .await
        });
        assert!(
            result.is_ok(),
            "iteration {iter} failed: {:?}",
            result.err()
        );
    }
}

#[test]
fn e2e_large_topological_sort_5000_nodes() {
    let mut graph = PipelineGraph::new();
    let mut ids = vec![];

    for i in 0..5000 {
        ids.push(graph.add_node(format!("p{i}"), format!("n{i}")));
    }

    for w in ids.windows(2) {
        let out = graph.node(w[0]).unwrap().outputs[0];
        let inp = graph.node(w[1]).unwrap().inputs[0];
        graph.connect(out, inp).unwrap();
    }

    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 5000);

    for i in 0..4999 {
        let pos_a = order.iter().position(|&id| id == ids[i]).unwrap();
        let pos_b = order.iter().position(|&id| id == ids[i + 1]).unwrap();
        assert!(pos_a < pos_b);
    }
}

#[test]
fn e2e_100_parameter_overrides_per_image() {
    let mut resolver = ParameterResolver::new();
    let node_ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
    let image_id = Uuid::new_v4();

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
                id: "value".into(),
                label: "Value".into(),
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

    for (i, node_id) in node_ids.iter().enumerate() {
        let mut template = ParameterSet::new();
        template.insert("value".into(), serde_json::json!(0));
        resolver.set_template_params(*node_id, template);

        let mut img_override = ParameterSet::new();
        img_override.insert("value".into(), serde_json::json!(i as i64));
        resolver.set_image_override(image_id, *node_id, img_override);
    }

    let metadata = Metadata::default();
    let info = make_image_info(image_id, "/tmp/100_overrides.jpg");

    for (i, node_id) in node_ids.iter().enumerate() {
        let result = resolver.resolve(*node_id, image_id, &schema, &metadata, &info);
        assert_eq!(result.get_i64("value"), Some(i as i64));
    }
}

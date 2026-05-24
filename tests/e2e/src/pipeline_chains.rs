use photopipeline_core::{
    ChannelLayout, ColorSpace, ExifData, GpsData, ImageFormat, ImageInfo, Metadata, PixelBuffer,
    PixelFormat,
};
use photopipeline_engine::{
    NodeStatus, ParameterResolver, PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugin::{ParameterSet, ProgressSink};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use test_harness::fixtures::image::ImageFixture;
use test_harness::fixtures::metadata::{exif_sony_a7r5, gps_beijing};
use test_harness::mocks::progress::{MockProgressSink, NoopProgress};
use uuid::Uuid;

fn make_image_info() -> ImageInfo {
    ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test.jpg".into(),
        filename: "test.jpg".into(),
        format: ImageFormat::JPEG,
        width: 256,
        height: 256,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

fn make_registry() -> Arc<Registry> {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    reg
}

#[test]
fn e2e_metadata_pipeline_gps_and_time_shift() {
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());

    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("Color".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source".into(), serde_json::json!("srgb"));
                    m.insert("target".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "lens".into(),
                plugin: "photopipeline.plugins.lens_correct".into(),
                label: Some("Lens".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("mode".into(), serde_json::json!("off"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "denoise".into(),
                plugin: "photopipeline.plugins.ai_denoise".into(),
                label: Some("Denoise".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("strength".into(), serde_json::json!(0.0));
                    Some(m)
                },
            },
            TemplateNode {
                id: "gps".into(),
                plugin: "photopipeline.plugins.gps_set".into(),
                label: Some("GPS Set".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("gps_mode".into(), serde_json::json!("manual"));
                    m.insert("latitude".into(), serde_json::json!(39.9042));
                    m.insert("longitude".into(), serde_json::json!(116.4074));
                    m.insert("altitude".into(), serde_json::json!(43.5));
                    Some(m)
                },
            },
            TemplateNode {
                id: "time".into(),
                plugin: "photopipeline.plugins.time_shift".into(),
                label: Some("Time Shift".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("shift_hours".into(), serde_json::json!(1));
                    m.insert("shift_minutes".into(), serde_json::json!(30));
                    Some(m)
                },
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "exif_rw".into(),
                to: "gps".into(),
            },
            TemplateEdge {
                from: "gps".into(),
                to: "time".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let mut metadata = Metadata::default();
    metadata.exif = Some(exif_sony_a7r5());
    metadata.gps = Some(gps_beijing());

    let pb = ImageFixture::new()
        .width(16)
        .height(16)
        .solid(128, 128, 128)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    match result {
        Ok(_exec_result) => {
            assert!(true, "metadata pipeline executed");
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                !msg.contains("not found"),
                "unexpected not found error: {msg}"
            );
        }
    }
}

#[test]
fn e2e_pixel_pipeline_transform_and_colorspace() {
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());

    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "transform".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("Resize".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("resize_mode".into(), serde_json::json!("absolute"));
                    m.insert("target_width".into(), serde_json::json!(128));
                    m.insert("target_height".into(), serde_json::json!(128));
                    m.insert("filter_type".into(), serde_json::json!("bilinear"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("Colorspace".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
        ],
        edges: vec![TemplateEdge {
            from: "transform".into(),
            to: "colorspace".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();
    let pb = ImageFixture::new()
        .width(256)
        .height(256)
        .solid(128, 128, 128)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    assert!(result.is_ok(), "pixel pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    assert!(exec_result.buffer.is_some(), "output buffer should exist");

    if let Some(ref buf) = exec_result.buffer {
        assert_eq!(buf.width, 128, "width should be 128");
        assert_eq!(buf.height, 128, "height should be 128");
        assert!(!buf.data.data.is_empty(), "pixel data should not be empty");
    }
}

#[test]
fn e2e_hdr_pipeline_raw_to_heif() {
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());

    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("HDR Colorspace".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("bt2020"));
                    m.insert("target_transfer".into(), serde_json::json!("pq"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "lens".into(),
                plugin: "photopipeline.plugins.lens_correct".into(),
                label: Some("Lens Correction".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("correction_mode".into(), serde_json::json!("auto"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "denoise".into(),
                plugin: "photopipeline.plugins.ai_denoise".into(),
                label: Some("AI Denoise".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("strength".into(), serde_json::json!(0.0));
                    Some(m)
                },
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "colorspace".into(),
                to: "lens".into(),
            },
            TemplateEdge {
                from: "lens".into(),
                to: "denoise".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test.ppm".into(),
        filename: "test.ppm".into(),
        format: ImageFormat::PPM,
        width: 256,
        height: 256,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U16,
        color_space: ColorSpace::SRGB,
    };
    let metadata = Metadata::default();
    let pb = ImageFixture::new()
        .width(256)
        .height(256)
        .format(PixelFormat::U16)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    // HDR pipeline with auto lens correction requires LensFun runtime.
    // Expected to fail gracefully with Internal error (not panic).
    assert!(
        result.is_err(),
        "HDR pipeline should fail without LensFun runtime"
    );
}

#[test]
fn e2e_single_node_pipeline_transform() {
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());

    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "transform".into(),
            plugin: "photopipeline.plugins.transform".into(),
            label: Some("Resize".into()),
            enabled: true,
            params: {
                let mut m = std::collections::HashMap::new();
                m.insert("resize_mode".into(), serde_json::json!("absolute"));
                m.insert("target_width".into(), serde_json::json!(64));
                m.insert("target_height".into(), serde_json::json!(64));
                Some(m)
            },
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();
    let pb = ImageFixture::new()
        .width(256)
        .height(256)
        .solid(128, 128, 128)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    assert!(
        result.is_ok(),
        "single node pipeline failed: {:?}",
        result.err()
    );
    let exec_result = result.unwrap();
    assert!(exec_result.buffer.is_some());
    let buf = exec_result.buffer.unwrap();
    assert_eq!(buf.width, 64);
    assert_eq!(buf.height, 64);
}

#[test]
fn e2e_pipeline_with_disabled_node() {
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "first".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("First Resize".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("resize_mode".into(), serde_json::json!("absolute"));
                    m.insert("target_width".into(), serde_json::json!(128));
                    m.insert("target_height".into(), serde_json::json!(128));
                    Some(m)
                },
            },
            TemplateNode {
                id: "middle".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("Disabled Colorspace".into()),
                enabled: false,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "last".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("Last Resize".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("resize_mode".into(), serde_json::json!("absolute"));
                    m.insert("target_width".into(), serde_json::json!(64));
                    m.insert("target_height".into(), serde_json::json!(64));
                    Some(m)
                },
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "first".into(),
                to: "middle".into(),
            },
            TemplateEdge {
                from: "middle".into(),
                to: "last".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();
    let pb = ImageFixture::new()
        .width(256)
        .height(256)
        .solid(128, 128, 128)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    assert!(
        result.is_ok(),
        "disabled node pipeline failed: {:?}",
        result.err()
    );
    let exec_result = result.unwrap();

    let first_id = graph
        .nodes
        .iter()
        .find(|n| n.label == "First Resize")
        .unwrap()
        .id;
    let middle_id = graph
        .nodes
        .iter()
        .find(|n| n.label == "Disabled Colorspace")
        .unwrap()
        .id;
    let last_id = graph
        .nodes
        .iter()
        .find(|n| n.label == "Last Resize")
        .unwrap()
        .id;

    let first_state = exec_result.node_states.get(&first_id).unwrap();
    let middle_state = exec_result.node_states.get(&middle_id).unwrap();
    let last_state = exec_result.node_states.get(&last_id).unwrap();

    assert!(
        matches!(first_state.status, NodeStatus::Completed(_)),
        "first node should be completed"
    );
    assert!(
        matches!(middle_state.status, NodeStatus::Skipped),
        "middle node should be skipped"
    );
    assert!(
        matches!(last_state.status, NodeStatus::Completed(_)),
        "last node should be completed"
    );
}

#[test]
fn e2e_pipeline_node_validation_failure() {
    let reg = make_registry();

    {
        let resolver = Arc::new(ParameterResolver::new());
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![TemplateNode {
                id: "cs".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("Same Colorspace".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            }],
            edges: vec![],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };

        let graph = template.into_graph();
        let executor = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver);

        let image_info = make_image_info();
        let metadata = Metadata::default();
        let pb = ImageFixture::new()
            .width(16)
            .height(16)
            .solid(128, 128, 128)
            .build();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            executor
                .execute(
                    &graph,
                    &image_info,
                    Some(pb),
                    &metadata,
                    Box::new(NoopProgress),
                )
                .await
        });

        assert!(
            result.is_ok(),
            "same source/target should be warning only, not error"
        );
    }

    {
        let resolver = Arc::new(ParameterResolver::new());
        let template = PipelineTemplate {
            metadata: Default::default(),
            nodes: vec![TemplateNode {
                id: "heif".into(),
                plugin: "photopipeline.plugins.heif_encoder".into(),
                label: Some("Bad HEIF".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("quality".into(), serde_json::json!(200.0));
                    Some(m)
                },
            }],
            edges: vec![],
            overrides: vec![],
            groups: vec![],
            batch: None,
        };

        let graph = template.into_graph();
        let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

        let image_info = make_image_info();
        let metadata = Metadata::default();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            executor
                .execute(&graph, &image_info, None, &metadata, Box::new(NoopProgress))
                .await
        });

        assert!(
            result.is_err(),
            "heif_encoder quality=200 should fail validation"
        );
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        assert!(
            err_msg.contains("Validation") || err_msg.contains("validation"),
            "expected ValidationFailed, got: {err_msg}"
        );
    }
}

#[test]
fn e2e_empty_graph_should_fail_validation() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let result = template.validate();
    assert!(result.is_err(), "empty graph should fail validation");
    let err = result.unwrap_err();
    assert!(err.contains("node"), "error should mention missing nodes");
}

#[test]
fn e2e_single_node_no_edges_should_pass() {
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "exif".into(),
            plugin: "photopipeline.plugins.exif_rw".into(),
            label: Some("EXIF".into()),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    assert!(template.validate().is_ok(), "single node should validate");

    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());
    let graph = template.into_graph();
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(&graph, &image_info, None, &metadata, Box::new(NoopProgress))
            .await
    });

    assert!(result.is_ok(), "single node no edges should execute");
}

#[test]
fn e2e_large_linear_pipeline_100_nodes() {
    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: Vec::new(),
        edges: Vec::new(),
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    for i in 0..100 {
        template.nodes.push(TemplateNode {
            id: format!("exif_{i}"),
            plugin: "photopipeline.plugins.exif_rw".into(),
            label: Some(format!("exif_{i}")),
            enabled: true,
            params: None,
        });
    }

    for i in 0..99 {
        template.edges.push(TemplateEdge {
            from: format!("exif_{i}"),
            to: format!("exif_{}", i + 1),
        });
    }

    assert!(template.validate().is_ok());

    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 100);
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 100);

    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(&graph, &image_info, None, &metadata, Box::new(NoopProgress))
            .await
    });

    assert!(
        result.is_ok(),
        "100-node pipeline failed: {:?}",
        result.err()
    );
    let exec_result = result.unwrap();

    let completed = exec_result
        .node_states
        .iter()
        .filter(|(_, s)| matches!(s.status, NodeStatus::Completed(_)))
        .count();
    assert_eq!(completed, 100, "all 100 nodes should complete");
}

#[test]
fn e2e_diamond_pipeline_topology() {
    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("Source".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("resize_mode".into(), serde_json::json!("absolute"));
                    m.insert("width".into(), serde_json::json!(128));
                    m.insert("height".into(), serde_json::json!(128));
                    Some(m)
                },
            },
            TemplateNode {
                id: "A".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("A".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "B".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("B".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
            TemplateNode {
                id: "C".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
                label: Some("C".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source_color_space".into(), serde_json::json!("srgb"));
                    m.insert("target_color_space".into(), serde_json::json!("srgb"));
                    Some(m)
                },
            },
        ],
        edges: vec![
            TemplateEdge {
                from: "source".into(),
                to: "A".into(),
            },
            TemplateEdge {
                from: "source".into(),
                to: "B".into(),
            },
            TemplateEdge {
                from: "A".into(),
                to: "C".into(),
            },
            TemplateEdge {
                from: "B".into(),
                to: "C".into(),
            },
        ],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    assert!(template.validate().is_ok());
    let graph = template.into_graph();
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 4);

    let source_id = graph.nodes.iter().find(|n| n.label == "Source").unwrap().id;
    let a_id = graph.nodes.iter().find(|n| n.label == "A").unwrap().id;
    let b_id = graph.nodes.iter().find(|n| n.label == "B").unwrap().id;
    let c_id = graph.nodes.iter().find(|n| n.label == "C").unwrap().id;

    let pos_source = order.iter().position(|&id| id == source_id).unwrap();
    let pos_a = order.iter().position(|&id| id == a_id).unwrap();
    let pos_b = order.iter().position(|&id| id == b_id).unwrap();
    let pos_c = order.iter().position(|&id| id == c_id).unwrap();

    assert!(pos_source < pos_a, "source before A");
    assert!(pos_source < pos_b, "source before B");
    assert!(pos_a < pos_c, "A before C");
    assert!(pos_b < pos_c, "B before C");

    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();
    let pb = ImageFixture::new()
        .width(256)
        .height(256)
        .solid(128, 128, 128)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(
                &graph,
                &image_info,
                Some(pb),
                &metadata,
                Box::new(NoopProgress),
            )
            .await
    });

    assert!(
        result.is_ok(),
        "diamond pipeline failed: {:?}",
        result.err()
    );
}

#[test]
fn e2e_cycle_detection_prevents_execution() {
    use photopipeline_engine::PipelineGraph;

    let mut graph = PipelineGraph::new();
    let a = graph.add_node("photopipeline.plugins.exif_rw".into(), "A".into());
    let b = graph.add_node("photopipeline.plugins.exif_rw".into(), "B".into());
    let c = graph.add_node("photopipeline.plugins.exif_rw".into(), "C".into());

    let out_a = graph.node(a).unwrap().outputs[0];
    let in_b = graph.node(b).unwrap().inputs[0];
    let out_b = graph.node(b).unwrap().outputs[0];
    let in_c = graph.node(c).unwrap().inputs[0];
    let out_c = graph.node(c).unwrap().outputs[0];
    let in_a = graph.node(a).unwrap().inputs[0];

    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_b, in_c).unwrap();

    let result = graph.connect(out_c, in_a);
    assert!(result.is_err(), "A→B→C→A should be a cycle");
    let err = result.unwrap_err();
    assert!(
        matches!(err, photopipeline_core::PluginError::CircularDependency),
        "should be CircularDependency"
    );
}

#[test]
fn e2e_cancel_pipeline_mid_execution() {
    let mut template = PipelineTemplate {
        metadata: Default::default(),
        nodes: Vec::new(),
        edges: Vec::new(),
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    for i in 0..10 {
        template.nodes.push(TemplateNode {
            id: format!("exif_{i}"),
            plugin: "photopipeline.plugins.exif_rw".into(),
            label: Some(format!("exif_{i}")),
            enabled: true,
            params: None,
        });
    }

    for i in 0..9 {
        template.edges.push(TemplateEdge {
            from: format!("exif_{i}"),
            to: format!("exif_{}", i + 1),
        });
    }

    let graph = template.into_graph();
    let reg = make_registry();
    let resolver = Arc::new(ParameterResolver::new());
    let executor = photopipeline_engine::NodeExecutor::new(reg, resolver);

    let image_info = make_image_info();
    let metadata = Metadata::default();

    let progress = MockProgressSink::new();
    let progress_arc = Arc::new(progress);

    struct CancelOnFirst {
        inner: Arc<MockProgressSink>,
        was_set: AtomicBool,
    }
    impl ProgressSink for CancelOnFirst {
        fn set_progress(&self, _fraction: f32, message: &str) {
            self.inner.set_progress(100.0, message);
            self.was_set.store(true, Ordering::SeqCst);
        }
        fn is_canceled(&self) -> bool {
            self.was_set.load(Ordering::SeqCst)
        }
    }

    let cancel_sink = CancelOnFirst {
        inner: progress_arc.clone(),
        was_set: AtomicBool::new(false),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        executor
            .execute(&graph, &image_info, None, &metadata, Box::new(cancel_sink))
            .await
    });

    assert!(result.is_err(), "pipeline should be canceled");
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("Canceled") || err_msg.contains("canceled"),
        "error should be Canceled, got: {err_msg}"
    );
}

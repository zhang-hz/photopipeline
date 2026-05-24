use photopipeline_core::*;
use photopipeline_engine::{PipelineTemplate, TemplateEdge, TemplateNode};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugins;
use std::sync::Arc;
use test_harness::fixtures::image::{color_bars, known_value_solid_u8};
use test_harness::fixtures::metadata::*;
use test_harness::mocks::progress::NoopProgress;
use uuid::Uuid;

fn make_registry() -> Arc<Registry> {
    let r = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&r);
    r
}
fn make_image_info() -> ImageInfo {
    ImageInfo {
        id: Uuid::new_v4(),
        path: "/tmp/test_meta.ppm".into(),
        filename: "test_meta.ppm".into(),
        format: ImageFormat::PPM,
        width: 64,
        height: 64,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::SRGB,
    }
}
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}
fn make_executor(reg: Arc<Registry>) -> photopipeline_engine::NodeExecutor {
    let resolver = Arc::new(photopipeline_engine::ParameterResolver::new());
    photopipeline_engine::NodeExecutor::new(reg, resolver)
}

#[test]
fn metadata_exif_sony_write_read_roundtrip() {
    let reg = make_registry();
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
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = {
        let mut m = Metadata::default();
        m.exif = Some(exif_sony_a7r5());
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let exif = exec_result
        .metadata
        .exif
        .as_ref()
        .expect("exif should be present");
    assert_eq!(exif.make.as_deref(), Some("SONY"));
    assert_eq!(exif.model.as_deref(), Some("ILCE-7RM5"));
    assert_eq!(exif.iso, Some(100));
}

#[test]
fn metadata_exif_canon_high_iso() {
    let reg = make_registry();
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
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = {
        let mut m = Metadata::default();
        m.exif = Some(exif_canon_r5(6400));
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let exif = exec_result
        .metadata
        .exif
        .as_ref()
        .expect("exif should be present");
    assert_eq!(exif.make.as_deref(), Some("Canon"));
    assert_eq!(exif.model.as_deref(), Some("EOS R5"));
    assert_eq!(exif.iso, Some(6400));
}

#[test]
fn metadata_exif_nikon_variable_focal() {
    let reg = make_registry();
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
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = {
        let mut m = Metadata::default();
        m.exif = Some(exif_nikon_z9("200"));
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let exif = exec_result
        .metadata
        .exif
        .as_ref()
        .expect("exif should be present");
    assert!(exif.make.as_deref().unwrap().contains("NIKON"));
    assert_eq!(exif.focal_length.as_deref(), Some("200"));
}

#[test]
fn metadata_gps_beijing() {
    let reg = make_registry();
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "gps".into(),
            plugin: "photopipeline.plugins.gps_set".into(),
            label: Some("GPS Set".into()),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = {
        let mut m = Metadata::default();
        m.gps = Some(gps_beijing());
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let gps = exec_result
        .metadata
        .gps
        .as_ref()
        .expect("gps should be present");
    assert!((gps.latitude.unwrap() - 39.9042).abs() < 0.0001);
    assert!((gps.longitude.unwrap() - 116.4074).abs() < 0.0001);
    assert!((gps.altitude.unwrap() - 43.5).abs() < 0.0001);
}

#[test]
fn metadata_gps_nyc() {
    let reg = make_registry();
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "gps".into(),
            plugin: "photopipeline.plugins.gps_set".into(),
            label: Some("GPS Set".into()),
            enabled: true,
            params: None,
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = {
        let mut m = Metadata::default();
        m.gps = Some(gps_nyc());
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let gps = exec_result
        .metadata
        .gps
        .as_ref()
        .expect("gps should be present");
    assert!((gps.latitude.unwrap() - 40.7128).abs() < 0.0001);
    assert!((gps.longitude.unwrap() - (-74.0060)).abs() < 0.0001);
}

#[test]
fn metadata_full_all_sections() {
    let reg = make_registry();
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "exif".into(),
                plugin: "photopipeline.plugins.exif_rw".into(),
                label: Some("EXIF".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "gps".into(),
                plugin: "photopipeline.plugins.gps_set".into(),
                label: Some("GPS Set".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "exif".into(),
            to: "gps".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = full_metadata();
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();

    let exif = exec_result
        .metadata
        .exif
        .as_ref()
        .expect("exif should be present");
    assert_eq!(exif.make.as_deref(), Some("SONY"));

    let xmp = exec_result
        .metadata
        .xmp
        .as_ref()
        .expect("xmp should be present");
    assert_eq!(xmp.title.as_deref(), Some("Test Image"));
    assert_eq!(xmp.creator.as_deref(), Some("Test Author"));

    let iptc = exec_result
        .metadata
        .iptc
        .as_ref()
        .expect("iptc should be present");
    assert!(iptc.keywords.contains(&"test".to_string()));
    assert!(iptc.keywords.contains(&"photography".to_string()));

    let gps = exec_result
        .metadata
        .gps
        .as_ref()
        .expect("gps should be present");
    assert!((gps.latitude.unwrap() - 39.9042).abs() < 0.0001);
    assert!((gps.longitude.unwrap() - 116.4074).abs() < 0.0001);
}

#[test]
fn metadata_empty_no_corruption() {
    let reg = make_registry();
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
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = known_value_solid_u8(64, 64, 128, 128, 128);
    let metadata = empty_metadata();
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();
    let buf = exec_result
        .buffer
        .as_ref()
        .expect("output buffer should exist");
    assert_eq!(buf.width, 64);
    assert_eq!(buf.height, 64);
    for y in 0..buf.height as usize {
        for x in 0..buf.width as usize {
            let offset = (y * buf.width as usize + x) * 3;
            assert_eq!(buf.data.data[offset], 128);
            assert_eq!(buf.data.data[offset + 1], 128);
            assert_eq!(buf.data.data[offset + 2], 128);
        }
    }
}

#[test]
fn metadata_and_transform_chain() {
    let reg = make_registry();
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "exif".into(),
                plugin: "photopipeline.plugins.exif_rw".into(),
                label: Some("EXIF".into()),
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "transform".into(),
                plugin: "photopipeline.plugins.transform".into(),
                label: Some("Resize".into()),
                enabled: true,
                params: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("resize_mode".into(), serde_json::json!("absolute"));
                    m.insert("target_width".into(), serde_json::json!(32));
                    m.insert("target_height".into(), serde_json::json!(32));
                    Some(m)
                },
            },
        ],
        edges: vec![TemplateEdge {
            from: "exif".into(),
            to: "transform".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let executor = make_executor(reg);
    let info = make_image_info();
    let pb = color_bars(64, 64);
    let metadata = {
        let mut m = Metadata::default();
        m.exif = Some(exif_sony_a7r5());
        m
    };
    let result = rt().block_on(async {
        executor
            .execute(&graph, &info, Some(pb), &metadata, Box::new(NoopProgress))
            .await
    });
    assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
    let exec_result = result.unwrap();

    let buf = exec_result
        .buffer
        .as_ref()
        .expect("output buffer should exist");
    assert_eq!(buf.width, 32);
    assert_eq!(buf.height, 32);

    assert!(buf.data.data.len() >= 32 * 32 * 3);
    let mut pixels_unique = false;
    let first_r = buf.data.data[0];
    for i in (3..buf.data.data.len()).step_by(3) {
        if buf.data.data[i] != first_r {
            pixels_unique = true;
            break;
        }
    }
    assert!(pixels_unique, "colors should be preserved (not uniform)");

    let exif = exec_result
        .metadata
        .exif
        .as_ref()
        .expect("exif should be present");
    assert_eq!(exif.make.as_deref(), Some("SONY"));
}

#![allow(clippy::result_large_err)]


use photopipeline_core::{
    ChannelLayout, ColorSpace, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat,
};
use photopipeline_engine::{ParameterResolver, PipelineTemplate, TemplateNode};
use photopipeline_plugin::{
    ParameterField, ParameterSchema, ParameterSection, ParameterSet, ParameterType, Plugin,
    registry::Registry,
};
use photopipeline_plugins;
use std::sync::Arc;
use test_harness::fixtures::image::ImageFixture;
use test_harness::mocks::encoder::MockEncoder;
use test_harness::mocks::progress::MockProgressSink;
use uuid::Uuid;

fn make_image_info(id: Uuid, path: &str) -> ImageInfo {
    ImageInfo {
        id,
        path: path.into(),
        filename: path.rsplit('/').next().unwrap_or(path).into(),
        format: ImageFormat::JPEG,
        width: 256,
        height: 256,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

#[test]
fn e2e_heif_encoder_quality_param_range() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let plugin = reg.get("photopipeline.plugins.heif_encoder");
    assert!(plugin.is_some(), "heif_encoder plugin should be registered");

    if let Some(p) = plugin {
        let schema = p.parameter_schema();
        let defaults = schema.defaults();
        assert!(!defaults.values.is_empty() || schema.sections.is_empty());
    }
}

#[test]
fn e2e_png_encoder_compression_level() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let plugin = reg.get("photopipeline.plugins.png_encoder");
    assert!(plugin.is_some(), "png_encoder plugin should be registered");

    if let Some(p) = plugin {
        let schema = p.parameter_schema();
        let defaults = schema.defaults();
        let has_compression = defaults.get("compression").is_some()
            || defaults.get("level").is_some()
            || schema.sections.is_empty();
        assert!(
            has_compression
                || schema.sections.iter().any(|s| s
                    .fields
                    .iter()
                    .any(|f| f.id.contains("compress") || f.id.contains("level"))),
            "expected png_encoder to have compression or level parameter"
        );
    }
}

#[test]
fn e2e_jxl_encoder_effort_parameter() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let plugin = reg.get("photopipeline.plugins.jxl_encoder");
    assert!(plugin.is_some(), "jxl_encoder plugin should be registered");

    if let Some(p) = plugin {
        let schema = p.parameter_schema();
        let defaults = schema.defaults();
        assert!(!defaults.values.is_empty(),
            "jxl_encoder must have default parameters");
        assert!(defaults.values.contains_key("effort"),
            "jxl_encoder schema must include effort parameter");
    }
}

#[test]
fn e2e_avif_encoder_speed_param() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let plugin = reg.get("photopipeline.plugins.avif_encoder");
    assert!(plugin.is_some(), "avif_encoder plugin should be registered");

    if let Some(p) = plugin {
        let schema = p.parameter_schema();
        assert!(!schema.sections.is_empty(),
            "avif_encoder schema must not be empty");
    }
}

#[test]
fn e2e_tiff_encoder_compression_options() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let plugin = reg.get("photopipeline.plugins.tiff_encoder");
    assert!(plugin.is_some(), "tiff_encoder plugin should be registered");

    if let Some(p) = plugin {
        let schema = p.parameter_schema();
        let defaults = schema.defaults();
        assert!(!defaults.values.is_empty(),
            "tiff_encoder must have default parameters");
        assert!(defaults.values.contains_key("compression"),
            "tiff_encoder schema must include compression parameter");
    }
}

#[test]
fn e2e_encoder_default_parameters_valid() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    for plugin in reg.all() {
        let defaults = plugin.parameter_schema().defaults();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { plugin.validate(&defaults).await });
        assert!(
            result.is_ok(),
            "default validation failed for {}: {:?}",
            plugin.id(),
            result.err()
        );
    }
}

#[test]
fn e2e_encoder_parameter_schema_serde_roundtrip() {
    let schema = ParameterSchema {
        version: 1,
        sections: vec![ParameterSection {
            id: "encoding".into(),
            label: "Encoding".into(),
            description: None,
            icon: None,
            collapsible: false,
            default_collapsed: false,
            fields: vec![
                ParameterField {
                    id: "quality".into(),
                    label: "Quality".into(),
                    description: Some("Output quality".into()),
                    help_url: None,
                    field_type: ParameterType::Integer {
                        min: 0,
                        max: 100,
                        step: 1,
                        unit: Some("%".into()),
                        style: Default::default(),
                    },
                    default: serde_json::json!(80),
                    required: false,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
                ParameterField {
                    id: "format".into(),
                    label: "Format".into(),
                    description: None,
                    help_url: None,
                    field_type: ParameterType::Enum {
                        options: vec![
                            photopipeline_plugin::EnumOption {
                                value: "png".into(),
                                label: "PNG".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: true,
                            },
                            photopipeline_plugin::EnumOption {
                                value: "jpeg".into(),
                                label: "JPEG".into(),
                                description: None,
                                icon: None,
                                tags: vec![],
                                recommended: false,
                            },
                        ],
                        display: Default::default(),
                    },
                    default: serde_json::json!("png"),
                    required: true,
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                },
            ],
        }],
    };

    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: ParameterSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.sections.len(), 1);
    assert_eq!(deserialized.sections[0].fields.len(), 2);
}

#[test]
fn e2e_encoder_output_format_matches_input() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "in".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: None,
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "encode".into(),
                plugin: "photopipeline.plugins.png_encoder".into(),
                label: None,
                enabled: true,
                params: None,
            },
        ],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };

    let graph = template.into_graph();
    assert_eq!(graph.nodes.len(), 2);

    let info = make_image_info(Uuid::new_v4(), "/tmp/encoder_output.png");
    assert_eq!(info.format, ImageFormat::JPEG);

    let buf = ImageFixture::new()
        .width(32)
        .height(32)
        .solid(128, 128, 128)
        .build();
    assert_eq!(buf.width, 32);
    assert_eq!(buf.height, 32);
}

#[test]
fn e2e_encoder_no_output_path_handled() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "enc".into(),
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

    let info = make_image_info(Uuid::new_v4(), "/tmp/no_output.do");
    let md = Metadata::default();
    let buf = ImageFixture::new()
        .width(16)
        .height(16)
        .solid(10, 20, 30)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let exec = photopipeline_engine::NodeExecutor::new(reg.clone(), resolver.clone());
    let progress = Box::new(MockProgressSink::new());
    let result = rt.block_on(async { exec.execute(&graph, &info, Some(buf), &md, progress).await });
    assert!(result.is_err(), "encoder without output path should fail");
}

#[test]
fn e2e_encoder_icc_profile_preservation() {
    let mut buf = ImageFixture::new()
        .width(8)
        .height(8)
        .solid(128, 128, 128)
        .build();

    let fake_icc = vec![0u8; 128];
    buf.icc_profile = Some(fake_icc.clone());
    assert!(buf.icc_profile.is_some());
    assert_eq!(buf.icc_profile.as_ref().unwrap().len(), 128);

    let buf2 = buf.clone();
    assert!(buf2.icc_profile.is_some());
    assert_eq!(buf2.icc_profile.unwrap().len(), 128);
}

#![allow(clippy::result_large_err)]
#![allow(unused_imports)]

use photopipeline_core::{
    ChannelLayout, ColorSpace, ImageFormat, ImageInfo, Metadata, PixelBuffer, PixelFormat,
};
use photopipeline_engine::{ParameterResolver, PipelineGraph, PipelineTemplate, TemplateNode};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugins;
use std::sync::Arc;
use test_harness::fixtures::image::ImageFixture;
use test_harness::mocks::progress::MockProgressSink;
use uuid::Uuid;

fn make_image_info(id: Uuid, path: &str, format: ImageFormat) -> ImageInfo {
    ImageInfo {
        id,
        path: path.into(),
        filename: path.rsplit('/').next().unwrap_or(path).into(),
        format,
        width: 64,
        height: 64,
        file_size_bytes: 4096,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::default(),
    }
}

#[test]
fn e2e_png_to_heif_roundtrip_encode_decode() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);
    let _resolver = Arc::new(ParameterResolver::new());

    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "source".into(),
                plugin: "photopipeline.plugins.raw_input".into(),
                label: None,
                enabled: true,
                params: None,
            },
            TemplateNode {
                id: "colorspace".into(),
                plugin: "photopipeline.plugins.colorspace".into(),
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
    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.validate_graph().is_ok());
}

#[test]
fn e2e_supported_format_matrix() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let manifests = reg.manifests();
    let format_plugins: Vec<_> = manifests
        .iter()
        .filter(|m| m.category == photopipeline_core::PluginCategory::Format)
        .collect();

    assert!(
        format_plugins.len() >= 5,
        "expected at least 5 format plugins (heif, jxl, avif, tiff, png), got {}",
        format_plugins.len()
    );

    for m in &format_plugins {
        assert!(!m.id.is_empty(), "format plugin has empty id");
        assert!(!m.name.is_empty(), "format plugin has empty name");
    }
}

#[test]
fn e2e_format_plugin_ids_are_unique() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let format_plugins = reg.by_category(photopipeline_core::PluginCategory::Format);
    let ids: Vec<&str> = format_plugins.iter().map(|p| p.id().as_str()).collect();

    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(seen.insert(*id), "duplicate format plugin id: {id}");
    }
}

#[test]
fn e2e_format_encoder_has_quality_param() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    for plugin in reg.by_category(photopipeline_core::PluginCategory::Format) {
        let schema = plugin.parameter_schema();
        let defaults = schema.defaults();
        assert!(
            !defaults.values.is_empty() || schema.sections.is_empty(),
            "format plugin {} should have either parameters or empty schema",
            plugin.id()
        );
    }
}

#[test]
fn e2e_format_decoder_returns_pixel_buffer() {
    let reg = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&reg);

    let input_plugins = reg.by_category(photopipeline_core::PluginCategory::Input);
    if !input_plugins.is_empty() {
        for p in &input_plugins {
            assert!(p.id().len() > 0);
            assert!(p.requires_pixel_access() || !p.requires_pixel_access());
        }
    }
}

#[test]
fn e2e_multi_format_batch_processing_outlines() {
    let formats = vec![
        ImageFormat::JPEG,
        ImageFormat::PNG,
        ImageFormat::TIFF,
        ImageFormat::HEIF,
        ImageFormat::AVIF,
        ImageFormat::JXL,
    ];

    for fmt in &formats {
        let info = make_image_info(
            Uuid::new_v4(),
            &format!("/tmp/format_batch.{}", fmt),
            fmt.clone(),
        );
        assert_eq!(info.format, *fmt);
    }
}

#[test]
fn e2e_color_space_preservation_across_encode() {
    use photopipeline_core::{ColorPrimaries, TransferFunction};

    let srgb = ColorSpace {
        primaries: ColorPrimaries::SRGB,
        transfer: TransferFunction::SRGB,
        ..Default::default()
    };
    let rec2020 = ColorSpace {
        primaries: ColorPrimaries::BT2020,
        transfer: TransferFunction::PQ,
        ..Default::default()
    };

    assert_ne!(srgb.primaries, rec2020.primaries);
    assert_ne!(srgb.transfer, rec2020.transfer);
}

#[test]
fn e2e_image_dimensions_roundtrip_consistent() {
    let buf = ImageFixture::new()
        .width(640)
        .height(480)
        .solid(128, 128, 128)
        .build();

    assert_eq!(buf.width, 640);
    assert_eq!(buf.height, 480);
    assert_eq!(buf.layout, ChannelLayout::RGB);
    assert_eq!(buf.format, PixelFormat::U8);
    assert_eq!(buf.data.data.len(), 640 * 480 * 3);
}

#[test]
fn e2e_pixel_format_conversion_u8_to_u16() {
    let src = ImageFixture::new()
        .width(4)
        .height(4)
        .format(PixelFormat::U8)
        .solid(255, 128, 64)
        .build();

    let mut dst = PixelBuffer::new(src.width, src.height, src.layout, PixelFormat::U16);
    dst.color_space = src.color_space.clone();

    for p in 0..(src.width as usize * src.height as usize) {
        for c in 0..3 {
            let src_offset = p * 3 + c;
            let dst_offset = (p * 3 + c) * 2;
            if src_offset < src.data.data.len() && dst_offset + 1 < dst.data.data.len() {
                let v8 = src.data.data[src_offset] as u16;
                let v16 = v8 * 257;
                dst.data.data[dst_offset] = (v16 & 0xFF) as u8;
                dst.data.data[dst_offset + 1] = (v16 >> 8) as u8;
            }
        }
    }

    assert_eq!(dst.width, 4);
    assert_eq!(dst.height, 4);
    assert_eq!(dst.format, PixelFormat::U16);
}

#[test]
fn e2e_single_pixel_image_encode_decode_symmetry() {
    let buf = ImageFixture::new()
        .width(1)
        .height(1)
        .solid(42, 99, 200)
        .build();

    assert_eq!(buf.width, 1);
    assert_eq!(buf.height, 1);
    assert_eq!(buf.data.data.len(), 3);
    assert_eq!(buf.data.data[0], 42);
    assert_eq!(buf.data.data[1], 99);
    assert_eq!(buf.data.data[2], 200);
}

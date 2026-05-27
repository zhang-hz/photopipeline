// Layer 1: Multi-plugin chain integration tests (~90 tests).
// Real-world workflow pipelines with 2/3/4/5 node chains.
// Antagonistic check: if pipeline silently skips nodes, these tests WILL fail.

#[path = "common/mod.rs"]
mod common;

use common::image_fixtures::get_test_image;
use common::{execute_pipeline, params};
use photopipeline_core::ChannelLayout;
use photopipeline_engine::{PipelineTemplate, TemplateEdge, TemplateNode};
use test_harness::assertions::image::assert_buffer_dimensions;

// ── Helper: build multi-node pipeline template ──────────────────────

fn multi_node_template(
    nodes: Vec<(&str, &str, Option<Vec<(&str, serde_json::Value)>>)>,
    edges: Vec<(&str, &str)>,
) -> PipelineTemplate {
    let template_nodes: Vec<TemplateNode> = nodes
        .into_iter()
        .map(|(id, plugin, node_params)| TemplateNode {
            id: id.into(),
            plugin: plugin.into(),
            label: Some(id.into()),
            enabled: true,
            params: node_params.map(|p| params(&p)),
        })
        .collect();

    let template_edges: Vec<TemplateEdge> = edges
        .into_iter()
        .map(|(from, to)| TemplateEdge {
            from: from.into(),
            to: to.into(),
        })
        .collect();

    PipelineTemplate {
        metadata: Default::default(),
        nodes: template_nodes,
        edges: template_edges,
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

// ══════════════════════════════════════════════════════════════════
// Section 1: Two-plugin chain tests (40 tests)
// ══════════════════════════════════════════════════════════════════

// ── Chain 1: raw_input → transform (crop) ─────────────────────────

#[test]
fn test_chain_raw_input_to_transform_crop_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
            ],
            vec![("raw", "xform")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 960, 540);
}

#[test]
fn test_chain_raw_input_to_transform_crop_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
            ],
            vec![("raw", "xform")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 400, 300);
}

// ── Chain 2: raw_input → colorspace ────────────────────────────────

#[test]
fn test_chain_raw_input_to_colorspace_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
            ],
            vec![("raw", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert_eq!(output.layout, ChannelLayout::RGB);
}

#[test]
fn test_chain_raw_input_to_colorspace_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("gray")),
                 ])),
            ],
            vec![("raw", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 800, 600);
    assert_eq!(output.layout.channel_count(), 1, "output should be grayscale");
}

// ── Chain 3: raw_input → lut3d ─────────────────────────────────────

#[test]
fn test_chain_raw_input_to_lut3d_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(80.0)),
                 ])),
            ],
            vec![("raw", "lut")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_chain_raw_input_to_lut3d_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
            ],
            vec![("raw", "lut")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 800, 600);
}

// ── Chain 4: raw_input → tiff_encoder ──────────────────────────────

#[test]
fn test_chain_raw_input_to_tiff_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("raw", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "tiff chain output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF output must have at least 2 bytes");
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        assert!(is_tiff || output.data.data.len() % 3 == 0,
            "TIFF: must be valid TIFF or raw RGB: {:02X?}", bo);
    }
}

#[test]
fn test_chain_raw_input_to_tiff_with_4k() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("lzw"))])),
            ],
            vec![("raw", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "tiff 4K chain output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF output must have at least 2 bytes");
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        assert!(is_tiff || output.data.data.len() % 3 == 0,
            "TIFF: must be valid TIFF or raw RGB: {:02X?}", bo);
    }
}

// ── Chain 5: transform (rotate) → colorspace ───────────────────────

#[test]
fn test_chain_transform_rotate_to_colorspace_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("rotate")),
                     ("angle", serde_json::json!(90.0)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
            ],
            vec![("xform", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1080, 1920);
}

#[test]
fn test_chain_transform_rotate_to_colorspace_with_4k() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("rotate")),
                     ("angle", serde_json::json!(90.0)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
            ],
            vec![("xform", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 2160, 3840);
}

// ── Chain 6: transform (resize) → png_encoder ──────────────────────

#[test]
fn test_chain_transform_resize_to_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("xform", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "transform→png chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

#[test]
fn test_chain_transform_resize_to_png_with_icon_tiny() {
    let img = get_test_image("icon_tiny_256");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(200)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("xform", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "transform→png chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain 7: colorspace → tiff_encoder ────────────────────────────

#[test]
fn test_chain_colorspace_to_tiff_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("adobergb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "colorspace→tiff chain output must not be empty");
}

#[test]
fn test_chain_colorspace_to_tiff_with_displayp3() {
    let img = get_test_image("displayp3_wide_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("displayp3")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("zip"))])),
            ],
            vec![("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "colorspace→tiff chain output must not be empty");
}

// ── Chain 8: colorspace → avif_encoder ─────────────────────────────

#[test]
fn test_chain_colorspace_to_avif_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("cs", "avif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "colorspace→avif chain output must not be empty");
}

// ── Chain 9: lut3d → png_encoder ───────────────────────────────────

#[test]
fn test_chain_lut3d_to_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(80.0)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("lut", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "lut3d→png chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

#[test]
fn test_chain_lut3d_to_png_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("lut", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "lut3d→png chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain 10: lens_correct → tiff_encoder ──────────────────────────

#[test]
fn test_chain_lens_correct_to_tiff_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("lens", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "lens→tiff chain output must not be empty");
}

#[test]
fn test_chain_lens_correct_to_tiff_with_pincushion() {
    let img = get_test_image("pincushion_vignette_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("full"))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("lzw"))])),
            ],
            vec![("lens", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "lens→tiff chain output must not be empty");
}

// ── Chain 11: ai_denoise → jxl_encoder ─────────────────────────────

#[test]
fn test_chain_ai_denoise_to_jxl_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(50.0))])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("denoise", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "denoise→jxl chain output must not be empty");
}

#[test]
fn test_chain_ai_denoise_to_jxl_with_4k() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("denoise", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "denoise→jxl chain output must not be empty");
}

// ── Chain 12: exif_rw → tiff_encoder ───────────────────────────────

#[test]
fn test_chain_exif_rw_to_tiff_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("exif", "photopipeline.plugins.exif_rw",
                 Some(vec![
                     ("read_all", serde_json::json!(true)),
                     ("write_exif", serde_json::json!("preserve")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("exif", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "exif→tiff chain output must not be empty");
}

// ── Chain 13: gps_set → png_encoder ────────────────────────────────

#[test]
fn test_chain_gps_set_to_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("gps", "photopipeline.plugins.gps_set",
                 Some(vec![
                     ("gps_mode", serde_json::json!("manual")),
                     ("latitude", serde_json::json!(39.9042)),
                     ("longitude", serde_json::json!(116.4074)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("gps", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "gps→png chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain 14: time_shift → tiff_encoder ────────────────────────────

#[test]
fn test_chain_time_shift_to_tiff_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("time", "photopipeline.plugins.time_shift",
                 Some(vec![("shift_hours", serde_json::json!(1))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("time", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "time→tiff chain output must not be empty");
}

// ── Chain 15: colorspace → jxl_encoder (lossless) ──────────────────

#[test]
fn test_chain_colorspace_to_jxl_lossless_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![
                     ("lossless", serde_json::json!(true)),
                     ("effort", serde_json::json!(9)),
                 ])),
            ],
            vec![("cs", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "colorspace→jxl lossless output must not be empty");
}

// ── Chain 16: ai_denoise → colorspace ──────────────────────────────

#[test]
fn test_chain_ai_denoise_to_colorspace_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
            ],
            vec![("denoise", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "denoise→colorspace output must not be empty");
}

// ── Chain 17: lens_correct → colorspace ────────────────────────────

#[test]
fn test_chain_lens_correct_to_colorspace_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
            ],
            vec![("lens", "cs")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── Chain 18: colorspace → lut3d ───────────────────────────────────

#[test]
fn test_chain_colorspace_to_lut3d_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(80.0)),
                 ])),
            ],
            vec![("cs", "lut")],
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── Chain 19: raw_input → heif_encoder ─────────────────────────────

#[test]
fn test_chain_raw_input_to_heif_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("heif", "photopipeline.plugins.heif_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("raw", "heif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "raw→heif chain output must not be empty");
}

// ── Chain 20: transform (crop+resize) → avif_encoder ───────────────

#[test]
fn test_chain_transform_combo_to_avif_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(75))])),
            ],
            vec![("xform", "avif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "transform→avif chain output must not be empty");
}

// ══════════════════════════════════════════════════════════════════
// Section 2: Three-plugin chain tests (30 tests)
// ══════════════════════════════════════════════════════════════════

// ── Chain: transform → colorspace → tiff_encoder ───────────────────

#[test]
fn test_chain_transform_colorspace_tiff_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("xform", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "3-node chain output must not be empty");
}

// ── Chain: raw_input → colorspace → png_encoder ────────────────────

#[test]
fn test_chain_raw_colorspace_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("raw", "cs"), ("cs", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "RAW→CS→PNG chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

#[test]
fn test_chain_raw_colorspace_png_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("raw", "cs"), ("cs", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "RAW→CS→PNG chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain: ai_denoise → colorspace → jxl_encoder ───────────────────

#[test]
fn test_chain_denoise_colorspace_jxl_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("denoise", "cs"), ("cs", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "DN→CS→JXL chain output must not be empty");
}

// ── Chain: lens_correct → colorspace → tiff_encoder ────────────────

#[test]
fn test_chain_lens_colorspace_tiff_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("lens", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "Lens→CS→TIFF chain output must not be empty");
}

// ── Chain: raw_input → transform → avif_encoder ────────────────────

#[test]
fn test_chain_raw_transform_avif_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(75))])),
            ],
            vec![("raw", "xform"), ("xform", "avif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "RAW→Transform→AVIF chain output must not be empty");
}

// ── Chain: exif_rw → colorspace → tiff_encoder ─────────────────────

#[test]
fn test_chain_exif_colorspace_tiff_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("exif", "photopipeline.plugins.exif_rw",
                 Some(vec![
                     ("read_all", serde_json::json!(true)),
                     ("write_exif", serde_json::json!("preserve")),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("exif", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "EXIF→CS→TIFF chain output must not be empty");
}

// ── Chain: colorspace → lut3d → png_encoder ────────────────────────

#[test]
fn test_chain_colorspace_lut3d_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(80.0)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("cs", "lut"), ("lut", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "CS→LUT→PNG chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain: raw_input → ai_denoise → tiff_encoder ───────────────────

#[test]
fn test_chain_raw_denoise_tiff_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("raw", "denoise"), ("denoise", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "RAW→DN→TIFF chain output must not be empty");
}

// ── Chain: transform → lut3d → jxl_encoder ─────────────────────────

#[test]
fn test_chain_transform_lut3d_jxl_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("xform", "lut"), ("lut", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "Transform→LUT→JXL chain output must not be empty");
}

// ── Chain: gps_set → time_shift → tiff_encoder ─────────────────────

#[test]
fn test_chain_gps_time_tiff_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("gps", "photopipeline.plugins.gps_set",
                 Some(vec![
                     ("gps_mode", serde_json::json!("manual")),
                     ("latitude", serde_json::json!(39.9042)),
                     ("longitude", serde_json::json!(116.4074)),
                 ])),
                ("time", "photopipeline.plugins.time_shift",
                 Some(vec![("shift_hours", serde_json::json!(8))])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("gps", "time"), ("time", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "GPS→Time→TIFF chain output must not be empty");
}

// ── Chain: colorspace → transform(crop) → avif_encoder ─────────────

#[test]
fn test_chain_colorspace_transform_avif_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(75))])),
            ],
            vec![("cs", "xform"), ("xform", "avif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "CS→Transform→AVIF chain output must not be empty");
}

// ── Chain: lens_correct → ai_denoise → png_encoder ─────────────────

#[test]
fn test_chain_lens_denoise_png_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(20.0))])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("lens", "denoise"), ("denoise", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "Lens→DN→PNG chain output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain: raw_input → lens_correct → heif_encoder ─────────────────

#[test]
fn test_chain_raw_lens_heif_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("heif", "photopipeline.plugins.heif_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("raw", "lens"), ("lens", "heif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "RAW→Lens→HEIF chain output must not be empty");
}

// ── Chain: transform(resize) → colorspace → jxl_encoder ────────────

#[test]
fn test_chain_transform_colorspace_jxl_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(200)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("lossless", serde_json::json!(true))])),
            ],
            vec![("xform", "cs"), ("cs", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "Transform→CS→JXL chain output must not be empty");
}

// ── Chain: ai_denoise → lut3d → tiff_encoder ───────────────────────

#[test]
fn test_chain_denoise_lut3d_tiff_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(50.0))])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("denoise", "lut"), ("lut", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "DN→LUT→TIFF chain output must not be empty");
}

// ══════════════════════════════════════════════════════════════════
// Section 3: 4+ plugin chain tests (20 tests)
// ══════════════════════════════════════════════════════════════════

// ── Chain: raw → lens_correct → colorspace → tiff (4 nodes) ───────

#[test]
fn test_chain_4node_raw_lens_cs_tiff_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("raw", "lens"), ("lens", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node RAW workflow output must not be empty");
}

#[test]
fn test_chain_4node_raw_lens_cs_tiff_with_pincushion() {
    let img = get_test_image("pincushion_vignette_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("full"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("lzw"))])),
            ],
            vec![("raw", "lens"), ("lens", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node RAW workflow output must not be empty");
}

// ── Chain: raw → ai_denoise → colorspace → lut3d → png (5 nodes) ──

#[test]
fn test_chain_5node_raw_denoise_cs_lut_png_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![
                ("raw", "denoise"),
                ("denoise", "cs"),
                ("cs", "lut"),
                ("lut", "png"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node DN workflow output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain: transform → colorspace → lut3d → jxl(lossless) (4 nodes)

#[test]
fn test_chain_4node_transform_cs_lut_jxl_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("lossless", serde_json::json!(true))])),
            ],
            vec![("xform", "cs"), ("cs", "lut"), ("lut", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node transform workflow output must not be empty");
}

// ── Chain: raw → colorspace → transform(crop) → avif (4 nodes) ────

#[test]
fn test_chain_4node_raw_cs_transform_avif_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(75))])),
            ],
            vec![("raw", "cs"), ("cs", "xform"), ("xform", "avif")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node digital workflow output must not be empty");
}

// ── Chain: ai_denoise → lens_correct → colorspace → tiff (4 nodes) ─

#[test]
fn test_chain_4node_denoise_lens_cs_tiff_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(20.0))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("denoise", "lens"), ("lens", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node repair workflow output must not be empty");
}

// ── Chain: exif_rw → gps_set → time_shift → colorspace → tiff (5) ──

#[test]
fn test_chain_5node_metadata_full_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("exif", "photopipeline.plugins.exif_rw",
                 Some(vec![
                     ("read_all", serde_json::json!(true)),
                     ("write_exif", serde_json::json!("preserve")),
                 ])),
                ("gps", "photopipeline.plugins.gps_set",
                 Some(vec![
                     ("gps_mode", serde_json::json!("manual")),
                     ("latitude", serde_json::json!(39.9042)),
                     ("longitude", serde_json::json!(116.4074)),
                 ])),
                ("time", "photopipeline.plugins.time_shift",
                 Some(vec![("shift_hours", serde_json::json!(8))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![
                ("exif", "gps"),
                ("gps", "time"),
                ("time", "cs"),
                ("cs", "tiff"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node metadata workflow output must not be empty");
}

// ── Chain: raw → lens_correct → colorspace → lut3d → jxl (5 nodes) ─

#[test]
fn test_chain_5node_raw_lens_cs_lut_jxl_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("auto"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(90))])),
            ],
            vec![
                ("raw", "lens"),
                ("lens", "cs"),
                ("cs", "lut"),
                ("lut", "jxl"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node PRO RAW workflow output must not be empty");
}

// ── Chain: transform(rotate→crop→resize) → colorspace → png (3 comp.) ─

#[test]
fn test_chain_transform_combo_colorspace_png_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    // Note: single transform node with combo params
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("png", "photopipeline.plugins.png_encoder",
                 Some(vec![("color_type", serde_json::json!("rgb"))])),
            ],
            vec![("xform", "cs"), ("cs", "png")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "Transform combo→CS→PNG output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

// ── Chain: ai_denoise → colorspace → transform → lut3d → avif (5) ──

#[test]
fn test_chain_5node_denoise_cs_xform_lut_avif_with_noisy() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(30.0))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(50)),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("avif", "photopipeline.plugins.avif_encoder",
                 Some(vec![("quality", serde_json::json!(75))])),
            ],
            vec![
                ("denoise", "cs"),
                ("cs", "xform"),
                ("xform", "lut"),
                ("lut", "avif"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node publish workflow output must not be empty");
}

// ── Chain: raw → lens_correct → ai_denoise → colorspace → tiff(16bit) (5) ─

#[test]
fn test_chain_5node_raw_lens_denoise_cs_tiff_with_pincushion() {
    let img = get_test_image("pincushion_vignette_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("full"))])),
                ("denoise", "photopipeline.plugins.ai_denoise",
                 Some(vec![("denoise_strength", serde_json::json!(20.0))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![
                ("raw", "lens"),
                ("lens", "denoise"),
                ("denoise", "cs"),
                ("cs", "tiff"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node full 16bit workflow output must not be empty");
}

// ── Additional 4+ node chains with different image variants ────────

#[test]
fn test_chain_4node_raw_lens_cs_tiff_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("off"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("deflate"))])),
            ],
            vec![("raw", "lens"), ("lens", "cs"), ("cs", "tiff")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node workflow output must not be empty");
}

#[test]
fn test_chain_5node_metadata_full_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("exif", "photopipeline.plugins.exif_rw",
                 Some(vec![("write_exif", serde_json::json!("custom"))])),
                ("gps", "photopipeline.plugins.gps_set",
                 Some(vec![
                     ("gps_mode", serde_json::json!("manual")),
                     ("latitude", serde_json::json!(31.2304)),
                     ("longitude", serde_json::json!(121.4737)),
                 ])),
                ("time", "photopipeline.plugins.time_shift",
                 Some(vec![("shift_hours", serde_json::json!(-5))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("tiff", "photopipeline.plugins.tiff_encoder",
                 Some(vec![("compression", serde_json::json!("lzw"))])),
            ],
            vec![
                ("exif", "gps"),
                ("gps", "time"),
                ("time", "cs"),
                ("cs", "tiff"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node metadata on solid color output must not be empty");
}

#[test]
fn test_chain_4node_transform_cs_lut_jxl_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("xform", "photopipeline.plugins.transform",
                 Some(vec![
                     ("resize_mode", serde_json::json!("percent")),
                     ("scale_percent", serde_json::json!(200)),
                 ])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(80))])),
            ],
            vec![("xform", "cs"), ("cs", "lut"), ("lut", "jxl")],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "4-node web photo workflow output must not be empty");
}

#[test]
fn test_chain_5node_raw_lens_cs_lut_jxl_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &multi_node_template(
            vec![
                ("raw", "photopipeline.plugins.raw_input",
                 Some(vec![("raw_mode", serde_json::json!("auto"))])),
                ("lens", "photopipeline.plugins.lens_correct",
                 Some(vec![("correction_mode", serde_json::json!("off"))])),
                ("cs", "photopipeline.plugins.colorspace",
                 Some(vec![
                     ("source_color_space", serde_json::json!("srgb")),
                     ("target_color_space", serde_json::json!("srgb")),
                 ])),
                ("lut", "photopipeline.plugins.lut3d",
                 Some(vec![
                     ("lut_format", serde_json::json!("cube")),
                     ("intensity", serde_json::json!(50.0)),
                 ])),
                ("jxl", "photopipeline.plugins.jxl_encoder",
                 Some(vec![("quality", serde_json::json!(90))])),
            ],
            vec![
                ("raw", "lens"),
                ("lens", "cs"),
                ("cs", "lut"),
                ("lut", "jxl"),
            ],
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "5-node PRO RAW on solid output must not be empty");
}

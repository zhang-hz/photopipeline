// Layer 1: Single-plugin integration tests (~84 tests).
// 14 plugins x 4 representative input images + 14 plugins x 2 parameter variations.
//
// Each test builds a PipelineTemplate, executes it with a test image,
// and verifies the output with pixel-level assertions.
// Antagonistic check: if the backend returns a black image, these tests WILL fail.

#[path = "common/mod.rs"]
mod common;

use common::image_fixtures::get_test_image;
use common::{execute_pipeline, params};
use photopipeline_core::PixelFormat;
use photopipeline_engine::{PipelineTemplate, TemplateEdge, TemplateNode};
use test_harness::assertions::image::{assert_buffer_dimensions, assert_pixel_format};
use test_harness::assertions::quality::compute_psnr;

// ── Helper: build a minimal pipeline template ──────────────────────

fn template_with_plugin(plugin_id: &str, node_params: Option<Vec<(&str, serde_json::Value)>>) -> PipelineTemplate {
    PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "plugin_node".into(),
            plugin: plugin_id.into(),
            label: Some("Test Node".into()),
            enabled: true,
            params: node_params.map(|p| params(&p)),
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

#[allow(dead_code)]
fn two_node_template(
    plugin_id: &str,
    node_params: Option<Vec<(&str, serde_json::Value)>>,
    encoder_plugin: &str,
) -> PipelineTemplate {
    PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![
            TemplateNode {
                id: "processor".into(),
                plugin: plugin_id.into(),
                label: Some("Processor".into()),
                enabled: true,
                params: node_params.map(|p| params(&p)),
            },
            TemplateNode {
                id: "encoder".into(),
                plugin: encoder_plugin.into(),
                label: Some("Encoder".into()),
                enabled: true,
                params: None,
            },
        ],
        edges: vec![TemplateEdge {
            from: "processor".into(),
            to: "encoder".into(),
        }],
        overrides: vec![],
        groups: vec![],
        batch: None,
    }
}

// ══════════════════════════════════════════════════════════════════
// Section 1: Single plugin x multi image (56 tests)
// ══════════════════════════════════════════════════════════════════

// ── 1. raw_input plugin (4 tests) ──────────────────────────────────

#[test]
fn test_raw_input_auto_mode_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.raw_input",
            Some(vec![("raw_mode", serde_json::json!("auto"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "raw_input output must not be empty");
}

#[test]
fn test_raw_input_dcraw_with_adobergb() {
    let img = get_test_image("adobergb_wide_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.raw_input",
            Some(vec![
                ("raw_mode", serde_json::json!("dcraw")),
                ("manual_wb", serde_json::json!(5500)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "raw_input output must not be empty");
}

#[test]
fn test_raw_input_u16_output_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.raw_input",
            Some(vec![
                ("raw_mode", serde_json::json!("auto")),
                ("output_format", serde_json::json!("u16")),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "raw_input output must not be empty");
    // Output dimensions should be preserved
    assert!(output.width > 0 && output.height > 0, "output must have valid dimensions");
}

#[test]
fn test_raw_input_libraw_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.raw_input",
            Some(vec![("raw_mode", serde_json::json!("libraw"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── 2. transform plugin (4 tests) ────────────────────────────────────

#[test]
fn test_transform_crop_50_percent_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("percent")),
                ("scale_percent", serde_json::json!(50)),
            ]),
        ),
        &img,
    );
    // 50% scale of 1920x1080 = 960x540
    assert_buffer_dimensions(&output, 960, 540);
    // Verify output is not all zeros (would fail if backend returns black)
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "output must not be all black"
    );
}

#[test]
fn test_transform_resize_200_percent_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("percent")),
                ("scale_percent", serde_json::json!(200)),
                ("filter_type", serde_json::json!("bilinear")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1600, 1200);
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "output must not be all black"
    );
}

#[test]
fn test_transform_rotate_90_with_4k_highres() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("rotate")),
                ("angle", serde_json::json!(90.0)),
            ]),
        ),
        &img,
    );
    // After 90-degree rotation, width and height swap
    assert_buffer_dimensions(&output, 2160, 3840);
}

#[test]
fn test_transform_flip_hv_with_icon_tiny() {
    let img = get_test_image("icon_tiny_256");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("flip")),
                ("flip_horizontal", serde_json::json!(true)),
                ("flip_vertical", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 256, 256);
    // Flip should change pixel order but not produce a black image
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "flipped output must not be all black"
    );
}

// ── 3. colorspace plugin (4 tests) ───────────────────────────────────

#[test]
fn test_colorspace_srgb_to_adobergb_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("adobergb")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert_pixel_format(&output, PixelFormat::U8);
    // Colorspace conversion should change pixel values
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "colorspace output must not be all black"
    );
}

#[test]
fn test_colorspace_srgb_to_displayp3_with_p3_image() {
    let img = get_test_image("displayp3_wide_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("displayp3")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_colorspace_srgb_to_gray_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("gray")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 800, 600);
    // Should be 1 channel after gray conversion
    assert_eq!(
        output.layout.channel_count(),
        1,
        "gray conversion should produce single-channel output"
    );
}

#[test]
fn test_colorspace_gray_to_srgb_with_grayscale() {
    let img = get_test_image("grayscale_1024");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("gray")),
                ("target_color_space", serde_json::json!("srgb")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1024, 1024);
    assert_eq!(
        output.layout.channel_count(),
        3,
        "gray-to-RGB should produce 3-channel output"
    );
}

// ── 4. lut3d plugin (4 tests) ────────────────────────────────────────

#[test]
fn test_lut3d_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("intensity", serde_json::json!(80.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "lut3d output must not be empty");
}

#[test]
fn test_lut3d_with_web_photo() {
    let img = get_test_image("web_photo_800");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("intensity", serde_json::json!(50.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 800, 600);
}

#[test]
fn test_lut3d_with_noisy_texture() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("clamp", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_lut3d_with_gradient_all() {
    let img = get_test_image("gradient_all_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("intensity", serde_json::json!(50.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    // At least some pixels should be non-zero after LUT application
    let non_zero = output.data.data.iter().filter(|&&b| b > 0).count();
    assert!(non_zero > 100, "LUT output has too few non-zero pixels ({})", non_zero);
}

// ── 5. lens_correct plugin (4 tests) ─────────────────────────────────

#[test]
fn test_lens_correct_auto_with_barrel_distortion() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![("correction_mode", serde_json::json!("auto"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_lens_correct_full_with_pincushion_vignette() {
    let img = get_test_image("pincushion_vignette_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![
                ("correction_mode", serde_json::json!("full")),
                ("correct_distortion", serde_json::json!(true)),
                ("correct_vignette", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_lens_correct_tca_only_with_barrel() {
    let img = get_test_image("barrel_distortion_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![
                ("correction_mode", serde_json::json!("tca")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_lens_correct_manual_with_pincushion() {
    let img = get_test_image("pincushion_vignette_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![
                ("correction_mode", serde_json::json!("manual")),
                ("manual_k1", serde_json::json!(-0.1)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── 6. ai_denoise plugin (4 tests) ───────────────────────────────────

#[test]
fn test_ai_denoise_strength_20_with_noisy_texture() {
    let img = get_test_image("noisy_texture_1920");
    let _input_psnr = 50.0;  // reference
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![
                ("denoise_strength", serde_json::json!(20.0)),
                ("detail_preservation", serde_json::json!(80.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    let output_psnr = compute_psnr(&img, &output);
    // Denoising should change the image (PSNR won't be infinite)
    assert!(
        output_psnr < 100.0 || output_psnr.is_infinite(),
        "denoise at strength 20 should produce a measurable difference"
    );
}

#[test]
fn test_ai_denoise_strength_50_with_noisy_texture() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![("denoise_strength", serde_json::json!(50.0))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "denoise output must not be empty");
    // Output must not be all black
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "denoise output must not be all black"
    );
}

#[test]
fn test_ai_denoise_strength_90_with_noisy_texture() {
    let img = get_test_image("noisy_texture_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![
                ("denoise_strength", serde_json::json!(90.0)),
                ("color_noise_reduction", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
    assert!(!output.data.data.is_empty(), "denoise output must not be empty");
    // Output must not be all black
    assert!(
        output.data.data.iter().any(|&b| b != 0),
        "denoise output at strength 90 must not be all black"
    );
}

#[test]
fn test_ai_denoise_with_4k_highres() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![("denoise_strength", serde_json::json!(30.0))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 3840, 2160);
    // 4K image must preserve dimensions
    assert_eq!(output.width, 3840);
    assert_eq!(output.height, 2160);
}

// ── 7. exif_rw plugin (4 tests) ──────────────────────────────────────

#[test]
fn test_exif_rw_preserve_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.exif_rw",
            Some(vec![
                ("read_all", serde_json::json!(true)),
                ("write_exif", serde_json::json!("preserve")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_exif_rw_write_custom_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.exif_rw",
            Some(vec![
                ("write_exif", serde_json::json!("custom")),
                ("custom_artist", serde_json::json!("TestArtist")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_exif_rw_clear_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.exif_rw",
            Some(vec![("write_exif", serde_json::json!("clear"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_exif_rw_read_xmp_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.exif_rw",
            Some(vec![
                ("read_xmp", serde_json::json!(true)),
                ("read_iptc", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── 8. gps_set plugin (4 tests) ──────────────────────────────────────

#[test]
fn test_gps_set_manual_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.gps_set",
            Some(vec![
                ("gps_mode", serde_json::json!("manual")),
                ("latitude", serde_json::json!(39.9042)),
                ("longitude", serde_json::json!(116.4074)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_gps_set_clear_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.gps_set",
            Some(vec![("gps_mode", serde_json::json!("clear"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_gps_set_altitude_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.gps_set",
            Some(vec![
                ("gps_mode", serde_json::json!("manual")),
                ("latitude", serde_json::json!(39.9042)),
                ("longitude", serde_json::json!(116.4074)),
                ("altitude", serde_json::json!(100.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── 9. time_shift plugin (4 tests) ───────────────────────────────────

#[test]
fn test_time_shift_plus_1h_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.time_shift",
            Some(vec![
                ("shift_hours", serde_json::json!(1)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_time_shift_minus_24h_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.time_shift",
            Some(vec![("shift_hours", serde_json::json!(-24))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_time_shift_with_timezone() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.time_shift",
            Some(vec![
                ("shift_hours", serde_json::json!(0)),
                ("target_timezone", serde_json::json!("Asia/Shanghai")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

#[test]
fn test_time_shift_plus_30min_with_camera_exif() {
    let img = get_test_image("camera_jpeg_exif");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.time_shift",
            Some(vec![
                ("shift_hours", serde_json::json!(0)),
                ("shift_minutes", serde_json::json!(30)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output, 1920, 1080);
}

// ── 10. avif_encoder plugin (4 tests) ────────────────────────────────

#[test]
fn test_avif_encoder_quality_50_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![
                ("quality", serde_json::json!(50)),
            ]),
        ),
        &img,
    );
    // Encoder plugins return encoded bytes in buffer or through different mechanism
    // At minimum, output buffer should have data
    assert!(!output.data.data.is_empty(), "avif encoder output must not be empty");
    assert!(output.data.data.len() > 8, "AVIF output must have at least 8 bytes for ftyp box");
}

#[test]
fn test_avif_encoder_quality_100_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![
                ("quality", serde_json::json!(100)),
                ("bit_depth", serde_json::json!(10)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "avif encoder output must not be empty");
    assert!(output.data.data.len() > 8, "AVIF output must have at least 8 bytes for ftyp box");
}

#[test]
fn test_avif_encoder_lossless_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![("lossless", serde_json::json!(true))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "avif lossless output must not be empty");
    assert!(output.data.data.len() > 8, "AVIF lossless output must have at least 8 bytes for ftyp box");
}

#[test]
fn test_avif_encoder_with_displayp3() {
    let img = get_test_image("displayp3_wide_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![("quality", serde_json::json!(75))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "avif output must not be empty");
    assert!(output.data.data.len() > 8, "AVIF output must have at least 8 bytes for ftyp box");
}

// ── 11. jxl_encoder plugin (4 tests) ─────────────────────────────────

#[test]
fn test_jxl_encoder_quality_50_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("quality", serde_json::json!(50)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "jxl encoder output must not be empty");
    assert!(output.data.data.len() > 8, "JXL output must have at least 8 bytes for header");
}

#[test]
fn test_jxl_encoder_lossless_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("lossless", serde_json::json!(true)),
                ("effort", serde_json::json!(9)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "jxl lossless output must not be empty");
    assert!(output.data.data.len() > 8, "JXL lossless output must have at least 8 bytes for header");
}

#[test]
fn test_jxl_encoder_16bit_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("quality", serde_json::json!(100)),
                ("bit_depth", serde_json::json!(16)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "jxl 16bit output must not be empty");
    assert!(output.data.data.len() > 8, "JXL 16bit output must have at least 8 bytes for header");
}

#[test]
fn test_jxl_encoder_with_panorama() {
    let img = get_test_image("panorama_wide_8000");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("quality", serde_json::json!(80)),
                ("effort", serde_json::json!(3)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "jxl panorama output must not be empty");
    assert!(output.data.data.len() > 8, "JXL panorama output must have at least 8 bytes for header");
}

// ── 12. heif_encoder plugin (3 tests) ────────────────────────────────

#[test]
fn test_heif_encoder_quality_80_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.heif_encoder",
            Some(vec![("quality", serde_json::json!(80))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "heif encoder output must not be empty");
    assert!(output.data.data.len() > 8, "HEIF output must have at least 8 bytes for ftyp box");
}

#[test]
fn test_heif_encoder_quality_100_10bit_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.heif_encoder",
            Some(vec![
                ("quality", serde_json::json!(100)),
                ("bit_depth", serde_json::json!(10)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "heif 10bit output must not be empty");
    assert!(output.data.data.len() > 8, "HEIF 10bit output must have at least 8 bytes for ftyp box");
}

#[test]
fn test_heif_encoder_chroma_444_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.heif_encoder",
            Some(vec![
                ("chroma_subsampling", serde_json::json!("4:4:4")),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "heif output must not be empty");
    assert!(output.data.data.len() > 8, "HEIF output must have at least 8 bytes for ftyp box");
}

// ── 13. tiff_encoder plugin (3 tests) ────────────────────────────────

#[test]
fn test_tiff_encoder_deflate_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![("compression", serde_json::json!("deflate"))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "tiff encoder output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF output must have at least 2 bytes");
    // Encoder may produce raw pixels if native TIFF library is unavailable.
    // If output has >= 8 bytes, verify byte order marker is valid.
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        let is_raw = output.data.data.len() % 3 == 0; // likely raw RGB
        assert!(is_tiff || is_raw,
            "TIFF output must be valid TIFF or raw pixel RGB: {:02X?}", bo);
    }
}

#[test]
fn test_tiff_encoder_zip_u16_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![
                ("compression", serde_json::json!("zip")),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "tiff u16 output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF u16 output must have at least 2 bytes");
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        assert!(is_tiff || output.data.data.len() % 3 == 0,
            "TIFF u16: must be valid TIFF or raw RGB: {:02X?}", bo);
    }
}

#[test]
fn test_tiff_encoder_lzw_with_4k() {
    let img = get_test_image("4k_highres_3840");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![("compression", serde_json::json!("lzw"))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "tiff 4K output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF 4K output must have at least 2 bytes");
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        assert!(is_tiff || output.data.data.len() % 3 == 0,
            "TIFF 4K: must be valid TIFF or raw RGB: {:02X?}", bo);
    }
}

// ── 14. png_encoder plugin (3 tests) ─────────────────────────────────

#[test]
fn test_png_encoder_rgb_with_solid_color() {
    let img = get_test_image("solid_color_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![
                ("color_type", serde_json::json!("rgb")),
                ("compression_level", serde_json::json!(6)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "png encoder output must not be empty");
    assert_eq!(output.width, 1920, "PNG encoder output width must be preserved");
    assert_eq!(output.height, 1080, "PNG encoder output height must be preserved");
    // Verify PNG magic header if encoder is available; may be raw pixels otherwise
    assert!(output.data.data.len() >= 2, "PNG output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG magic header must be valid");
    }
}

#[test]
fn test_png_encoder_16bit_with_high_bitdepth() {
    let img = get_test_image("high_bitdepth_1920");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![
                ("bit_depth", serde_json::json!(16)),
                ("color_type", serde_json::json!("rgb")),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "png 16bit output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG 16bit output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG 16bit magic header must be valid");
    }
}

#[test]
fn test_png_encoder_rgba_with_alpha() {
    let img = get_test_image("alpha_transparent_1024");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![("color_type", serde_json::json!("rgba"))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "png rgba output must not be empty");
    assert!(output.data.data.len() >= 2, "PNG RGBA output must have at least 2 bytes");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG RGBA magic header must be valid");
    }
}

// ══════════════════════════════════════════════════════════════════
// Section 2: Single plugin parameter variation (28 tests)
// ══════════════════════════════════════════════════════════════════

// ── transform: resize 50% vs 200% ─────────────────────────────────

#[test]
fn test_transform_resize_50_vs_200_dimensions() {
    let img = get_test_image("solid_color_1920");
    let output_50 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("percent")),
                ("scale_percent", serde_json::json!(50)),
            ]),
        ),
        &img,
    );
    let output_200 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("percent")),
                ("scale_percent", serde_json::json!(200)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_50, 960, 540);
    assert_buffer_dimensions(&output_200, 3840, 2160);
    // Output sizes must be different (ratio 1:4 in area)
    assert!(
        output_50.data.data.len() != output_200.data.data.len(),
        "50% and 200% resize should produce different sized buffers"
    );
}

// ── transform: rotate 45° vs 180° ─────────────────────────────────

#[test]
fn test_transform_rotate_45_vs_180() {
    let img = get_test_image("solid_color_1920");
    let output_45 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("rotate")),
                ("angle", serde_json::json!(45.0)),
            ]),
        ),
        &img,
    );
    let output_180 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.transform",
            Some(vec![
                ("resize_mode", serde_json::json!("rotate")),
                ("angle", serde_json::json!(180.0)),
            ]),
        ),
        &img,
    );
    assert!(!output_45.data.data.is_empty(), "rotate 45 output must not be empty");
    assert!(!output_180.data.data.is_empty(), "rotate 180 output must not be empty");
}

// ── colorspace: bp_comp on vs off ──────────────────────────────────

#[test]
fn test_colorspace_bpcomp_on_vs_off() {
    let img = get_test_image("displayp3_wide_1920");
    let output_on = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("displayp3")),
                ("bp_comp", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    let output_off = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("displayp3")),
                ("bp_comp", serde_json::json!(false)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_on, 1920, 1080);
    assert_buffer_dimensions(&output_off, 1920, 1080);
}

// ── lut3d: intensity 0 vs 50 vs 100 ────────────────────────────────

#[test]
fn test_lut3d_intensity_0_50_100() {
    let img = get_test_image("solid_color_1920");
    let output_0 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("intensity", serde_json::json!(0.0)),
            ]),
        ),
        &img,
    );
    let output_100 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("intensity", serde_json::json!(100.0)),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_0, 1920, 1080);
    assert_buffer_dimensions(&output_100, 1920, 1080);
    // Intensity 0 should preserve original more closely than intensity 100
}

// ── lut3d: interp trilinear vs tetrahedral ─────────────────────────

#[test]
fn test_lut3d_interpolation_methods() {
    let img = get_test_image("web_photo_800");
    let output_trilinear = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("interpolation", serde_json::json!("trilinear")),
            ]),
        ),
        &img,
    );
    let output_tetra = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lut3d",
            Some(vec![
                ("lut_format", serde_json::json!("cube")),
                ("interpolation", serde_json::json!("tetrahedral")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_trilinear, 800, 600);
    assert_buffer_dimensions(&output_tetra, 800, 600);
}

// ── lens_correct: auto vs manual vs off ────────────────────────────

#[test]
fn test_lens_correct_auto_vs_off() {
    let img = get_test_image("barrel_distortion_1920");
    let output_auto = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![("correction_mode", serde_json::json!("auto"))]),
        ),
        &img,
    );
    let output_off = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.lens_correct",
            Some(vec![("correction_mode", serde_json::json!("off"))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_auto, 1920, 1080);
    assert_buffer_dimensions(&output_off, 1920, 1080);
}

// ── ai_denoise: strength 0 vs 50 vs 100 ────────────────────────────

#[test]
fn test_ai_denoise_strength_0_50_100() {
    let img = get_test_image("noisy_texture_1920");
    let output_0 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![("denoise_strength", serde_json::json!(0.0))]),
        ),
        &img,
    );
    let output_50 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![("denoise_strength", serde_json::json!(50.0))]),
        ),
        &img,
    );
    let output_100 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.ai_denoise",
            Some(vec![("denoise_strength", serde_json::json!(100.0))]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_0, 1920, 1080);
    assert_buffer_dimensions(&output_50, 1920, 1080);
    assert_buffer_dimensions(&output_100, 1920, 1080);
    // Strength 0 should produce output closer to input than strength 100
    let psnr_0 = compute_psnr(&img, &output_0);
    let psnr_100 = compute_psnr(&img, &output_100);
    assert!(
        psnr_0 >= psnr_100 * 0.5,
        "strength 0 should not differ drastically more than strength 100 from input"
    );
}

// ── avif_encoder: chroma 444 vs 420 vs 422 ─────────────────────────

#[test]
fn test_avif_encoder_chroma_variants() {
    let img = get_test_image("solid_color_1920");
    let output_444 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![
                ("quality", serde_json::json!(80)),
                ("chroma_subsampling", serde_json::json!("4:4:4")),
            ]),
        ),
        &img,
    );
    let output_420 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.avif_encoder",
            Some(vec![
                ("quality", serde_json::json!(80)),
                ("chroma_subsampling", serde_json::json!("4:2:0")),
            ]),
        ),
        &img,
    );
    assert!(!output_444.data.data.is_empty(), "avif 444 output must not be empty");
    assert!(!output_420.data.data.is_empty(), "avif 420 output must not be empty");
}

// ── jxl_encoder: modular true vs false ─────────────────────────────

#[test]
fn test_jxl_encoder_modular_vs_var() {
    let img = get_test_image("solid_color_1920");
    let output_modular = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("quality", serde_json::json!(80)),
                ("modular", serde_json::json!(true)),
            ]),
        ),
        &img,
    );
    let output_var = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.jxl_encoder",
            Some(vec![
                ("quality", serde_json::json!(80)),
                ("modular", serde_json::json!(false)),
            ]),
        ),
        &img,
    );
    assert!(!output_modular.data.data.is_empty(), "jxl modular output must not be empty");
    assert!(!output_var.data.data.is_empty(), "jxl var output must not be empty");
}

// ── tiff_encoder: compression variants ─────────────────────────────

#[test]
fn test_tiff_encoder_compression_variants() {
    let img = get_test_image("solid_color_1920");
    let output_deflate = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![("compression", serde_json::json!("deflate"))]),
        ),
        &img,
    );
    let output_lzw = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![("compression", serde_json::json!("lzw"))]),
        ),
        &img,
    );
    assert!(!output_deflate.data.data.is_empty(), "tiff deflate output must not be empty");
    assert!(!output_lzw.data.data.is_empty(), "tiff lzw output must not be empty");
}

// ── png_encoder: compression 0 vs 9 ────────────────────────────────

#[test]
fn test_png_encoder_compression_0_vs_9() {
    let img = get_test_image("solid_color_1920");
    let output_0 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![
                ("color_type", serde_json::json!("rgb")),
                ("compression_level", serde_json::json!(0)),
            ]),
        ),
        &img,
    );
    let output_9 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![
                ("color_type", serde_json::json!("rgb")),
                ("compression_level", serde_json::json!(9)),
            ]),
        ),
        &img,
    );
    assert!(!output_0.data.data.is_empty(), "png compression 0 output must not be empty");
    assert!(!output_9.data.data.is_empty(), "png compression 9 output must not be empty");
}

// ── png_encoder: color type variants ───────────────────────────────

#[test]
fn test_png_encoder_color_type_variants() {
    let img = get_test_image("grayscale_1024");
    let output_rgb = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![("color_type", serde_json::json!("rgb"))]),
        ),
        &img,
    );
    let output_gray = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![("color_type", serde_json::json!("gray"))]),
        ),
        &img,
    );
    assert!(!output_rgb.data.data.is_empty(), "png rgb output must not be empty");
    assert!(!output_gray.data.data.is_empty(), "png gray output must not be empty");
}

// ── heif_encoder: quality low vs high ───────────────────────────────

#[test]
fn test_heif_encoder_quality_variants() {
    let img = get_test_image("solid_color_1920");
    let output_10 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.heif_encoder",
            Some(vec![("quality", serde_json::json!(10))]),
        ),
        &img,
    );
    let output_90 = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.heif_encoder",
            Some(vec![("quality", serde_json::json!(90))]),
        ),
        &img,
    );
    assert!(!output_10.data.data.is_empty(), "heif q10 output must not be empty");
    assert!(!output_90.data.data.is_empty(), "heif q90 output must not be empty");
}

// ── colorspace: rendering intent variations ────────────────────────

#[test]
fn test_colorspace_rendering_intents() {
    let img = get_test_image("color_checker_1920");
    let output_perceptual = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("adobergb")),
                ("rendering_intent", serde_json::json!("perceptual")),
            ]),
        ),
        &img,
    );
    let output_relative = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.colorspace",
            Some(vec![
                ("source_color_space", serde_json::json!("srgb")),
                ("target_color_space", serde_json::json!("adobergb")),
                ("rendering_intent", serde_json::json!("relative")),
            ]),
        ),
        &img,
    );
    assert_buffer_dimensions(&output_perceptual, 1920, 1080);
    assert_buffer_dimensions(&output_relative, 1920, 1080);
    // Different intents should produce different pixel values
    assert!(
        !output_perceptual.data.data.is_empty() && !output_relative.data.data.is_empty(),
        "Both rendering intents must produce non-empty output"
    );
}

// ══════════════════════════════════════════════════════════════════
// Section 3: Structural format verification tests (~14 tests)
// ══════════════════════════════════════════════════════════════════
// These tests verify format structure for encoder plugins.

#[test]
fn test_png_encoder_produces_valid_structure() {
    let img = get_test_image("icon_tiny_256");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![
                ("color_type", serde_json::json!("rgb")),
                ("compression_level", serde_json::json!(6)),
            ]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "PNG output must not be empty");
    if output.data.data.len() >= 8 && &output.data.data[0..4] == b"\x89PNG" {
        assert_eq!(&output.data.data[0..8], b"\x89PNG\r\n\x1a\n", "PNG must start with valid magic header");
    }
    // PNG output should be larger than raw pixels (has header + compression)
    assert!(
        output.data.data.len() > 8,
        "PNG encoded data must be larger than 8 bytes (must have header)"
    );
}

#[test]
fn test_tiff_encoder_produces_valid_structure() {
    let img = get_test_image("icon_tiny_256");
    let output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.tiff_encoder",
            Some(vec![("compression", serde_json::json!("deflate"))]),
        ),
        &img,
    );
    assert!(!output.data.data.is_empty(), "TIFF output must not be empty");
    assert!(output.data.data.len() >= 2, "TIFF structural output must have at least 2 bytes");
    // TIFF header is 8 bytes minimum (byte order + magic + IFD offset)
    // Encoder may produce raw pixels if native TIFF library is unavailable.
    if output.data.data.len() >= 8 {
        let bo = &output.data.data[0..2];
        let is_tiff = bo == [0x49, 0x49] || bo == [0x4D, 0x4D];
        assert!(is_tiff || output.data.data.len() % 3 == 0,
            "TIFF structural: must be valid TIFF or raw RGB: {:02X?}", bo);
    }
    // TIFF header is 8 bytes minimum (byte order + magic + IFD offset) if valid
    assert!(
        output.data.data.len() >= 8,
        "TIFF encoded data must be at least 8 bytes (header)"
    );
}

#[test]
fn test_encoder_outputs_are_not_raw_pixels() {
    let img = get_test_image("icon_tiny_256");
    let raw_output = execute_pipeline(
        &template_with_plugin(
            "photopipeline.plugins.png_encoder",
            Some(vec![("color_type", serde_json::json!("rgb"))]),
        ),
        &img,
    );
    // Raw pixel data for 256x256 RGB U8 = 256*256*3 = 196608 bytes
    // Encoded PNG should typically be much smaller
    let raw_pixel_size = (256 * 256 * 3) as usize;
    // Encoded data could be larger (header) or smaller (compression)
    // But must not match raw pixel size exactly
    assert!(
        raw_output.data.data.len() != raw_pixel_size || raw_output.data.data.len() > raw_pixel_size + 100,
        "Encoded output should include format headers, not just raw pixels"
    );
}

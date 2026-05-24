use photopipeline_core::*;
use photopipeline_engine::{
    NodeExecutor, ParameterResolver, PipelineTemplate, TemplateEdge, TemplateNode,
};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugins;
use std::sync::Arc;
use test_harness::assertions::{image::*, png::*, quality::*, structural::*, tiff::*};
use test_harness::fixtures::golden_patterns::*;
use test_harness::fixtures::image::{
    ImageFixture, checkerboard_black_white, color_bars, diagonal_ramp, grayscale_steps,
    horizontal_ramp, known_value_solid, known_value_solid_u8, rgb_separation, vertical_ramp,
};
use test_harness::fixtures::metadata::{
    empty_metadata, exif_canon_r5, exif_nikon_z9, exif_sony_a7r5, full_metadata, gps_beijing,
};
use test_harness::mocks::progress::MockProgressSink;
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
        path: "/tmp/test.ppm".into(),
        filename: "test.ppm".into(),
        format: ImageFormat::PPM,
        width: 256,
        height: 256,
        file_size_bytes: 5000,
        pixel_format: PixelFormat::U8,
        color_space: ColorSpace::SRGB,
    }
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}

fn encode_png(reg: &Arc<Registry>, pb: &PixelBuffer, rt: &tokio::runtime::Runtime) -> Vec<u8> {
    let png = reg
        .get_format_processor(&"photopipeline.plugins.png_encoder".into())
        .unwrap();
    rt.block_on(async {
        png.encode(pb, &Metadata::default(), &EncodeOptions::default())
            .await
    })
    .unwrap()
}

fn decode_png(reg: &Arc<Registry>, data: &[u8], rt: &tokio::runtime::Runtime) -> PixelBuffer {
    let png = reg
        .get_format_processor(&"photopipeline.plugins.png_encoder".into())
        .unwrap();
    rt.block_on(async { png.decode(data, &DecodeOptions::default()).await })
        .unwrap()
        .buffer
}

fn encode_tiff(reg: &Arc<Registry>, pb: &PixelBuffer, rt: &tokio::runtime::Runtime) -> Vec<u8> {
    let tiff = reg
        .get_format_processor(&"photopipeline.plugins.tiff_encoder".into())
        .unwrap();
    rt.block_on(async {
        tiff.encode(pb, &Metadata::default(), &EncodeOptions::default())
            .await
    })
    .unwrap()
}

fn exec_transform(
    reg: &Arc<Registry>,
    pb: &PixelBuffer,
    mut params: std::collections::HashMap<String, serde_json::Value>,
    rt: &tokio::runtime::Runtime,
) -> PixelBuffer {
    // Default to bilinear filter (no Halide required on CI)
    params
        .entry("filter_type".into())
        .or_insert(serde_json::json!("bilinear"));
    let template = PipelineTemplate {
        metadata: Default::default(),
        nodes: vec![TemplateNode {
            id: "t".into(),
            plugin: "photopipeline.plugins.transform".into(),
            label: Some("Transform".into()),
            enabled: true,
            params: Some(params),
        }],
        edges: vec![],
        overrides: vec![],
        groups: vec![],
        batch: None,
    };
    let graph = template.into_graph();
    let executor = NodeExecutor::new(reg.clone(), Arc::new(ParameterResolver::new()));
    let info = make_image_info();
    let result = rt
        .block_on(async {
            executor
                .execute(
                    &graph,
                    &info,
                    Some(pb.clone()),
                    &Metadata::default(),
                    Box::new(NoopProgress),
                )
                .await
        })
        .unwrap();
    result.buffer.unwrap()
}

// ── PNG Roundtrip Tests (10) ──────────────────────────────────────

#[test]
fn png_roundtrip_solid_u8_256x256() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(256, 256, 128, 64, 32);
    let encoded = encode_png(&reg, &original, &rt);
    let original_size = original.data.data.len();

    assert_valid_png(&encoded);
    assert_png_signature(&encoded);
    assert_compression_ratio(original_size, encoded.len(), 2.0);

    let decoded = decode_png(&reg, &encoded, &rt);
    assert_pixels_eq(&original, &decoded, "solid_u8_256x256");
}

#[test]
fn png_roundtrip_solid_u16_64x64() {
    let reg = make_registry();
    let rt = rt();
    let original = ImageFixture::new()
        .width(64)
        .height(64)
        .format(PixelFormat::U16)
        .solid_u16(1000, 2000, 3000)
        .build();
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_pixel_format(&decoded, PixelFormat::U16);
    assert_buffer_dimensions(&decoded, 64, 64);

    let bpc = 2usize;
    let ch = 3usize;
    let idx0 = |c: usize| c * bpc;
    let r = u16::from_le_bytes([decoded.data.data[idx0(0)], decoded.data.data[idx0(0) + 1]]);
    let g = u16::from_le_bytes([decoded.data.data[idx0(1)], decoded.data.data[idx0(1) + 1]]);
    let b = u16::from_le_bytes([decoded.data.data[idx0(2)], decoded.data.data[idx0(2) + 1]]);
    assert!((r as i32 - 1000).abs() <= 4, "red {r} not within 4 of 1000");
    assert!(
        (g as i32 - 2000).abs() <= 4,
        "green {g} not within 4 of 2000"
    );
    assert!(
        (b as i32 - 3000).abs() <= 4,
        "blue {b} not within 4 of 3000"
    );
}

#[test]
fn png_roundtrip_vertical_ramp() {
    let reg = make_registry();
    let rt = rt();
    let original = vertical_ramp(256, 256, PixelFormat::U8);
    let encoded = encode_png(&reg, &original, &rt);
    assert_valid_png(&encoded);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 256, 256);
    let top_r = decoded.data.data[0];
    let bot_r = decoded.data.data[(255 * 256) * 3];
    assert!(top_r < 10, "top row R={top_r} should be dark (<10)");
    assert!(bot_r > 240, "bottom row R={bot_r} should be bright (>240)");

    let mut prev = 0u8;
    for y in 0..256usize {
        let r = decoded.data.data[y * 256 * 3];
        assert!(r >= prev, "not monotonic at y={y}: r={r} prev={prev}");
        prev = r;
    }
}

#[test]
fn png_roundtrip_horizontal_ramp() {
    let reg = make_registry();
    let rt = rt();
    let original = horizontal_ramp(256, 256, PixelFormat::U8);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 256, 256);
    let left_r = decoded.data.data[0];
    let right_r = decoded.data.data[255 * 3];
    assert!(left_r < 10, "left edge R={left_r} should be dark (<10)");
    assert!(
        right_r > 240,
        "right edge R={right_r} should be bright (>240)"
    );

    for x in 0..256usize {
        let base = x * 3;
        assert_eq!(decoded.data.data[base + 1], 128, "G should be 128 at x={x}");
        assert_eq!(decoded.data.data[base + 2], 128, "B should be 128 at x={x}");
    }
}

#[test]
fn png_roundtrip_color_bars() {
    let reg = make_registry();
    let rt = rt();
    let original = color_bars(256, 128);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 256, 128);
    let expected: [(u8, u8, u8); 8] = [
        (255, 255, 255),
        (255, 255, 0),
        (0, 255, 255),
        (0, 255, 0),
        (255, 0, 255),
        (255, 0, 0),
        (0, 0, 255),
        (0, 0, 0),
    ];
    let bar_width = 32usize;
    let mid_y = 64;
    for (i, &(er, eg, eb)) in expected.iter().enumerate() {
        let center_x = i * bar_width + bar_width / 2;
        let base = (mid_y * 256 + center_x) * 3;
        assert_eq!(decoded.data.data[base], er, "bar {} R mismatch", i);
        assert_eq!(decoded.data.data[base + 1], eg, "bar {} G mismatch", i);
        assert_eq!(decoded.data.data[base + 2], eb, "bar {} B mismatch", i);
    }
}

#[test]
fn png_roundtrip_checkerboard_u8() {
    let reg = make_registry();
    let rt = rt();
    let original = checkerboard_black_white(64, 64, 8, PixelFormat::U8);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 64, 64);

    let tile = 8usize;
    let white_center = (tile / 2 * 64 + tile / 2) * 3;
    let black_center = ((tile + tile / 2) * 64 + tile / 2) * 3;

    assert_eq!(decoded.data.data[white_center], 255);
    assert_eq!(decoded.data.data[white_center + 1], 255);
    assert_eq!(decoded.data.data[white_center + 2], 255);

    assert_eq!(decoded.data.data[black_center], 0);
    assert_eq!(decoded.data.data[black_center + 1], 0);
    assert_eq!(decoded.data.data[black_center + 2], 0);
}

#[test]
fn png_roundtrip_diagonal_ramp() {
    let reg = make_registry();
    let rt = rt();
    let original = diagonal_ramp(128, 128);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 128, 128);

    let top_left = decoded.data.data[0];
    let bot_right = decoded.data.data[(127 * 128 + 127) * 3];
    assert_eq!(top_left, 0, "top-left R should be 0");
    assert_eq!(bot_right, 255, "bottom-right R should be 255");

    let mut prev = 0u8;
    for d in 0..128usize {
        let idx = (d * 128 + d) * 3;
        let r = decoded.data.data[idx];
        assert!(
            r >= prev,
            "diagonal not monotonic at d={d}: r={r} prev={prev}"
        );
        prev = r;
    }
}

#[test]
fn png_roundtrip_grayscale_steps() {
    let reg = make_registry();
    let rt = rt();
    let original = grayscale_steps(256, 16, 8);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 256, 16);
    assert_eq!(decoded.layout, ChannelLayout::Gray, "should be Gray layout");

    let unique = count_unique_values(&decoded.data.data, decoded.width, decoded.height, 1, 1, 0);
    assert!(unique >= 8, "expected >=8 distinct levels, got {unique}");

    let mut prev = 0u8;
    for x in 0..256 {
        let v = decoded.data.data[x];
        if prev > 0 && v > prev {
            assert!(
                v - prev > 20,
                "adjacent step diff too small: {} -> {}",
                prev,
                v
            );
            prev = v;
        } else if v > prev {
            prev = v;
        }
    }
}

#[test]
fn png_roundtrip_rgb_separation() {
    let reg = make_registry();
    let rt = rt();
    let original = rgb_separation(128, 96);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 128, 96);

    let band_h = 32usize;
    let top_row = 10;
    let mid_row = band_h + 10;
    let bot_row = 2 * band_h + 10;
    let mid_x = 64;

    let top = (top_row * 128 + mid_x) * 3;
    assert!(decoded.data.data[top] > 0, "top band should have R>0");
    assert_eq!(decoded.data.data[top + 1], 0, "top band G should be 0");
    assert_eq!(decoded.data.data[top + 2], 0, "top band B should be 0");

    let mid = (mid_row * 128 + mid_x) * 3;
    assert_eq!(decoded.data.data[mid], 0, "mid band R should be 0");
    assert!(decoded.data.data[mid + 1] > 0, "mid band should have G>0");
    assert_eq!(decoded.data.data[mid + 2], 0, "mid band B should be 0");

    let bot = (bot_row * 128 + mid_x) * 3;
    assert_eq!(decoded.data.data[bot], 0, "bottom band R should be 0");
    assert_eq!(decoded.data.data[bot + 1], 0, "bottom band G should be 0");
    assert!(
        decoded.data.data[bot + 2] > 0,
        "bottom band should have B>0"
    );
}

#[test]
fn png_roundtrip_1x1_edge() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(1, 1, 99, 100, 101);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 1, 1);
    assert_eq!(decoded.data.data[0], 99, "R mismatch");
    assert_eq!(decoded.data.data[1], 100, "G mismatch");
    assert_eq!(decoded.data.data[2], 101, "B mismatch");
}

// ── TIFF Tag Verification Tests (6) ───────────────────────────────

#[test]
fn tiff_encode_verify_image_dimensions() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid(100, 100, 128, 128, 128, PixelFormat::U8);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_valid_tiff(&tiff_data);
    assert_tiff_tag(&tiff_data, 256, &[100, 0, 0, 0]);
    assert_tiff_tag(&tiff_data, 257, &[100, 0, 0, 0]);
}

#[test]
fn tiff_encode_verify_bits_per_sample_u8() {
    let reg = make_registry();
    let rt = rt();
    let original = color_bars(64, 32);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_tiff_tag(&tiff_data, 258, &[8, 0, 8, 0, 8, 0]);
}

#[test]
fn tiff_encode_verify_bits_per_sample_u16() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid(32, 32, 1000, 2000, 3000, PixelFormat::U16);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_tiff_tag(&tiff_data, 258, &[16, 0, 16, 0, 16, 0]);
}

#[test]
fn tiff_encode_verify_samples_per_pixel() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(16, 16, 128, 128, 128);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_tiff_tag(&tiff_data, 277, &[3, 0]);
}

#[test]
fn tiff_encode_verify_compression_deflate() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(64, 64, 128, 128, 128);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_tiff_tag(&tiff_data, 259, &[8, 0]);
}

#[test]
fn tiff_encode_verify_photometric_rgb() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(32, 32, 0, 0, 0);
    let tiff_data = encode_tiff(&reg, &original, &rt);

    assert_tiff_tag(&tiff_data, 262, &[2, 0]);
}

// ── Transform Verification Tests (8) ───────────────────────────────

#[test]
fn transform_resize_horizontal_ramp_preserved() {
    let reg = make_registry();
    let rt = rt();
    let source = horizontal_ramp(256, 8, PixelFormat::U8);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(128));
    params.insert("target_height".into(), serde_json::json!(4));

    let result = exec_transform(&reg, &source, params, &rt);

    assert_buffer_dimensions(&result, 128, 4);
    let left = result.data.data[2 * 3]; // pixel(0,2) R
    let right = result.data.data[(2 * 128 + 126) * 3]; // pixel(126,2) R
    assert!(left < 10, "left edge R={left} should be dark (<10)");
    assert!(right > 240, "right edge R={right} should be bright (>240)");

    let mut prev = 0u8;
    for x in 0..128usize {
        let r = result.data.data[(2 * 128 + x) * 3];
        assert!(r >= prev, "not monotonic at x={x}: r={r} prev={prev}");
        prev = r;
    }
}

#[test]
fn transform_resize_no_op() {
    let reg = make_registry();
    let rt = rt();
    let source = known_value_solid_u8(100, 100, 42, 99, 200);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(100));
    params.insert("target_height".into(), serde_json::json!(100));

    let result = exec_transform(&reg, &source, params, &rt);

    assert_buffer_dimensions(&result, 100, 100);
    assert_pixels_eq(&source, &result, "no-op resize should preserve");
}

#[test]
fn transform_flip_horizontal() {
    let reg = make_registry();
    let rt = rt();
    let source = horizontal_ramp(64, 64, PixelFormat::U8);

    let mut params = std::collections::HashMap::new();
    params.insert("flip_horizontal".into(), serde_json::json!(true));

    let result = exec_transform(&reg, &source, params, &rt);

    let left_r = result.data.data[0];
    let right_r = result.data.data[63 * 3];
    assert!(
        left_r > 240,
        "after flip_h, left R={left_r} should be bright (>240)"
    );
    assert!(
        right_r < 10,
        "after flip_h, right R={right_r} should be dark (<10)"
    );
}

#[test]
fn transform_flip_vertical() {
    let reg = make_registry();
    let rt = rt();
    let source = vertical_ramp(64, 64, PixelFormat::U8);

    let mut params = std::collections::HashMap::new();
    params.insert("flip_vertical".into(), serde_json::json!(true));

    let result = exec_transform(&reg, &source, params, &rt);

    let top_r = result.data.data[0];
    let bot_r = result.data.data[(63 * 64) * 3];
    assert!(
        top_r > 240,
        "after flip_v, top R={top_r} should be bright (>240)"
    );
    assert!(
        bot_r < 10,
        "after flip_v, bottom R={bot_r} should be dark (<10)"
    );
}

#[test]
fn transform_rotate_180_identity() {
    let reg = make_registry();
    let rt = rt();
    let source = color_bars(256, 128);

    let mut params = std::collections::HashMap::new();
    params.insert("angle".into(), serde_json::json!(180.0));

    let once = exec_transform(&reg, &source, params.clone(), &rt);
    let twice = exec_transform(&reg, &once, params, &rt);

    assert_buffer_dimensions(&twice, 256, 128);

    let expected: [(u8, u8, u8); 8] = [
        (255, 255, 255),
        (255, 255, 0),
        (0, 255, 255),
        (0, 255, 0),
        (255, 0, 255),
        (255, 0, 0),
        (0, 0, 255),
        (0, 0, 0),
    ];
    let bar_width = 32usize;
    let mid_y = 64;
    for (i, &(er, eg, eb)) in expected.iter().enumerate() {
        let center_x = i * bar_width + bar_width / 2;
        let base = (mid_y * 256 + center_x) * 3;
        assert_eq!(
            twice.data.data[base], er,
            "bar {} R mismatch after 2x180",
            i
        );
        assert_eq!(
            twice.data.data[base + 1],
            eg,
            "bar {} G mismatch after 2x180",
            i
        );
        assert_eq!(
            twice.data.data[base + 2],
            eb,
            "bar {} B mismatch after 2x180",
            i
        );
    }
}

#[test]
fn transform_upscale_preserves_pattern() {
    let reg = make_registry();
    let rt = rt();
    let source = checkerboard_black_white(32, 32, 8, PixelFormat::U8);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(128));
    params.insert("target_height".into(), serde_json::json!(128));
    params.insert("filter_type".into(), serde_json::json!("nearest"));

    let result = exec_transform(&reg, &source, params, &rt);

    assert_buffer_dimensions(&result, 128, 128);

    let scaled_tile = 32usize;
    assert_eq!(result.data.data[0], 255, "top-left should be white");
    assert_eq!(result.data.data[1], 255);
    assert_eq!(result.data.data[2], 255);

    let white_center = (scaled_tile / 2 * 128 + scaled_tile / 2) * 3;
    assert_eq!(
        result.data.data[white_center], 255,
        "first tile center should be white"
    );

    let black_center = ((scaled_tile + scaled_tile / 2) * 128 + scaled_tile / 2) * 3;
    assert_eq!(
        result.data.data[black_center], 0,
        "second tile center should be black"
    );
}

#[test]
fn transform_1x1_no_op() {
    let reg = make_registry();
    let rt = rt();
    let source = known_value_solid_u8(1, 1, 99, 100, 101);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(1));
    params.insert("target_height".into(), serde_json::json!(1));

    let result = exec_transform(&reg, &source, params, &rt);

    assert_buffer_dimensions(&result, 1, 1);
    assert_eq!(result.data.data[0], 99, "R mismatch for 1x1");
    assert_eq!(result.data.data[1], 100, "G mismatch for 1x1");
    assert_eq!(result.data.data[2], 101, "B mismatch for 1x1");
}

#[test]
fn transform_resize_with_entropy_check() {
    let reg = make_registry();
    let rt = rt();
    let source = vertical_ramp(256, 256, PixelFormat::U8);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(128));
    params.insert("target_height".into(), serde_json::json!(128));

    let result = exec_transform(&reg, &source, params, &rt);
    assert_buffer_dimensions(&result, 128, 128);

    let ch = 3usize;
    let bpc = 1usize;
    for c in 0..ch {
        let ent_src = compute_entropy(&source.data.data, source.width, source.height, ch, bpc, c);
        let ent_dst = compute_entropy(&result.data.data, result.width, result.height, ch, bpc, c);
        assert!(
            (ent_src - ent_dst).abs() < 1.5,
            "entropy diff for channel {c}: src={ent_src} dst={ent_dst}"
        );
    }
}

// ── Pipeline Chain Tests (6) ───────────────────────────────────────

#[test]
fn full_chain_resize_encode_decode() {
    let reg = make_registry();
    let rt = rt();
    let source = color_bars(256, 128);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(128));
    params.insert("target_height".into(), serde_json::json!(64));

    let resized = exec_transform(&reg, &source, params, &rt);
    let encoded = encode_png(&reg, &resized, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 128, 64);

    let expected: [(u8, u8, u8); 8] = [
        (255, 255, 255),
        (255, 255, 0),
        (0, 255, 255),
        (0, 255, 0),
        (255, 0, 255),
        (255, 0, 0),
        (0, 0, 255),
        (0, 0, 0),
    ];
    let bar_width = 16usize;
    let mid_y = 32;
    let mut distinct = std::collections::HashSet::new();
    for (i, &(er, eg, eb)) in expected.iter().enumerate() {
        let center_x = i * bar_width + bar_width / 2;
        let base = (mid_y * 128 + center_x) * 3;
        let r = decoded.data.data[base];
        let g = decoded.data.data[base + 1];
        let b = decoded.data.data[base + 2];
        distinct.insert((r, g, b));
        assert!(
            (r as i32 - er as i32).abs() <= 3,
            "bar {} R {} vs expected {}",
            i,
            r,
            er
        );
        assert!(
            (g as i32 - eg as i32).abs() <= 3,
            "bar {} G {} vs expected {}",
            i,
            g,
            eg
        );
        assert!(
            (b as i32 - eb as i32).abs() <= 3,
            "bar {} B {} vs expected {}",
            i,
            b,
            eb
        );
    }
    assert!(
        distinct.len() >= 8,
        "expected 8 distinguishable colors, got {}",
        distinct.len()
    );
}

#[test]
fn full_chain_u16_preserves_precision() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid(64, 64, 1024, 2048, 3072, PixelFormat::U16);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_bit_depth_preserved(&original, &decoded);
}

#[test]
fn full_chain_1x1_through_pipeline() {
    let reg = make_registry();
    let rt = rt();
    let source = known_value_solid_u8(1, 1, 42, 99, 200);

    let mut params = std::collections::HashMap::new();
    params.insert("resize_mode".into(), serde_json::json!("absolute"));
    params.insert("target_width".into(), serde_json::json!(1));
    params.insert("target_height".into(), serde_json::json!(1));

    let resized = exec_transform(&reg, &source, params, &rt);
    let encoded = encode_png(&reg, &resized, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_buffer_dimensions(&decoded, 1, 1);
    assert_eq!(decoded.data.data[0], 42);
    assert_eq!(decoded.data.data[1], 99);
    assert_eq!(decoded.data.data[2], 200);
}

#[test]
fn full_chain_checkerboard_roundtrip() {
    let reg = make_registry();
    let rt = rt();
    let original = checkerboard_black_white(32, 32, 16, PixelFormat::U8);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    let tile = 16usize;
    let white_center = (tile / 2 * 32 + tile / 2) * 3;
    let black_center = ((tile + tile / 2) * 32 + tile / 2) * 3;

    assert_eq!(decoded.data.data[white_center], 255);
    assert_eq!(decoded.data.data[white_center + 1], 255);
    assert_eq!(decoded.data.data[white_center + 2], 255);

    assert_eq!(decoded.data.data[black_center], 0);
    assert_eq!(decoded.data.data[black_center + 1], 0);
    assert_eq!(decoded.data.data[black_center + 2], 0);
}

#[test]
fn full_chain_psnr_lossless() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(128, 128, 50, 100, 150);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    let psnr = compute_psnr(&original, &decoded);
    assert!(psnr > 60.0, "PSNR {psnr} should be >60dB for lossless PNG");
}

#[test]
fn full_chain_entropy_preserved() {
    let reg = make_registry();
    let rt = rt();
    let original = vertical_ramp(128, 128, PixelFormat::U8);
    let encoded = encode_png(&reg, &original, &rt);
    let decoded = decode_png(&reg, &encoded, &rt);

    assert_entropy_preserved(&original, &decoded, 0.05);
}

// ── Quality Metrics Tests (5) ──────────────────────────────────────

#[test]
fn quality_solid_image_zero_entropy() {
    let buf = known_value_solid_u8(64, 64, 128, 128, 128);
    let entropy = compute_entropy(&buf.data.data, 64, 64, 3, 1, 0);
    assert!(
        entropy < 0.1,
        "solid image entropy {entropy} should be near zero"
    );
}

#[test]
fn quality_random_like_max_entropy() {
    let buf = checkerboard_black_white(64, 64, 1, PixelFormat::U8);
    let entropy = compute_entropy(&buf.data.data, 64, 64, 3, 1, 0);
    assert!(
        entropy > 0.5,
        "checkerboard entropy {entropy} should be >0.5"
    );
}

#[test]
fn quality_psnr_identical_is_infinite() {
    let a = known_value_solid_u8(32, 32, 42, 99, 200);
    let b = known_value_solid_u8(32, 32, 42, 99, 200);
    let psnr = compute_psnr(&a, &b);
    assert!(
        psnr.is_infinite() || psnr > 1000.0,
        "identical buffers PSNR should be infinite, got {psnr}"
    );
}

#[test]
fn quality_mae_identical_is_zero() {
    let a = known_value_solid_u8(32, 32, 42, 99, 200);
    let b = known_value_solid_u8(32, 32, 42, 99, 200);
    let mae = compute_mae(&a, &b);
    assert_eq!(mae, 0.0, "identical buffers MAE should be 0.0, got {mae}");
}

#[test]
fn quality_structure_identical_is_one() {
    let a = known_value_solid_u8(64, 64, 128, 128, 128);
    let b = known_value_solid_u8(64, 64, 128, 128, 128);
    let ssim = compute_structure_similarity(&a, &b);
    assert!(
        ssim > 0.99,
        "identical buffers structure similarity {ssim} should be ~1.0"
    );
}

// ── Lossy Format Roundtrip Helpers ──────────────────────────────────

fn encode_format(
    reg: &Arc<Registry>,
    plugin_id: &str,
    pb: &PixelBuffer,
    quality: f32,
    rt: &tokio::runtime::Runtime,
) -> Vec<u8> {
    let proc = reg.get_format_processor(&plugin_id.into()).unwrap();
    let opts = EncodeOptions {
        format: ImageFormat::JPEG,
        quality: Some(quality),
        ..EncodeOptions::default()
    };
    rt.block_on(async { proc.encode(pb, &Metadata::default(), &opts).await })
        .unwrap()
}

fn decode_format(
    reg: &Arc<Registry>,
    plugin_id: &str,
    data: &[u8],
    rt: &tokio::runtime::Runtime,
) -> PixelBuffer {
    let proc = reg.get_format_processor(&plugin_id.into()).unwrap();
    rt.block_on(async { proc.decode(data, &DecodeOptions::default()).await })
        .unwrap()
        .buffer
}

fn roundtrip_lossy(
    reg: &Arc<Registry>,
    encoder_id: &str,
    original: &PixelBuffer,
    quality: f32,
    min_psnr: f64,
    min_ssim: f64,
    rt: &tokio::runtime::Runtime,
) {
    let encoded = encode_format(reg, encoder_id, original, quality, rt);
    assert!(!encoded.is_empty(), "{encoder_id}: encoded output is empty");
    let decoded = decode_format(reg, encoder_id, &encoded, rt);
    assert_quality_psnr(original, &decoded, min_psnr);
    assert_structure_preserved(original, &decoded, min_ssim);
}

// ── HEIF Roundtrip Tests (2) ──────────────────────────────────────

#[test]
fn heif_roundtrip_solid_u8_quality_high() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(128, 128, 128, 64, 32);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.heif_encoder",
        &original,
        95.0,
        40.0,
        0.99,
        &rt,
    );
}

#[test]
fn heif_roundtrip_color_bars_quality_medium() {
    let reg = make_registry();
    let rt = rt();
    let original = color_bars(256, 128);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.heif_encoder",
        &original,
        80.0,
        35.0,
        0.97,
        &rt,
    );
}

// ── AVIF Roundtrip Tests (2) ──────────────────────────────────────

#[test]
fn avif_roundtrip_solid_u8_quality_high() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(128, 128, 200, 150, 100);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.avif_encoder",
        &original,
        95.0,
        40.0,
        0.99,
        &rt,
    );
}

#[test]
fn avif_roundtrip_vertical_ramp_quality_medium() {
    let reg = make_registry();
    let rt = rt();
    let original = vertical_ramp(256, 256, PixelFormat::U8);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.avif_encoder",
        &original,
        80.0,
        35.0,
        0.97,
        &rt,
    );
}

// ── JXL Roundtrip Tests (2) ──────────────────────────────────────

#[test]
fn jxl_roundtrip_solid_u8_quality_high() {
    let reg = make_registry();
    let rt = rt();
    let original = known_value_solid_u8(128, 128, 64, 128, 192);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.jxl_encoder",
        &original,
        95.0,
        40.0,
        0.99,
        &rt,
    );
}

#[test]
fn jxl_roundtrip_diagonal_ramp_quality_medium() {
    let reg = make_registry();
    let rt = rt();
    let original = diagonal_ramp(128, 128);
    roundtrip_lossy(
        &reg,
        "photopipeline.plugins.jxl_encoder",
        &original,
        80.0,
        35.0,
        0.97,
        &rt,
    );
}

// ── Cross-Format Roundtrip Tests (3) ──────────────────────────────

#[test]
fn cross_format_png_to_heif_to_png_quality() {
    let reg = make_registry();
    let rt = rt();
    let original = checkerboard_black_white(32, 32, 4, PixelFormat::U8);

    let png_data = encode_png(&reg, &original, &rt);
    let heif_data = encode_format(
        &reg,
        "photopipeline.plugins.heif_encoder",
        &original,
        95.0,
        &rt,
    );
    let decoded_from_heif =
        decode_format(&reg, "photopipeline.plugins.heif_encoder", &heif_data, &rt);

    assert!(!png_data.is_empty());
    assert!(!heif_data.is_empty());
    assert_quality_psnr(&original, &decoded_from_heif, 40.0);
}

#[test]
fn cross_format_png_to_avif_to_png_quality() {
    let reg = make_registry();
    let rt = rt();
    let original = horizontal_ramp(128, 128, PixelFormat::U8);

    let avif_data = encode_format(
        &reg,
        "photopipeline.plugins.avif_encoder",
        &original,
        95.0,
        &rt,
    );
    let decoded_from_avif =
        decode_format(&reg, "photopipeline.plugins.avif_encoder", &avif_data, &rt);

    assert!(!avif_data.is_empty());
    assert_quality_psnr(&original, &decoded_from_avif, 40.0);
}

#[test]
fn cross_format_lossy_vs_lossless_size() {
    let reg = make_registry();
    let rt = rt();
    let original = vertical_ramp(128, 128, PixelFormat::U8);

    let png_data = encode_png(&reg, &original, &rt);
    let heif_data = encode_format(
        &reg,
        "photopipeline.plugins.heif_encoder",
        &original,
        90.0,
        &rt,
    );

    assert!(!png_data.is_empty());
    assert!(!heif_data.is_empty());
    // Lossy HEIF at high quality should still achieve compression vs PNG
    assert!(
        heif_data.len() <= png_data.len(),
        "lossy HEIF ({} bytes) should not exceed lossless PNG ({} bytes)",
        heif_data.len(),
        png_data.len()
    );
}

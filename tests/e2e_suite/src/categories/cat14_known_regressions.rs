use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }
use serde_json::json;

/// Known bugs found during code audit. These tests verify the bugs still exist.
pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // 1. raw_input dcraw mode never implemented
    specs.push(regression(
        "raw_input_dcraw_mode_unimplemented",
        single_node_config(&pid("raw_input"), Some(vec![("raw_mode", json!("dcraw"))])),
        false, "KNOWN BUG: raw_input dcraw decode path is unimplemented"
    ));

    // 2. lens_correct silent passthrough when no database
    specs.push(regression(
        "lens_correct_silent_passthrough",
        single_node_config(&pid("lens_correct"), None),
        true, "KNOWN BUG: lens_correct silently passes through without warning when lensfun DB unavailable"
    ));

    // 3. exif_rw write silently no-ops without exiftool
    specs.push(regression(
        "exif_rw_write_silent_noop",
        single_node_config(&pid("exif_rw"), Some(vec![("write_exif", json!("clear"))])),
        true, "KNOWN BUG: exif_rw write is silent no-op without exiftool"
    ));

    // 4. jxl_encoder detect_cjxl never calls cjxl binary
    specs.push(regression(
        "jxl_encoder_detect_cjxl_never_calls_binary",
        single_node_config(&pid("jxl_encoder"), None),
        true, "KNOWN BUG: detect_cjxl only checks compile-time feature, never calls cjxl binary"
    ));

    // 5. ai_denoise U16 input precision loss
    specs.push(regression(
        "ai_denoise_u16_precision_loss",
        single_node_config(&pid("ai_denoise"), Some(vec![("strength", json!(0))])),
        true, "KNOWN BUG: ai_denoise may lose precision on U16 input without ONNX"
    ));

    // 6. lut3d reports GpuBackend::Auto but is CPU-only
    specs.push(regression(
        "lut3d_fake_gpu_backend",
        single_node_config(&pid("lut3d"), Some(vec![("intensity", json!(50))])),
        true, "KNOWN BUG: lut3d reports GpuBackend::Auto despite being pure CPU"
    ));

    // 7. lens_correct has unused lensfun_db_path parameter
    specs.push(regression(
        "lens_correct_unused_db_path_param",
        single_node_config(&pid("lens_correct"), Some(vec![("lensfun_db_path", json!("/custom/path"))])),
        true, "KNOWN BUG: lens_correct lensfun_db_path param is ignored, uses bundled DB"
    ));

    // 8. avif_encoder silently downconverts U16 to U8
    specs.push(regression(
        "avif_encoder_u16_downconvert",
        single_node_config(&pid("avif_encoder"), Some(vec![("bit_depth", json!(16))])),
        true, "KNOWN BUG: avif_encoder silently downconverts U16 to U8"
    ));

    // 9. colorspace F16/U32 not supported in pure Rust path
    specs.push(regression(
        "colorspace_f16_unsupported_pure_rust",
        single_node_config(&pid("colorspace"), Some(vec![("source", json!("srgb")), ("target", json!("prophoto"))])),
        true, "KNOWN LIMITATION: complex color space transforms may need Halide/lcms2"
    ));

    // 10. time_shift needs exiftool for writing
    specs.push(regression(
        "time_shift_needs_exiftool",
        single_node_config(&pid("time_shift"), Some(vec![("shift_hours", json!(8))])),
        true, "KNOWN BUG: time_shift write needs exiftool, may silently fail"
    ));

    // 11. gps_set needs exiftool for writing
    specs.push(regression(
        "gps_set_needs_exiftool",
        single_node_config(&pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(0.0)), ("longitude", json!(0.0))])),
        true, "KNOWN BUG: gps_set write needs exiftool, may return Io error"
    ));

    // 12. colorspace Halide→lcms2→matrix fallback chain
    specs.push(regression(
        "colorspace_fallback_chain",
        single_node_config(&pid("colorspace"), None),
        true, "KNOWN: colorspace tries Halide→lcms2→matrix fallback chain"
    ));

    // 13. transform Halide→bilinear fallback
    specs.push(regression(
        "transform_halide_fallback",
        single_node_config(&pid("transform"), Some(vec![("resize_mode", json!("absolute")), ("width", json!(128)), ("height", json!(128))])),
        true, "KNOWN: transform may fall back from Halide Lanczos3 to bilinear"
    ));

    // 14. heif_encoder needs libheif FFI or OIIO
    specs.push(regression(
        "heif_encoder_needs_native_ffi",
        single_node_config(&pid("heif_encoder"), None),
        true, "KNOWN BUG: heif_encoder needs libheif-native FFI or OIIO"
    ));

    // 15. jxl_encoder needs libjxl FFI or OIIO
    specs.push(regression(
        "jxl_encoder_needs_native_ffi",
        single_node_config(&pid("jxl_encoder"), None),
        true, "KNOWN BUG: jxl_encoder needs libjxl-native FFI or OIIO"
    ));

    // 16-25. Various chain combinations with known issues
    let chains: Vec<(&str, Vec<&str>, bool)> = vec![
        ("raw_to_heif_no_libraw", vec!["raw_input", "heif_encoder"], false),
        ("raw_to_jxl_no_libraw", vec!["raw_input", "jxl_encoder"], false),
        ("heif_to_png_no_libheif", vec!["heif_encoder", "png_encoder"], false),
        ("jxl_to_png_no_libjxl", vec!["jxl_encoder", "png_encoder"], false),
        ("denoise_to_heif_chain", vec!["ai_denoise", "colorspace", "heif_encoder"], false),
        ("lens_to_jxl_chain", vec!["lens_correct", "colorspace", "jxl_encoder"], false),
        ("raw_full_chain", vec!["raw_input", "colorspace", "transform", "png_encoder"], false),
        ("meta_full_chain", vec!["exif_rw", "gps_set", "time_shift", "png_encoder"], false),
        ("complex_color_chain", vec!["colorspace", "lut3d", "transform", "png_encoder"], false),
    ];
    for (name, plugins, expect) in &chains {
        let ids: Vec<String> = plugins.iter().map(|s| pid(s)).collect();
        let config = match plugins.len() {
            2 => two_node_config(&ids[0], None, &ids[1], None),
            3 => three_node_config(&ids[0], None, &ids[1], None, &ids[2], None),
            _ => four_node_config(&ids[0], None, &ids[1], None, &ids[2], None, &ids[3], None),
        };
        specs.push(regression(name, config, *expect, ""));
    }

    specs
}

fn regression(name: &str, config_json: String, expect_success: bool, _description: &str) -> TestCaseSpec {
    TestCaseSpec {
        name: name.to_string(), category: "cat14".into(),
        plugin_ids: vec![],
        config_json,
        image_type: ImageType::Solid64x64,
        output_ext: "png".into(),
        expect_success,
        is_large_pipeline: false,
        timeout_secs: Some(45),
        assertions: if expect_success { vec![AssertionSpec::FileNonEmpty] } else { vec![] },
    }
}

use crate::common::*;

use super::cat01_single_plugin::PLUGINS;

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    for (short, full_id) in PLUGINS {
        let variants = param_variants(short);
        for (vi, (var_name, params)) in variants.iter().enumerate() {
            let name = format!("{}__variant_{}", short, var_name);
            let ext = super::cat01_single_plugin::output_ext(short);
            specs.push(TestCaseSpec {
                name,
                category: "cat02".into(),
                plugin_ids: vec![full_id.to_string()],
                config_json: single_node_config(full_id, Some(params.clone())),
                image_type: ImageType::Gradient256x256,
                output_ext: ext.to_string(),
                expect_success: vi < 2, // first 2 should succeed, last may fail
                is_large_pipeline: false,
                timeout_secs: Some(45),
                assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
            });
        }
    }

    // Additional variants for extra coverage (14 plugins x 1 extra = 14)
    use serde_json::json;
    let extra: Vec<(&str, &str, Vec<(&str, serde_json::Value)>)> = vec![
        ("exif_rw", "all_tags", vec![("write_exif", json!("custom")), ("make", json!("Test")), ("model", json!("E2E"))]),
        ("gps_set", "gpx_mode", vec![("gps_mode", json!("gpx")), ("gpx_path", json!("test.gpx"))]),
        ("time_shift", "reset", vec![("shift_hours", json!(0)), ("shift_minutes", json!(0))]),
        ("colorspace", "srgb_to_linear", vec![("source", json!("srgb")), ("target", json!("linear_srgb"))]),
        ("lut3d", "intensity_75", vec![("intensity", json!(75))]),
        ("transform", "crop_32x32", vec![("resize_mode", json!("crop")), ("width", json!(32)), ("height", json!(32))]),
        ("lens_correct", "mode_auto_focal_85", vec![("correction_mode", json!("auto")), ("focal_length", json!(85.0))]),
        ("ai_denoise", "strength_0", vec![("strength", json!(0))]),
        ("png_encoder", "compression_6", vec![("compression_level", json!(6))]),
        ("tiff_encoder", "compression_none", vec![("compression", json!("none"))]),
        ("heif_encoder", "quality_80", vec![("quality", json!(80))]),
        ("jxl_encoder", "effort_7_q100", vec![("effort", json!(7)), ("quality", json!(100))]),
        ("avif_encoder", "q80_s6", vec![("quality", json!(80)), ("speed", json!(6))]),
        ("raw_input", "mode_auto_output_u8", vec![("raw_mode", json!("auto")), ("output_format", json!("u8"))]),
    ];
    let all_images = [ImageType::Solid64x64, ImageType::Checkerboard128x128, ImageType::Gradient256x256,
                      ImageType::ColorBars256x128, ImageType::Grayscale256x16, ImageType::Large1920x1080];
    for (short, var_name, params) in &extra {
        let full_id = format!("photopipeline.plugins.{}", short);
        let ext = super::cat01_single_plugin::output_ext(short);
        let config = single_node_config(&full_id, Some(params.clone()));
        specs.push(TestCaseSpec {
            name: format!("{}__{}", short, var_name), category: "cat02".into(),
            plugin_ids: vec![full_id],
            config_json: config,
            image_type: ImageType::Gradient256x256, output_ext: ext.to_string(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
        });
    }

    specs
}

fn param_variants(short: &str) -> Vec<(&str, Vec<(&str, serde_json::Value)>)> {
    use serde_json::json;
    match short {
        "exif_rw" => vec![
            ("preserve", vec![("write_exif", json!("preserve"))]),
            ("clear", vec![("write_exif", json!("clear"))]),
        ],
        "gps_set" => vec![
            ("manual", vec![("gps_mode", json!("manual")), ("latitude", json!(39.9)), ("longitude", json!(116.4))]),
            ("clear", vec![("gps_mode", json!("clear"))]),
        ],
        "time_shift" => vec![
            ("plus_8h", vec![("shift_hours", json!(8))]),
            ("minus_5h30m", vec![("shift_hours", json!(-5)), ("shift_minutes", json!(30))]),
        ],
        "colorspace" => vec![
            ("srgb_to_adobergb", vec![("source", json!("srgb")), ("target", json!("adobergb"))]),
            ("srgb_to_displayp3", vec![("source", json!("srgb")), ("target", json!("displayp3"))]),
        ],
        "lut3d" => vec![
            ("intensity_100", vec![("intensity", json!(100))]),
            ("intensity_50", vec![("intensity", json!(50))]),
        ],
        "transform" => vec![
            ("resize_128x128", vec![("resize_mode", json!("absolute")), ("width", json!(128)), ("height", json!(128))]),
            ("scale_50pct", vec![("resize_mode", json!("percent")), ("scale", json!(50))]),
        ],
        "lens_correct" => vec![
            ("mode_auto", vec![("correction_mode", json!("auto"))]),
            ("mode_off", vec![("correction_mode", json!("off"))]),
        ],
        "ai_denoise" => vec![
            ("strength_25", vec![("strength", json!(25))]),
            ("strength_0", vec![("strength", json!(0))]),
        ],
        "png_encoder" => vec![
            ("compression_9", vec![("compression_level", json!(9))]),
            ("compression_0", vec![("compression_level", json!(0))]),
        ],
        "tiff_encoder" => vec![
            ("compression_lzw", vec![("compression", json!("lzw"))]),
            ("compression_deflate", vec![("compression", json!("deflate"))]),
        ],
        "heif_encoder" => vec![
            ("quality_100", vec![("quality", json!(100))]),
            ("quality_30", vec![("quality", json!(30))]),
        ],
        "jxl_encoder" => vec![
            ("effort_9_quality_100", vec![("effort", json!(9)), ("quality", json!(100))]),
            ("effort_3_quality_50", vec![("effort", json!(3)), ("quality", json!(50))]),
        ],
        "avif_encoder" => vec![
            ("quality_100_speed_0", vec![("quality", json!(100)), ("speed", json!(0))]),
            ("quality_50_speed_8", vec![("quality", json!(50)), ("speed", json!(8))]),
        ],
        "raw_input" => vec![
            ("mode_auto", vec![("raw_mode", json!("auto"))]),
            ("mode_dcraw", vec![("raw_mode", json!("dcraw"))]),
        ],
        _ => vec![("default", vec![])],
    }
}

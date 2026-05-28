use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }
use serde_json::json;

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // For each plugin: min-valid, max-valid, below-min(fail), at-default
    let boundaries: Vec<(&str, Vec<(&str, Vec<(&str, serde_json::Value)>, bool)>)> = vec![
        ("exif_rw", vec![
            ("preserve", vec![("write_exif", json!("preserve"))], true),
            ("custom_full", vec![("write_exif", json!("custom")), ("make", json!("Sony")), ("model", json!("A7R5"))], true),
            ("invalid_mode", vec![("write_exif", json!("invalid_enum"))], false),
            ("default", vec![], true),
        ]),
        ("gps_set", vec![
            ("lat_min90_lon_min180", vec![("latitude", json!(-90.0)), ("longitude", json!(-180.0))], true),
            ("lat_90_lon_180", vec![("latitude", json!(90.0)), ("longitude", json!(180.0))], true),
            ("lat_91_invalid", vec![("latitude", json!(91.0))], false),
            ("default_zero", vec![("gps_mode", json!("manual")), ("latitude", json!(0.0)), ("longitude", json!(0.0))], true),
        ]),
        ("time_shift", vec![
            ("minus_23h", vec![("shift_hours", json!(-23))], true),
            ("plus_23h", vec![("shift_hours", json!(23))], true),
            ("h24_out_of_range", vec![("shift_hours", json!(24))], false),
            ("default_0", vec![("shift_hours", json!(0))], true),
        ]),
        ("colorspace", vec![
            ("srgb_to_srgb", vec![("source", json!("srgb")), ("target", json!("srgb"))], true),
            ("srgb_to_prophoto", vec![("source", json!("srgb")), ("target", json!("prophoto"))], true),
            ("invalid_target", vec![("source", json!("srgb")), ("target", json!("invalid_space"))], false),
            ("default_srgb", vec![], true),
        ]),
        ("lut3d", vec![
            ("intensity_0", vec![("intensity", json!(0))], true),
            ("intensity_100", vec![("intensity", json!(100))], true),
            ("intensity_101", vec![("intensity", json!(101))], false),
            ("default_100", vec![("intensity", json!(100))], true),
        ]),
        ("transform", vec![
            ("scale_1pct", vec![("resize_mode", json!("percent")), ("scale", json!(1))], true),
            ("scale_400pct", vec![("resize_mode", json!("percent")), ("scale", json!(400))], true),
            ("scale_401pct", vec![("resize_mode", json!("percent")), ("scale", json!(401))], false),
            ("default_100", vec![("resize_mode", json!("percent")), ("scale", json!(100))], true),
        ]),
        ("lens_correct", vec![
            ("mode_off", vec![("correction_mode", json!("off"))], true),
            ("mode_auto", vec![("correction_mode", json!("auto")), ("focal_length", json!(50.0))], true),
            ("focal_neg1", vec![("correction_mode", json!("manual")), ("focal_length", json!(-1.0))], false),
            ("default_off", vec![], true),
        ]),
        ("ai_denoise", vec![
            ("strength_0", vec![("strength", json!(0))], true),
            ("strength_100", vec![("strength", json!(100))], true),
            ("strength_101", vec![("strength", json!(101))], false),
            ("default_0", vec![("strength", json!(0))], true),
        ]),
        ("png_encoder", vec![
            ("compression_0", vec![("compression_level", json!(0))], true),
            ("compression_9", vec![("compression_level", json!(9))], true),
            ("compression_10", vec![("compression_level", json!(10))], false),
            ("default_6", vec![("compression_level", json!(6))], true),
        ]),
        ("tiff_encoder", vec![
            ("compression_none", vec![("compression", json!("none"))], true),
            ("compression_packbits", vec![("compression", json!("packbits"))], true),
            ("compression_invalid", vec![("compression", json!("bzip2"))], false),
            ("default_none", vec![], true),
        ]),
        ("heif_encoder", vec![
            ("quality_0", vec![("quality", json!(0))], true),
            ("quality_100", vec![("quality", json!(100))], true),
            ("quality_101", vec![("quality", json!(101))], false),
            ("default_80", vec![("quality", json!(80))], true),
        ]),
        ("jxl_encoder", vec![
            ("effort_1_q_minus1", vec![("effort", json!(1)), ("quality", json!(-1))], true),
            ("effort_9_q_100", vec![("effort", json!(9)), ("quality", json!(100))], true),
            ("effort_10", vec![("effort", json!(10))], false),
            ("default_7", vec![("effort", json!(7)), ("quality", json!(100))], true),
        ]),
        ("avif_encoder", vec![
            ("q0_s10", vec![("quality", json!(0)), ("speed", json!(10))], true),
            ("q100_s0", vec![("quality", json!(100)), ("speed", json!(0))], true),
            ("q101", vec![("quality", json!(101))], false),
            ("default_q80_s6", vec![("quality", json!(80)), ("speed", json!(6))], true),
        ]),
        ("raw_input", vec![
            ("mode_auto", vec![("raw_mode", json!("auto"))], true),
            ("mode_dcraw", vec![("raw_mode", json!("dcraw"))], false),
            ("invalid_mode", vec![("raw_mode", json!("invalid_decoder"))], false),
            ("default", vec![], true),
        ]),
    ];

    for (short, variants) in &boundaries {
        for (var_name, params, expect_ok) in variants {
            let name = format!("{}__{}", short, var_name);
            specs.push(TestCaseSpec {
                name, category: "cat08".into(), plugin_ids: vec![pid(short)],
                config_json: single_node_config(&pid(short), Some(params.clone())),
                image_type: ImageType::Gradient256x256, output_ext: "png".into(),
                expect_success: *expect_ok, is_large_pipeline: false, timeout_secs: Some(30),
                assertions: if *expect_ok { vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)] } else { vec![] },
            });
        }
    }

    specs
}

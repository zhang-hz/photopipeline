use crate::common::*;

/// All 14 plugin IDs
pub const PLUGINS: &[(&str, &str)] = &[
    ("exif_rw", "photopipeline.plugins.exif_rw"),
    ("gps_set", "photopipeline.plugins.gps_set"),
    ("time_shift", "photopipeline.plugins.time_shift"),
    ("colorspace", "photopipeline.plugins.colorspace"),
    ("lut3d", "photopipeline.plugins.lut3d"),
    ("transform", "photopipeline.plugins.transform"),
    ("lens_correct", "photopipeline.plugins.lens_correct"),
    ("ai_denoise", "photopipeline.plugins.ai_denoise"),
    ("png_encoder", "photopipeline.plugins.png_encoder"),
    ("tiff_encoder", "photopipeline.plugins.tiff_encoder"),
    ("heif_encoder", "photopipeline.plugins.heif_encoder"),
    ("jxl_encoder", "photopipeline.plugins.jxl_encoder"),
    ("avif_encoder", "photopipeline.plugins.avif_encoder"),
    ("raw_input", "photopipeline.plugins.raw_input"),
];

const IMAGES: &[ImageType] = &[
    ImageType::Solid64x64,
    ImageType::Checkerboard128x128,
    ImageType::Gradient256x256,
    ImageType::ColorBars256x128,
    ImageType::Grayscale256x16,
    ImageType::Large1920x1080,
];

/// Output format for each plugin category
pub fn output_ext(short: &str) -> &str {
    match short {
        "png_encoder" | "raw_input" | "ai_denoise" => "png",
        "tiff_encoder" => "tiff",
        "heif_encoder" => "heif",
        "jxl_encoder" => "jxl",
        "avif_encoder" => "avif",
        _ => "png", // metadata/pixel plugins produce pixel buffer → PNG output
    }
}

/// Build a single-node config with default params for a given plugin.
/// For ai_denoise, uses strength=0 (pass-through, no ONNX needed).
fn default_config(short: &str, full_id: &str) -> String {
    if *short == *"ai_denoise" {
        single_node_config(full_id, Some(vec![("strength", serde_json::json!(0))]))
    } else {
        single_node_config(full_id, None)
    }
}

/// Basic output assertions for each output format
fn format_assertions(short: &str) -> Vec<AssertionSpec> {
    let mut base = vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)];
    match short {
        "png_encoder" | "raw_input" | "ai_denoise" | "colorspace" | "transform"
        | "lut3d" | "lens_correct" | "exif_rw" | "gps_set" | "time_shift" => {
            base.push(AssertionSpec::PngSignatureValid)
        }
        "tiff_encoder" => base.push(AssertionSpec::TiffMagicValid),
        _ => {}
    }
    base
}

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    for (short, full_id) in PLUGINS {
        for img in IMAGES {
            let name = format!("{}__{}", short, img.name());
            let ext = output_ext(short);
            specs.push(TestCaseSpec {
                name,
                category: "cat01".into(),
                plugin_ids: vec![full_id.to_string()],
                config_json: default_config(short, full_id),
                image_type: *img,
                output_ext: ext.to_string(),
                expect_success: true,
                is_large_pipeline: *img == ImageType::Large1920x1080,
                timeout_secs: if *img == ImageType::Large1920x1080 { Some(120) } else { None },
                assertions: format_assertions(short),
            });
        }
    }

    specs
}

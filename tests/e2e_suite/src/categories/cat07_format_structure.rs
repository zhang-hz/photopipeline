use crate::common::*;

fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // PNG structure tests (8)
    for (name, assertions) in &[
        ("png_rgb8_sig", vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid, AssertionSpec::FormatIsValidPng]),
        ("png_gray8", vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid]),
        ("png_rgba8", vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid]),
        ("png_compression_9", vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid, AssertionSpec::FileSizeGt(200)]),
    ] {
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("png_encoder")],
            config_json: single_node_config(&pid("png_encoder"), None),
            image_type: ImageType::ColorBars256x128, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: assertions.clone(),
        });
    }
    // Additional PNG variants
    for (name, img) in &[("png_solid", ImageType::Solid64x64), ("png_checker", ImageType::Checkerboard128x128),
                          ("png_gradient", ImageType::Gradient256x256), ("png_large", ImageType::Large1920x1080)] {
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("png_encoder")],
            config_json: single_node_config(&pid("png_encoder"), None),
            image_type: *img, output_ext: "png".into(), expect_success: true,
            is_large_pipeline: *img == ImageType::Large1920x1080, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
        });
    }

    // TIFF structure tests (8)
    for (name, assertions) in &[
        ("tiff_lzw", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_deflate", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_solid", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid, AssertionSpec::FileSizeGt(500)]),
        ("tiff_checker", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_gradient", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_color_bars", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_grayscale", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
        ("tiff_large", vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid]),
    ] {
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("tiff_encoder")],
            config_json: single_node_config(&pid("tiff_encoder"), None),
            image_type: ImageType::Gradient256x256, output_ext: "tiff".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: assertions.clone(),
        });
    }

    // HEIF tests (8) — may need libheif
    for (name, params) in &[
        ("heif_q100", vec![("quality", serde_json::json!(100))]),
        ("heif_q50", vec![("quality", serde_json::json!(50))]),
        ("heif_q30", vec![("quality", serde_json::json!(30))]),
        ("heif_solid", vec![]),
        ("heif_checker", vec![]),
        ("heif_gradient", vec![]),
        ("heif_color_bars", vec![]),
        ("heif_large", vec![]),
    ] {
        let params = if params.is_empty() { None } else { Some(params.clone()) };
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("heif_encoder")],
            config_json: single_node_config(&pid("heif_encoder"), params),
            image_type: ImageType::ColorBars256x128, output_ext: "heif".into(),
            expect_success: true, is_large_pipeline: *name == "heif_large", timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // JXL tests (8)
    for (name, params) in &[
        ("jxl_q100_e9", vec![("quality", serde_json::json!(100)), ("effort", serde_json::json!(9))]),
        ("jxl_q50_e3", vec![("quality", serde_json::json!(50)), ("effort", serde_json::json!(3))]),
        ("jxl_lossless", vec![("lossless", serde_json::json!(true))]),
        ("jxl_solid", vec![]),
        ("jxl_checker", vec![]),
        ("jxl_gradient", vec![]),
        ("jxl_color_bars", vec![]),
        ("jxl_large", vec![]),
    ] {
        let params = if params.is_empty() { None } else { Some(params.clone()) };
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("jxl_encoder")],
            config_json: single_node_config(&pid("jxl_encoder"), params),
            image_type: ImageType::Gradient256x256, output_ext: "jxl".into(),
            expect_success: true, is_large_pipeline: *name == "jxl_large", timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // AVIF tests (8)
    for (name, params) in &[
        ("avif_q100_s0", vec![("quality", serde_json::json!(100)), ("speed", serde_json::json!(0))]),
        ("avif_q50_s8", vec![("quality", serde_json::json!(50)), ("speed", serde_json::json!(8))]),
        ("avif_solid", vec![]),
        ("avif_checker", vec![]),
        ("avif_gradient", vec![]),
        ("avif_color_bars", vec![]),
        ("avif_grayscale", vec![]),
        ("avif_large", vec![]),
    ] {
        let params = if params.is_empty() { None } else { Some(params.clone()) };
        specs.push(TestCaseSpec {
            name: name.to_string(), category: "cat07".into(),
            plugin_ids: vec![pid("avif_encoder")],
            config_json: single_node_config(&pid("avif_encoder"), params),
            image_type: ImageType::ColorBars256x128, output_ext: "avif".into(),
            expect_success: true, is_large_pipeline: *name == "avif_large", timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    specs
}

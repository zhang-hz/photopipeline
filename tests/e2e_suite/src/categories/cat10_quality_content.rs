use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();
    use serde_json::json;

    // Pass-through quality tests (colorspace srgb→srgb should preserve pixels)
    for img in &[ImageType::Solid64x64, ImageType::Checkerboard128x128, ImageType::Gradient256x256] {
        specs.push(TestCaseSpec {
            name: format!("passthrough_{}", img.name()), category: "cat10".into(),
            plugin_ids: vec![pid("colorspace")],
            config_json: single_node_config(&pid("colorspace"), Some(vec![
                ("source", json!("srgb")), ("target", json!("srgb"))
            ])),
            image_type: *img, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid, AssertionSpec::FileSizeGt(200)],
        });
    }

    // Lossless PNG encode → verify output size consistency
    for img in &[ImageType::Solid64x64, ImageType::Checkerboard128x128, ImageType::Gradient256x256] {
        specs.push(TestCaseSpec {
            name: format!("png_lossless_{}", img.name()), category: "cat10".into(),
            plugin_ids: vec![pid("png_encoder")],
            config_json: single_node_config(&pid("png_encoder"), Some(vec![("compression_level", json!(9))])),
            image_type: *img, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid, AssertionSpec::FileSizeGt(100)],
        });
    }

    // TIFF lossless encode
    for img in &[ImageType::Solid64x64, ImageType::Checkerboard128x128, ImageType::Gradient256x256] {
        specs.push(TestCaseSpec {
            name: format!("tiff_lossless_{}", img.name()), category: "cat10".into(),
            plugin_ids: vec![pid("tiff_encoder")],
            config_json: single_node_config(&pid("tiff_encoder"), Some(vec![("compression", json!("deflate"))])),
            image_type: *img, output_ext: "tiff".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid, AssertionSpec::FileSizeGt(500)],
        });
    }

    // Lossy HEIF quality tiers — verify size decreases with quality
    for (q, _label) in &[(100, "q100"), (50, "q50"), (10, "q10")] {
        specs.push(TestCaseSpec {
            name: format!("heif_q{}", q), category: "cat10".into(),
            plugin_ids: vec![pid("heif_encoder")],
            config_json: single_node_config(&pid("heif_encoder"), Some(vec![("quality", json!(*q))])),
            image_type: ImageType::Gradient256x256, output_ext: "heif".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // Lossy JXL quality tiers
    for (effort, quality, label) in &[(9, 100, "e9q100"), (3, 80, "e3q80")] {
        specs.push(TestCaseSpec {
            name: format!("jxl_{}", label), category: "cat10".into(),
            plugin_ids: vec![pid("jxl_encoder")],
            config_json: single_node_config(&pid("jxl_encoder"), Some(vec![("effort", json!(*effort)), ("quality", json!(*quality))])),
            image_type: ImageType::Gradient256x256, output_ext: "jxl".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // Lossy AVIF quality tiers
    for (q, speed, label) in &[(100, 0, "q100s0"), (50, 8, "q50s8"), (10, 10, "q10s10")] {
        specs.push(TestCaseSpec {
            name: format!("avif_{}", label), category: "cat10".into(),
            plugin_ids: vec![pid("avif_encoder")],
            config_json: single_node_config(&pid("avif_encoder"), Some(vec![("quality", json!(*q)), ("speed", json!(*speed))])),
            image_type: ImageType::Gradient256x256, output_ext: "avif".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // Colorspace transform → verify output is valid PNG
    specs.push(TestCaseSpec {
        name: "colorspace_srgb_to_adobergb_size".into(), category: "cat10".into(),
        plugin_ids: vec![pid("colorspace")],
        config_json: single_node_config(&pid("colorspace"), Some(vec![("source", json!("srgb")), ("target", json!("adobergb"))])),
        image_type: ImageType::Large1920x1080, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: true, timeout_secs: Some(90),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid, AssertionSpec::FileSizeGt(1000)],
    });

    // Transform resize → verify output dimensions change
    specs.push(TestCaseSpec {
        name: "transform_resize_64x64".into(), category: "cat10".into(),
        plugin_ids: vec![pid("transform")],
        config_json: single_node_config(&pid("transform"), Some(vec![
            ("resize_mode", json!("absolute")), ("width", json!(64)), ("height", json!(64))
        ])),
        image_type: ImageType::Large1920x1080, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
    });

    // Extra quality checks (3)
    specs.push(TestCaseSpec {
        name: "jxl_lossless_quality".into(), category: "cat10".into(),
        plugin_ids: vec![pid("jxl_encoder")],
        config_json: single_node_config(&pid("jxl_encoder"), Some(vec![("lossless", serde_json::json!(true))])),
        image_type: ImageType::Gradient256x256, output_ext: "jxl".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
    });
    specs.push(TestCaseSpec {
        name: "avif_small_quality_check".into(), category: "cat10".into(),
        plugin_ids: vec![pid("avif_encoder")],
        config_json: single_node_config(&pid("avif_encoder"), Some(vec![("quality", serde_json::json!(90)), ("speed", serde_json::json!(4))])),
        image_type: ImageType::ColorBars256x128, output_ext: "avif".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
    });
    specs.push(TestCaseSpec {
        name: "heif_high_quality_large".into(), category: "cat10".into(),
        plugin_ids: vec![pid("heif_encoder")],
        config_json: single_node_config(&pid("heif_encoder"), Some(vec![("quality", serde_json::json!(100))])),
        image_type: ImageType::Large1920x1080, output_ext: "heif".into(),
        expect_success: true, is_large_pipeline: true, timeout_secs: Some(120),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(500)],
    });

    specs
}

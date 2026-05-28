use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }
use serde_json::json;

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // Encode → verify output for each format (5)
    for (plugin, ext) in &[("png_encoder", "png"), ("tiff_encoder", "tiff"),
                            ("heif_encoder", "heif"), ("jxl_encoder", "jxl"), ("avif_encoder", "avif")] {
        specs.push(TestCaseSpec {
            name: format!("roundtrip_encode_{}", plugin), category: "cat13".into(),
            plugin_ids: vec![pid(plugin)],
            config_json: single_node_config(&pid(plugin), None),
            image_type: ImageType::Gradient256x256, output_ext: ext.to_string(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    // Round-trip with transform → resize then encode (3)
    for (plugin, ext) in &[("png_encoder", "png"), ("jxl_encoder", "jxl"), ("avif_encoder", "avif")] {
        specs.push(TestCaseSpec {
            name: format!("roundtrip_resize_{}", plugin), category: "cat13".into(),
            plugin_ids: vec![pid("transform"), pid(plugin)],
            config_json: two_node_config(
                &pid("transform"), Some(vec![("resize_mode", json!("absolute")), ("width", json!(100)), ("height", json!(100))]),
                &pid(plugin), None,
            ),
            image_type: ImageType::Gradient256x256, output_ext: ext.to_string(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
        });
    }

    // Round-trip with format conversion → colorspace then encode (2)
    specs.push(TestCaseSpec {
        name: "roundtrip_colorspace_heif".into(), category: "cat13".into(),
        plugin_ids: vec![pid("colorspace"), pid("heif_encoder")],
        config_json: two_node_config(
            &pid("colorspace"), Some(vec![("source", json!("srgb")), ("target", json!("displayp3"))]),
            &pid("heif_encoder"), None,
        ),
        image_type: ImageType::Gradient256x256, output_ext: "heif".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
    });
    specs.push(TestCaseSpec {
        name: "roundtrip_colorspace_tiff".into(), category: "cat13".into(),
        plugin_ids: vec![pid("colorspace"), pid("tiff_encoder")],
        config_json: two_node_config(
            &pid("colorspace"), Some(vec![("source", json!("srgb")), ("target", json!("adobergb"))]),
            &pid("tiff_encoder"), None,
        ),
        image_type: ImageType::ColorBars256x128, output_ext: "tiff".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid, AssertionSpec::FileSizeGt(500)],
    });

    // Metadata + encode roundtrip (2)
    specs.push(TestCaseSpec {
        name: "roundtrip_meta_png".into(), category: "cat13".into(),
        plugin_ids: vec![pid("exif_rw"), pid("png_encoder")],
        config_json: two_node_config(
            &pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]),
            &pid("png_encoder"), None,
        ),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
    });
    specs.push(TestCaseSpec {
        name: "roundtrip_meta_jxl".into(), category: "cat13".into(),
        plugin_ids: vec![pid("exif_rw"), pid("jxl_encoder")],
        config_json: two_node_config(
            &pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]),
            &pid("jxl_encoder"), None,
        ),
        image_type: ImageType::Gradient256x256, output_ext: "jxl".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Additional roundtrip (3)
    specs.push(TestCaseSpec {
        name: "roundtrip_raw_to_png_encode".into(), category: "cat13".into(),
        plugin_ids: vec![pid("raw_input"), pid("png_encoder")],
        config_json: two_node_config(&pid("raw_input"), None, &pid("png_encoder"), None),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
    });
    specs.push(TestCaseSpec {
        name: "roundtrip_lut3d_to_jxl".into(), category: "cat13".into(),
        plugin_ids: vec![pid("lut3d"), pid("jxl_encoder")],
        config_json: two_node_config(&pid("lut3d"), Some(vec![("intensity", serde_json::json!(80))]), &pid("jxl_encoder"), None),
        image_type: ImageType::ColorBars256x128, output_ext: "jxl".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "roundtrip_denoise_to_png".into(), category: "cat13".into(),
        plugin_ids: vec![pid("ai_denoise"), pid("png_encoder")],
        config_json: two_node_config(&pid("ai_denoise"), Some(vec![("strength", serde_json::json!(0))]), &pid("png_encoder"), None),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
    });

    specs
}

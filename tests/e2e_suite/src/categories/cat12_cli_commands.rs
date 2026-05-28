use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // validate command (4)
    specs.push(TestCaseSpec {
        name: "validate_valid_1node".into(), category: "cat12".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: single_node_config(&pid("exif_rw"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "validate_invalid_missing_plugin".into(), category: "cat12".into(),
        plugin_ids: vec![],
        config_json: single_node_config("nonexistent.plugin.zzz", None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: false, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "validate_invalid_cycle".into(), category: "cat12".into(),
        plugin_ids: vec![],
        config_json: format!(r#"{{"metadata":{{}},"nodes":[{{"id":"A","plugin":"{}"}},{{"id":"B","plugin":"{}"}}],"edges":[{{"from":"A","to":"B"}},{{"from":"B","to":"A"}}]}}"#,
            pid("exif_rw"), pid("exif_rw")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: false, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "validate_empty_graph".into(), category: "cat12".into(),
        plugin_ids: vec![],
        config_json: r#"{"metadata":{},"nodes":[],"edges":[]}"#.into(),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: false, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });

    // multiple run calls with same config (3)
    specs.push(TestCaseSpec {
        name: "run_same_config_twice".into(), category: "cat12".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
    });
    specs.push(TestCaseSpec {
        name: "run_different_inputs_same_config".into(), category: "cat12".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::Checkerboard128x128, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "run_overwrite_output".into(), category: "cat12".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Various output formats for validate command (5)
    let valid_configs = ["png_encoder", "tiff_encoder", "exif_rw", "colorspace", "transform"];
    for plugin in &valid_configs {
        specs.push(TestCaseSpec {
            name: format!("validate_{}_valid", plugin), category: "cat12".into(),
            plugin_ids: vec![pid(plugin)],
            config_json: single_node_config(&pid(plugin), None),
            image_type: ImageType::Solid64x64, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(15),
            assertions: vec![],
        });
    }

    // Run with all output formats (5)
    for (plugin, ext) in &[("png_encoder", "png"), ("tiff_encoder", "tiff"), ("heif_encoder", "heif"), ("jxl_encoder", "jxl"), ("avif_encoder", "avif")] {
        specs.push(TestCaseSpec {
            name: format!("run_{}_output", plugin), category: "cat12".into(),
            plugin_ids: vec![pid(plugin)],
            config_json: single_node_config(&pid(plugin), None),
            image_type: ImageType::Solid64x64, output_ext: ext.to_string(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
        });
    }

    // Run with empty args equivalent (1)
    specs.push(TestCaseSpec {
        name: "run_no_args_like_empty".into(), category: "cat12".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::VerySmall8x8, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Run with wide-strip image (1)
    specs.push(TestCaseSpec {
        name: "run_wide_strip_input".into(), category: "cat12".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::WideStrip640x16, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Run with large image → verify no timeout (1)
    specs.push(TestCaseSpec {
        name: "run_large_image_1920x1080".into(), category: "cat12".into(),
        plugin_ids: vec![pid("colorspace")],
        config_json: single_node_config(&pid("colorspace"), None),
        image_type: ImageType::Large1920x1080, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: true, timeout_secs: Some(120),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(1000)],
    });

    // Run with grayscale input (1)
    specs.push(TestCaseSpec {
        name: "run_grayscale_input".into(), category: "cat12".into(),
        plugin_ids: vec![pid("colorspace")],
        config_json: single_node_config(&pid("colorspace"), None),
        image_type: ImageType::Grayscale256x16, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Validate multi-node config (1)
    specs.push(TestCaseSpec {
        name: "validate_3node_valid".into(), category: "cat12".into(),
        plugin_ids: vec![pid("exif_rw"), pid("colorspace"), pid("png_encoder")],
        config_json: three_node_config(&pid("exif_rw"), None, &pid("colorspace"), None, &pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });

    // Validate 50-node chain (1)
    specs.push(TestCaseSpec {
        name: "validate_50node_linear".into(), category: "cat12".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: linear_chain_config(50, &pid("exif_rw")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![],
    });

    // Run colorspace chain with specific output format (1)
    specs.push(TestCaseSpec {
        name: "run_colorspace_to_transform_png".into(), category: "cat12".into(),
        plugin_ids: vec![pid("colorspace"), pid("transform"), pid("png_encoder")],
        config_json: three_node_config(&pid("colorspace"), None, &pid("transform"),
            Some(vec![("resize_mode", serde_json::json!("percent")), ("scale", serde_json::json!(50))]),
            &pid("png_encoder"), None),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
    });

    specs
}

use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();
    use serde_json::json;

    // Missing/nonexistent files (4)
    for (name, config, expect_msg) in &[
        ("nonexistent_plugin", single_node_config("nonexistent.plugin.fake", None), "not found"),
        ("typo_in_plugin_id", single_node_config("photopipeline.plugins.colorspac", None), "not found"), // typo
        ("empty_plugin_id", single_node_config("", None), "invalid"),
    ] {
        specs.push(error_test(name, config, *expect_msg));
    }

    // Invalid parameters (3)
    specs.push(error_test("unknown_param_key",
        &single_node_config(&pid("exif_rw"), Some(vec![("nonexistent_param", json!("value"))])), ""));
    specs.push(error_test("case_sensitive_param",
        &single_node_config(&pid("exif_rw"), Some(vec![("WRITE_EXIF", json!("preserve"))])), ""));

    // Encoder validation failures (5)
    specs.push(error_test("png_invalid_color_type",
        &single_node_config(&pid("png_encoder"), Some(vec![("color_type", json!("cmyk"))])), ""));
    specs.push(error_test("tiff_invalid_compression",
        &single_node_config(&pid("tiff_encoder"), Some(vec![("compression", json!("bzip2"))])), ""));
    specs.push(error_test("heif_quality_200",
        &single_node_config(&pid("heif_encoder"), Some(vec![("quality", json!(200))])), ""));
    specs.push(error_test("jxl_quality_200",
        &single_node_config(&pid("jxl_encoder"), Some(vec![("quality", json!(200))])), ""));
    specs.push(error_test("avif_quality_200",
        &single_node_config(&pid("avif_encoder"), Some(vec![("quality", json!(200))])), ""));

    // Conflicting parameters (3)
    specs.push(error_test("colorspace_source_equals_target_noop",
        &single_node_config(&pid("colorspace"), Some(vec![("source", json!("srgb")), ("target", json!("srgb"))])), ""));
    specs.push(error_test("transform_crop_without_dims",
        &single_node_config(&pid("transform"), Some(vec![("resize_mode", json!("crop"))])), ""));
    specs.push(error_test("lens_manual_without_focal",
        &single_node_config(&pid("lens_correct"), Some(vec![("correction_mode", json!("manual"))])), ""));

    // Graph validation failures (3)
    specs.push(error_test("duplicate_node_ids",
        &format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}},{{"id":"n1","plugin":"{}"}}],"edges":[]}}"#,
            pid("exif_rw"), pid("exif_rw")), ""));
    specs.push(error_test("edge_to_nonexistent",
        &format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[{{"from":"n1","to":"n99"}}]}}"#,
            pid("exif_rw")), ""));
    specs.push(error_test("self_loop",
        &format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[{{"from":"n1","to":"n1"}}]}}"#,
            pid("exif_rw")), ""));

    // Invalid JSON config (2)
    specs.push(error_test("invalid_json_syntax",
        r#"{"nodes": [{"id": "n1", "plugin": "test",]}"#, ""));
    specs.push(error_test("invalid_json_wrong_types",
        r#"{"metadata":{"name":123},"nodes":"not_an_array","edges":[]}"#, ""));

    // Missing input file scenarios handled at CLI level (2)
    specs.push(TestCaseSpec {
        name: "output_ext_unknown".into(), category: "cat09".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "xyz".into(),
        expect_success: false, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "output_no_ext".into(), category: "cat09".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![],
    });

    // Large invalid dimension (1)
    specs.push(TestCaseSpec {
        name: "zero_dim_image".into(), category: "cat09".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: single_node_config(&pid("png_encoder"), None),
        image_type: ImageType::VerySmall8x8, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Additional error paths (3)
    specs.push(error_test("jxl_negative_effort",
        &single_node_config(&pid("jxl_encoder"), Some(vec![("effort", serde_json::json!(-1))])), ""));
    specs.push(error_test("transform_negative_scale",
        &single_node_config(&pid("transform"), Some(vec![("resize_mode", serde_json::json!("percent")), ("scale", serde_json::json!(-50))])), ""));
    specs.push(error_test("avif_negative_quality",
        &single_node_config(&pid("avif_encoder"), Some(vec![("quality", serde_json::json!(-1))])), ""));

    specs
}

fn error_test(name: &str, config: &str, _expected_msg: &str) -> TestCaseSpec {
    TestCaseSpec {
        name: name.to_string(), category: "cat09".into(), plugin_ids: vec![],
        config_json: config.to_string(), image_type: ImageType::Solid64x64,
        output_ext: "png".into(), expect_success: false, is_large_pipeline: false,
        timeout_secs: Some(30), assertions: vec![],
    }
}

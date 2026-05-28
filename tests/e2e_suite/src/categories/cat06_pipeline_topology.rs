use crate::common::*;

fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();
    let plugin = pid("exif_rw");

    // Image variety with fixed pipeline (6)
    for img in &[ImageType::Solid64x64, ImageType::Checkerboard128x128, ImageType::Gradient256x256,
                  ImageType::ColorBars256x128, ImageType::Grayscale256x16, ImageType::Large1920x1080] {
        specs.push(TestCaseSpec {
            name: format!("image_{}", img.name()), category: "cat06".into(),
            plugin_ids: vec![plugin.clone()],
            config_json: single_node_config(&plugin, None), image_type: *img,
            output_ext: "png".into(), expect_success: true,
            is_large_pipeline: *img == ImageType::Large1920x1080,
            timeout_secs: Some(60), assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
        });
    }

    // Diamond topology (1)
    specs.push(TestCaseSpec {
        name: "diamond".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("colorspace"), pid("transform"), pid("png_encoder")],
        config_json: diamond_config(&pid("exif_rw"), &pid("colorspace"), &pid("transform"), &pid("png_encoder")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(90),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
    });

    // Disabled node scenarios (4)
    for pos in 0..3 {
        specs.push(TestCaseSpec {
            name: format!("disabled_pos{}", pos), category: "cat06".into(),
            plugin_ids: vec![plugin.clone()],
            config_json: disabled_node_config(&plugin, pos),
            image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
            is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty],
        });
    }

    // Large pipelines (2)
    for n in &[50usize, 100] {
        specs.push(TestCaseSpec {
            name: format!("linear_{}_nodes", n), category: "cat06".into(),
            plugin_ids: vec![plugin.clone()],
            config_json: linear_chain_config(*n, &plugin),
            image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
            is_large_pipeline: true, timeout_secs: Some(180),
            assertions: vec![AssertionSpec::FileNonEmpty],
        });
    }

    // Invalid configs (3) — expect failure
    specs.push(TestCaseSpec {
        name: "empty_graph".into(), category: "cat06".into(), plugin_ids: vec![],
        config_json: r#"{"metadata":{"name":"empty"},"nodes":[],"edges":[]}"#.into(),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: false,
        is_large_pipeline: false, timeout_secs: Some(30), assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "cycle_detected".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("colorspace")],
        config_json: format!(r#"{{"metadata":{{}},"nodes":[{{"id":"A","plugin":"{}","enabled":true}},{{"id":"B","plugin":"{}","enabled":true}}],"edges":[{{"from":"A","to":"B"}},{{"from":"B","to":"A"}}]}}"#,
            pid("exif_rw"), pid("colorspace")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: false,
        is_large_pipeline: false, timeout_secs: Some(30), assertions: vec![],
    });

    // Fan-out: single source → 3 encoders (1)
    specs.push(TestCaseSpec {
        name: "fan_out_3".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("png_encoder"), pid("tiff_encoder"), pid("jxl_encoder")],
        config_json: build_json(
            vec![
                serde_json::json!({"id":"src","plugin":pid("exif_rw"),"enabled":true}),
                serde_json::json!({"id":"a","plugin":pid("png_encoder"),"enabled":true}),
                serde_json::json!({"id":"b","plugin":pid("tiff_encoder"),"enabled":true}),
                serde_json::json!({"id":"c","plugin":pid("jxl_encoder"),"enabled":true}),
            ],
            vec![
                serde_json::json!({"from":"src","to":"a"}),
                serde_json::json!({"from":"src","to":"b"}),
                serde_json::json!({"from":"src","to":"c"}),
            ],
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
    });

    // Fan-in: 2 metadata processors → single encoder (1)
    specs.push(TestCaseSpec {
        name: "fan_in_2".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("gps_set"), pid("png_encoder")],
        config_json: build_json(
            vec![
                serde_json::json!({"id":"a","plugin":pid("exif_rw"),"enabled":true}),
                serde_json::json!({"id":"b","plugin":pid("gps_set"),"enabled":true}),
                serde_json::json!({"id":"out","plugin":pid("png_encoder"),"enabled":true}),
            ],
            vec![
                serde_json::json!({"from":"a","to":"out"}),
                serde_json::json!({"from":"b","to":"out"}),
            ],
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Disabled first/last node (2)
    specs.push(TestCaseSpec {
        name: "disabled_first_node".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: disabled_node_config(&pid("exif_rw"), 0),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "disabled_last_node".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: disabled_node_config(&pid("exif_rw"), 2),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Single node no edges (1)
    specs.push(TestCaseSpec {
        name: "single_node_no_edges".into(), category: "cat06".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: build_json(
            vec![serde_json::json!({"id":"n1","plugin":pid("png_encoder"),"enabled":true})],
            vec![],
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(50)],
    });

    // 20-node chain stress (1)
    specs.push(TestCaseSpec {
        name: "linear_20_nodes".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: linear_chain_config(20, &pid("exif_rw")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: true, timeout_secs: Some(120),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Diamond with different plugins (2)
    let diamond_pid = pid("png_encoder");
    specs.push(TestCaseSpec {
        name: "diamond_metadata".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("gps_set"), pid("time_shift"), diamond_pid.clone()],
        config_json: diamond_config(&pid("exif_rw"), &pid("gps_set"), &pid("time_shift"), &diamond_pid),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(90),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
    });

    // TOML-style config (via JSON with metadata) (2)
    specs.push(TestCaseSpec {
        name: "config_with_groups".into(), category: "cat06".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: format!(r#"{{"metadata":{{"name":"grouped"}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[],"groups":[{{"name":"high_iso","condition":"exif.iso > 800","params":{{}}}}]}}"#,
            pid("png_encoder")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "config_with_overrides".into(), category: "cat06".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[],"overrides":[{{"image":"test.png","params":{{}}}}]}}"#,
            pid("png_encoder")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Batch config (1)
    specs.push(TestCaseSpec {
        name: "config_with_batch".into(), category: "cat06".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[],"batch":{{"parallel":2,"resume":false}}}}"#,
            pid("png_encoder")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Edge case topologies (5)
    specs.push(TestCaseSpec {
        name: "disconnected_components".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("colorspace")],
        config_json: build_json(
            vec![
                serde_json::json!({"id":"a","plugin":pid("exif_rw"),"enabled":true}),
                serde_json::json!({"id":"b","plugin":pid("colorspace"),"enabled":true}),
            ],
            vec![], // no edges — disconnected
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "all_nodes_disabled".into(), category: "cat06".into(),
        plugin_ids: vec![pid("png_encoder")],
        config_json: build_json(
            vec![
                serde_json::json!({"id":"n1","plugin":pid("png_encoder"),"enabled":false}),
                serde_json::json!({"id":"n2","plugin":pid("png_encoder"),"enabled":false}),
            ],
            vec![serde_json::json!({"from":"n1","to":"n2"})],
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "fan_out_5_encoders".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw"), pid("png_encoder"), pid("tiff_encoder"), pid("jxl_encoder"), pid("avif_encoder"), pid("heif_encoder")],
        config_json: build_json(
            vec![
                serde_json::json!({"id":"src","plugin":pid("exif_rw"),"enabled":true}),
                serde_json::json!({"id":"a","plugin":pid("png_encoder"),"enabled":true}),
                serde_json::json!({"id":"b","plugin":pid("tiff_encoder"),"enabled":true}),
                serde_json::json!({"id":"c","plugin":pid("jxl_encoder"),"enabled":true}),
                serde_json::json!({"id":"d","plugin":pid("avif_encoder"),"enabled":true}),
                serde_json::json!({"id":"e","plugin":pid("heif_encoder"),"enabled":true}),
            ],
            vec![
                serde_json::json!({"from":"src","to":"a"}), serde_json::json!({"from":"src","to":"b"}),
                serde_json::json!({"from":"src","to":"c"}), serde_json::json!({"from":"src","to":"d"}),
                serde_json::json!({"from":"src","to":"e"}),
            ],
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(60),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "edge_missing_source".into(), category: "cat06".into(),
        plugin_ids: vec![],
        config_json: format!(r#"{{"metadata":{{}},"nodes":[{{"id":"n1","plugin":"{}"}}],"edges":[{{"from":"n99","to":"n1"}}]}}"#, pid("exif_rw")),
        image_type: ImageType::Solid64x64, output_ext: "png".into(), expect_success: false,
        is_large_pipeline: false, timeout_secs: Some(15),
        assertions: vec![],
    });
    specs.push(TestCaseSpec {
        name: "5node_linear_chain".into(), category: "cat06".into(),
        plugin_ids: vec![pid("exif_rw")],
        config_json: linear_chain_config(5, &pid("exif_rw")),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(), expect_success: true,
        is_large_pipeline: false, timeout_secs: Some(30),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    specs
}

use crate::common::*;
fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }
use serde_json::json;

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();
    let encoders = ["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder", "avif_encoder"];

    // EXIF read → encode → basic check (5)
    for e in &encoders {
        specs.push(TestCaseSpec {
            name: format!("exif_read__{}", e), category: "cat11".into(),
            plugin_ids: vec![pid("exif_rw"), pid(e)],
            config_json: two_node_config(&pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]), &pid(e), None),
            image_type: ImageType::Gradient256x256, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // GPS set → encode (5)
    for e in &encoders {
        specs.push(TestCaseSpec {
            name: format!("gps_set__{}", e), category: "cat11".into(),
            plugin_ids: vec![pid("gps_set"), pid(e)],
            config_json: two_node_config(&pid("gps_set"), Some(vec![
                ("gps_mode", json!("manual")), ("latitude", json!(39.9)), ("longitude", json!(116.4))
            ]), &pid(e), None),
            image_type: ImageType::Solid64x64, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // GPS + time_shift chain → encode (3)
    for e in &["png_encoder", "tiff_encoder", "jxl_encoder"] {
        specs.push(TestCaseSpec {
            name: format!("gps_time__{}", e), category: "cat11".into(),
            plugin_ids: vec![pid("gps_set"), pid("time_shift"), pid(e)],
            config_json: three_node_config(
                &pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(39.9)), ("longitude", json!(116.4))]),
                &pid("time_shift"), Some(vec![("shift_hours", json!(8))]),
                &pid(e), None,
            ),
            image_type: ImageType::Solid64x64, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(90),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // Full metadata chain (4)
    for e in &["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder"] {
        specs.push(TestCaseSpec {
            name: format!("full_meta__{}", e), category: "cat11".into(),
            plugin_ids: vec![pid("exif_rw"), pid("gps_set"), pid("time_shift"), pid(e)],
            config_json: four_node_config(
                &pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]),
                &pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(39.9)), ("longitude", json!(116.4))]),
                &pid("time_shift"), Some(vec![("shift_hours", json!(8))]),
                &pid(e), None,
            ),
            image_type: ImageType::Gradient256x256, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(120),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // Metadata clear (3)
    specs.push(TestCaseSpec {
        name: "gps_clear".into(), category: "cat11".into(),
        plugin_ids: vec![pid("gps_set"), pid("png_encoder")],
        config_json: two_node_config(&pid("gps_set"), Some(vec![("gps_mode", json!("clear"))]), &pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "exif_clear".into(), category: "cat11".into(),
        plugin_ids: vec![pid("exif_rw"), pid("png_encoder")],
        config_json: two_node_config(&pid("exif_rw"), Some(vec![("write_exif", json!("clear"))]), &pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "time_shift_reset".into(), category: "cat11".into(),
        plugin_ids: vec![pid("time_shift"), pid("png_encoder")],
        config_json: two_node_config(&pid("time_shift"), Some(vec![("shift_hours", json!(0))]), &pid("png_encoder"), None),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });

    // Additional metadata scenarios (4)
    specs.push(TestCaseSpec {
        name: "exif_and_gps_png".into(), category: "cat11".into(),
        plugin_ids: vec![pid("exif_rw"), pid("gps_set"), pid("png_encoder")],
        config_json: three_node_config(
            &pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]),
            &pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(35.6)), ("longitude", json!(139.7))]),
            &pid("png_encoder"), None,
        ),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(90),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::PngSignatureValid],
    });
    specs.push(TestCaseSpec {
        name: "time_shift_and_exif_png".into(), category: "cat11".into(),
        plugin_ids: vec![pid("time_shift"), pid("exif_rw"), pid("png_encoder")],
        config_json: three_node_config(
            &pid("time_shift"), Some(vec![("shift_hours", json!(3))]),
            &pid("exif_rw"), Some(vec![("write_exif", json!("preserve"))]),
            &pid("png_encoder"), None,
        ),
        image_type: ImageType::Solid64x64, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(90),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "gps_tokyo_manual".into(), category: "cat11".into(),
        plugin_ids: vec![pid("gps_set"), pid("png_encoder")],
        config_json: two_node_config(
            &pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(35.6762)), ("longitude", json!(139.6503))]),
            &pid("png_encoder"), None,
        ),
        image_type: ImageType::Gradient256x256, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty],
    });
    specs.push(TestCaseSpec {
        name: "gps_london_manual".into(), category: "cat11".into(),
        plugin_ids: vec![pid("gps_set"), pid("tiff_encoder")],
        config_json: two_node_config(
            &pid("gps_set"), Some(vec![("gps_mode", json!("manual")), ("latitude", json!(51.5074)), ("longitude", json!(-0.1278))]),
            &pid("tiff_encoder"), None,
        ),
        image_type: ImageType::Solid64x64, output_ext: "tiff".into(),
        expect_success: true, is_large_pipeline: false, timeout_secs: Some(45),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::TiffMagicValid],
    });

    specs
}

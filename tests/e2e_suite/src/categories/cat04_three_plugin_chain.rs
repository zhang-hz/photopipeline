use crate::common::*;

fn pid(short: &str) -> String { format!("photopipeline.plugins.{}", short) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    // raw_input → pixel → encode (25)
    let pixels = ["colorspace", "lut3d", "transform", "lens_correct", "ai_denoise"];
    let encoders = ["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder", "avif_encoder"];
    for p in &pixels {
        for e in &encoders {
            let name = format!("raw__{}__{}", p, e);
            specs.push(TestCaseSpec {
                name, category: "cat04".into(),
                plugin_ids: vec![pid("raw_input"), pid(p), pid(e)],
                config_json: three_node_config(&pid("raw_input"), None, &pid(p), None, &pid(e), None),
                image_type: ImageType::Large1920x1080, output_ext: "png".into(),
                expect_success: true, is_large_pipeline: true, timeout_secs: Some(120),
                assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
            });
        }
    }

    // pixel → pixel → encode (15)
    let chains: Vec<(&str, &str, &str)> = vec![
        ("colorspace", "transform", "png_encoder"),
        ("transform", "lens_correct", "heif_encoder"),
        ("colorspace", "lut3d", "png_encoder"),
        ("lut3d", "ai_denoise", "png_encoder"),
        ("transform", "ai_denoise", "jxl_encoder"),
        ("colorspace", "ai_denoise", "jxl_encoder"),
        ("lens_correct", "colorspace", "png_encoder"),
        ("colorspace", "transform", "tiff_encoder"),
        ("colorspace", "lut3d", "jxl_encoder"),
        ("lut3d", "ai_denoise", "tiff_encoder"),
        ("transform", "ai_denoise", "png_encoder"),
        ("transform", "ai_denoise", "heif_encoder"),
        ("colorspace", "ai_denoise", "png_encoder"),
        ("transform", "colorspace", "avif_encoder"),
        ("lens_correct", "transform", "png_encoder"),
    ];
    for (p1, p2, e) in &chains {
        let name = format!("pp__{}__{}__{}", p1, p2, e);
        specs.push(TestCaseSpec {
            name, category: "cat04".into(),
            plugin_ids: vec![pid(p1), pid(p2), pid(e)],
            config_json: three_node_config(&pid(p1), None, &pid(p2), None, &pid(e), None),
            image_type: ImageType::Large1920x1080, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: true, timeout_secs: Some(120),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // metadata → pixel → encode (5)
    for i in 0..5 {
        let e = encoders[i];
        let name = format!("meta_gps__colorspace__{}", e);
        specs.push(TestCaseSpec {
            name, category: "cat04".into(),
            plugin_ids: vec![pid("gps_set"), pid("colorspace"), pid(e)],
            config_json: three_node_config(&pid("gps_set"), None, &pid("colorspace"), None, &pid(e), None),
            image_type: ImageType::Gradient256x256, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(90),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    // pixel → metadata → encode (5)
    for i in 0..5 {
        let e = encoders[i];
        let name = format!("pixel_transform__exif__{}", e);
        specs.push(TestCaseSpec {
            name, category: "cat04".into(),
            plugin_ids: vec![pid("transform"), pid("exif_rw"), pid(e)],
            config_json: three_node_config(&pid("transform"), None, &pid("exif_rw"), None, &pid(e), None),
            image_type: ImageType::Gradient256x256, output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(90),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
        });
    }

    specs
}

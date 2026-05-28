use crate::common::*;

fn pid(s: &str) -> String { format!("photopipeline.plugins.{}", s) }

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();
    let encoders = ["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder", "avif_encoder"];

    // raw→colorspace→transform→encode (5)
    for e in &encoders {
        specs.push(test("raw_cs_transform", &["raw_input","colorspace","transform",e], true));
    }
    // raw→colorspace→ai_denoise→encode (5)
    for e in &encoders {
        specs.push(test("raw_cs_denoise", &["raw_input","colorspace","ai_denoise",e], true));
    }
    // raw→transform→lens_correct→encode (4)
    for e in &["png_encoder","tiff_encoder","jxl_encoder","avif_encoder"] {
        specs.push(test("raw_transform_lens", &["raw_input","transform","lens_correct",e], true));
    }
    // Full pixel: raw→colorspace→transform→lens_correct (3)
    for e in &["png_encoder","tiff_encoder","jxl_encoder"] {
        specs.push(test("raw_full_pixel", &["raw_input","colorspace","transform","lens_correct",e], true));
    }
    // LUT: raw→transform→colorspace→lut3d (3)
    for e in &["png_encoder","jxl_encoder","avif_encoder"] {
        specs.push(test("raw_lut", &["raw_input","transform","colorspace","lut3d",e], true));
    }
    // Full metadata: exif→gps→time→encode (3)
    for e in &["png_encoder","tiff_encoder","jxl_encoder"] {
        specs.push(test("full_meta", &["exif_rw","gps_set","time_shift",e], false));
    }

    // Additional cross-category workflows (3)
    specs.push(test("meta_exif_gps_time_png", &["exif_rw","gps_set","time_shift","png_encoder"], false));
    specs.push(test("raw_cs_lut_jxl", &["raw_input","colorspace","lut3d","jxl_encoder"], true));
    specs.push(test("raw_transform_denoise_png", &["raw_input","transform","ai_denoise","png_encoder"], true));

    specs
}

fn test(name: &str, plugins: &[&str], _is_raw: bool) -> TestCaseSpec {
    let full_ids: Vec<String> = plugins.iter().map(|s| pid(s)).collect();
    let config = four_node_config(&full_ids[0], None, &full_ids[1], None, &full_ids[2], None, &full_ids[3], None);
    TestCaseSpec {
        name: name.to_string(), category: "cat05".into(), plugin_ids: full_ids,
        config_json: config, image_type: ImageType::Large1920x1080, output_ext: "png".into(),
        expect_success: true, is_large_pipeline: true, timeout_secs: Some(120),
        assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(200)],
    }
}

use crate::common::*;

/// Metadata → Encode chains
fn metadata_encode_chains() -> Vec<(&'static str, &'static str, &'static str)> {
    let metadata = ["exif_rw", "gps_set", "time_shift"];
    let encoders = ["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder", "avif_encoder"];
    let mut chains = Vec::new();
    for m in &metadata {
        for e in &encoders {
            chains.push((*m, *e, "solid_64"));
        }
    }
    chains
}

/// Pixel → Encode chains
fn pixel_encode_chains() -> Vec<(&'static str, &'static str, &'static str)> {
    let pixels = ["colorspace", "lut3d", "transform", "lens_correct", "ai_denoise"];
    let encoders = ["png_encoder", "tiff_encoder", "heif_encoder", "jxl_encoder", "avif_encoder"];
    let mut chains = Vec::new();
    for p in &pixels {
        for e in &encoders {
            chains.push((*p, *e, "checkerboard_128"));
        }
    }
    chains
}

/// Input → Pixel chains
fn decode_pixel_chains() -> Vec<(&'static str, &'static str, &'static str)> {
    let pixels = ["colorspace", "lut3d", "transform", "lens_correct", "ai_denoise"];
    pixels.iter().map(|p| ("raw_input", *p, "gradient_256")).collect()
}

/// Pixel → Pixel chains
fn pixel_pixel_chains() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("colorspace", "transform", "solid_64"),
        ("colorspace", "lut3d", "color_bars"),
        ("transform", "lens_correct", "checkerboard_128"),
        ("transform", "ai_denoise", "grayscale"),
        ("lut3d", "colorspace", "gradient_256"),
        ("lens_correct", "ai_denoise", "solid_64"),
        ("colorspace", "lens_correct", "large_1920"),
        ("transform", "colorspace", "color_bars"),
        ("lut3d", "transform", "checkerboard_128"),
        ("ai_denoise", "lens_correct", "solid_64"),
        ("lut3d", "lens_correct", "gradient_256"),
        ("colorspace", "ai_denoise", "color_bars"),
        ("transform", "lut3d", "checkerboard_128"),
        ("lens_correct", "transform", "grayscale"),
        ("ai_denoise", "colorspace", "solid_64"),
    ]
}

fn plugin_id(short: &str) -> String {
    format!("photopipeline.plugins.{}", short)
}

fn image_for(name: &str) -> ImageType {
    match name {
        "solid_64" => ImageType::Solid64x64,
        "checkerboard_128" => ImageType::Checkerboard128x128,
        "gradient_256" => ImageType::Gradient256x256,
        "color_bars" => ImageType::ColorBars256x128,
        "grayscale" => ImageType::Grayscale256x16,
        "large_1920" => ImageType::Large1920x1080,
        _ => ImageType::Solid64x64,
    }
}

pub fn specs() -> Vec<TestCaseSpec> {
    let mut specs = Vec::new();

    for (p1, p2, img_name) in &metadata_encode_chains() {
        let name = format!("meta__{}__{}", p1, p2);
        specs.push(TestCaseSpec {
            name, category: "cat03".into(),
            plugin_ids: vec![plugin_id(p1), plugin_id(p2)],
            config_json: two_node_config(&plugin_id(p1), None, &plugin_id(p2), None),
            image_type: image_for(img_name), output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    for (p1, p2, img_name) in &pixel_encode_chains() {
        let name = format!("pixel__{}__{}", p1, p2);
        specs.push(TestCaseSpec {
            name, category: "cat03".into(),
            plugin_ids: vec![plugin_id(p1), plugin_id(p2)],
            config_json: two_node_config(&plugin_id(p1), None, &plugin_id(p2), None),
            image_type: image_for(img_name), output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    for (p1, p2, img_name) in &decode_pixel_chains() {
        let name = format!("decode__{}__{}", p1, p2);
        specs.push(TestCaseSpec {
            name, category: "cat03".into(),
            plugin_ids: vec![plugin_id(p1), plugin_id(p2)],
            config_json: two_node_config(&plugin_id(p1), None, &plugin_id(p2), None),
            image_type: image_for(img_name), output_ext: "png".into(),
            expect_success: true, is_large_pipeline: false, timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    for (p1, p2, img_name) in &pixel_pixel_chains() {
        let name = format!("pixel2__{}__{}", p1, p2);
        specs.push(TestCaseSpec {
            name, category: "cat03".into(),
            plugin_ids: vec![plugin_id(p1), plugin_id(p2)],
            config_json: two_node_config(&plugin_id(p1), None, &plugin_id(p2), None),
            image_type: image_for(img_name), output_ext: "png".into(),
            expect_success: true, is_large_pipeline: img_name.ends_with("large_1920"), timeout_secs: Some(60),
            assertions: vec![AssertionSpec::FileNonEmpty, AssertionSpec::FileSizeGt(100)],
        });
    }

    specs
}

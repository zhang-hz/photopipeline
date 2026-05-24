#![allow(clippy::result_large_err)]

use photopipeline_core::{EncodeOptions, ImageFormat, Metadata, PixelBuffer};
use photopipeline_plugin::registry::Registry;
use photopipeline_plugins;
use std::path::Path;
use std::sync::Arc;
use test_harness::assertions::golden::assert_golden_bytes;
use test_harness::fixtures::golden_patterns::*;

fn make_registry() -> Arc<Registry> {
    let r = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&r);
    r
}

fn encode_png(image: &PixelBuffer) -> Vec<u8> {
    let reg = make_registry();
    let processor = reg
        .get_format_processor(&"photopipeline.plugins.png_encoder".to_string())
        .expect("png format processor not found");
    let options = EncodeOptions {
        format: ImageFormat::PNG,
        ..Default::default()
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        processor
            .encode(image, &Metadata::default(), &options)
            .await
    })
    .expect("PNG encode failed")
}

#[test]
fn golden_png_solid_u8() {
    let input = solid_64x64_rgb_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/solid_64x64_128_64_32.png"),
        "solid_u8",
    );
}

#[test]
fn golden_png_vertical_ramp() {
    let input = vertical_ramp_256x256_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/vertical_ramp_256x256_u8.png"),
        "vertical_ramp",
    );
}

#[test]
fn golden_png_horizontal_ramp() {
    let input = horizontal_ramp_256x256_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/horizontal_ramp_256x256_u8.png"),
        "horizontal_ramp",
    );
}

#[test]
fn golden_png_color_bars() {
    let input = color_bars_256x128_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/color_bars_256x128.png"),
        "color_bars",
    );
}

#[test]
fn golden_png_checkerboard() {
    let input = checkerboard_64x64_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/checkerboard_64x64_u8.png"),
        "checkerboard",
    );
}

#[test]
fn golden_png_grayscale_steps() {
    let input = grayscale_steps_256x16_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/grayscale_steps_256x16_8steps.png"),
        "grayscale",
    );
}

#[test]
fn golden_png_solid_u16() {
    let input = solid_64x64_rgb_u16();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/solid_64x64_1000_2000_3000_u16.png"),
        "solid_u16",
    );
}

#[test]
fn golden_png_diagonal_ramp() {
    let input = diagonal_ramp_128x128_u8();
    let encoded = encode_png(&input);
    assert_golden_bytes(
        &encoded,
        Path::new("tests/fixtures/golden/png/diagonal_ramp_128x128.png"),
        "diagonal_ramp",
    );
}

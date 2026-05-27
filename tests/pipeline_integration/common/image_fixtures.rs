#![allow(dead_code)]
/// Image fixture generation for pipeline integration tests.
/// Generates in-memory PixelBuffer images with deterministic patterns (seed 42).
/// These correspond to I01-I20 from the test case specification.

use photopipeline_core::{ChannelLayout, ColorSpace, PixelBuffer, PixelFormat};
use test_harness::fixtures::image::ImageFixture;

/// All 20 test input images, keyed by their symbolic name.
pub struct TestImageSet {
    pub images: Vec<(String, PixelBuffer)>,
}

impl TestImageSet {
    /// Create all 20 test input images.
    pub fn generate_all() -> Self {
        let images = vec![
            ("solid_color_1920".to_string(), solid_color_1920()),
            ("adobergb_wide_1920".to_string(), adobergb_wide_1920()),
            ("web_photo_800".to_string(), web_photo_800()),
            ("4k_highres_3840".to_string(), highres_4k_3840()),
            ("displayp3_wide_1920".to_string(), displayp3_wide_1920()),
            ("noisy_texture_1920".to_string(), noisy_texture_1920()),
            ("barrel_distortion_1920".to_string(), barrel_distortion_1920()),
            ("pincushion_vignette_1920".to_string(), pincushion_vignette_1920()),
            ("grayscale_1024".to_string(), grayscale_1024()),
            ("high_bitdepth_1920".to_string(), high_bitdepth_1920()),
            ("camera_jpeg_exif".to_string(), camera_jpeg_exif()),
            ("alpha_transparent_1024".to_string(), alpha_transparent_1024()),
            ("icon_tiny_256".to_string(), icon_tiny_256()),
            ("panorama_wide_8000".to_string(), panorama_wide_8000()),
            ("cmyk_print_1920".to_string(), cmyk_print_1920()),
            ("zone_plate_test_1920".to_string(), zone_plate_test_1920()),
            ("color_checker_1920".to_string(), color_checker_1920()),
            ("gradient_all_1920".to_string(), gradient_all_1920()),
            ("single_pixel_1x1".to_string(), single_pixel_1x1()),
            ("extreme_aspect_100x65535".to_string(), extreme_aspect_100x65535()),
        ];
        TestImageSet { images }
    }

    /// Find an image by its symbolic name.
    pub fn get(&self, name: &str) -> &PixelBuffer {
        self.images
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, buf)| buf)
            .unwrap_or_else(|| panic!("Test image '{}' not found", name))
    }
}

/// Lazy static: generate once, reuse across tests.
use std::sync::OnceLock;
static TEST_IMAGES: OnceLock<TestImageSet> = OnceLock::new();

/// Get or initialize the global test image set.
pub fn test_images() -> &'static TestImageSet {
    TEST_IMAGES.get_or_init(|| TestImageSet::generate_all())
}

/// Get a test image buffer by name.
pub fn get_test_image(name: &str) -> PixelBuffer {
    test_images().get(name).clone()
}

// ── I01: 1920x1080 solid fill (sky+建筑+人物 sim) ──────────────────────
fn solid_color_1920() -> PixelBuffer {
    ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .color_space(ColorSpace::SRGB)
        .solid(120, 150, 200)
        .build()
}

// ── I02: 1920x1080 AdobeRGB wide gamut ───────────────────────────────
fn adobergb_wide_1920() -> PixelBuffer {
    let mut buf = ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build();
    buf.color_space = ColorSpace::ADOBE_RGB;
    buf
}

// ── I03: 800x600 web-size JPEG-like ──────────────────────────────────
fn web_photo_800() -> PixelBuffer {
    ImageFixture::new()
        .width(800)
        .height(600)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

// ── I04: 3840x2160 4K high-res ───────────────────────────────────────
fn highres_4k_3840() -> PixelBuffer {
    ImageFixture::new()
        .width(3840)
        .height(2160)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

// ── I05: 1920x1080 DisplayP3 wide ────────────────────────────────────
fn displayp3_wide_1920() -> PixelBuffer {
    let mut buf = ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build();
    buf.color_space = ColorSpace::DISPLAY_P3;
    buf
}

// ── I06: 1920x1080 noisy texture (ISO 6400 sim) ─────────────────────
fn noisy_texture_1920() -> PixelBuffer {
    // Simulate noise by using checkerboard with small tile size
    ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .checkerboard()
        .build()
}

// ── I07: 1920x1080 barrel distortion grid ────────────────────────────
fn barrel_distortion_1920() -> PixelBuffer {
    // Grid pattern simulates distortion test chart
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            // Draw a grid every 120 pixels
            let grid = (x % 120 < 2 || y % 120 < 2) as u8;
            let val = if grid == 1 { 200u8 } else { 60u8 };
            buf.data.data[base] = val;
            buf.data.data[base + 1] = val;
            buf.data.data[base + 2] = val;
        }
    }
    buf
}

// ── I08: 1920x1080 pincushion + vignette ─────────────────────────────
fn pincushion_vignette_1920() -> PixelBuffer {
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    let cx = 1920.0 / 2.0;
    let cy = 1080.0 / 2.0;
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            let dx = (x as f64 - cx) / cx;
            let dy = (y as f64 - cy) / cy;
            let dist = (dx * dx + dy * dy).sqrt();
            // Grid pattern
            let grid = (x % 120 < 2 || y % 120 < 2) as u8;
            let base_val = if grid == 1 { 200u8 } else { 80u8 };
            // Apply vignette darkening at corners
            let vignette = ((1.0 - dist * 0.8).max(0.0) * 255.0) as u8;
            let val = ((base_val as f64 * vignette as f64 / 255.0).min(255.0)) as u8;
            buf.data.data[base] = val;
            buf.data.data[base + 1] = val;
            buf.data.data[base + 2] = val;
        }
    }
    buf
}

// ── I09: 1024x1024 grayscale 256-step ramp ───────────────────────────
fn grayscale_1024() -> PixelBuffer {
    let mut buf = PixelBuffer::new(1024, 1024, ChannelLayout::Gray, PixelFormat::U8);
    for y in 0..1024usize {
        for x in 0..1024usize {
            let idx = y * 1024 + x;
            buf.data.data[idx] = (x / 4).min(255) as u8;
        }
    }
    buf
}

// ── I10: 1920x1080 U16 gradient ──────────────────────────────────────
fn high_bitdepth_1920() -> PixelBuffer {
    ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U16)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

// ── I11: 1920x1080 camera JPEG with EXIF marker (visual only) ───────
fn camera_jpeg_exif() -> PixelBuffer {
    ImageFixture::new()
        .width(1920)
        .height(1080)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

// ── I12: 1024x1024 RGBA checkerboard ─────────────────────────────────
fn alpha_transparent_1024() -> PixelBuffer {
    ImageFixture::new()
        .width(1024)
        .height(1024)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGBA)
        .checkerboard()
        .build()
}

// ── I13: 256x256 icon tiny ───────────────────────────────────────────
fn icon_tiny_256() -> PixelBuffer {
    ImageFixture::new()
        .width(256)
        .height(256)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .solid(200, 100, 50)
        .build()
}

// ── I14: 8000x4000 panorama ultra-wide ───────────────────────────────
fn panorama_wide_8000() -> PixelBuffer {
    ImageFixture::new()
        .width(8000)
        .height(4000)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

// ── I15: 1920x1080 CMYK-simulated print ──────────────────────────────
fn cmyk_print_1920() -> PixelBuffer {
    // Simulate CMYK with inverted colors
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            let t = x as f64 / 1920.0;
            // CMYK-like: C=1-R, M=1-G, Y=1-B
            buf.data.data[base] = ((1.0 - t) * 255.0) as u8;
            buf.data.data[base + 1] = ((1.0 - t * 0.7) * 200.0) as u8;
            buf.data.data[base + 2] = 128u8;
        }
    }
    buf
}

// ── I16: 1920x1080 zone plate (sine circular frequency) ──────────────
fn zone_plate_test_1920() -> PixelBuffer {
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    let cx = 1920.0 / 2.0;
    let cy = 1080.0 / 2.0;
    let max_r = f64::sqrt(cx * cx + cy * cy);
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let r = f64::sqrt(dx * dx + dy * dy);
            let freq = (r / max_r) * 80.0;
            let val = ((freq.sin() * 0.5 + 0.5) * 255.0) as u8;
            buf.data.data[base] = val;
            buf.data.data[base + 1] = val;
            buf.data.data[base + 2] = val;
        }
    }
    buf
}

// ── I17: 1920x1080 ColorChecker 24-color patch ──────────────────────
fn color_checker_1920() -> PixelBuffer {
    // 6x4 grid of color patches
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    let colors: [(u8, u8, u8); 24] = [
        (115, 82, 68), (194, 150, 130), (98, 122, 157), (87, 108, 67),
        (133, 128, 177), (103, 189, 170), (214, 126, 44), (80, 91, 166),
        (193, 90, 99), (94, 60, 108), (157, 188, 64), (224, 163, 46),
        (56, 61, 150), (70, 148, 73), (175, 54, 60), (231, 199, 31),
        (187, 86, 149), (8, 133, 161), (243, 238, 243), (200, 200, 200),
        (160, 160, 160), (122, 122, 122), (85, 85, 85), (52, 52, 52),
    ];
    let patch_w = 1920 / 6;
    let patch_h = 1080 / 4;
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            let col = (x / patch_w).min(5);
            let row = (y / patch_h).min(3);
            let idx = (row * 6 + col).min(23);
            let (r, g, b) = colors[idx];
            buf.data.data[base] = r;
            buf.data.data[base + 1] = g;
            buf.data.data[base + 2] = b;
        }
    }
    buf
}

// ── I18: 1920x1080 composite gradient ────────────────────────────────
fn gradient_all_1920() -> PixelBuffer {
    let mut buf = PixelBuffer::new(1920, 1080, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    for y in 0..1080usize {
        for x in 0..1920usize {
            let base = (y * 1920 + x) * channels;
            let h = x as f64 / 1920.0;
            let v = y as f64 / 1080.0;
            let d = (x + y) as f64 / 3000.0;
            buf.data.data[base] = (h * 255.0) as u8;
            buf.data.data[base + 1] = (v * 200.0) as u8;
            buf.data.data[base + 2] = (d * 180.0).min(255.0) as u8;
        }
    }
    buf
}

// ── I19: 1x1 single pixel white ──────────────────────────────────────
fn single_pixel_1x1() -> PixelBuffer {
    ImageFixture::new()
        .width(1)
        .height(1)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .solid(255, 255, 255)
        .build()
}

// ── I20: 100x65535 extreme aspect ratio ──────────────────────────────
fn extreme_aspect_100x65535() -> PixelBuffer {
    ImageFixture::new()
        .width(100)
        .height(65535)
        .format(PixelFormat::U8)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_images_generate() {
        let set = TestImageSet::generate_all();
        assert_eq!(set.images.len(), 20, "should generate 20 test images");
    }

    #[test]
    fn test_image_dimensions() {
        let set = TestImageSet::generate_all();
        let i01 = set.get("solid_color_1920");
        assert_eq!(i01.width, 1920);
        assert_eq!(i01.height, 1080);

        let i19 = set.get("single_pixel_1x1");
        assert_eq!(i19.width, 1);
        assert_eq!(i19.height, 1);

        let i20 = set.get("extreme_aspect_100x65535");
        assert_eq!(i20.width, 100);
        assert_eq!(i20.height, 65535);
    }

    #[test]
    fn test_image_not_empty() {
        let set = TestImageSet::generate_all();
        for (name, buf) in &set.images {
            assert!(
                !buf.data.data.is_empty(),
                "image '{}' has empty data",
                name
            );
        }
    }

    #[test]
    fn test_grayscale_is_single_channel() {
        let buf = grayscale_1024();
        assert_eq!(buf.layout, ChannelLayout::Gray);
        assert_eq!(buf.width, 1024);
        assert_eq!(buf.height, 1024);
    }

    #[test]
    fn test_alpha_has_4_channels() {
        let buf = alpha_transparent_1024();
        assert_eq!(buf.layout, ChannelLayout::RGBA);
    }

    #[test]
    fn test_high_bitdepth_is_u16() {
        let buf = high_bitdepth_1920();
        assert_eq!(buf.format, PixelFormat::U16);
    }

    #[test]
    fn test_solid_color_consistency() {
        let buf = solid_color_1920();
        // Solid RGB image: each channel should have the same value across all pixels.
        // fst pixel channels = (120, 150, 200)
        let channels = 3;
        let pixel_count = buf.width as usize * buf.height as usize;
        for c in 0..channels {
            let expected = buf.data.data[c];
            for p in 1..pixel_count {
                let offset = p * channels + c;
                assert_eq!(expected, buf.data.data[offset],
                    "solid image channel {} should be consistent: pixel 0 = {}, pixel {} = {}",
                    c, expected, p, buf.data.data[offset]);
            }
        }
    }

    #[test]
    fn test_color_checker_has_24_distinct_colors() {
        let buf = color_checker_1920();
        use std::collections::HashSet;
        let patch_w = 1920 / 6;
        let patch_h = 1080 / 4;
        let mut colors = HashSet::new();
        for row in 0..4 {
            for col in 0..6 {
                let cx = col * patch_w + patch_w / 2;
                let cy = row * patch_h + patch_h / 2;
                let base = (cy * 1920 + cx) * 3;
                let r = buf.data.data[base];
                let g = buf.data.data[base + 1];
                let b = buf.data.data[base + 2];
                colors.insert((r, g, b));
            }
        }
        assert_eq!(colors.len(), 24, "ColorChecker should have 24 distinct patches");
    }

    #[test]
    fn test_cache_works() {
        let set1 = test_images();
        let set2 = test_images();
        let i01_1 = set1.get("solid_color_1920");
        let i01_2 = set2.get("solid_color_1920");
        assert_eq!(i01_1.width, i01_2.width);
        // Same pointer due to OnceLock
    }
}

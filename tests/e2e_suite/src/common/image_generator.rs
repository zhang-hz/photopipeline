use super::ImageType;

/// Generate a test image as PNG bytes for the given image type.
/// Reuses test_harness image generators.
pub fn generate(img: ImageType) -> Vec<u8> {
    let (_w, _h) = img.dimensions();
    match img {
        ImageType::Solid64x64 => make_solid_rgb_png(64, 64, 128, 64, 32),
        ImageType::Checkerboard128x128 => make_checkerboard_png(128, 128, 32),
        ImageType::Gradient256x256 => make_horizontal_gradient_png(256, 256),
        ImageType::ColorBars256x128 => make_color_bars_png(),
        ImageType::Grayscale256x16 => make_grayscale_steps_png(256, 16, 8),
        ImageType::Large1920x1080 => make_solid_rgb_png(1920, 1080, 100, 150, 200),
        ImageType::VerySmall8x8 => make_solid_rgb_png(8, 8, 64, 128, 192),
        ImageType::WideStrip640x16 => make_solid_rgb_png(640, 16, 200, 100, 50),
    }
}

fn make_solid_rgb_png(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    let img = image::ImageBuffer::from_fn(w, h, |_, _| image::Rgb([r, g, b]));
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

fn make_checkerboard_png(w: u32, h: u32, tile: u32) -> Vec<u8> {
    let img = image::ImageBuffer::from_fn(w, h, |x, y| {
        let bright = ((x / tile) + (y / tile)) % 2 == 0;
        if bright { image::Rgb([200u8, 200, 200]) } else { image::Rgb([40u8, 40, 40]) }
    });
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

fn make_horizontal_gradient_png(w: u32, h: u32) -> Vec<u8> {
    let img = image::ImageBuffer::from_fn(w, h, |x, _y| {
        let v = (x as f64 / w as f64 * 255.0) as u8;
        image::Rgb([v, 128, 255 - v])
    });
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

fn make_color_bars_png() -> Vec<u8> {
    let colors: [(u8, u8, u8); 8] = [
        (255, 255, 255), (255, 255, 0), (0, 255, 255), (0, 255, 0),
        (255, 0, 255), (255, 0, 0), (0, 0, 255), (0, 0, 0),
    ];
    let w = 256u32;
    let h = 128u32;
    let bar_w = w / 8;
    let img = image::ImageBuffer::from_fn(w, h, |x, _y| {
        let idx = (x / bar_w).min(7) as usize;
        let (r, g, b) = colors[idx];
        image::Rgb([r, g, b])
    });
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

fn make_grayscale_steps_png(w: u32, h: u32, steps: u32) -> Vec<u8> {
    let step_w = w.max(1) / steps.max(1);
    let img = image::ImageBuffer::from_fn(w, h, |x, _y| {
        let v = ((x / step_w) as f64 / (steps - 1) as f64 * 255.0) as u8;
        image::Luma([v])
    });
    let mut buf = std::io::Cursor::new(Vec::new());
    let dynamic = image::DynamicImage::ImageLuma8(img);
    dynamic.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

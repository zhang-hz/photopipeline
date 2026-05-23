use photopipeline_core::*;

pub fn assert_pixels_eq(a: &PixelBuffer, b: &PixelBuffer, msg: &str) {
    assert_eq!(
        a.width, b.width,
        "{msg}: width mismatch: {} vs {}",
        a.width, b.width
    );
    assert_eq!(
        a.height, b.height,
        "{msg}: height mismatch: {} vs {}",
        a.height, b.height
    );
    assert_eq!(
        a.layout, b.layout,
        "{msg}: layout mismatch: {:?} vs {:?}",
        a.layout, b.layout
    );
    assert_eq!(
        a.format, b.format,
        "{msg}: format mismatch: {:?} vs {:?}",
        a.format, b.format
    );
    assert_eq!(
        a.data.data.len(),
        b.data.data.len(),
        "{msg}: data length mismatch: {} vs {}",
        a.data.data.len(),
        b.data.data.len()
    );
    for (i, (byte_a, byte_b)) in a.data.data.iter().zip(b.data.data.iter()).enumerate() {
        assert_eq!(
            byte_a, byte_b,
            "{msg}: pixel data differs at byte {i}: {byte_a} vs {byte_b}"
        );
    }
}

pub fn assert_buffer_dimensions(buf: &PixelBuffer, w: u32, h: u32) {
    assert_eq!(buf.width, w, "expected width {}, got {}", w, buf.width);
    assert_eq!(buf.height, h, "expected height {}, got {}", h, buf.height);
}

pub fn assert_pixel_range(buf: &PixelBuffer, min: f64, max: f64) {
    let bpc = buf.format.bytes_per_channel();
    let channels = buf.layout.channel_count() as usize;
    let pixel_count = (buf.width as usize) * (buf.height as usize);

    for p in 0..pixel_count {
        for c in 0..channels {
            let offset = (p * channels + c) * bpc;
            if offset + bpc <= buf.data.data.len() {
                let value: f64 = match buf.format {
                    PixelFormat::U8 => buf.data.data[offset] as f64 / 255.0,
                    PixelFormat::U16 => {
                        let v =
                            u16::from_le_bytes([buf.data.data[offset], buf.data.data[offset + 1]]);
                        v as f64 / 65535.0
                    }
                    PixelFormat::F16 => {
                        let v =
                            u16::from_le_bytes([buf.data.data[offset], buf.data.data[offset + 1]]);
                        f16_to_f64(v)
                    }
                    PixelFormat::U32 => {
                        let mut bytes = [0u8; 4];
                        bytes.copy_from_slice(&buf.data.data[offset..offset + 4]);
                        u32::from_le_bytes(bytes) as f64 / u32::MAX as f64
                    }
                    PixelFormat::F32 => {
                        let mut bytes = [0u8; 4];
                        bytes.copy_from_slice(&buf.data.data[offset..offset + 4]);
                        f32::from_le_bytes(bytes) as f64
                    }
                };
                assert!(
                    value >= min && value <= max,
                    "pixel {p} channel {c} value {value} not in range [{min}, {max}]"
                );
            }
        }
    }
}

pub fn assert_pixel_format(buf: &PixelBuffer, format: PixelFormat) {
    assert_eq!(
        buf.format, format,
        "expected format {:?}, got {:?}",
        format, buf.format
    );
}

fn f16_to_f64(v: u16) -> f64 {
    let sign = (v >> 15) as f64;
    let exp = ((v >> 10) & 0x1F) as f64;
    let frac = (v & 0x3FF) as f64;
    if exp == 0.0 {
        (sign * -2.0 + 1.0) * 2.0f64.powi(-14) * (frac / 1024.0)
    } else if exp < 31.0 {
        (sign * -2.0 + 1.0) * 2.0f64.powi((exp as i32) - 15) * (1.0 + frac / 1024.0)
    } else {
        if frac == 0.0 { f64::INFINITY } else { f64::NAN }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::image::ImageFixture;

    #[test]
    fn assert_pixels_eq_identical() {
        let buf = ImageFixture::new()
            .width(4)
            .height(4)
            .solid(10, 20, 30)
            .build();
        let buf2 = buf.clone();
        assert_pixels_eq(&buf, &buf2, "identical buffers");
    }

    #[test]
    #[should_panic]
    fn assert_pixels_eq_different_data() {
        let buf1 = ImageFixture::new().solid(0, 0, 0).build();
        let buf2 = ImageFixture::new().solid(255, 255, 255).build();
        assert_pixels_eq(&buf1, &buf2, "different buffers");
    }

    #[test]
    fn assert_buffer_dimensions_matches() {
        let buf = ImageFixture::new().width(32).height(64).build();
        assert_buffer_dimensions(&buf, 32, 64);
    }

    #[test]
    #[should_panic]
    fn assert_buffer_dimensions_mismatch() {
        let buf = ImageFixture::new().width(32).height(64).build();
        assert_buffer_dimensions(&buf, 100, 200);
    }

    #[test]
    fn assert_pixel_range_within_bounds() {
        let buf = ImageFixture::new()
            .width(4)
            .height(4)
            .solid(128, 128, 128)
            .build();
        assert_pixel_range(&buf, 0.0, 1.0);
    }

    #[test]
    #[should_panic]
    fn assert_pixel_range_out_of_bounds() {
        let buf = ImageFixture::new()
            .width(4)
            .height(4)
            .solid(0, 0, 0)
            .build();
        assert_pixel_range(&buf, 0.5, 1.0);
    }

    #[test]
    fn assert_pixel_format_matches() {
        let buf = ImageFixture::new().format(PixelFormat::U16).build();
        assert_pixel_format(&buf, PixelFormat::U16);
    }

    #[test]
    #[should_panic]
    fn assert_pixel_format_mismatch() {
        let buf = ImageFixture::new().format(PixelFormat::U8).build();
        assert_pixel_format(&buf, PixelFormat::F32);
    }
}

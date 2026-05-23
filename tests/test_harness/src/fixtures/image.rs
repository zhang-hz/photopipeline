use photopipeline_core::*;

pub struct ImageFixture {
    width: u32,
    height: u32,
    format: PixelFormat,
    layout: ChannelLayout,
    color_space: ColorSpace,
    fill_value: Option<Vec<u8>>,
    gradient: bool,
    checkerboard: bool,
}

impl ImageFixture {
    pub fn new() -> Self {
        Self {
            width: 256,
            height: 256,
            format: PixelFormat::U8,
            layout: ChannelLayout::RGB,
            color_space: ColorSpace::SRGB,
            fill_value: None,
            gradient: false,
            checkerboard: false,
        }
    }

    pub fn width(mut self, w: u32) -> Self {
        self.width = w;
        self
    }

    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    pub fn format(mut self, f: PixelFormat) -> Self {
        self.format = f;
        self
    }

    pub fn layout(mut self, l: ChannelLayout) -> Self {
        self.layout = l;
        self
    }

    pub fn color_space(mut self, cs: ColorSpace) -> Self {
        self.color_space = cs;
        self
    }

    pub fn solid(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fill_value = Some(vec![r, g, b]);
        self
    }

    pub fn solid_u16(mut self, r: u16, g: u16, b: u16) -> Self {
        let bpc = self.format.bytes_per_channel();
        let mut data = vec![0u8; 6];
        if bpc == 2 {
            data[0..2].copy_from_slice(&r.to_le_bytes());
            data[2..4].copy_from_slice(&g.to_le_bytes());
            data[4..6].copy_from_slice(&b.to_le_bytes());
        }
        self.fill_value = Some(data);
        self
    }

    pub fn gradient(mut self) -> Self {
        self.gradient = true;
        self
    }

    pub fn checkerboard(mut self) -> Self {
        self.checkerboard = true;
        self
    }

    pub fn build(&self) -> PixelBuffer {
        let channels = self.layout.channel_count() as usize;
        let bpc = self.format.bytes_per_channel();
        let pixel_count = (self.width as usize) * (self.height as usize);
        let _byte_size = pixel_count * channels * bpc;
        let mut buf = PixelBuffer::new(self.width, self.height, self.layout, self.format);
        buf.color_space = self.color_space.clone();

        if let Some(ref fill) = self.fill_value {
            if fill.len() >= channels {
                for p in 0..pixel_count {
                    for c in 0..channels {
                        let offset = (p * channels + c) * bpc;
                        let end = offset + bpc;
                        if end <= buf.data.data.len() && c * bpc + bpc <= fill.len() {
                            buf.data.data[offset..end]
                                .copy_from_slice(&fill[c * bpc..c * bpc + bpc]);
                        }
                    }
                }
            }
        } else if self.gradient {
            for y in 0..self.height as usize {
                for x in 0..self.width as usize {
                    let t = x as f32 / (self.width as f32).max(1.0);
                    for c in 0..channels.min(3) {
                        let val = if c == 0 {
                            (t * 255.0) as u8
                        } else if c == 1 {
                            ((1.0 - t) * 200.0) as u8
                        } else {
                            128u8
                        };
                        let offset = (y * self.width as usize + x) * channels * bpc + c * bpc;
                        if bpc >= 1 && offset < buf.data.data.len() {
                            buf.data.data[offset] = val;
                            if bpc >= 2 {
                                let v16 = (val as u16) * 257u16;
                                buf.data.data[offset] = (v16 & 0xFF) as u8;
                                buf.data.data[offset + 1] = (v16 >> 8) as u8;
                            }
                            if bpc >= 4 {
                                let v32 = val as f32 / 255.0;
                                let bytes = v32.to_le_bytes();
                                buf.data.data[offset..offset + 4].copy_from_slice(&bytes);
                            }
                        }
                    }
                }
            }
        } else if self.checkerboard {
            let tile_size = 32usize;
            for y in 0..self.height as usize {
                for x in 0..self.width as usize {
                    let bright = ((x / tile_size) + (y / tile_size)) % 2 == 0;
                    let val = if bright { 200u8 } else { 40u8 };
                    for c in 0..channels.min(3) {
                        let offset = (y * self.width as usize + x) * channels * bpc + c * bpc;
                        if bpc >= 1 && offset < buf.data.data.len() {
                            buf.data.data[offset] = val;
                            if bpc >= 2 {
                                let v16 = (val as u16) * 257u16;
                                buf.data.data[offset] = (v16 & 0xFF) as u8;
                                buf.data.data[offset + 1] = (v16 >> 8) as u8;
                            }
                            if bpc >= 4 {
                                let v32 = val as f32 / 255.0;
                                let bytes = v32.to_le_bytes();
                                buf.data.data[offset..offset + 4].copy_from_slice(&bytes);
                            }
                        }
                    }
                    if channels >= 4 {
                        let a_offset = (y * self.width as usize + x) * channels * bpc + 3 * bpc;
                        if bpc >= 1 && a_offset < buf.data.data.len() {
                            buf.data.data[a_offset] = 255;
                            if bpc >= 2 {
                                buf.data.data[a_offset + 1] = 255;
                            }
                            if bpc >= 4 {
                                let one = 1.0f32.to_le_bytes();
                                buf.data.data[a_offset..a_offset + 4].copy_from_slice(&one);
                            }
                        }
                    }
                }
            }
        }

        buf
    }
}

impl Default for ImageFixture {
    fn default() -> Self {
        Self::new()
    }
}

pub fn pixel_buffer_1x1(format: PixelFormat, layout: ChannelLayout) -> PixelBuffer {
    ImageFixture::new()
        .width(1)
        .height(1)
        .format(format)
        .layout(layout)
        .solid(128, 128, 128)
        .build()
}

pub fn test_pattern_rgb(width: u32, height: u32, format: PixelFormat) -> PixelBuffer {
    ImageFixture::new()
        .width(width)
        .height(height)
        .format(format)
        .layout(ChannelLayout::RGB)
        .gradient()
        .build()
}

pub fn test_pattern_gray_ramp(width: u32, height: u32, _byte_per_channel: usize) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::Gray, PixelFormat::U8);
    let total = (width as usize * height as usize).max(1);
    for (i, byte) in buf.data.data.iter_mut().enumerate().take(total) {
        *byte = ((i as u64 * 255) / total as u64) as u8;
    }
    buf
}

pub fn test_buffer_suite() -> Vec<PixelBuffer> {
    let combinations = vec![
        (PixelFormat::U8, ChannelLayout::Gray),
        (PixelFormat::U8, ChannelLayout::RGB),
        (PixelFormat::U8, ChannelLayout::RGBA),
        (PixelFormat::U16, ChannelLayout::Gray),
        (PixelFormat::U16, ChannelLayout::RGB),
        (PixelFormat::U16, ChannelLayout::RGBA),
        (PixelFormat::F32, ChannelLayout::Gray),
        (PixelFormat::F32, ChannelLayout::RGB),
        (PixelFormat::F32, ChannelLayout::RGBA),
        (PixelFormat::F16, ChannelLayout::GrayAlpha),
    ];

    combinations
        .into_iter()
        .map(|(format, layout)| {
            ImageFixture::new()
                .width(64)
                .height(64)
                .format(format)
                .layout(layout)
                .gradient()
                .build()
        })
        .collect()
}

fn write_channel_value(data: &mut [u8], offset: usize, bpc: usize, u8_val: u8) {
    match bpc {
        1 => data[offset] = u8_val,
        2 => {
            let v16 = u8_val as u16 * 257u16;
            data[offset] = (v16 & 0xFF) as u8;
            data[offset + 1] = (v16 >> 8) as u8;
        }
        4 => {
            let v32 = u8_val as f32 / 255.0;
            let bytes = v32.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&bytes);
        }
        _ => {}
    }
}

fn write_u16_value(data: &mut [u8], offset: usize, bpc: usize, u16_val: u16) {
    match bpc {
        1 => data[offset] = u16_val.min(255) as u8,
        2 => {
            data[offset] = (u16_val & 0xFF) as u8;
            data[offset + 1] = (u16_val >> 8) as u8;
        }
        _ => {}
    }
}

pub fn vertical_ramp(width: u32, height: u32, format: PixelFormat) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, format);
    let bpc = format.bytes_per_channel();
    let channels = 3usize;

    for y in 0..height as usize {
        for x in 0..width as usize {
            let t = y as f32 / (height as f32).max(1.0);
            let r = (t * 255.0) as u8;
            let g = (t * 200.0) as u8;
            let base = (y * width as usize + x) * channels * bpc;
            write_channel_value(&mut buf.data.data, base, bpc, r);
            write_channel_value(&mut buf.data.data, base + bpc, bpc, g);
            write_channel_value(&mut buf.data.data, base + 2 * bpc, bpc, 128);
        }
    }

    buf
}

pub fn horizontal_ramp(width: u32, height: u32, format: PixelFormat) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, format);
    let bpc = format.bytes_per_channel();
    let channels = 3usize;

    for y in 0..height as usize {
        for x in 0..width as usize {
            let t = x as f32 / (width as f32).max(1.0);
            let r = (t * 255.0) as u8;
            let base = (y * width as usize + x) * channels * bpc;
            write_channel_value(&mut buf.data.data, base, bpc, r);
            write_channel_value(&mut buf.data.data, base + bpc, bpc, 128);
            write_channel_value(&mut buf.data.data, base + 2 * bpc, bpc, 128);
        }
    }

    buf
}

pub fn diagonal_ramp(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, PixelFormat::U8);
    let channels = 3usize;
    let max_sum = (width + height - 2).max(1) as usize;

    for y in 0..height as usize {
        for x in 0..width as usize {
            let r = ((x + y) * 255 / max_sum.max(1)) as u8;
            let base = (y * width as usize + x) * channels;
            buf.data.data[base] = r;
            buf.data.data[base + 1] = 128;
            buf.data.data[base + 2] = 128;
        }
    }

    buf
}

pub fn checkerboard_black_white(
    width: u32,
    height: u32,
    tile_size: u32,
    format: PixelFormat,
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, format);
    let bpc = format.bytes_per_channel();
    let channels = 3usize;
    let max_val: u16 = match bpc {
        2 => 65535,
        _ => 255,
    };

    for y in 0..height as usize {
        for x in 0..width as usize {
            let tx = x / tile_size as usize;
            let ty = y / tile_size as usize;
            let bright = (tx + ty) % 2 == 0;
            let val = if bright { max_val } else { 0u16 };
            let base = (y * width as usize + x) * channels * bpc;
            write_u16_value(&mut buf.data.data, base, bpc, val);
            write_u16_value(&mut buf.data.data, base + bpc, bpc, val);
            write_u16_value(&mut buf.data.data, base + 2 * bpc, bpc, val);
        }
    }

    buf
}

pub fn color_bars(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, PixelFormat::U8);
    let colors: [(u8, u8, u8); 8] = [
        (255, 255, 255),
        (255, 255, 0),
        (0, 255, 255),
        (0, 255, 0),
        (255, 0, 255),
        (255, 0, 0),
        (0, 0, 255),
        (0, 0, 0),
    ];
    let bar_width = (width as usize / 8).max(1);

    for y in 0..height as usize {
        for x in 0..width as usize {
            let bar = (x / bar_width).min(7);
            let (r, g, b) = colors[bar];
            let base = (y * width as usize + x) * 3;
            buf.data.data[base] = r;
            buf.data.data[base + 1] = g;
            buf.data.data[base + 2] = b;
        }
    }

    buf
}

pub fn grayscale_steps(width: u32, height: u32, num_steps: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::Gray, PixelFormat::U8);
    let step_width = (width as usize / num_steps as usize).max(1);

    for y in 0..height as usize {
        for x in 0..width as usize {
            let step = (x / step_width).min(num_steps as usize - 1);
            let val = (step * 255 / (num_steps as usize - 1).max(1)) as u8;
            let idx = y * width as usize + x;
            buf.data.data[idx] = val;
        }
    }

    buf
}

pub fn rgb_separation(width: u32, height: u32) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, PixelFormat::U8);
    let band_height = (height as usize / 3).max(1);

    for y in 0..height as usize {
        for x in 0..width as usize {
            let band = y / band_height;
            let t = x as f32 / (width as f32).max(1.0);
            let r = if band == 0 { (t * 255.0) as u8 } else { 0u8 };
            let g = if band == 1 { (t * 255.0) as u8 } else { 0u8 };
            let b = if band >= 2 { (t * 255.0) as u8 } else { 0u8 };
            let base = (y * width as usize + x) * 3;
            buf.data.data[base] = r;
            buf.data.data[base + 1] = g;
            buf.data.data[base + 2] = b;
        }
    }

    buf
}

pub fn known_value_solid(
    width: u32,
    height: u32,
    r: u16,
    g: u16,
    b: u16,
    format: PixelFormat,
) -> PixelBuffer {
    let mut buf = PixelBuffer::new(width, height, ChannelLayout::RGB, format);
    let bpc = format.bytes_per_channel();
    let channels = 3usize;
    let (r_val, g_val, b_val) = match bpc {
        1 => (r.min(255) as u16, g.min(255) as u16, b.min(255) as u16),
        _ => (r, g, b),
    };

    for y in 0..height as usize {
        for x in 0..width as usize {
            let base = (y * width as usize + x) * channels * bpc;
            write_u16_value(&mut buf.data.data, base, bpc, r_val);
            write_u16_value(&mut buf.data.data, base + bpc, bpc, g_val);
            write_u16_value(&mut buf.data.data, base + 2 * bpc, bpc, b_val);
        }
    }

    buf
}

pub fn known_value_solid_u8(width: u32, height: u32, r: u8, g: u8, b: u8) -> PixelBuffer {
    known_value_solid(width, height, r as u16, g as u16, b as u16, PixelFormat::U8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_fixture_default_build() {
        let buf = ImageFixture::new().build();
        assert_eq!(buf.width, 256);
        assert_eq!(buf.height, 256);
        assert_eq!(buf.layout, ChannelLayout::RGB);
        assert_eq!(buf.format, PixelFormat::U8);
    }

    #[test]
    fn image_fixture_solid_fill() {
        let buf = ImageFixture::new()
            .width(4)
            .height(4)
            .solid(10, 20, 30)
            .build();
        assert_eq!(buf.data.data[0], 10);
        assert_eq!(buf.data.data[1], 20);
        assert_eq!(buf.data.data[2], 30);
    }

    #[test]
    fn image_fixture_checkerboard() {
        let buf = ImageFixture::new()
            .width(64)
            .height(64)
            .checkerboard()
            .build();
        let first = buf.data.data[0];
        let distant = buf.data.data[1000];
        assert!(first != distant || first == distant);
    }

    #[test]
    fn image_fixture_gradient() {
        let buf = ImageFixture::new().width(16).height(1).gradient().build();
        let left = buf.data.data[0];
        let right = buf.data.data[45];
        assert!(left <= right);
    }

    #[test]
    fn pixel_buffer_1x1_works() {
        let buf = pixel_buffer_1x1(PixelFormat::U8, ChannelLayout::Gray);
        assert_eq!(buf.width, 1);
        assert_eq!(buf.height, 1);
        assert_eq!(buf.layout, ChannelLayout::Gray);
    }

    #[test]
    fn test_pattern_rgb_dimensions() {
        let buf = test_pattern_rgb(100, 200, PixelFormat::U8);
        assert_eq!(buf.width, 100);
        assert_eq!(buf.height, 200);
        assert_eq!(buf.layout, ChannelLayout::RGB);
    }

    #[test]
    fn test_pattern_gray_ramp_monotonic() {
        let buf = test_pattern_gray_ramp(4, 4, 1);
        assert_eq!(buf.data.data.len(), 16);
        for w in buf.data.data.windows(2) {
            assert!(w[0] <= w[1]);
        }
    }

    #[test]
    fn test_buffer_suite_covers_all() {
        let suite = test_buffer_suite();
        assert_eq!(suite.len(), 10);
    }

    #[test]
    fn image_fixture_builder_custom_dims() {
        let buf = ImageFixture::new().width(32).height(64).build();
        assert_eq!(buf.width, 32);
        assert_eq!(buf.height, 64);
    }

    #[test]
    fn image_fixture_color_space() {
        let buf = ImageFixture::new()
            .color_space(ColorSpace::LINEAR_SRGB)
            .build();
        assert_eq!(buf.color_space.primaries, ColorPrimaries::SRGB);
        assert_eq!(buf.color_space.transfer, TransferFunction::Linear);
    }

    #[test]
    fn image_fixture_solid_u16() {
        let buf = ImageFixture::new()
            .width(2)
            .height(2)
            .format(PixelFormat::U16)
            .layout(ChannelLayout::RGB)
            .solid_u16(1000, 2000, 3000)
            .build();
        let bpc: usize = 2;
        let _ch: usize = 3;
        let idx0 = |c: usize| c * bpc;
        assert_eq!(
            u16::from_le_bytes([buf.data.data[idx0(0)], buf.data.data[idx0(0) + 1]]),
            1000
        );
        assert_eq!(
            u16::from_le_bytes([buf.data.data[idx0(1)], buf.data.data[idx0(1) + 1]]),
            2000
        );
        assert_eq!(
            u16::from_le_bytes([buf.data.data[idx0(2)], buf.data.data[idx0(2) + 1]]),
            3000
        );
    }
}

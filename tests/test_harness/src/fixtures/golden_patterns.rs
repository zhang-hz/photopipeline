use crate::fixtures::image::*;
use photopipeline_core::{PixelBuffer, PixelFormat};

pub fn solid_64x64_rgb_u8() -> PixelBuffer {
    known_value_solid(64, 64, 128, 64, 32, PixelFormat::U8)
}

pub fn solid_64x64_rgb_u16() -> PixelBuffer {
    known_value_solid(64, 64, 1000, 2000, 3000, PixelFormat::U16)
}

pub fn vertical_ramp_256x256_u8() -> PixelBuffer {
    vertical_ramp(256, 256, PixelFormat::U8)
}

pub fn horizontal_ramp_256x256_u8() -> PixelBuffer {
    horizontal_ramp(256, 256, PixelFormat::U8)
}

pub fn color_bars_256x128_u8() -> PixelBuffer {
    color_bars(256, 128)
}

pub fn checkerboard_64x64_u8() -> PixelBuffer {
    checkerboard_black_white(64, 64, 8, PixelFormat::U8)
}

pub fn grayscale_steps_256x16_u8() -> PixelBuffer {
    grayscale_steps(256, 16, 8)
}

pub fn diagonal_ramp_128x128_u8() -> PixelBuffer {
    diagonal_ramp(128, 128)
}

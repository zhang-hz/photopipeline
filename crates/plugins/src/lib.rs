pub mod exif_rw;
pub mod gps_set;
pub mod time_shift;
pub mod colorspace;
pub mod lut3d;
pub mod transform;
pub mod lens_correct;
pub mod ai_denoise;
pub mod heif_encoder;
pub mod jxl_encoder;
pub mod avif_encoder;
pub mod tiff_encoder;
pub mod png_encoder;
pub mod raw_input;

use std::sync::Arc;
use photopipeline_plugin::registry::Registry;

pub fn register_all(registry: &Registry) {
    registry.register(Arc::new(exif_rw::ExifRwPlugin::new())).ok();
    registry.register(Arc::new(gps_set::GpsSetPlugin::new())).ok();
    registry.register(Arc::new(time_shift::TimeShiftPlugin::new())).ok();
    registry.register(Arc::new(colorspace::ColorSpacePlugin::new())).ok();
    registry.register(Arc::new(lut3d::Lut3dPlugin::new())).ok();
    registry.register(Arc::new(transform::TransformPlugin::new())).ok();
    registry.register(Arc::new(lens_correct::LensCorrectPlugin::new())).ok();
    registry.register(Arc::new(ai_denoise::AiDenoisePlugin::new())).ok();
    registry.register(Arc::new(heif_encoder::HeifEncoderPlugin::new())).ok();
    registry.register(Arc::new(jxl_encoder::JxlEncoderPlugin::new())).ok();
    registry.register(Arc::new(avif_encoder::AvifEncoderPlugin::new())).ok();
    registry.register(Arc::new(tiff_encoder::TiffEncoderPlugin::new())).ok();
    registry.register(Arc::new(png_encoder::PngEncoderPlugin::new())).ok();
    registry.register(Arc::new(raw_input::RawInputPlugin::new())).ok();
}

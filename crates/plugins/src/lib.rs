#![allow(clippy::result_large_err)]
#![allow(clippy::manual_memcpy)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::useless_vec)]
#![allow(clippy::new_without_default)]
#![allow(clippy::same_item_push)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::manual_strip)]
#![allow(clippy::unnecessary_parentheses)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::comparison_chain)]
pub mod ai_denoise;
pub mod avif_encoder;
pub mod colorspace;
pub mod exif_rw;
pub mod gps_set;
pub mod heif_encoder;
pub mod jxl_encoder;
pub mod lens_correct;
pub mod lut3d;
pub mod png_encoder;
pub mod raw_input;
pub mod tiff_encoder;
pub mod time_shift;
pub mod transform;

use photopipeline_plugin::registry::Registry;
use photopipeline_plugin::{
    AiProcessor, FormatProcessor, MetadataProcessor, PixelProcessor, Plugin,
};
use std::sync::Arc;

pub fn register_all(registry: &Registry) {
    {
        let p: Arc<exif_rw::ExifRwPlugin> = Arc::new(exif_rw::ExifRwPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_metadata_processor(p.clone() as Arc<dyn MetadataProcessor>);
    }
    {
        let p: Arc<gps_set::GpsSetPlugin> = Arc::new(gps_set::GpsSetPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_metadata_processor(p.clone() as Arc<dyn MetadataProcessor>);
    }
    {
        let p: Arc<time_shift::TimeShiftPlugin> = Arc::new(time_shift::TimeShiftPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_metadata_processor(p.clone() as Arc<dyn MetadataProcessor>);
    }
    {
        let p: Arc<colorspace::ColorSpacePlugin> = Arc::new(colorspace::ColorSpacePlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    }
    {
        let p: Arc<lut3d::Lut3dPlugin> = Arc::new(lut3d::Lut3dPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    }
    {
        let p: Arc<transform::TransformPlugin> = Arc::new(transform::TransformPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    }
    {
        let p: Arc<lens_correct::LensCorrectPlugin> =
            Arc::new(lens_correct::LensCorrectPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
    }
    {
        let p: Arc<ai_denoise::AiDenoisePlugin> = Arc::new(ai_denoise::AiDenoisePlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_pixel_processor(p.clone() as Arc<dyn PixelProcessor>);
        let _ = registry.register_ai_processor(p.clone() as Arc<dyn AiProcessor>);
    }
    {
        let p: Arc<heif_encoder::HeifEncoderPlugin> =
            Arc::new(heif_encoder::HeifEncoderPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
    {
        let p: Arc<jxl_encoder::JxlEncoderPlugin> = Arc::new(jxl_encoder::JxlEncoderPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
    {
        let p: Arc<avif_encoder::AvifEncoderPlugin> =
            Arc::new(avif_encoder::AvifEncoderPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
    {
        let p: Arc<tiff_encoder::TiffEncoderPlugin> =
            Arc::new(tiff_encoder::TiffEncoderPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
    {
        let p: Arc<png_encoder::PngEncoderPlugin> = Arc::new(png_encoder::PngEncoderPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
    {
        let p: Arc<raw_input::RawInputPlugin> = Arc::new(raw_input::RawInputPlugin::new());
        let _ = registry.register(p.clone() as Arc<dyn Plugin>);
        let _ = registry.register_format_processor(p.clone() as Arc<dyn FormatProcessor>);
    }
}

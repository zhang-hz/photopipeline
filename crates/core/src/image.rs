use strum::{Display, EnumString};
use serde::{Deserialize, Serialize};
use crate::color::ColorSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum PixelFormat {
    #[strum(serialize = "u8")]
    U8,
    #[strum(serialize = "u16")]
    U16,
    #[strum(serialize = "u32")]
    U32,
    #[strum(serialize = "f16")]
    F16,
    #[strum(serialize = "f32")]
    F32,
}

impl PixelFormat {
    pub fn bytes_per_channel(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 | Self::F16 => 2,
            Self::U32 | Self::F32 => 4,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::F16 | Self::F32)
    }

    pub fn is_high_precision(&self) -> bool {
        !matches!(self, Self::U8)
    }

    pub fn max_value_u16(&self) -> u16 {
        match self {
            Self::U8 => 255,
            Self::U16 => u16::MAX,
            Self::U32 | Self::F16 | Self::F32 => u16::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelLayout {
    Gray,
    GrayAlpha,
    RGB,
    RGBA,
    Planar(u8),
    Custom(u8),
}

impl ChannelLayout {
    pub fn channel_count(&self) -> u8 {
        match self {
            Self::Gray => 1,
            Self::GrayAlpha => 2,
            Self::RGB => 3,
            Self::RGBA => 4,
            Self::Planar(n) | Self::Custom(n) => *n,
        }
    }

    pub fn is_interleaved(&self) -> bool {
        matches!(self, Self::Gray | Self::GrayAlpha | Self::RGB | Self::RGBA)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct TileSpec {
    pub coord: TileCoord,
    pub x_offset: u32,
    pub y_offset: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct TileLayout {
    pub image_width: u32,
    pub image_height: u32,
    pub tile_size: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub overlap: u32,
    pub total_tiles: u32,
}

impl TileLayout {
    pub fn new(image_width: u32, image_height: u32, tile_size: u32, overlap: u32) -> Self {
        let stride = tile_size.saturating_sub(overlap).max(1);
        let tx = (image_width + stride - 1) / stride;
        let ty = (image_height + stride - 1) / stride;
        Self {
            image_width,
            image_height,
            tile_size,
            tiles_x: tx,
            tiles_y: ty,
            overlap,
            total_tiles: tx * ty,
        }
    }

    pub fn tile_spec(&self, x: u32, y: u32) -> TileSpec {
        let stride = self.tile_size.saturating_sub(self.overlap).max(1);
        let xo = x * stride;
        let yo = y * stride;
        let w = (self.tile_size).min(self.image_width.saturating_sub(xo));
        let h = (self.tile_size).min(self.image_height.saturating_sub(yo));
        TileSpec {
            coord: TileCoord { x, y },
            x_offset: xo,
            y_offset: yo,
            width: w,
            height: h,
        }
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = TileSpec> + '_ {
        (0..self.tiles_y).flat_map(move |y| {
            (0..self.tiles_x).map(move |x| self.tile_spec(x, y))
        })
    }
}

#[derive(Debug, Clone)]
pub struct AlignedBuffer {
    pub data: Vec<u8>,
    pub alignment: usize,
}

impl AlignedBuffer {
    pub fn new(size: usize, alignment: usize) -> Self {
        let mut data = Vec::with_capacity(size + alignment);
        unsafe { data.set_len(size); }
        Self { data, alignment }
    }

    pub fn as_u16_slice(&self) -> &[u16] {
        bytemuck::cast_slice(&self.data)
    }

    pub fn as_f32_slice(&self) -> &[f32] {
        bytemuck::cast_slice(&self.data)
    }
}

#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub layout: ChannelLayout,
    pub format: PixelFormat,
    pub color_space: ColorSpace,
    pub icc_profile: Option<Vec<u8>>,
    pub data: AlignedBuffer,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32, layout: ChannelLayout, format: PixelFormat) -> Self {
        let channels = layout.channel_count() as usize;
        let bytes = width as usize * height as usize * channels * format.bytes_per_channel();
        Self {
            width,
            height,
            layout,
            format,
            color_space: ColorSpace::default(),
            icc_profile: None,
            data: AlignedBuffer::new(bytes, 64),
        }
    }

    pub fn byte_size(&self) -> usize {
        self.data.data.len()
    }

    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn u16_samples(&self, channel: usize) -> Option<&[u16]> {
        let count = self.width as usize * self.height as usize;
        let offset = channel * count * std::mem::size_of::<u16>();
        match self.layout {
            ChannelLayout::Planar(n) if (channel as u8) < n => {
                bytemuck::cast_slice(&self.data.data[offset..]).get(..count)
            }
            _ => None,
        }
    }

    pub fn gpu_handle(&self) -> Option<GpuBufferHandle> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct GpuBufferHandle {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: crate::types::GpuBackend,
}

#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub handle: u64,
    pub size_bytes: u64,
    pub backend: crate::types::GpuBackend,
}

#[derive(Debug, Clone)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
    pub dtype: TensorDtype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorDtype {
    F32,
    F16,
    I8,
    U8,
}

#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub pixel_format: Option<PixelFormat>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub read_metadata: bool,
    pub apply_transfer: bool,
    pub icc_profile: Option<Vec<u8>>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            pixel_format: None,
            max_width: None,
            max_height: None,
            read_metadata: true,
            apply_transfer: false,
            icc_profile: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub buffer: PixelBuffer,
    pub metadata: crate::metadata::Metadata,
    pub format: crate::types::ImageFormat,
}

#[derive(Debug, Clone)]
pub struct EncodeOptions {
    pub format: crate::types::ImageFormat,
    pub quality: Option<f32>,
    pub lossless: bool,
    pub bit_depth: u8,
    pub chroma_subsampling: Option<ChromaSubsampling>,
    pub encoder: Option<String>,
    pub effort: Option<u8>,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            format: crate::types::ImageFormat::HEIF,
            quality: Some(95.0),
            lossless: false,
            bit_depth: 10,
            chroma_subsampling: Some(ChromaSubsampling::Yuv444),
            encoder: None,
            effort: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSubsampling {
    Yuv444,
    Yuv422,
    Yuv420,
}

#[derive(Debug, Clone)]
pub struct FormatProbe {
    pub path: Option<std::path::PathBuf>,
    pub extension: Option<String>,
    pub magic_bytes: Option<Vec<u8>>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HardwareRequirement {
    pub requires_cpu: bool,
    pub requires_gpu: bool,
    pub min_ram_mb: u64,
    pub preferred_backend: Option<crate::types::GpuBackend>,
}

impl Default for HardwareRequirement {
    fn default() -> Self {
        Self {
            requires_cpu: true,
            requires_gpu: false,
            min_ram_mb: 256,
            preferred_backend: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: std::collections::HashMap<String, String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self { enabled: true, settings: Default::default() }
    }
}

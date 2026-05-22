use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum ColorPrimaries {
    #[strum(serialize = "bt709")]
    BT709,
    #[strum(serialize = "bt2020")]
    BT2020,
    #[strum(serialize = "display_p3")]
    DisplayP3,
    #[strum(serialize = "sRGB")]
    SRGB,
    #[strum(serialize = "adobe_rgb")]
    AdobeRGB,
    #[strum(serialize = "pro_photo")]
    ProPhoto,
    #[strum(serialize = "aces")]
    ACES,
    #[strum(serialize = "aces_cg")]
    ACEScg,
    #[strum(serialize = "cie_xyz")]
    CIEXYZ,
    #[strum(serialize = "dci_p3")]
    DCIP3,
    #[strum(serialize = "rec2100")]
    Rec2100,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString)]
pub enum TransferFunction {
    #[strum(serialize = "linear")]
    Linear,
    #[strum(serialize = "srgb")]
    SRGB,
    #[strum(serialize = "gamma22")]
    Gamma22,
    #[strum(serialize = "gamma24")]
    Gamma24,
    #[strum(serialize = "gamma26")]
    Gamma26,
    #[strum(serialize = "gamma28")]
    Gamma28,
    #[strum(serialize = "pq")]
    PQ,
    #[strum(serialize = "hlg")]
    HLG,
    #[strum(serialize = "slog3")]
    SLog3,
    #[strum(serialize = "log_c")]
    LogC,
    #[strum(serialize = "custom")]
    Custom(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorSpace {
    pub primaries: ColorPrimaries,
    pub transfer: TransferFunction,
    pub white_point: WhitePoint,
    pub hdr_nits: Option<f32>,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self {
            primaries: ColorPrimaries::SRGB,
            transfer: TransferFunction::SRGB,
            white_point: WhitePoint::D65,
            hdr_nits: None,
        }
    }
}

impl ColorSpace {
    pub const SRGB: Self = Self {
        primaries: ColorPrimaries::SRGB,
        transfer: TransferFunction::SRGB,
        white_point: WhitePoint::D65,
        hdr_nits: None,
    };

    pub const ADOBE_RGB: Self = Self {
        primaries: ColorPrimaries::AdobeRGB,
        transfer: TransferFunction::Gamma22,
        white_point: WhitePoint::D65,
        hdr_nits: None,
    };

    pub const DISPLAY_P3: Self = Self {
        primaries: ColorPrimaries::DisplayP3,
        transfer: TransferFunction::SRGB,
        white_point: WhitePoint::D65,
        hdr_nits: None,
    };

    pub const REC2020_PQ: Self = Self {
        primaries: ColorPrimaries::BT2020,
        transfer: TransferFunction::PQ,
        white_point: WhitePoint::D65,
        hdr_nits: Some(1000.0),
    };

    pub const ACES_CG: Self = Self {
        primaries: ColorPrimaries::ACEScg,
        transfer: TransferFunction::Linear,
        white_point: WhitePoint::D60,
        hdr_nits: None,
    };

    pub const LINEAR_SRGB: Self = Self {
        primaries: ColorPrimaries::SRGB,
        transfer: TransferFunction::Linear,
        white_point: WhitePoint::D65,
        hdr_nits: None,
    };

    pub fn is_hdr(&self) -> bool {
        self.hdr_nits.unwrap_or(0.0) > 203.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString)]
pub enum WhitePoint {
    #[strum(serialize = "d50")]
    D50,
    #[strum(serialize = "d55")]
    D55,
    #[strum(serialize = "d60")]
    D60,
    #[strum(serialize = "d65")]
    D65,
    #[strum(serialize = "d75")]
    D75,
    #[strum(serialize = "dci")]
    DCI,
    #[strum(serialize = "e")]
    E,
    #[strum(serialize = "custom")]
    Custom(f32, f32),
}

impl Default for WhitePoint {
    fn default() -> Self {
        Self::D65
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum RenderingIntent {
    #[strum(serialize = "perceptual")]
    Perceptual,
    #[strum(serialize = "relative_colorimetric")]
    RelativeColorimetric,
    #[strum(serialize = "saturation")]
    Saturation,
    #[strum(serialize = "absolute_colorimetric")]
    AbsoluteColorimetric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorConversionSpec {
    pub source: ColorSpace,
    pub target: ColorSpace,
    pub intent: RenderingIntent,
    pub black_point_compensation: bool,
    pub gamut_mapping: GamutMapping,
    pub icc_profile: Option<Vec<u8>>,
    pub ocio_config: Option<String>,
    pub ocio_display: Option<String>,
    pub ocio_view: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum GamutMapping {
    #[strum(serialize = "clip")]
    Clip,
    #[strum(serialize = "compress")]
    Compress,
    #[strum(serialize = "luminance_preserve")]
    LuminancePreserve,
}

// RGB color in linear float, for color operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorRGB {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0 };

    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

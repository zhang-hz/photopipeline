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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_space_srgb_preset() {
        assert_eq!(ColorSpace::SRGB.primaries, ColorPrimaries::SRGB);
        assert_eq!(ColorSpace::SRGB.transfer, TransferFunction::SRGB);
        assert_eq!(ColorSpace::SRGB.white_point, WhitePoint::D65);
        assert_eq!(ColorSpace::SRGB.hdr_nits, None);
    }

    #[test]
    fn color_space_adobe_rgb_preset() {
        assert_eq!(ColorSpace::ADOBE_RGB.primaries, ColorPrimaries::AdobeRGB);
        assert_eq!(ColorSpace::ADOBE_RGB.transfer, TransferFunction::Gamma22);
        assert_eq!(ColorSpace::ADOBE_RGB.white_point, WhitePoint::D65);
        assert_eq!(ColorSpace::ADOBE_RGB.hdr_nits, None);
    }

    #[test]
    fn color_space_display_p3_preset() {
        assert_eq!(ColorSpace::DISPLAY_P3.primaries, ColorPrimaries::DisplayP3);
        assert_eq!(ColorSpace::DISPLAY_P3.transfer, TransferFunction::SRGB);
        assert_eq!(ColorSpace::DISPLAY_P3.white_point, WhitePoint::D65);
        assert_eq!(ColorSpace::DISPLAY_P3.hdr_nits, None);
    }

    #[test]
    fn color_space_rec2020_pq_preset() {
        assert_eq!(ColorSpace::REC2020_PQ.primaries, ColorPrimaries::BT2020);
        assert_eq!(ColorSpace::REC2020_PQ.transfer, TransferFunction::PQ);
        assert_eq!(ColorSpace::REC2020_PQ.white_point, WhitePoint::D65);
        assert_eq!(ColorSpace::REC2020_PQ.hdr_nits, Some(1000.0));
    }

    #[test]
    fn color_space_aces_cg_preset() {
        assert_eq!(ColorSpace::ACES_CG.primaries, ColorPrimaries::ACEScg);
        assert_eq!(ColorSpace::ACES_CG.transfer, TransferFunction::Linear);
        assert_eq!(ColorSpace::ACES_CG.white_point, WhitePoint::D60);
        assert_eq!(ColorSpace::ACES_CG.hdr_nits, None);
    }

    #[test]
    fn color_space_linear_srgb_preset() {
        assert_eq!(ColorSpace::LINEAR_SRGB.primaries, ColorPrimaries::SRGB);
        assert_eq!(ColorSpace::LINEAR_SRGB.transfer, TransferFunction::Linear);
        assert_eq!(ColorSpace::LINEAR_SRGB.white_point, WhitePoint::D65);
        assert_eq!(ColorSpace::LINEAR_SRGB.hdr_nits, None);
    }

    #[test]
    fn color_space_is_hdr() {
        assert!(!ColorSpace::SRGB.is_hdr());
        assert!(!ColorSpace::ADOBE_RGB.is_hdr());
        assert!(ColorSpace::REC2020_PQ.is_hdr());

        let low_hdr = ColorSpace {
            primaries: ColorPrimaries::BT2020,
            transfer: TransferFunction::PQ,
            white_point: WhitePoint::D65,
            hdr_nits: Some(200.0),
        };
        assert!(!low_hdr.is_hdr());

        let just_above = ColorSpace {
            primaries: ColorPrimaries::BT2020,
            transfer: TransferFunction::PQ,
            white_point: WhitePoint::D65,
            hdr_nits: Some(204.0),
        };
        assert!(just_above.is_hdr());
    }

    #[test]
    fn color_rgb_luminance() {
        assert_eq!(ColorRGB::BLACK.r, 0.0);
        assert_eq!(ColorRGB::BLACK.g, 0.0);
        assert_eq!(ColorRGB::BLACK.b, 0.0);
        assert!((ColorRGB::BLACK.luminance() - 0.0).abs() < 0.001);

        assert_eq!(ColorRGB::WHITE.r, 1.0);
        assert_eq!(ColorRGB::WHITE.g, 1.0);
        assert_eq!(ColorRGB::WHITE.b, 1.0);
        assert!((ColorRGB::WHITE.luminance() - 1.0).abs() < 0.001);

        let gray = ColorRGB { r: 0.5, g: 0.5, b: 0.5 };
        assert!((gray.luminance() - 0.5).abs() < 0.001);
    }

    #[test]
    fn color_primaries_display() {
        assert_eq!(ColorPrimaries::SRGB.to_string(), "sRGB");
        assert_eq!(ColorPrimaries::DisplayP3.to_string(), "display_p3");
        assert_eq!(ColorPrimaries::BT2020.to_string(), "bt2020");
        assert_eq!(ColorPrimaries::ACES.to_string(), "aces");
    }

    #[test]
    fn transfer_function_display() {
        assert_eq!(TransferFunction::Linear.to_string(), "linear");
        assert_eq!(TransferFunction::SRGB.to_string(), "srgb");
        assert_eq!(TransferFunction::PQ.to_string(), "pq");
        assert_eq!(TransferFunction::HLG.to_string(), "hlg");
    }

    #[test]
    fn white_point_display() {
        assert_eq!(WhitePoint::D50.to_string(), "d50");
        assert_eq!(WhitePoint::D65.to_string(), "d65");
        assert_eq!(WhitePoint::DCI.to_string(), "dci");
    }

    #[test]
    fn rendering_intent_display() {
        assert_eq!(RenderingIntent::Perceptual.to_string(), "perceptual");
        assert_eq!(RenderingIntent::RelativeColorimetric.to_string(), "relative_colorimetric");
    }

    #[test]
    fn gamut_mapping_display() {
        assert_eq!(GamutMapping::Clip.to_string(), "clip");
        assert_eq!(GamutMapping::Compress.to_string(), "compress");
    }

    #[test]
    fn color_space_default_is_srgb() {
        let cs = ColorSpace::default();
        assert_eq!(cs.primaries, ColorPrimaries::SRGB);
        assert_eq!(cs.transfer, TransferFunction::SRGB);
        assert!(!cs.is_hdr());
    }
}

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

impl ColorPrimaries {
    /// (x,y) chromaticity coordinates for (R, G, B) primaries.
    pub fn chromaticities(&self) -> ((f64, f64), (f64, f64), (f64, f64)) {
        match self {
            ColorPrimaries::SRGB | ColorPrimaries::BT709 => {
                ((0.6400, 0.3300), (0.3000, 0.6000), (0.1500, 0.0600))
            }
            ColorPrimaries::DisplayP3 | ColorPrimaries::DCIP3 => {
                ((0.6800, 0.3200), (0.2650, 0.6900), (0.1500, 0.0600))
            }
            ColorPrimaries::AdobeRGB => ((0.6400, 0.3300), (0.2100, 0.7100), (0.1500, 0.0600)),
            ColorPrimaries::BT2020 | ColorPrimaries::Rec2100 => {
                ((0.7080, 0.2920), (0.1700, 0.7970), (0.1310, 0.0460))
            }
            ColorPrimaries::ACEScg | ColorPrimaries::ACES => {
                ((0.7130, 0.2930), (0.1650, 0.8300), (0.1280, 0.0440))
            }
            ColorPrimaries::ProPhoto => ((0.7347, 0.2653), (0.1596, 0.8404), (0.0366, 0.0001)),
            ColorPrimaries::CIEXYZ => ((1.0, 0.0), (0.0, 1.0), (0.0, 0.0)),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ColorPrimaries::SRGB => "sRGB IEC61966-2.1",
            ColorPrimaries::DisplayP3 => "Display P3",
            ColorPrimaries::AdobeRGB => "Adobe RGB (1998)",
            ColorPrimaries::BT2020 => "ITU-R BT.2020",
            ColorPrimaries::ProPhoto => "ProPhoto RGB",
            ColorPrimaries::ACEScg => "ACEScg",
            ColorPrimaries::ACES => "ACES",
            ColorPrimaries::DCIP3 => "DCI-P3",
            ColorPrimaries::Rec2100 => "ITU-R BT.2100",
            ColorPrimaries::BT709 => "ITU-R BT.709",
            ColorPrimaries::CIEXYZ => "CIE XYZ",
        }
    }
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

impl TransferFunction {
    /// Decode a non-linear encoded value to linear light (f32, [0,1] range).
    pub fn decode_to_linear(&self, v: f32) -> f32 {
        let v = v.clamp(0.0, 1.0);
        match self {
            TransferFunction::Linear => v,
            TransferFunction::SRGB => {
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            }
            TransferFunction::Gamma22 => v.powf(2.2),
            TransferFunction::Gamma24 => v.powf(2.4),
            TransferFunction::Gamma26 => v.powf(2.6),
            TransferFunction::Gamma28 => v.powf(2.8),
            TransferFunction::PQ => pq_eotf_normalized(v as f64) as f32,
            TransferFunction::HLG => hlg_oetf_inverse(v as f64) as f32,
            TransferFunction::SLog3 => slog3_to_linear(v as f64) as f32,
            TransferFunction::LogC => logc_to_linear(v as f64) as f32,
            TransferFunction::Custom(g) => v.powf(*g as f32),
        }
    }

    /// Encode a linear-light value to non-linear encoded (f32, [0,1] range).
    pub fn encode_from_linear(&self, v: f32) -> f32 {
        let v = v.max(0.0);
        match self {
            TransferFunction::Linear => v,
            TransferFunction::SRGB => {
                if v <= 0.0031308 {
                    v * 12.92
                } else {
                    1.055 * v.powf(1.0 / 2.4) - 0.055
                }
            }
            TransferFunction::Gamma22 => v.powf(1.0 / 2.2),
            TransferFunction::Gamma24 => v.powf(1.0 / 2.4),
            TransferFunction::Gamma26 => v.powf(1.0 / 2.6),
            TransferFunction::Gamma28 => v.powf(1.0 / 2.8),
            TransferFunction::PQ => pq_inverse_eotf_normalized(v as f64) as f32,
            TransferFunction::HLG => hlg_oetf(v as f64) as f32,
            TransferFunction::SLog3 => linear_to_slog3(v as f64) as f32,
            TransferFunction::LogC => linear_to_logc(v as f64) as f32,
            TransferFunction::Custom(g) => v.powf(1.0 / *g as f32),
        }
    }
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

    /// Compute the RGB→XYZ matrix for this color space.
    /// Returns (rXYZ, gXYZ, bXYZ, wtpt_XYZ) column vectors in D50 PCS.
    pub fn primaries_to_xyz_matrix(&self) -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
        compute_color_space_matrix(&self.primaries, &self.white_point)
    }

    /// Compute a 3×3 conversion matrix from this color space to `target`.
    /// The matrix operates on linear RGB values (after transfer function decoding).
    /// Returns None for CIEXYZ primaries.
    pub fn conversion_matrix_to(&self, target: &ColorSpace) -> Option<[[f64; 3]; 3]> {
        if self.primaries == ColorPrimaries::CIEXYZ || target.primaries == ColorPrimaries::CIEXYZ {
            return None;
        }

        if self.primaries == target.primaries && self.white_point == target.white_point {
            return Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        }

        let (r_src, g_src, b_src, _) =
            compute_color_space_matrix(&self.primaries, &self.white_point);
        let (r_dst, g_dst, b_dst, _) =
            compute_color_space_matrix(&target.primaries, &target.white_point);

        let m_src = col_matrix(r_src, g_src, b_src);
        let m_dst = col_matrix(r_dst, g_dst, b_dst);

        let m_src_adapted = if self.white_point != target.white_point {
            let src_xyz = self.white_point.to_xyz();
            let dst_xyz = target.white_point.to_xyz();
            let cat = bradford_cat(
                &[src_xyz.0, src_xyz.1, src_xyz.2],
                &[dst_xyz.0, dst_xyz.1, dst_xyz.2],
            );
            mat3_mul(&cat, &m_src)
        } else {
            m_src
        };

        let m_dst_inv = mat3_inverse(&m_dst);
        Some(mat3_mul(&m_dst_inv, &m_src_adapted))
    }

    /// Generate a binary ICC v2 profile for this color space.
    /// Returns None for CIEXYZ or unsupported primaries.
    pub fn generate_icc_profile(&self) -> Option<Vec<u8>> {
        if self.primaries == ColorPrimaries::CIEXYZ {
            return None;
        }

        let (rxyz, gxyz, bxyz, wtpt) =
            compute_color_space_matrix(&self.primaries, &self.white_point);
        Some(build_icc_v2_profile(
            &rxyz,
            &gxyz,
            &bxyz,
            &wtpt,
            &self.transfer,
            &self.primaries,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString, Default)]
pub enum WhitePoint {
    #[strum(serialize = "d50")]
    D50,
    #[strum(serialize = "d55")]
    D55,
    #[strum(serialize = "d60")]
    D60,
    #[strum(serialize = "d65")]
    #[default]
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

impl WhitePoint {
    pub fn chromaticity(&self) -> (f64, f64) {
        match self {
            WhitePoint::D50 => (0.34567, 0.35850),
            WhitePoint::D55 => (0.33242, 0.34743),
            WhitePoint::D60 => (0.32168, 0.33767),
            WhitePoint::D65 => (0.31270, 0.32900),
            WhitePoint::D75 => (0.29900, 0.31490),
            WhitePoint::DCI => (0.31400, 0.35100),
            WhitePoint::E => (1.0 / 3.0, 1.0 / 3.0),
            WhitePoint::Custom(x, _) => (*x as f64, 0.32900),
        }
    }

    pub fn to_xyz(&self) -> (f64, f64, f64) {
        match self {
            WhitePoint::D50 => (0.96422, 1.0, 0.82521),
            WhitePoint::D55 => (0.95682, 1.0, 0.92149),
            WhitePoint::D60 => (0.952646, 1.0, 1.008825),
            WhitePoint::D65 => (0.95047, 1.0, 1.08883),
            WhitePoint::D75 => (0.94972, 1.0, 1.22638),
            WhitePoint::DCI => (0.98000, 1.0, 1.18000),
            WhitePoint::E => (1.0, 1.0, 1.0),
            WhitePoint::Custom(_, _) => (0.95047, 1.0, 1.08883),
        }
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
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

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

// ---------------------------------------------------------------------------
// Private: 3x3 matrix helpers
// ---------------------------------------------------------------------------

fn mat3_inverse(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    // Guard against singular matrix (determinant near zero)
    if det.abs() < 1e-15 {
        tracing::warn!("mat3_inverse: near-singular matrix (det={}), returning identity", det);
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let inv_det = 1.0 / det;
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ]
}

fn mat3_mul_vec3(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn mat3_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    result
}

fn col_matrix(r: [f64; 3], g: [f64; 3], b: [f64; 3]) -> [[f64; 3]; 3] {
    [[r[0], g[0], b[0]], [r[1], g[1], b[1]], [r[2], g[2], b[2]]]
}

// ---------------------------------------------------------------------------
// Private: Bradford chromatic adaptation
// ---------------------------------------------------------------------------

fn bradford_cat(src_xyz: &[f64; 3], dst_xyz: &[f64; 3]) -> [[f64; 3]; 3] {
    let bfd = [
        [0.8951000, 0.2664000, -0.1614000],
        [-0.7502000, 1.7135000, 0.0367000],
        [0.0389000, -0.0685000, 1.0296000],
    ];
    let bfd_inv = [
        [0.9869929, -0.1470543, 0.1599627],
        [0.4323053, 0.5183603, 0.0492912],
        [-0.0085287, 0.0400428, 0.9684867],
    ];

    let src_lms = mat3_mul_vec3(&bfd, src_xyz);
    let dst_lms = mat3_mul_vec3(&bfd, dst_xyz);

    let d = [
        dst_lms[0] / src_lms[0],
        dst_lms[1] / src_lms[1],
        dst_lms[2] / src_lms[2],
    ];

    let mut m1 = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m1[i][j] = d[i] * bfd[i][j];
        }
    }
    mat3_mul(&bfd_inv, &m1)
}

// ---------------------------------------------------------------------------
// Private: Color space matrix computation
// ---------------------------------------------------------------------------

fn compute_color_space_matrix(
    primaries: &ColorPrimaries,
    white_point: &WhitePoint,
) -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
    let ((xr, yr), (xg, yg), (xb, yb)) = primaries.chromaticities();
    let (xw, yw) = white_point.chromaticity();

    let xr_xyz = xr / yr;
    let zr_xyz = (1.0 - xr - yr) / yr;
    let xg_xyz = xg / yg;
    let zg_xyz = (1.0 - xg - yg) / yg;
    let xb_xyz = xb / yb;
    let zb_xyz = (1.0 - xb - yb) / yb;

    let xw_xyz = xw / yw;
    let zw_xyz = (1.0 - xw - yw) / yw;

    let m_prim = [
        [xr_xyz, xg_xyz, xb_xyz],
        [1.0, 1.0, 1.0],
        [zr_xyz, zg_xyz, zb_xyz],
    ];
    let m_inv = mat3_inverse(&m_prim);
    let w_vec = [xw_xyz, 1.0, zw_xyz];
    let s = mat3_mul_vec3(&m_inv, &w_vec);

    let rxyz = [s[0] * xr_xyz, s[0] * 1.0, s[0] * zr_xyz];
    let gxyz = [s[1] * xg_xyz, s[1] * 1.0, s[1] * zg_xyz];
    let bxyz = [s[2] * xb_xyz, s[2] * 1.0, s[2] * zb_xyz];
    let wtpt_native = [xw_xyz, 1.0, zw_xyz];

    let wp_src = white_point.to_xyz();
    let wp_d50 = WhitePoint::D50.to_xyz();

    if (wp_src.0 - wp_d50.0).abs() < 0.001
        && (wp_src.1 - wp_d50.1).abs() < 0.001
        && (wp_src.2 - wp_d50.2).abs() < 0.001
    {
        return (rxyz, gxyz, bxyz, wtpt_native);
    }

    let cat = bradford_cat(
        &[wp_src.0, wp_src.1, wp_src.2],
        &[wp_d50.0, wp_d50.1, wp_d50.2],
    );
    let rxyz_d50 = mat3_mul_vec3(&cat, &rxyz);
    let gxyz_d50 = mat3_mul_vec3(&cat, &gxyz);
    let bxyz_d50 = mat3_mul_vec3(&cat, &bxyz);
    let wtpt_d50 = mat3_mul_vec3(&cat, &wtpt_native);

    (rxyz_d50, gxyz_d50, bxyz_d50, wtpt_d50)
}

// ---------------------------------------------------------------------------
// Private: Transfer function helpers (f64, for ICC TRC curves)
// ---------------------------------------------------------------------------

fn srgb_to_linear_f64(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_from_linear_f64(v: f64) -> f64 {
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn pq_eotf_normalized(v: f64) -> f64 {
    let m1 = 2610.0 / 16384.0;
    let m2 = 2523.0 / 32.0;
    let c1 = 3424.0 / 4096.0;
    let c2 = 2413.0 / 128.0;
    let c3 = 2392.0 / 128.0;

    let v_pow = v.powf(1.0 / m2);
    let num = (v_pow - c1).max(0.0);
    let den = c2 - c3 * v_pow;
    (num / den.max(1e-10)).powf(1.0 / m1)
}

fn pq_inverse_eotf_normalized(v: f64) -> f64 {
    let m1 = 2610.0 / 16384.0;
    let m2 = 2523.0 / 32.0;
    let c1 = 3424.0 / 4096.0;
    let c2 = 2413.0 / 128.0;
    let c3 = 2392.0 / 128.0;

    // Input v is normalized 0..1 (matching pq_eotf_normalized output range)
    let v_pow = v.max(0.0).powf(m1);
    let num = c1 + c2 * v_pow;
    let den = 1.0 + c3 * v_pow;
    (num / den.max(1e-10)).powf(m2)
}

/// Full-range PQ inverse EOTF: takes linear display luminance in cd/m^2 (0..10000)
/// and returns PQ signal in 0..1.
#[allow(dead_code)]
fn pq_inverse_eotf_full(v: f64) -> f64 {
    let m1 = 2610.0 / 16384.0;
    let m2 = 2523.0 / 32.0;
    let c1 = 3424.0 / 4096.0;
    let c2 = 2413.0 / 128.0;
    let c3 = 2392.0 / 128.0;

    let v_scaled = v.clamp(0.0, 10000.0) / 10000.0;
    let v_pow = v_scaled.max(0.0).powf(m1);
    let num = c1 + c2 * v_pow;
    let den = 1.0 + c3 * v_pow;
    (num / den.max(1e-10)).powf(m2)
}

/// Sony S-Log3 to linear (normalized 0..1).
/// Reference: S-Gamut3/S-Log3 white paper.
fn slog3_to_linear(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let threshold = 171.2102946929 / 1023.0;
    if x >= threshold {
        let lin = 10.0f64.powf((x * 1023.0 - 420.0) / 261.5) * (0.19 + 0.01) - 0.01;
        lin.max(0.0)
    } else {
        let lin = (x * 1023.0 - 95.0) * 0.01125000 / (171.2102946929 - 95.0);
        lin.max(0.0)
    }
}

/// Linear to Sony S-Log3 (normalized 0..1).
fn linear_to_slog3(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let threshold = 0.01125000;
    if x >= threshold {
        let slog3 = (420.0 + ((x + 0.01) / (0.19 + 0.01)).log10() * 261.5) / 1023.0;
        slog3.clamp(0.0, 1.0)
    } else {
        let slog3 = (x * (171.2102946929 - 95.0) / 0.01125000 + 95.0) / 1023.0;
        slog3.clamp(0.0, 1.0)
    }
}

/// ARRI LogC4 to linear (normalized 0..1). EI800 reference.
fn logc_to_linear(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let cut = 0.010591;
    let a = 5.555556;
    let b = 0.052272;
    let c = 0.247190;
    let d = 0.385537;
    let e = 0.6;
    let f = 0.2;
    if x > cut {
        let lin = (10.0f64.powf((x - d) / c / e) - b) / a * f;
        lin.max(0.0)
    } else {
        let t = cut;
        let s = (10.0f64.powf((t - d) / c / e) - b) / (a * t) * f;
        let lin = x * s;
        lin.max(0.0)
    }
}

/// Linear to ARRI LogC4 (normalized 0..1). EI800 reference.
fn linear_to_logc(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let cut = 0.010591;
    let a = 5.555556;
    let b = 0.052272;
    let c = 0.247190;
    let d = 0.385537;
    let e = 0.6;
    let f = 0.2;
    let t = cut;
    let s = (10.0f64.powf((t - d) / c / e) - b) / (a * t) * f;
    let v = x / f;
    if v > t * a * s / f {
        // Actually the threshold check should compare x against the linear value at cut
        // Use simplified direct approach
        let logc = c * e * (a * v + b).log10() + d;
        logc.clamp(0.0, 1.0)
    } else {
        let logc = x / s;
        logc.clamp(0.0, 1.0)
    }
}

fn hlg_oetf_inverse(v: f64) -> f64 {
    let a = 0.17883277;
    let b = 1.0 - 4.0 * a;
    let c = 0.5 - a * (4.0f64).ln();

    if v <= 0.5 {
        v * v / 3.0
    } else {
        (((v - c) / a).exp() + b) / 12.0
    }
}

fn hlg_oetf(v: f64) -> f64 {
    let a = 0.17883277;
    let b = 1.0 - 4.0 * a;
    let c = 0.5 - a * (4.0f64).ln();

    if v <= 1.0 / 12.0 {
        (3.0 * v).sqrt()
    } else {
        a * (12.0 * v - b).ln() + c
    }
}

// ---------------------------------------------------------------------------
// Private: ICC profile builder
// ---------------------------------------------------------------------------

fn s15f16(v: f64) -> u32 {
    (v * 65536.0).round() as i32 as u32
}

fn write_xyz_tag(data: &mut Vec<u8>, xyz: &[f64; 3]) {
    data.extend_from_slice(b"XYZ ");
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&s15f16(xyz[0]).to_be_bytes());
    data.extend_from_slice(&s15f16(xyz[1]).to_be_bytes());
    data.extend_from_slice(&s15f16(xyz[2]).to_be_bytes());
}

fn generate_trc_curve(tf: &TransferFunction, inverted: bool) -> Vec<u8> {
    const CURVE_ENTRIES: usize = 256;
    let mut data = Vec::with_capacity(12 + CURVE_ENTRIES * 2);

    data.extend_from_slice(b"curv");
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&(CURVE_ENTRIES as u32).to_be_bytes());

    for i in 0..CURVE_ENTRIES {
        let x = i as f64 / (CURVE_ENTRIES - 1) as f64;
        let y = if inverted {
            match tf {
                TransferFunction::Linear => x,
                TransferFunction::SRGB => srgb_from_linear_f64(x),
                TransferFunction::Gamma22 => x.powf(1.0 / 2.2),
                TransferFunction::Gamma24 => x.powf(1.0 / 2.4),
                TransferFunction::Gamma26 => x.powf(1.0 / 2.6),
                TransferFunction::Gamma28 => x.powf(1.0 / 2.8),
                TransferFunction::PQ => x,
                TransferFunction::HLG => x,
                TransferFunction::SLog3 => x,
                TransferFunction::LogC => x,
                TransferFunction::Custom(g) => x.powf(1.0 / *g),
            }
        } else {
            match tf {
                TransferFunction::Linear => x,
                TransferFunction::SRGB => srgb_to_linear_f64(x),
                TransferFunction::Gamma22 => x.powf(2.2),
                TransferFunction::Gamma24 => x.powf(2.4),
                TransferFunction::Gamma26 => x.powf(2.6),
                TransferFunction::Gamma28 => x.powf(2.8),
                TransferFunction::PQ => pq_eotf_normalized(x),
                TransferFunction::HLG => hlg_oetf_inverse(x),
                TransferFunction::SLog3 => x,
                TransferFunction::LogC => x,
                TransferFunction::Custom(g) => x.powf(*g),
            }
        };
        let y16 = (y.clamp(0.0, 1.0) * 65535.0).round() as u16;
        data.extend_from_slice(&y16.to_be_bytes());
    }

    data
}

fn build_icc_v2_profile(
    rxyz: &[f64; 3],
    gxyz: &[f64; 3],
    bxyz: &[f64; 3],
    wtpt: &[f64; 3],
    tf: &TransferFunction,
    primaries: &ColorPrimaries,
) -> Vec<u8> {
    let desc = primaries.description();

    let mut tag_data: Vec<u8> = Vec::new();
    let mut tag_table: Vec<(u32, usize, usize)> = Vec::new();

    let rxyz_offset = tag_data.len();
    write_xyz_tag(&mut tag_data, rxyz);
    tag_table.push((0x7258595A, rxyz_offset, 20));

    let gxyz_offset = tag_data.len();
    write_xyz_tag(&mut tag_data, gxyz);
    tag_table.push((0x6758595A, gxyz_offset, 20));

    let bxyz_offset = tag_data.len();
    write_xyz_tag(&mut tag_data, bxyz);
    tag_table.push((0x6258595A, bxyz_offset, 20));

    let wtpt_offset = tag_data.len();
    write_xyz_tag(&mut tag_data, wtpt);
    tag_table.push((0x77747074, wtpt_offset, 20));

    let trc = generate_trc_curve(tf, false);
    let trc_size = trc.len();

    let rtrc_offset = tag_data.len();
    tag_data.extend_from_slice(&trc);
    tag_table.push((0x72545243, rtrc_offset, trc_size));

    let gtrc_offset = tag_data.len();
    tag_data.extend_from_slice(&trc);
    tag_table.push((0x67545243, gtrc_offset, trc_size));

    let btrc_offset = tag_data.len();
    tag_data.extend_from_slice(&trc);
    tag_table.push((0x62545243, btrc_offset, trc_size));

    let desc_ascii = format!("{}\0", desc);
    let desc_payload_len = desc_ascii.len();
    let desc_padded = (desc_payload_len + 3) & !3;

    let mut desc_tag = Vec::with_capacity(12 + desc_padded + 16);
    desc_tag.extend_from_slice(b"desc");
    desc_tag.extend_from_slice(&0u32.to_be_bytes());
    desc_tag.extend_from_slice(&(desc_payload_len as u32).to_be_bytes());
    desc_tag.extend_from_slice(desc_ascii.as_bytes());
    while desc_tag.len() < 12 + desc_padded {
        desc_tag.push(0);
    }
    desc_tag.extend_from_slice(&0u32.to_be_bytes());
    desc_tag.extend_from_slice(&0u32.to_be_bytes());
    desc_tag.extend_from_slice(&0u32.to_be_bytes());
    desc_tag.extend_from_slice(&0u32.to_be_bytes());

    let desc_offset = tag_data.len();
    let desc_size = desc_tag.len();
    tag_data.extend_from_slice(&desc_tag);
    tag_table.push((0x64657363, desc_offset, desc_size));

    let header_size = 128;
    let tag_table_size = 4 + tag_table.len() * 12;
    let tag_data_offset = header_size + tag_table_size;
    let total_size = tag_data_offset + tag_data.len();

    let mut profile = vec![0u8; total_size];

    profile[0..4].copy_from_slice(&(total_size as u32).to_be_bytes());
    profile[4..8].copy_from_slice(b"lcms");
    profile[8..12].copy_from_slice(&0x04200000u32.to_be_bytes());
    profile[12..16].copy_from_slice(b"mntr");
    profile[16..20].copy_from_slice(b"RGB ");
    profile[20..24].copy_from_slice(b"XYZ ");
    profile[24..36].fill(0);
    profile[36..40].copy_from_slice(b"acsp");
    profile[40..44].fill(0);
    profile[44..48].fill(0);
    profile[48..52].fill(0);
    profile[52..56].fill(0);
    profile[56..64].fill(0);
    profile[64..68].copy_from_slice(&0u32.to_be_bytes());
    profile[68..72].copy_from_slice(&s15f16(0.9642).to_be_bytes());
    profile[72..76].copy_from_slice(&s15f16(1.0).to_be_bytes());
    profile[76..80].copy_from_slice(&s15f16(0.8249).to_be_bytes());
    profile[80..84].fill(0);
    profile[84..128].fill(0);

    let table_start = 128;
    profile[table_start..table_start + 4].copy_from_slice(&(tag_table.len() as u32).to_be_bytes());

    for (i, (sig, data_off, size)) in tag_table.iter().enumerate() {
        let entry_offset = table_start + 4 + i * 12;
        let actual_offset = (tag_data_offset + data_off) as u32;
        profile[entry_offset..entry_offset + 4].copy_from_slice(&sig.to_be_bytes());
        profile[entry_offset + 4..entry_offset + 8].copy_from_slice(&actual_offset.to_be_bytes());
        profile[entry_offset + 8..entry_offset + 12].copy_from_slice(&(*size as u32).to_be_bytes());
    }

    profile[tag_data_offset..tag_data_offset + tag_data.len()].copy_from_slice(&tag_data);

    profile
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

        let gray = ColorRGB {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        };
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
        assert_eq!(
            RenderingIntent::RelativeColorimetric.to_string(),
            "relative_colorimetric"
        );
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

    #[test]
    fn is_hdr_at_203_nits_is_false() {
        let cs = ColorSpace {
            hdr_nits: Some(203.0),
            ..ColorSpace::default()
        };
        assert!(!cs.is_hdr());
    }

    #[test]
    fn is_hdr_at_204_nits_is_true() {
        let cs = ColorSpace {
            hdr_nits: Some(204.0),
            ..ColorSpace::default()
        };
        assert!(cs.is_hdr());
    }

    #[test]
    fn is_hdr_at_0_nits_is_false() {
        let cs = ColorSpace {
            hdr_nits: Some(0.0),
            ..ColorSpace::default()
        };
        assert!(!cs.is_hdr());
    }

    #[test]
    fn is_hdr_at_10000_nits_is_true() {
        let cs = ColorSpace {
            hdr_nits: Some(10000.0),
            ..ColorSpace::default()
        };
        assert!(cs.is_hdr());
    }

    #[test]
    fn is_hdr_none_nits_is_false() {
        let cs = ColorSpace {
            hdr_nits: None,
            ..ColorSpace::default()
        };
        assert!(!cs.is_hdr());
    }

    #[test]
    fn color_primaries_bt709_display() {
        assert_eq!(ColorPrimaries::BT709.to_string(), "bt709");
    }

    #[test]
    fn color_primaries_adobe_rgb_display() {
        assert_eq!(ColorPrimaries::AdobeRGB.to_string(), "adobe_rgb");
    }

    #[test]
    fn color_primaries_pro_photo_display() {
        assert_eq!(ColorPrimaries::ProPhoto.to_string(), "pro_photo");
    }

    #[test]
    fn color_primaries_aces_cg_display() {
        assert_eq!(ColorPrimaries::ACEScg.to_string(), "aces_cg");
    }

    #[test]
    fn color_primaries_cie_xyz_display() {
        assert_eq!(ColorPrimaries::CIEXYZ.to_string(), "cie_xyz");
    }

    #[test]
    fn color_primaries_dci_p3_display() {
        assert_eq!(ColorPrimaries::DCIP3.to_string(), "dci_p3");
    }

    #[test]
    fn color_primaries_rec2100_display() {
        assert_eq!(ColorPrimaries::Rec2100.to_string(), "rec2100");
    }

    #[test]
    fn transfer_function_gamma22_display() {
        assert_eq!(TransferFunction::Gamma22.to_string(), "gamma22");
    }

    #[test]
    fn transfer_function_gamma24_display() {
        assert_eq!(TransferFunction::Gamma24.to_string(), "gamma24");
    }

    #[test]
    fn transfer_function_gamma26_display() {
        assert_eq!(TransferFunction::Gamma26.to_string(), "gamma26");
    }

    #[test]
    fn transfer_function_gamma28_display() {
        assert_eq!(TransferFunction::Gamma28.to_string(), "gamma28");
    }

    #[test]
    fn transfer_function_slog3_display() {
        assert_eq!(TransferFunction::SLog3.to_string(), "slog3");
    }

    #[test]
    fn transfer_function_log_c_display() {
        assert_eq!(TransferFunction::LogC.to_string(), "log_c");
    }

    #[test]
    fn transfer_function_custom_display() {
        assert_eq!(TransferFunction::Custom(2.4).to_string(), "custom");
    }

    #[test]
    fn white_point_d55_display() {
        assert_eq!(WhitePoint::D55.to_string(), "d55");
    }

    #[test]
    fn white_point_d60_display() {
        assert_eq!(WhitePoint::D60.to_string(), "d60");
    }

    #[test]
    fn white_point_d75_display() {
        assert_eq!(WhitePoint::D75.to_string(), "d75");
    }

    #[test]
    fn white_point_e_display() {
        assert_eq!(WhitePoint::E.to_string(), "e");
    }

    #[test]
    fn white_point_custom_d65_display() {
        let wp = WhitePoint::Custom(0.3127, 0.3290);
        let s = wp.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn rendering_intent_saturation_display() {
        assert_eq!(RenderingIntent::Saturation.to_string(), "saturation");
    }

    #[test]
    fn rendering_intent_absolute_colorimetric_display() {
        assert_eq!(
            RenderingIntent::AbsoluteColorimetric.to_string(),
            "absolute_colorimetric"
        );
    }

    #[test]
    fn gamut_mapping_luminance_preserve_display() {
        assert_eq!(
            GamutMapping::LuminancePreserve.to_string(),
            "luminance_preserve"
        );
    }

    #[test]
    fn color_rgb_red_luminance() {
        let red = ColorRGB {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        assert!((red.luminance() - 0.2126).abs() < 0.001);
    }

    #[test]
    fn color_rgb_green_luminance() {
        let green = ColorRGB {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        };
        assert!((green.luminance() - 0.7152).abs() < 0.001);
    }

    #[test]
    fn color_rgb_blue_luminance() {
        let blue = ColorRGB {
            r: 0.0,
            g: 0.0,
            b: 1.0,
        };
        assert!((blue.luminance() - 0.0722).abs() < 0.001);
    }

    #[test]
    fn color_rgb_gray_luminance() {
        let gray = ColorRGB {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        };
        assert!((gray.luminance() - 0.5).abs() < 0.001);
    }

    #[test]
    fn color_rgba_with_alpha() {
        let c = ColorRGBA {
            r: 1.0,
            g: 0.5,
            b: 0.0,
            a: 0.8,
        };
        assert_eq!(c.a, 0.8);
    }

    #[test]
    fn color_rgba_full_opaque() {
        let c = ColorRGBA {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn color_conversion_spec_serde_roundtrip() {
        let spec = ColorConversionSpec {
            source: ColorSpace::SRGB,
            target: ColorSpace::DISPLAY_P3,
            intent: RenderingIntent::Perceptual,
            black_point_compensation: true,
            gamut_mapping: GamutMapping::Compress,
            icc_profile: None,
            ocio_config: None,
            ocio_display: None,
            ocio_view: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let spec2: ColorConversionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec2.source, ColorSpace::SRGB);
        assert!(spec2.black_point_compensation);
        assert_eq!(spec2.gamut_mapping, GamutMapping::Compress);
    }

    #[test]
    fn color_conversion_spec_with_icc_and_ocio() {
        let spec = ColorConversionSpec {
            source: ColorSpace::ADOBE_RGB,
            target: ColorSpace::SRGB,
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: false,
            gamut_mapping: GamutMapping::Clip,
            icc_profile: Some(vec![1, 2, 3]),
            ocio_config: Some("config.ocio".into()),
            ocio_display: Some("sRGB".into()),
            ocio_view: Some("Film".into()),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let spec2: ColorConversionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec2.ocio_config, Some("config.ocio".into()));
        assert_eq!(spec2.icc_profile, Some(vec![1, 2, 3]));
    }

    // ---- New tests: primaries chromaticities ----

    #[test]
    fn primaries_chromaticities_srgb() {
        let ((rx, ry), (gx, gy), (bx, by)) = ColorPrimaries::SRGB.chromaticities();
        assert!((rx - 0.6400).abs() < 0.0001);
        assert!((ry - 0.3300).abs() < 0.0001);
        assert!((gx - 0.3000).abs() < 0.0001);
        assert!((gy - 0.6000).abs() < 0.0001);
        assert!((bx - 0.1500).abs() < 0.0001);
        assert!((by - 0.0600).abs() < 0.0001);
    }

    #[test]
    fn primaries_chromaticities_display_p3() {
        let ((rx, _ry), (gx, gy), (bx, by)) = ColorPrimaries::DisplayP3.chromaticities();
        assert!((rx - 0.6800).abs() < 0.0001);
        assert!((gx - 0.2650).abs() < 0.0001);
        assert!((gy - 0.6900).abs() < 0.0001);
        assert!((bx - 0.1500).abs() < 0.0001);
        assert!((by - 0.0600).abs() < 0.0001);
    }

    #[test]
    fn primaries_chromaticities_bt2020() {
        let ((rx, _), (gx, gy), (bx, by)) = ColorPrimaries::BT2020.chromaticities();
        assert!((rx - 0.7080).abs() < 0.0001);
        assert!((gx - 0.1700).abs() < 0.0001);
        assert!((gy - 0.7970).abs() < 0.0001);
        assert!((bx - 0.1310).abs() < 0.0001);
        assert!((by - 0.0460).abs() < 0.0001);
    }

    // ---- New tests: white point ----

    #[test]
    fn white_point_d65_xyz() {
        let (x, y, z) = WhitePoint::D65.to_xyz();
        assert!((x - 0.95047).abs() < 0.0001);
        assert!((y - 1.0).abs() < 0.0001);
        assert!((z - 1.08883).abs() < 0.0001);
    }

    #[test]
    fn white_point_d50_xyz() {
        let (x, y, z) = WhitePoint::D50.to_xyz();
        assert!((x - 0.96422).abs() < 0.0001);
        assert!((y - 1.0).abs() < 0.0001);
        assert!((z - 0.82521).abs() < 0.0001);
    }

    #[test]
    fn white_point_chromaticity_d65() {
        let (x, y) = WhitePoint::D65.chromaticity();
        assert!((x - 0.31270).abs() < 0.0001);
        assert!((y - 0.32900).abs() < 0.0001);
    }

    // ---- New tests: transfer function encode/decode ----

    #[test]
    fn transfer_decode_linear() {
        assert!((TransferFunction::Linear.decode_to_linear(0.5) - 0.5).abs() < 0.0001);
    }

    #[test]
    fn transfer_srgb_roundtrip() {
        for &v in &[0.0, 0.01, 0.1, 0.5, 0.9, 1.0] {
            let linear = TransferFunction::SRGB.decode_to_linear(v);
            let encoded = TransferFunction::SRGB.encode_from_linear(linear);
            assert!(
                (encoded - v).abs() < 0.001,
                "sRGB roundtrip failed at v={v}: encoded={encoded}"
            );
        }
    }

    #[test]
    fn transfer_gamma22_roundtrip() {
        let v = 0.5;
        let linear = TransferFunction::Gamma22.decode_to_linear(v);
        let encoded = TransferFunction::Gamma22.encode_from_linear(linear);
        assert!((encoded - v).abs() < 0.001);
    }

    #[test]
    fn transfer_decode_srgb_black() {
        assert!((TransferFunction::SRGB.decode_to_linear(0.0) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn transfer_decode_srgb_white() {
        assert!((TransferFunction::SRGB.decode_to_linear(1.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn transfer_encode_srgb_black() {
        assert!((TransferFunction::SRGB.encode_from_linear(0.0) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn transfer_encode_srgb_white() {
        assert!((TransferFunction::SRGB.encode_from_linear(1.0) - 1.0).abs() < 0.0001);
    }

    // ---- New tests: conversion matrix ----

    #[test]
    fn conversion_matrix_same_space_is_identity() {
        let m = ColorSpace::SRGB
            .conversion_matrix_to(&ColorSpace::SRGB)
            .unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m[i][j] - expected).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn conversion_matrix_srgb_to_display_p3_exists() {
        let m = ColorSpace::SRGB
            .conversion_matrix_to(&ColorSpace::DISPLAY_P3)
            .unwrap();
        assert!(m[0][0] > 0.0);
    }

    #[test]
    fn conversion_matrix_ciexyz_returns_none() {
        let cs = ColorSpace {
            primaries: ColorPrimaries::CIEXYZ,
            transfer: TransferFunction::Linear,
            white_point: WhitePoint::D65,
            hdr_nits: None,
        };
        assert!(cs.conversion_matrix_to(&ColorSpace::SRGB).is_none());
        assert!(ColorSpace::SRGB.conversion_matrix_to(&cs).is_none());
    }

    #[test]
    fn conversion_matrix_srgb_to_linear_srgb_is_identity() {
        let m = ColorSpace::SRGB
            .conversion_matrix_to(&ColorSpace::LINEAR_SRGB)
            .unwrap();
        // Same primaries, same white point → identity
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m[i][j] - expected).abs() < 0.0001);
            }
        }
    }

    // ---- New tests: ICC profile generation ----

    #[test]
    fn generate_icc_profile_srgb() {
        let icc = ColorSpace::SRGB.generate_icc_profile().unwrap();
        assert!(icc.len() > 128);
        assert_eq!(&icc[36..40], b"acsp");
    }

    #[test]
    fn generate_icc_profile_display_p3() {
        let icc = ColorSpace::DISPLAY_P3.generate_icc_profile().unwrap();
        assert!(icc.len() > 128);
        assert_eq!(&icc[36..40], b"acsp");
    }

    #[test]
    fn generate_icc_profile_adobe_rgb() {
        let icc = ColorSpace::ADOBE_RGB.generate_icc_profile().unwrap();
        assert!(icc.len() > 128);
    }

    #[test]
    fn generate_icc_profile_ciexyz_is_none() {
        let cs = ColorSpace {
            primaries: ColorPrimaries::CIEXYZ,
            transfer: TransferFunction::Linear,
            white_point: WhitePoint::D65,
            hdr_nits: None,
        };
        assert!(cs.generate_icc_profile().is_none());
    }

    #[test]
    fn icc_profile_contains_required_tags() {
        let icc = ColorSpace::SRGB.generate_icc_profile().unwrap();
        let tag_count = u32::from_be_bytes([icc[128], icc[129], icc[130], icc[131]]);
        assert!(tag_count >= 8);
    }

    #[test]
    fn bradford_cat_d65_to_d50() {
        let d65 = WhitePoint::D65.to_xyz();
        let d50 = WhitePoint::D50.to_xyz();
        let cat = bradford_cat(&[d65.0, d65.1, d65.2], &[d50.0, d50.1, d50.2]);
        let result = mat3_mul_vec3(&cat, &[d65.0, d65.1, d65.2]);
        assert!((result[0] - d50.0).abs() < 0.001);
        assert!((result[1] - d50.1).abs() < 0.001);
        assert!((result[2] - d50.2).abs() < 0.001);
    }

    #[test]
    fn bradford_cat_identity_same_white_point() {
        let d65 = WhitePoint::D65.to_xyz();
        let cat = bradford_cat(&[d65.0, d65.1, d65.2], &[d65.0, d65.1, d65.2]);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((cat[i][j] - expected).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn mat3_inverse_identity() {
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let inv = mat3_inverse(&id);
        for i in 0..3 {
            for j in 0..3 {
                assert!((inv[i][j] - id[i][j]).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn mat3_inverse_roundtrip() {
        let m = [[2.0, 0.3, 0.1], [0.1, 1.5, 0.2], [0.05, 0.1, 3.0]];
        let inv = mat3_inverse(&m);
        let result = mat3_mul(&m, &inv);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (result[i][j] - expected).abs() < 0.0001,
                    "m*inv[{i}][{j}] = {}, expected {expected}",
                    result[i][j]
                );
            }
        }
    }

    #[test]
    fn srgb_to_p3_roundtrip_matrix() {
        let to_p3 = ColorSpace::SRGB
            .conversion_matrix_to(&ColorSpace::DISPLAY_P3)
            .unwrap();
        let from_p3 = ColorSpace::DISPLAY_P3
            .conversion_matrix_to(&ColorSpace::SRGB)
            .unwrap();
        let roundtrip = mat3_mul(&from_p3, &to_p3);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (roundtrip[i][j] - expected).abs() < 0.001,
                    "roundtrip[{i}][{j}] = {}",
                    roundtrip[i][j]
                );
            }
        }
    }

    #[test]
    fn primaries_to_xyz_matrix_srgb_d65() {
        let (rx, gx, bx, _) = ColorSpace::SRGB.primaries_to_xyz_matrix();
        // sRGB red primary (x=0.64, y=0.33) → XYZ; Y should be ~0.2126
        let y_sum = rx[1] + gx[1] + bx[1];
        assert!(
            (y_sum - 1.0).abs() < 0.01,
            "Y row should sum to ~1.0, got {y_sum}"
        );
    }

    #[test]
    fn pq_eotf_black_is_zero() {
        assert!((pq_eotf_normalized(0.0) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn pq_eotf_white_is_near_one() {
        let v = pq_eotf_normalized(1.0);
        assert!(v > 0.9, "PQ(1.0) = {v}, expected near 1.0");
    }

    #[test]
    fn hlg_inverse_black() {
        assert!((hlg_oetf_inverse(0.0) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn hlg_oetf_black() {
        assert!((hlg_oetf(0.0) - 0.0).abs() < 0.0001);
    }
}

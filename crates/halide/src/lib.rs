use photopipeline_core::{ColorPrimaries, ColorSpace, TransferFunction};

#[cfg(halide_found)]
mod ffi {
    extern "C" {
        pub fn halide_colorspace_convert(
            input: *const f32,
            width: i32,
            height: i32,
            channels: i32,
            src_primaries: i32,
            src_transfer: i32,
            dst_primaries: i32,
            dst_transfer: i32,
            output: *mut f32,
        ) -> i32;

        pub fn halide_resize(
            input: *const f32,
            in_w: i32,
            in_h: i32,
            channels: i32,
            output: *mut f32,
            out_w: i32,
            out_h: i32,
            filter_type: i32,
        ) -> i32;

        pub fn halide_tonemap(
            input: *const f32,
            width: i32,
            height: i32,
            channels: i32,
            output: *mut f32,
            algorithm: i32,
            max_luminance: f32,
        ) -> i32;
    }
}

fn primaries_to_halide(cp: ColorPrimaries) -> i32 {
    match cp {
        ColorPrimaries::SRGB => 0,
        ColorPrimaries::BT709 => 1,
        ColorPrimaries::BT2020 => 2,
        ColorPrimaries::DisplayP3 => 3,
        ColorPrimaries::AdobeRGB => 4,
        ColorPrimaries::ACES => 5,
        ColorPrimaries::ACEScg => 6,
        _ => 0,
    }
}

fn transfer_to_halide(tf: TransferFunction) -> i32 {
    match tf {
        TransferFunction::SRGB => 0,
        TransferFunction::Linear => 1,
        TransferFunction::Gamma22 => 2,
        TransferFunction::Gamma24 => 3,
        TransferFunction::PQ => 4,
        TransferFunction::HLG => 5,
        _ => 0,
    }
}

pub struct HalideContext;

impl HalideContext {
    pub fn available() -> bool {
        cfg!(halide_found)
    }

    pub fn colorspace_convert(
        input: &[f32],
        width: u32,
        height: u32,
        channels: u32,
        src: &ColorSpace,
        dst: &ColorSpace,
    ) -> Result<Vec<f32>, String> {
        #[cfg(not(halide_found))]
        {
            let _ = (input, width, height, channels, src, dst);
            return Err("Halide runtime not available".into());
        }
        #[cfg(halide_found)]
        {
            let expected_len = width as usize * height as usize * channels as usize;
            if input.len() < expected_len {
                return Err(format!(
                    "input slice too short: {} < {}",
                    input.len(),
                    expected_len
                ));
            }
            let src_prim = primaries_to_halide(src.primaries);
            let src_tf = transfer_to_halide(src.transfer);
            let dst_prim = primaries_to_halide(dst.primaries);
            let dst_tf = transfer_to_halide(dst.transfer);

            let mut output = vec![0.0f32; expected_len];
            let ret = unsafe {
                ffi::halide_colorspace_convert(
                    input.as_ptr(),
                    width as i32,
                    height as i32,
                    channels as i32,
                    src_prim,
                    src_tf,
                    dst_prim,
                    dst_tf,
                    output.as_mut_ptr(),
                )
            };
            if ret != 0 {
                return Err(format!(
                    "halide_colorspace_convert returned error code {}",
                    ret
                ));
            }
            Ok(output)
        }
    }

    pub fn resize(
        input: &[f32],
        in_w: u32,
        in_h: u32,
        channels: u32,
        out_w: u32,
        out_h: u32,
        filter: &str,
    ) -> Result<Vec<f32>, String> {
        #[cfg(not(halide_found))]
        {
            let _ = (input, in_w, in_h, channels, out_w, out_h, filter);
            return Err("Halide runtime not available".into());
        }
        #[cfg(halide_found)]
        {
            let expected_in = in_w as usize * in_h as usize * channels as usize;
            if input.len() < expected_in {
                return Err(format!(
                    "input slice too short: {} < {}",
                    input.len(),
                    expected_in
                ));
            }
            let filter_code = match filter {
                "lanczos3" => 0,
                "bilinear" => 1,
                "nearest" => 2,
                _ => 0,
            };

            let expected_out = out_w as usize * out_h as usize * channels as usize;
            let mut output = vec![0.0f32; expected_out];
            let ret = unsafe {
                ffi::halide_resize(
                    input.as_ptr(),
                    in_w as i32,
                    in_h as i32,
                    channels as i32,
                    output.as_mut_ptr(),
                    out_w as i32,
                    out_h as i32,
                    filter_code,
                )
            };
            if ret != 0 {
                return Err(format!("halide_resize returned error code {}", ret));
            }
            Ok(output)
        }
    }

    pub fn tonemap(
        input: &[f32],
        width: u32,
        height: u32,
        channels: u32,
        algorithm: &str,
        max_luminance: f32,
    ) -> Result<Vec<f32>, String> {
        #[cfg(not(halide_found))]
        {
            let _ = (input, width, height, channels, algorithm, max_luminance);
            return Err("Halide runtime not available".into());
        }
        #[cfg(halide_found)]
        {
            let expected_len = width as usize * height as usize * channels as usize;
            if input.len() < expected_len {
                return Err(format!(
                    "input slice too short: {} < {}",
                    input.len(),
                    expected_len
                ));
            }
            let algo_code = match algorithm {
                "reinhard" => 0,
                "aces" => 1,
                _ => 0,
            };

            let mut output = vec![0.0f32; expected_len];
            let ret = unsafe {
                ffi::halide_tonemap(
                    input.as_ptr(),
                    width as i32,
                    height as i32,
                    channels as i32,
                    output.as_mut_ptr(),
                    algo_code,
                    max_luminance,
                )
            };
            if ret != 0 {
                return Err(format!("halide_tonemap returned error code {}", ret));
            }
            Ok(output)
        }
    }
}

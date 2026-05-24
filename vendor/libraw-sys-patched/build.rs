use std::env;
use std::path::PathBuf;

fn main() {
    let vcpkg_root = env::var("VCPKG_ROOT").unwrap_or_else(|_| "C:/vcpkg".to_string());
    let vcpkg_lib = format!("{}/installed/x64-windows/lib", vcpkg_root);
    let vcpkg_include = format!("{}/installed/x64-windows/include", vcpkg_root);

    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");

    // Link against vcpkg's x64-windows DLL import library for libraw_r
    println!("cargo:rustc-link-search=native={}", vcpkg_lib);
    println!("cargo:rustc-link-lib=raw_r");

    // libraw_r depends on lcms2 — keep static (pure C, no CRT init issues)
    println!("cargo:rustc-link-lib=static=lcms2");

    // All cfg flags: vcpkg's libraw 0.22.1 is full-featured
    println!("cargo:rustc-cfg=have_iparams_software");
    println!("cargo:rustc-cfg=have_iparams_xtrans");
    println!("cargo:rustc-cfg=have_iparams_xtrans_abs");
    println!("cargo:rustc-cfg=have_iparams_xmplen");
    println!("cargo:rustc-cfg=have_iparams_xmpdata");
    println!("cargo:rustc-cfg=have_output_params_sony_arw2_hack");
    println!("cargo:rustc-cfg=have_output_params_afd_noise_att");
    println!("cargo:rustc-cfg=have_output_params_afd_noise_thres");
    println!("cargo:rustc-cfg=have_output_params_afd_luminance_passes");
    println!("cargo:rustc-cfg=have_output_params_afd_chrominance_method");
    println!("cargo:rustc-cfg=have_output_params_afd_luminance_only");
    println!("cargo:rustc-cfg=have_output_params_no_auto_scale");
    println!("cargo:rustc-cfg=have_output_params_no_interpolation");
    println!("cargo:rustc-cfg=have_output_params_sraw_ycc");
    println!("cargo:rustc-cfg=have_output_params_force_foveon_x3f");
    println!("cargo:rustc-cfg=have_output_params_x3f_flags");
    println!("cargo:rustc-cfg=have_output_params_sony_arw2_options");
    println!("cargo:rustc-cfg=have_output_params_sony_arw2_posterization_thr");
    println!("cargo:rustc-cfg=have_output_params_coolscan_nef_gamma");
    println!("cargo:rustc-cfg=have_colordata_black_stat");
    println!("cargo:rustc-cfg=have_colordata_baseline_exposure");
    println!("cargo:rustc-cfg=have_colordata_olympus_sensor_calibration");
    println!("cargo:rustc-cfg=have_colordata_fuji_expo_mid_point_shift");
    println!("cargo:rustc-cfg=have_colordata_digital_back_color");
    println!("cargo:rustc-cfg=have_rawdata_ph1_rblack");
    println!("cargo:rustc-cfg=have_ph1");
    println!("cargo:rustc-cfg=have_ph1_black_off");
    println!("cargo:rustc-cfg=have_lensinfo");
    println!("cargo:rustc-cfg=have_nikonlens");
    println!("cargo:rustc-cfg=have_dnglens");
    println!("cargo:rustc-cfg=have_makernotes_lens");
    println!("cargo:rustc-cfg=have_dng_color");
    println!("cargo:rustc-cfg=have_canon_makernotes");
    println!("cargo:rustc-cfg=have_gps_info");
}

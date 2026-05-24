extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    libjxl_src::build();

    let out_dir = std::env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=wrapper.h");
    libjxl_src::print_cargo_link_from(&out_dir);

    let include_dir = format!("{}/include", out_dir);
    println!("cargo:include={}", include_dir);

    let vcpkg_root = env::var("VCPKG_ROOT").unwrap_or_else(|_| "C:/vcpkg".to_string());
    #[cfg(target_os = "windows")]
    let vcpkg_include = format!("{}/installed/x64-windows/include", vcpkg_root);
    #[cfg(not(target_os = "windows"))]
    let vcpkg_include = format!("{}/installed/include", vcpkg_root);

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I")
        .clang_arg(include_dir)
        .clang_arg("-I")
        .clang_arg(vcpkg_include)
        .allowlist_function("Jxl.*")
        .derive_default(true)
        .size_t_is_usize(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

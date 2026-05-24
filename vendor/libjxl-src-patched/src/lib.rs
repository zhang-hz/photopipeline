use std::fs;
use std::path::Path;

pub fn build() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let include_dir = format!("{}/include", out_dir);
    fs::create_dir_all(&include_dir).ok();

    let vcpkg_root = std::env::var("VCPKG_ROOT")
        .unwrap_or_else(|_| "C:/vcpkg".to_string());

    let vcpkg_include = format!("{}/installed/x64-windows/include", vcpkg_root);
    let jxl_include = format!("{}/jxl", vcpkg_include);
    let dst_link = format!("{}/jxl", include_dir);

    // Create symlink/junction: $OUT_DIR/include/jxl → VCPKG/include/jxl
    if Path::new(&jxl_include).exists() && !Path::new(&dst_link).exists() {
        symlink_or_copy(&jxl_include, &dst_link);
    }

    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:include={}", vcpkg_include);
}

pub fn out_dir() -> &'static str {
    std::env!("OUT_DIR")
}

pub fn print_cargo_link() {
    print_cargo_link_from(out_dir())
}

pub fn print_cargo_link_from(_dst: &str) {
    let vcpkg_root = std::env::var("VCPKG_ROOT")
        .unwrap_or_else(|_| "C:/vcpkg".to_string());

    let lib_dir = format!("{}/installed/x64-windows/lib", vcpkg_root);

    println!("cargo:rustc-link-search=native={}", lib_dir);

    // x64-windows DLL import libraries
    println!("cargo:rustc-link-lib=jxl");
    println!("cargo:rustc-link-lib=jxl_threads");
    println!("cargo:rustc-link-lib=jxl_cms");
    println!("cargo:rustc-link-lib=brotlicommon");
    println!("cargo:rustc-link-lib=brotlidec");
    println!("cargo:rustc-link-lib=brotlienc");
    println!("cargo:rustc-link-lib=hwy");

    // jxl_cms depends on lcms2 — keep static (pure C, no CRT init issues)
    println!("cargo:rustc-link-lib=static=lcms2");
}

#[cfg(windows)]
fn symlink_or_copy(src: &str, dst: &str) {
    // Try junction first, fall back to copy
    use std::process::Command;
    let result = Command::new("cmd")
        .args(&["/C", "mklink", "/J", dst, src])
        .output();
    if result.is_err() || !result.unwrap().status.success() {
        // Fall back to copying the directory
        copy_dir(src, dst);
    }
}

#[cfg(not(windows))]
fn symlink_or_copy(src: &str, dst: &str) {
    std::os::unix::fs::symlink(src, dst).unwrap_or_else(|_| {
        copy_dir(src, dst);
    });
}

fn copy_dir(src: &str, dst: &str) {
    if let Ok(entries) = fs::read_dir(src) {
        fs::create_dir_all(dst).ok();
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dst_path = Path::new(dst).join(entry.file_name());
            if src_path.is_dir() {
                copy_dir(&src_path.to_string_lossy(), &dst_path.to_string_lossy());
            } else {
                fs::copy(&src_path, &dst_path).ok();
            }
        }
    }
}

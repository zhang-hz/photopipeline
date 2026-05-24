fn main() {
    let vcpkg_root = std::env::var("VCPKG_ROOT").unwrap_or_else(|_| "C:/vcpkg".to_string());
    let lib_dir = format!("{}/installed/x64-windows/lib", vcpkg_root);

    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-env-changed=PHOTOPIPELINE_EXIFTOOL");
    println!("cargo:rustc-link-search=native={}", lib_dir);

    // libheif + deps (x64-windows DLL)
    println!("cargo:rustc-link-lib=heif");
    println!("cargo:rustc-link-lib=libx265");
    println!("cargo:rustc-link-lib=de265");

    // libjxl + deps (x64-windows DLL)
    println!("cargo:rustc-link-lib=jxl");
    println!("cargo:rustc-link-lib=jxl_threads");
    println!("cargo:rustc-link-lib=jxl_cms");
    println!("cargo:rustc-link-lib=brotlicommon");
    println!("cargo:rustc-link-lib=brotlidec");
    println!("cargo:rustc-link-lib=brotlienc");
    println!("cargo:rustc-link-lib=hwy");

    // libraw + deps (x64-windows DLL)
    println!("cargo:rustc-link-lib=raw_r");

    // lcms2 — keep static (pure C, no CRT init issues; lcms2-sys compiles from source)
    println!("cargo:rustc-link-lib=static=lcms2");

    // zlib (x64-windows DLL)
    println!("cargo:rustc-link-lib=z");

    // ExifTool embed path
    let exiftool_path = find_exiftool();
    println!("cargo:rustc-env=PHOTOPIPELINE_EXIFTOOL={}", exiftool_path);
    println!("cargo:rerun-if-changed=vendor/exiftool");
}

fn find_exiftool() -> String {
    if let Ok(p) = std::env::var("PHOTOPIPELINE_EXIFTOOL")
        && std::path::Path::new(&p).exists()
    {
        return p;
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let repo_root = std::path::Path::new(&manifest_dir).join("../..");

    #[cfg(target_os = "windows")]
    let candidates: &[(&str, &str)] = &[("exiftool.exe", "vendor/exiftool")];

    #[cfg(not(target_os = "windows"))]
    let candidates: &[(&str, &str)] = &[
        ("exiftool", "/usr/bin"),
        ("exiftool", "/usr/local/bin"),
        ("exiftool", "/opt/homebrew/bin"),
    ];

    for (name, dir) in candidates {
        let full = repo_root.join(dir).join(name);
        if full.exists() {
            return full.to_string_lossy().to_string();
        }
    }

    // scan vendor/exiftool subdirs (versioned dirs)
    let vendor_dir = repo_root.join("vendor/exiftool");
    if let Ok(entries) = std::fs::read_dir(&vendor_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let exe = path.join("exiftool.exe");
                if exe.exists() {
                    return exe.to_string_lossy().to_string();
                }
            }
        }
    }

    // fallback: system PATH
    "exiftool".to_string()
}

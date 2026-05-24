use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(halide_found)");
    println!("cargo::rustc-check-cfg=cfg(halide_generators_found)");
    let target_triple = std::env::var("TARGET").unwrap_or_else(|_| String::from("unknown"));
    let prebuilt_dir = Path::new("halide_generators")
        .join("prebuilt")
        .join(&target_triple);

    let halide_found = if prebuilt_dir.exists() {
        println!("cargo:rustc-link-search=native={}", prebuilt_dir.display());
        println!("cargo:rustc-link-lib=static=halide_pipeline");
        println!("cargo:rerun-if-changed={}", prebuilt_dir.display());
        println!(
            "cargo:warning=Halide prebuilt found at {}",
            prebuilt_dir.display()
        );
        true
    } else {
        match pkg_config::Config::new()
            .atleast_version("14.0")
            .probe("Halide")
        {
            Ok(lib) => {
                for path in &lib.link_paths {
                    println!("cargo:rustc-link-search=native={}", path.display());
                }
                for lib_name in &lib.libs {
                    println!("cargo:rustc-link-lib={}", lib_name);
                }
                true
            }
            Err(_) => {
                println!(
                    "cargo:warning=Halide not found via prebuilt or pkg-config; Halide features disabled"
                );
                false
            }
        }
    };

    if halide_found {
        println!("cargo:rustc-cfg=halide_found");
    }

    println!("cargo:rerun-if-changed=build.rs");

    let generators_dir = Path::new("halide_generators/build");
    if generators_dir.exists() {
        println!("cargo:rustc-cfg=halide_generators_found");
        println!("cargo:rerun-if-changed=halide_generators/build");
    }
}

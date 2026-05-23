fn main() {
    let halide_lib = pkg_config::Config::new()
        .atleast_version("14.0")
        .probe("Halide");

    match halide_lib {
        Ok(lib) => {
            for path in &lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
            for lib_name in &lib.libs {
                println!("cargo:rustc-link-lib={}", lib_name);
            }
            println!("cargo:rustc-cfg=halide_found");
            println!("cargo:rerun-if-changed=build.rs");
        }
        Err(_) => {
            println!("cargo:rustc-cfg=halide_not_found");
            println!("cargo:warning=Halide not found via pkg-config; Halide features disabled");
        }
    }

    let build_dir = std::path::Path::new("halide_generators/build");
    if build_dir.exists() {
        println!("cargo:rustc-cfg=halide_generators_found");
        println!("cargo:rerun-if-changed=../halide_generators/build");
    }
}

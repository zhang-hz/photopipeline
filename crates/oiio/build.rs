fn main() {
    let oiio_lib = pkg_config::Config::new()
        .atleast_version("2.4")
        .probe("OpenImageIO");

    match oiio_lib {
        Ok(lib) => {
            for path in &lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
            for lib_name in &lib.libs {
                println!("cargo:rustc-link-lib={}", lib_name);
            }
            println!("cargo:rustc-cfg=oiio_found");
        }
        Err(_) => {
            println!("cargo:rustc-cfg=oiio_not_found");
            println!("cargo:warning=OpenImageIO not found via pkg-config; OIIO features disabled");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}

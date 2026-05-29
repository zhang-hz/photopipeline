fn main() {
    // Generate proto Rust code directly into the source tree
    // so it's compiled as normal Rust without include!() macro issues.
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("src/proto_gen")
        .compile_protos(
            &["photopipeline/plugin.proto", "photopipeline/image.proto",
              "photopipeline/pipeline.proto", "photopipeline/batch.proto"],
            &["photopipeline"],
        )
        .expect("Failed to compile proto files");

    tauri_build::build();
}

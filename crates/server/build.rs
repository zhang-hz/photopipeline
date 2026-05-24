fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_include = find_protobuf_include();
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &[
                "../../proto/pipeline.proto",
                "../../proto/image.proto",
                "../../proto/batch.proto",
            ],
            &["../../proto/", proto_include.as_str()],
        )?;
    Ok(())
}

fn find_protobuf_include() -> String {
    if let Ok(dir) = std::env::var("PROTOC_INCLUDE")
        && std::path::Path::new(&dir).is_dir()
    {
        return dir;
    }
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_default()
        .replace('\\', "/");
    let candidates: [String; 4] = [
        "/opt/homebrew/include".to_string(),
        "/usr/local/include".to_string(),
        "C:/vcpkg/installed/x64-windows-static/include".to_string(),
        format!(
            "{}/Microsoft/WinGet/Packages/Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe/include",
            local_app_data
        ),
    ];
    for c in &candidates {
        let p = std::path::Path::new(c).join("google/protobuf/struct.proto");
        if p.exists() {
            return c.clone();
        }
    }
    ".".to_string()
}

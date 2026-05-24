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
    let mut candidates: Vec<String> = vec![
        "/opt/homebrew/include".to_string(),
        "/usr/local/include".to_string(),
        format!(
            "{}/Microsoft/WinGet/Packages/Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe/include",
            local_app_data
        ),
    ];
    // VCPKG_ROOT paths (try both dynamic and static triplets)
    if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        candidates.insert(0, format!("{}/installed/x64-windows/include", vcpkg_root));
        candidates.insert(
            1,
            format!("{}/installed/x64-windows-static/include", vcpkg_root),
        );
    } else {
        candidates.insert(0, "C:/vcpkg/installed/x64-windows/include".to_string());
        candidates.insert(
            1,
            "C:/vcpkg/installed/x64-windows-static/include".to_string(),
        );
    }
    for c in &candidates {
        let p = std::path::Path::new(c).join("google/protobuf/struct.proto");
        if p.exists() {
            return c.clone();
        }
    }
    ".".to_string()
}

mod commands;
mod grpc;
mod proto;

use grpc::GrpcClients;
use tauri::Manager;
use tokio::sync::Mutex;

/// Apply window vibrancy based on the current platform.
fn apply_vibrancy(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::apply_mica;
        if let Err(_e) = apply_mica(window, None) {
            use window_vibrancy::apply_acrylic;
            let _ = apply_acrylic(window, Some((18, 18, 18, 125)));
        }
    }

    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
        let _ = apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None);
    }

    #[cfg(target_os = "linux")]
    {
        eprintln!("[photopipeline] Linux: vibrancy not available, using CSS solid fallback");
    }

    let _ = window;
}

/// Locate the photopipeline backend binary.
fn find_backend_binary() -> String {
    let candidates = [
        "photopipeline".to_string(),
        "./photopipeline".to_string(),
        "../target/release/photopipeline".to_string(),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }

    "photopipeline".to_string()
}

/// Spawn the photopipeline serve process on localhost:50051.
fn spawn_backend(binary: &str) -> Result<std::process::Child, std::io::Error> {
    std::process::Command::new(binary)
        .args(["serve", "--port", "50051"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Apply window vibrancy
            let window = app.get_webview_window("main").expect("main window not found");
            apply_vibrancy(&window);

            // Spawn backend process (non-blocking)
            let binary = find_backend_binary();
            match spawn_backend(&binary) {
                Ok(_) => eprintln!("[photopipeline] Backend started"),
                Err(e) => eprintln!("[photopipeline] Backend failed: {}", e),
            }

            // Initialize gRPC clients (non-blocking; will connect lazily)
            let rt = tokio::runtime::Runtime::new().unwrap();
            let grpc_clients = rt.block_on(async {
                GrpcClients::connect("http://localhost:50051")
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("[photopipeline] gRPC connection failed: {}", e);
                        panic!("Cannot start without gRPC connection to backend");
                    })
            });
            app.manage(Mutex::new(grpc_clients));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::plugin_cmds::list_plugins,
            commands::plugin_cmds::get_node_schema,
            commands::image_cmds::load_images,
            commands::image_cmds::get_thumbnail,
            commands::image_cmds::decode_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

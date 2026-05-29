mod commands;
mod grpc;
mod proto;

use grpc::GrpcClients;
use tauri::Manager;
use tokio::sync::Mutex;

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

fn find_backend_binary() -> String {
    for path in ["photopipeline", "./photopipeline", "../target/release/photopipeline"] {
        if std::path::Path::new(path).exists() { return path.to_string(); }
    }
    "photopipeline".to_string()
}

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
            let window = app.get_webview_window("main").expect("main window not found");
            apply_vibrancy(&window);

            let binary = find_backend_binary();
            match spawn_backend(&binary) {
                Ok(_) => eprintln!("[photopipeline] Backend started"),
                Err(e) => eprintln!("[photopipeline] Backend spawn failed: {}", e),
            }

            // Try gRPC connection, fall back to offline mode
            let rt = tokio::runtime::Runtime::new().unwrap();
            let grpc_clients = rt.block_on(async {
                match GrpcClients::connect("http://localhost:50051").await {
                    Ok(clients) => {
                        eprintln!("[photopipeline] gRPC connected");
                        Some(clients)
                    }
                    Err(e) => {
                        eprintln!("[photopipeline] gRPC unavailable: {} — running in offline (mock) mode", e);
                        None
                    }
                }
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

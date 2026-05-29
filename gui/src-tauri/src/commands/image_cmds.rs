use crate::grpc::GrpcClients;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub pixel_format: String,
    pub color_space: String,
    pub metadata: Option<MetadataInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub date_time_original: Option<String>,
    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailResult {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn load_images(
    grpc: State<'_, Mutex<GrpcClients>>,
    paths: Vec<String>,
) -> Result<Vec<ImageInfo>, String> {
    let mut clients = grpc.lock().await;
    let mut results = Vec::new();
    for path in &paths {
        match clients.image.load(path).await {
            Ok(info) => {
                results.push(ImageInfo {
                    id: info.id,
                    path: info.path,
                    filename: info.filename,
                    format: info.format,
                    width: info.width,
                    height: info.height,
                    file_size_bytes: info.file_size_bytes,
                    pixel_format: info.pixel_format,
                    color_space: info.color_space,
                    metadata: info.metadata.map(|m| MetadataInfo {
                        make: m.make,
                        model: m.model,
                        lens_model: m.lens_model,
                        date_time_original: m.date_time_original,
                        exposure_time: m.exposure_time,
                        f_number: m.f_number,
                        iso: m.iso,
                        focal_length: m.focal_length,
                        latitude: m.latitude,
                        longitude: m.longitude,
                        altitude: m.altitude,
                    }),
                });
            }
            Err(e) => {
                eprintln!("[photopipeline] Failed to load {}: {}", path, e);
                // Continue loading other files
            }
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn get_thumbnail(
    grpc: State<'_, Mutex<GrpcClients>>,
    path: String,
    max_size: u32,
) -> Result<ThumbnailResult, String> {
    let mut clients = grpc.lock().await;
    let data = clients.image.get_thumbnail(&path, max_size).await?;
    Ok(ThumbnailResult {
        data: data.data,
        width: data.width,
        height: data.height,
    })
}

#[tauri::command]
pub async fn decode_preview(
    grpc: State<'_, Mutex<GrpcClients>>,
    path: String,
    max_width: u32,
    max_height: u32,
) -> Result<Vec<u8>, String> {
    let mut clients = grpc.lock().await;
    clients
        .image
        .decode_preview(&path, max_width, max_height)
        .await
}

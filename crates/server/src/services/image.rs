use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use photopipeline_core::{
    ChannelLayout, EncodeOptions, ImageFormat, Metadata, PixelBuffer, PixelFormat,
};

use crate::SharedState;
use crate::pb::image::{
    DecodeRequest, EncodeProgress, EncodeRequest, ImageData, ImageInfo as ProtoImageInfo,
    ImagePath, PixelDataChunk, ThumbnailRequest, image_service_server::ImageService,
};

pub struct ImageServiceImpl {
    state: Arc<SharedState>,
}

impl ImageServiceImpl {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

fn parse_pixel_format(s: &str) -> PixelFormat {
    match s.to_lowercase().as_str() {
        "u8" => PixelFormat::U8,
        "u16" => PixelFormat::U16,
        "u32" => PixelFormat::U32,
        "f16" => PixelFormat::F16,
        "f32" => PixelFormat::F32,
        _ => PixelFormat::U16,
    }
}

fn parse_image_format(s: &str) -> ImageFormat {
    match s.to_lowercase().as_str() {
        "heif" | "heic" => ImageFormat::HEIF,
        "avif" => ImageFormat::AVIF,
        "jxl" => ImageFormat::JXL,
        "png" => ImageFormat::PNG,
        "tiff" | "tif" => ImageFormat::TIFF,
        "jpeg" | "jpg" => ImageFormat::JPEG,
        "webp" => ImageFormat::WEBP,
        "exr" => ImageFormat::OpenEXR,
        "bmp" => ImageFormat::BMP,
        _ => ImageFormat::Unknown(s.to_string()),
    }
}

fn find_format_processor_for_format(
    registry: &photopipeline_plugin::Registry,
    format: &ImageFormat,
) -> Option<Arc<dyn photopipeline_plugin::FormatProcessor>> {
    for manifest in registry.manifests() {
        if manifest.category == photopipeline_core::PluginCategory::Format {
            if let Some(fp) = registry.get_format_processor(&manifest.id) {
                if fp.can_encode(format) {
                    return Some(fp);
                }
            }
        }
    }
    None
}

#[tonic::async_trait]
impl ImageService for ImageServiceImpl {
    async fn load(&self, request: Request<ImagePath>) -> Result<Response<ProtoImageInfo>, Status> {
        let path = request.into_inner().path;
        tracing::info!("load: path={}", path);

        if !std::path::Path::new(&path).exists() {
            return Err(Status::not_found(format!("file not found: {}", path)));
        }

        let filename = std::path::Path::new(&path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        let format = crate::detect_format_from_ext(&path);
        let format_str = format.to_string();

        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let (width, height) = match image::ImageReader::open(&path) {
            Ok(reader) => reader
                .into_dimensions()
                .map(|(w, h)| (w, h))
                .unwrap_or((0, 0)),
            Err(_) => (0, 0),
        };

        Ok(Response::new(ProtoImageInfo {
            id: Uuid::new_v4().to_string(),
            path: path.clone(),
            filename,
            format: format_str,
            width,
            height,
            file_size_bytes,
            pixel_format: "u16".into(),
            color_space: "srgb".into(),
            metadata: None,
        }))
    }

    type DecodeStream = ReceiverStream<Result<PixelDataChunk, Status>>;

    async fn decode(
        &self,
        request: Request<DecodeRequest>,
    ) -> Result<Response<Self::DecodeStream>, Status> {
        let req = request.into_inner();
        tracing::info!("decode: path={}", req.path);

        let path = req.path.clone();
        if !std::path::Path::new(&path).exists() {
            return Err(Status::not_found(format!("file not found: {}", path)));
        }

        let decode_req = req.clone();
        let (tx, rx) = mpsc::channel(256);

        tokio::spawn(async move {
            match image::ImageReader::open(&path) {
                Ok(reader) => match reader.with_guessed_format() {
                    Ok(reader) => match reader.decode() {
                        Ok(img) => {
                            let max_w = decode_req.max_width.unwrap_or(u32::MAX);
                            let max_h = decode_req.max_height.unwrap_or(u32::MAX);
                            let resized = if img.width() > max_w || img.height() > max_h {
                                let scale_w = max_w as f64 / img.width() as f64;
                                let scale_h = max_h as f64 / img.height() as f64;
                                let scale = scale_w.min(scale_h);
                                img.resize(
                                    (img.width() as f64 * scale) as u32,
                                    (img.height() as f64 * scale) as u32,
                                    image::imageops::FilterType::Lanczos3,
                                )
                            } else {
                                img
                            };

                            let rgb = resized.to_rgba8();
                            let data = rgb.into_raw();
                            let total = data.len() as u32;
                            let chunk_size = (256 * 1024).min(total as usize).max(1);

                            let mut offset = 0u32;
                            while offset < total {
                                let end = ((offset as usize) + chunk_size).min(total as usize);
                                let slice = data[offset as usize..end].to_vec();
                                let is_last = end >= total as usize;
                                let _ = tx.send(Ok(PixelDataChunk {
                                    offset,
                                    data: slice,
                                    total_size: total,
                                    is_last,
                                }));
                                offset = end as u32;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(Status::internal(format!("decode failed: {}", e))));
                        }
                    },
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(format!(
                            "unknown image format: {}",
                            e
                        ))));
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(Status::internal(format!(
                        "failed to open image: {}",
                        e
                    ))));
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type EncodeStream = ReceiverStream<Result<EncodeProgress, Status>>;

    async fn encode(
        &self,
        request: Request<EncodeRequest>,
    ) -> Result<Response<Self::EncodeStream>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "encode: output_path={}, format={}, {}x{}",
            req.output_path,
            req.format,
            req.width,
            req.height
        );

        let format = parse_image_format(&req.format);
        let pixel_format = parse_pixel_format(&req.pixel_format);
        let layout = match req.layout.to_lowercase().as_str() {
            "gray" => ChannelLayout::Gray,
            "gray_alpha" | "grayalpha" => ChannelLayout::GrayAlpha,
            "rgb" => ChannelLayout::RGB,
            "rgba" => ChannelLayout::RGBA,
            _ => ChannelLayout::RGB,
        };

        let mut buffer = PixelBuffer::new(req.width, req.height, layout, pixel_format);
        let copy_len = buffer.data.data.len().min(req.pixel_data.len());
        buffer.data.data[..copy_len].copy_from_slice(&req.pixel_data[..copy_len]);

        let metadata = Metadata::default();
        let opts = EncodeOptions {
            format: format.clone(),
            quality: req.quality,
            lossless: req.lossless,
            bit_depth: req.bit_depth.max(8) as u8,
            chroma_subsampling: req.chroma_subsampling.as_deref().map(|cs| match cs {
                "yuv444" => photopipeline_core::ChromaSubsampling::Yuv444,
                "yuv422" => photopipeline_core::ChromaSubsampling::Yuv422,
                _ => photopipeline_core::ChromaSubsampling::Yuv420,
            }),
            encoder: req.encoder.clone(),
            effort: req.effort.map(|e| e as u8),
            compression: None,
            embed_profile: None,
        };

        let (tx, rx) = mpsc::channel(256);
        let output_path = req.output_path.clone();
        let registry = self.state.registry.clone();

        tokio::spawn(async move {
            let format_proc = find_format_processor_for_format(&registry, &format);

            if let Some(proc) = format_proc {
                let _ = tx.send(Ok(EncodeProgress {
                    fraction: 0.1,
                    message: "Starting encoding with format processor...".into(),
                    bytes_written: 0,
                    done: false,
                }));

                match proc.encode(&buffer, &metadata, &opts).await {
                    Ok(encoded) => {
                        let bytes_written = encoded.len() as u64;
                        if let Err(e) = std::fs::write(&output_path, &encoded) {
                            let _ = tx.send(Err(Status::internal(format!(
                                "failed to write output: {}",
                                e
                            ))));
                            return;
                        }
                        let _ = tx.send(Ok(EncodeProgress {
                            fraction: 1.0,
                            message: format!("Encoded to {}", output_path),
                            bytes_written,
                            done: true,
                        }));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(format!("encode failed: {}", e))));
                    }
                }
            } else {
                let _ = tx.send(Ok(EncodeProgress {
                    fraction: 0.1,
                    message: "Falling back to raw write...".into(),
                    bytes_written: 0,
                    done: false,
                }));

                match std::fs::write(&output_path, &buffer.data.data) {
                    Ok(()) => {
                        let bytes_written = buffer.data.data.len() as u64;
                        let _ = tx.send(Ok(EncodeProgress {
                            fraction: 1.0,
                            message: format!("Raw data written to {}", output_path),
                            bytes_written,
                            done: true,
                        }));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(format!(
                            "failed to write output: {}",
                            e
                        ))));
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_thumbnail(
        &self,
        request: Request<ThumbnailRequest>,
    ) -> Result<Response<ImageData>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "get_thumbnail: path={}, max_size={}",
            req.path,
            req.max_size
        );

        if !std::path::Path::new(&req.path).exists() {
            return Err(Status::not_found(format!("file not found: {}", req.path)));
        }

        let max_size = req.max_size.max(1);

        let reader = image::ImageReader::open(&req.path)
            .map_err(|e| Status::internal(format!("failed to open image: {}", e)))?
            .with_guessed_format()
            .map_err(|e| Status::internal(format!("unknown image format: {}", e)))?;

        let img = reader
            .decode()
            .map_err(|e| Status::internal(format!("failed to decode image: {}", e)))?;

        let (w, h) = (img.width(), img.height());
        let (tw, th) = if w > h {
            let scale = max_size as f64 / w as f64;
            (max_size, (h as f64 * scale).max(1.0) as u32)
        } else {
            let scale = max_size as f64 / h as f64;
            ((w as f64 * scale).max(1.0) as u32, max_size)
        };

        let thumb = img.thumbnail(tw, th);
        let mut buf = std::io::Cursor::new(Vec::new());
        thumb
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .map_err(|e| Status::internal(format!("failed to write thumbnail: {}", e)))?;

        let dimension = (thumb.width(), thumb.height());
        Ok(Response::new(ImageData {
            data: buf.into_inner(),
            width: dimension.0,
            height: dimension.1,
            format: "jpeg".into(),
        }))
    }
}

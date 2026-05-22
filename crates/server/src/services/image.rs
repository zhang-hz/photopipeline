use std::sync::Arc;
use parking_lot::RwLock;
use tonic::{Request, Response, Status};
use futures::stream;

use crate::pb::image::{
    image_service_server::ImageService,
    DecodeRequest, EncodeProgress, EncodeRequest, ImageData, ImageInfo,
    ImagePath, PixelDataChunk, ThumbnailRequest,
};
use crate::SharedState;

pub struct ImageServiceImpl {
    state: Arc<RwLock<SharedState>>,
}

impl ImageServiceImpl {
    pub fn new(state: Arc<RwLock<SharedState>>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl ImageService for ImageServiceImpl {
    async fn load(
        &self,
        request: Request<ImagePath>,
    ) -> Result<Response<ImageInfo>, Status> {
        let path = request.into_inner().path;

        Ok(Response::new(ImageInfo {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.clone(),
            filename: std::path::Path::new(&path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default(),
            format: "raw".into(),
            width: 0,
            height: 0,
            file_size_bytes: 0,
            pixel_format: "u16".into(),
            color_space: "srgb".into(),
            metadata: None,
        }))
    }

    type DecodeStream = stream::Iter<std::vec::IntoIter<Result<PixelDataChunk, Status>>>;

    async fn decode(
        &self,
        request: Request<DecodeRequest>,
    ) -> Result<Response<Self::DecodeStream>, Status> {
        let _req = request.into_inner();

        let chunks = vec![PixelDataChunk {
            offset: 0,
            data: vec![],
            total_size: 0,
            is_last: true,
        }];

        let iter = chunks.into_iter().map(Ok).collect::<Vec<_>>().into_iter();
        Ok(Response::new(stream::iter(iter)))
    }

    type EncodeStream = stream::Iter<std::vec::IntoIter<Result<EncodeProgress, Status>>>;

    async fn encode(
        &self,
        request: Request<EncodeRequest>,
    ) -> Result<Response<Self::EncodeStream>, Status> {
        let req = request.into_inner();

        let progress = vec![
            EncodeProgress {
                fraction: 0.5,
                message: "Encoding...".into(),
                bytes_written: 0,
                done: false,
            },
            EncodeProgress {
                fraction: 1.0,
                message: format!("Encoded to {}", req.output_path),
                bytes_written: req.pixel_data.len() as u64,
                done: true,
            },
        ];

        let iter = progress.into_iter().map(Ok).collect::<Vec<_>>().into_iter();
        Ok(Response::new(stream::iter(iter)))
    }

    async fn get_thumbnail(
        &self,
        request: Request<ThumbnailRequest>,
    ) -> Result<Response<ImageData>, Status> {
        let req = request.into_inner();

        Ok(Response::new(ImageData {
            data: vec![],
            width: req.max_size,
            height: req.max_size,
            format: "jpeg".into(),
        }))
    }
}

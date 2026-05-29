use crate::proto::{image_service_client::ImageServiceClient, DecodeRequest, ImageData, ImageInfo, ImagePath, ThumbnailRequest};
use tonic::{transport::Channel, Request};

pub struct ImageClient { inner: ImageServiceClient<Channel> }
impl ImageClient {
    pub fn new(channel: Channel) -> Self { Self { inner: ImageServiceClient::new(channel) } }
    pub async fn load(&mut self, path: &str) -> Result<ImageInfo, String> {
        self.inner.load(Request::new(ImagePath { path: path.to_string() })).await
            .map(|r| r.into_inner()).map_err(|e| format!("ImageService: {}", e))
    }
    pub async fn get_thumbnail(&mut self, path: &str, max_size: u32) -> Result<ImageData, String> {
        self.inner.get_thumbnail(Request::new(ThumbnailRequest { path: path.to_string(), max_size })).await
            .map(|r| r.into_inner()).map_err(|e| format!("ImageService: {}", e))
    }
    pub async fn decode_preview(&mut self, path: &str, max_w: u32, max_h: u32) -> Result<Vec<u8>, String> {
        let mut stream = self.inner.decode(Request::new(DecodeRequest {
            path: path.to_string(), pixel_format: None, max_width: Some(max_w), max_height: Some(max_h),
            read_metadata: false, apply_transfer: true,
        })).await.map_err(|e| format!("ImageService: {}", e))?.into_inner();
        let mut buf = Vec::new();
        while let Some(chunk) = stream.message().await.map_err(|e| format!("stream: {}", e))? {
            buf.extend_from_slice(&chunk.data);
        }
        Ok(buf)
    }
}

use crate::proto::{batch_service_client::BatchServiceClient, BatchId, BatchProgress, BatchSpec};
use tonic::{codec::Streaming, transport::Channel, Request};

pub struct BatchClient { inner: BatchServiceClient<Channel> }
impl BatchClient {
    pub fn new(channel: Channel) -> Self { Self { inner: BatchServiceClient::new(channel) } }
    pub async fn submit_batch(&mut self, cfg: &str, pat: &str, out: &str, par: i32, resume: bool) -> Result<String, String> {
        self.inner.submit_batch(Request::new(BatchSpec {
            pipeline_config_path: cfg.to_string(), file_pattern: pat.to_string(), output_dir: out.to_string(),
            parallel: par, resume,
        })).await.map(|r| r.into_inner().id).map_err(|e| format!("Batch: {}", e))
    }
    pub async fn get_progress(&mut self, id: &str) -> Result<Streaming<BatchProgress>, String> {
        self.inner.get_progress(Request::new(BatchId { id: id.to_string() })).await
            .map(|r| r.into_inner()).map_err(|e| format!("Batch: {}", e))
    }
    pub async fn cancel(&mut self, id: &str) -> Result<(), String> {
        self.inner.cancel(Request::new(BatchId { id: id.to_string() })).await
            .map(|_| ()).map_err(|e| format!("Batch: {}", e))
    }
}

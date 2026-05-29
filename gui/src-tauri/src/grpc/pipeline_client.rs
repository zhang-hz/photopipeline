use crate::proto::{pipeline_service_client::PipelineServiceClient, ExecuteRequest, PipelineSpec, ValidationResult};
use tonic::{codec::Streaming, transport::Channel, Request};

pub struct PipelineClient { inner: PipelineServiceClient<Channel> }
impl PipelineClient {
    pub fn new(channel: Channel) -> Self { Self { inner: PipelineServiceClient::new(channel) } }
    pub async fn create_pipeline(&mut self, spec: PipelineSpec) -> Result<String, String> {
        self.inner.create_pipeline(Request::new(spec)).await
            .map(|r| r.into_inner().id).map_err(|e| format!("Pipeline: {}", e))
    }
    pub async fn validate(&mut self, spec: PipelineSpec) -> Result<ValidationResult, String> {
        self.inner.validate(Request::new(spec)).await
            .map(|r| r.into_inner()).map_err(|e| format!("Pipeline: {}", e))
    }
    pub async fn execute(&mut self, pid: &str, img: &str, out: &str) -> Result<Streaming<crate::proto::ExecuteProgress>, String> {
        self.inner.execute(Request::new(ExecuteRequest {
            pipeline_id: pid.to_string(), image_path: img.to_string(), output_path: out.to_string(),
        })).await.map(|r| r.into_inner()).map_err(|e| format!("Pipeline: {}", e))
    }
}

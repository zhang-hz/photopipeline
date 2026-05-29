pub mod batch_client;
pub mod image_client;
pub mod pipeline_client;
pub mod plugin_client;

use tonic::transport::Channel;

pub struct GrpcClients {
    pub plugin: plugin_client::PluginClient,
    pub image: image_client::ImageClient,
    pub pipeline: pipeline_client::PipelineClient,
    pub batch: batch_client::BatchClient,
}

impl GrpcClients {
    pub async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(addr.to_string())
            .map_err(|e| format!("Invalid address: {}", e))?
            .connect().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        Ok(Self {
            plugin: plugin_client::PluginClient::new(channel.clone()),
            image: image_client::ImageClient::new(channel.clone()),
            pipeline: pipeline_client::PipelineClient::new(channel.clone()),
            batch: batch_client::BatchClient::new(channel),
        })
    }
}

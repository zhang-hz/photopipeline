use crate::proto::{plugin_service_client::PluginServiceClient, NodeSchemaResponse, PluginEntry, PluginIdRequest};
use tonic::{transport::Channel, Request};

pub struct PluginClient { inner: PluginServiceClient<Channel> }
impl PluginClient {
    pub fn new(channel: Channel) -> Self { Self { inner: PluginServiceClient::new(channel) } }
    pub async fn list_plugins(&mut self) -> Result<Vec<PluginEntry>, String> {
        self.inner.list_plugins(Request::new(())).await
            .map(|r| r.into_inner().plugins)
            .map_err(|e| format!("PluginService: {}", e))
    }
    pub async fn get_node_schema(&mut self, id: &str) -> Result<NodeSchemaResponse, String> {
        self.inner.get_node_schema(Request::new(PluginIdRequest { id: id.to_string() })).await
            .map(|r| r.into_inner())
            .map_err(|e| format!("PluginService: {}", e))
    }
}

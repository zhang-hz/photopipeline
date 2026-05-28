use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::pb::plugin::{
    plugin_service_server::PluginService, NodeSchemaResponse, PluginCatalogResponse, PluginEntry,
    PluginIdRequest,
};
use crate::{schema_to_prost_struct, SharedState};

pub struct PluginServiceImpl {
    state: Arc<SharedState>,
}

impl PluginServiceImpl {
    pub fn new(state: Arc<SharedState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl PluginService for PluginServiceImpl {
    async fn list_plugins(
        &self,
        _request: Request<()>,
    ) -> Result<Response<PluginCatalogResponse>, Status> {
        let manifests = self.state.registry.manifests();
        let categories = self.state.registry.categories();

        let plugins: Vec<PluginEntry> = manifests
            .into_iter()
            .map(|m| PluginEntry {
                id: m.id.to_string(),
                name: m.name,
                version: m.version.to_string(),
                category: m.category.to_string(),
                description: m.description,
                tags: m.tags,
                requires_pixel_access: m.requires_pixel_access,
                requires_network: m.requires_network,
                requires_filesystem: m.requires_filesystem,
                min_ram_mb: m.min_ram_mb,
                dependencies: m.dependencies.into_iter().collect(),
            })
            .collect();

        let categories: Vec<String> =
            categories.into_iter().map(|c| c.to_string()).collect();

        Ok(Response::new(PluginCatalogResponse {
            plugins,
            categories,
        }))
    }

    async fn get_node_schema(
        &self,
        request: Request<PluginIdRequest>,
    ) -> Result<Response<NodeSchemaResponse>, Status> {
        let pid = request.into_inner();
        let plugin = self
            .state
            .registry
            .get(&pid.id)
            .ok_or_else(|| Status::not_found(format!("plugin not found: {}", pid.id)))?;

        let schema = plugin.parameter_schema();
        let gui = plugin.gui_schema();

        let parameter_schema = Some(schema_to_prost_struct(schema));
        let gui_schema = serde_json::to_value(gui).ok().map(|v| {
            let pv = crate::json_to_prost_value(&v);
            match pv.kind {
                Some(prost_types::value::Kind::StructValue(s)) => s,
                _ => prost_types::Struct::default(),
            }
        });

        Ok(Response::new(NodeSchemaResponse {
            plugin_id: pid.id,
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            category: plugin.category().to_string(),
            description: plugin.description().to_string(),
            parameter_schema,
            gui_schema,
        }))
    }
}

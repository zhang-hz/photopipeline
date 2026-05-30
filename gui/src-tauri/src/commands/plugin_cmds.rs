use crate::grpc::GrpcClients;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub id: String, pub name: String, pub version: String, pub category: String,
    pub description: String, pub tags: Vec<String>,
    pub requires_pixel_access: bool, pub requires_network: bool, pub requires_filesystem: bool,
    pub min_ram_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSchemaResponse {
    pub plugin_id: String, pub name: String, pub version: String, pub category: String,
    pub description: String, pub parameter_schema: serde_json::Value, pub gui_schema: serde_json::Value,
}

fn prost_struct_to_json(s: &prost_types::Struct) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &s.fields { map.insert(k.clone(), prost_value_to_json(v)); }
    serde_json::Value::Object(map)
}
fn prost_value_to_json(v: &prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match &v.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(0.into())),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(Kind::StructValue(s)) => prost_struct_to_json(s),
        Some(Kind::ListValue(list)) => serde_json::Value::Array(list.values.iter().map(prost_value_to_json).collect()),
        None => serde_json::Value::Null,
    }
}

fn mock_plugins() -> Vec<PluginEntry> {
    serde_json::from_str(include_str!("../data/mock_plugins.json")).unwrap_or_default()
}

fn mock_schema_for(plugin_id: &str) -> Option<NodeSchemaResponse> {
    let db: serde_json::Value = serde_json::from_str(include_str!("../data/mock_schemas.json")).unwrap_or_default();
    db.get(plugin_id).map(|s| NodeSchemaResponse {
        plugin_id: s["plugin_id"].as_str().unwrap_or(plugin_id).to_string(),
        name: s["name"].as_str().unwrap_or("").to_string(),
        version: "1.0.0".to_string(),
        category: s["category"].as_str().unwrap_or("").to_string(),
        description: s["description"].as_str().unwrap_or("").to_string(),
        parameter_schema: s["parameter_schema"].clone(),
        gui_schema: s["gui_schema"].clone(),
    })
}

#[tauri::command]
pub async fn list_plugins(
    grpc: State<'_, Mutex<Option<GrpcClients>>>,
) -> Result<Vec<PluginEntry>, String> {
    let mut guard = grpc.lock().await;
    if let Some(clients) = guard.as_mut() {
        if let Ok(entries) = clients.plugin.list_plugins().await {
            return Ok(entries.into_iter().map(|e| PluginEntry {
                id: e.id, name: e.name, version: e.version, category: e.category,
                description: e.description, tags: e.tags,
                requires_pixel_access: e.requires_pixel_access, requires_network: e.requires_network,
                requires_filesystem: e.requires_filesystem, min_ram_mb: e.min_ram_mb,
            }).collect());
        }
    }
    Ok(mock_plugins())
}

#[tauri::command]
pub async fn get_node_schema(
    grpc: State<'_, Mutex<Option<GrpcClients>>>,
    plugin_id: String,
) -> Result<NodeSchemaResponse, String> {
    let mut guard = grpc.lock().await;
    if let Some(clients) = guard.as_mut() {
        if let Ok(schema) = clients.plugin.get_node_schema(&plugin_id).await {
            return Ok(NodeSchemaResponse {
                plugin_id: schema.plugin_id, name: schema.name, version: schema.version,
                category: schema.category, description: schema.description,
                parameter_schema: schema.parameter_schema.as_ref().map(|s| prost_struct_to_json(s)).unwrap_or(serde_json::Value::Null),
                gui_schema: schema.gui_schema.as_ref().map(|s| prost_struct_to_json(s)).unwrap_or(serde_json::Value::Null),
            });
        }
    }
    mock_schema_for(&plugin_id).ok_or_else(|| format!("No schema for plugin: {}", plugin_id))
}

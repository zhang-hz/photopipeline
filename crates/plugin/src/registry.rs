use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use crate::trait_def::*;
use photopipeline_core::{PluginId, PluginCategory, PluginResult};

pub struct Registry {
    entries: DashMap<PluginId, RegistryEntry>,
    manifests: DashMap<PluginId, PluginManifest>,
    load_order: RwLock<Vec<PluginId>>,
    metadata_processors: DashMap<PluginId, Arc<dyn MetadataProcessor>>,
    pixel_processors: DashMap<PluginId, Arc<dyn PixelProcessor>>,
    format_processors: DashMap<PluginId, Arc<dyn FormatProcessor>>,
    gpu_processors: DashMap<PluginId, Arc<dyn GpuProcessor>>,
    ai_processors: DashMap<PluginId, Arc<dyn AiProcessor>>,
    external_tool_processors: DashMap<PluginId, Arc<dyn ExternalToolProcessor>>,
}

struct RegistryEntry {
    plugin: Arc<dyn Plugin>,
    enabled: bool,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            manifests: DashMap::new(),
            load_order: RwLock::new(Vec::new()),
            metadata_processors: DashMap::new(),
            pixel_processors: DashMap::new(),
            format_processors: DashMap::new(),
            gpu_processors: DashMap::new(),
            ai_processors: DashMap::new(),
            external_tool_processors: DashMap::new(),
        }
    }

    pub fn register(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()> {
        let id = plugin.id().clone();
        let manifest = PluginManifest {
            id: id.clone(),
            name: plugin.name().to_string(),
            version: plugin.version(),
            category: plugin.category(),
            description: plugin.description().to_string(),
            tags: plugin.tags().to_vec(),
            requires_pixel_access: plugin.requires_pixel_access(),
            requires_network: false,
            requires_filesystem: false,
            min_ram_mb: plugin.supported_hardware().min_ram_mb,
            dependencies: Default::default(),
        };

        self.manifests.insert(id.clone(), manifest);
        self.entries.insert(id.clone(), RegistryEntry {
            plugin,
            enabled: true,
        });
        self.load_order.write().push(id);
        Ok(())
    }

    pub fn unregister(&self, id: &PluginId) -> Option<Arc<dyn Plugin>> {
        self.manifests.remove(id);
        self.load_order.write().retain(|i| i != id);
        self.metadata_processors.remove(id);
        self.pixel_processors.remove(id);
        self.format_processors.remove(id);
        self.gpu_processors.remove(id);
        self.ai_processors.remove(id);
        self.external_tool_processors.remove(id);
        self.entries.remove(id).map(|(_, entry)| entry.plugin)
    }

    pub fn get(&self, id: &PluginId) -> Option<Arc<dyn Plugin>> {
        self.entries.get(id).map(|e| e.value().plugin.clone())
    }

    pub fn get_metadata_processor(&self, id: &PluginId) -> Option<Arc<dyn MetadataProcessor>> {
        self.metadata_processors.get(id).map(|e| e.value().clone())
    }

    pub fn get_pixel_processor(&self, id: &PluginId) -> Option<Arc<dyn PixelProcessor>> {
        self.pixel_processors.get(id).map(|e| e.value().clone())
    }

    pub fn get_format_processor(&self, id: &PluginId) -> Option<Arc<dyn FormatProcessor>> {
        self.format_processors.get(id).map(|e| e.value().clone())
    }

    pub fn get_gpu_processor(&self, id: &PluginId) -> Option<Arc<dyn GpuProcessor>> {
        self.gpu_processors.get(id).map(|e| e.value().clone())
    }

    pub fn get_ai_processor(&self, id: &PluginId) -> Option<Arc<dyn AiProcessor>> {
        self.ai_processors.get(id).map(|e| e.value().clone())
    }

    pub fn get_external_tool_processor(&self, id: &PluginId) -> Option<Arc<dyn ExternalToolProcessor>> {
        self.external_tool_processors.get(id).map(|e| e.value().clone())
    }

    pub fn query(&self, q: &PluginQuery) -> Vec<Arc<dyn Plugin>> {
        self.entries.iter()
            .filter(|entry| {
                let plugin = &entry.value().plugin;
                if q.enabled_only && !entry.value().enabled {
                    return false;
                }
                if let Some(ref cat) = q.category {
                    if plugin.category() != *cat {
                        return false;
                    }
                }
                if !q.tags.is_empty() {
                    let plugin_tags: std::collections::HashSet<_> =
                        plugin.tags().iter().map(|s| s.as_str()).collect();
                    if !q.tags.iter().all(|t| plugin_tags.contains(t.as_str())) {
                        return false;
                    }
                }
                if let Some(ref kw) = q.keyword {
                    let lower = kw.to_lowercase();
                    if !plugin.name().to_lowercase().contains(&lower)
                        && !plugin.description().to_lowercase().contains(&lower)
                    {
                        return false;
                    }
                }
                if let Some(req_pixel) = q.requires_pixel {
                    if plugin.requires_pixel_access() != req_pixel {
                        return false;
                    }
                }
                true
            })
            .map(|e| e.value().plugin.clone())
            .collect()
    }

    pub fn by_category(&self, cat: PluginCategory) -> Vec<Arc<dyn Plugin>> {
        self.query(&PluginQuery { category: Some(cat), ..Default::default() })
    }

    pub fn all(&self) -> Vec<Arc<dyn Plugin>> {
        self.entries.iter().map(|e| e.value().plugin.clone()).collect()
    }

    pub fn manifest(&self, id: &PluginId) -> Option<PluginManifest> {
        self.manifests.get(id).map(|m| m.clone())
    }

    pub fn manifests(&self) -> Vec<PluginManifest> {
        self.manifests.iter().map(|m| m.value().clone()).collect()
    }

    pub fn categories(&self) -> Vec<PluginCategory> {
        let mut cats: Vec<_> = self.manifests.iter()
            .map(|m| m.value().category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    pub fn is_loaded(&self, id: &PluginId) -> bool {
        self.entries.contains_key(id)
    }

    pub fn register_metadata_processor(&self, plugin: Arc<dyn MetadataProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.metadata_processors.insert(id, plugin);
        Ok(())
    }

    pub fn register_pixel_processor(&self, plugin: Arc<dyn PixelProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.pixel_processors.insert(id, plugin);
        Ok(())
    }

    pub fn register_format_processor(&self, plugin: Arc<dyn FormatProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.format_processors.insert(id, plugin);
        Ok(())
    }

    pub fn register_gpu_processor(&self, plugin: Arc<dyn GpuProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.gpu_processors.insert(id, plugin);
        Ok(())
    }

    pub fn register_ai_processor(&self, plugin: Arc<dyn AiProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.ai_processors.insert(id, plugin);
        Ok(())
    }

    pub fn register_external_tool_processor(&self, plugin: Arc<dyn ExternalToolProcessor>) -> PluginResult<()> {
        let id = plugin.id().clone();
        self.external_tool_processors.insert(id, plugin);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use photopipeline_core::{
        HardwareRequirement, PluginId, PluginResult, PluginVersion,
        PluginCategory, ValidationIssue,
    };
    use crate::{ParameterSchema, ParameterSet};

    #[derive(Debug)]
    struct MockPlugin {
        id: PluginId,
        name: String,
        version: PluginVersion,
        category: PluginCategory,
        description: String,
        tags: Vec<String>,
        requires_pixel: bool,
        hardware: HardwareRequirement,
        schema: ParameterSchema,
        gui_schema: photopipeline_core::GuiSchema,
    }

    impl MockPlugin {
        fn new(id: &str, name: &str, category: PluginCategory) -> Self {
            Self {
                id: id.into(),
                name: name.into(),
                version: PluginVersion::new(1, 0, 0),
                category,
                description: "mock plugin".into(),
                tags: vec!["test".into()],
                requires_pixel: false,
                hardware: Default::default(),
                schema: ParameterSchema::empty(),
                gui_schema: Default::default(),
            }
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn id(&self) -> &PluginId { &self.id }
        fn name(&self) -> &str { &self.name }
        fn version(&self) -> PluginVersion { self.version.clone() }
        fn category(&self) -> PluginCategory { self.category.clone() }
        fn description(&self) -> &str { &self.description }
        fn tags(&self) -> &[String] { &self.tags }
        fn requires_pixel_access(&self) -> bool { self.requires_pixel }
        fn produces_pixel_output(&self) -> bool { false }
        fn supported_hardware(&self) -> HardwareRequirement { self.hardware.clone() }
        fn parameter_schema(&self) -> &ParameterSchema { &self.schema }
        fn gui_schema(&self) -> &photopipeline_core::GuiSchema { &self.gui_schema }

        async fn initialize(&mut self, _cfg: &PluginConfig) -> PluginResult<()> { Ok(()) }
        async fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
        async fn validate(&self, _params: &ParameterSet) -> PluginResult<Vec<ValidationIssue>> { Ok(vec![]) }
    }

    #[test]
    fn registry_new_is_empty() {
        let reg = Registry::new();
        assert!(reg.all().is_empty());
        assert!(reg.manifests().is_empty());
        assert!(reg.categories().is_empty());
    }

    #[test]
    fn registry_register_and_get() {
        let reg = Registry::new();
        let plugin = Arc::new(MockPlugin::new("test.plugin", "Test Plugin", PluginCategory::Color));
        reg.register(plugin.clone()).unwrap();

        let found = reg.get(&"test.plugin".into());
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Test Plugin");
    }

    #[test]
    fn registry_unregister() {
        let reg = Registry::new();
        let plugin = Arc::new(MockPlugin::new("test.plugin", "Test Plugin", PluginCategory::Color));
        reg.register(plugin).unwrap();

        let removed = reg.unregister(&"test.plugin".into());
        assert!(removed.is_some());
        assert!(reg.get(&"test.plugin".into()).is_none());
        assert!(!reg.is_loaded(&"test.plugin".into()));
    }

    #[test]
    fn registry_unregister_nonexistent() {
        let reg = Registry::new();
        assert!(reg.unregister(&"nonexistent".into()).is_none());
    }

    #[test]
    fn registry_manifests_list() {
        let reg = Registry::new();
        reg.register(Arc::new(MockPlugin::new("p1", "P1", PluginCategory::Input))).unwrap();
        reg.register(Arc::new(MockPlugin::new("p2", "P2", PluginCategory::Enhance))).unwrap();

        let manifests = reg.manifests();
        assert_eq!(manifests.len(), 2);
        let ids: Vec<&str> = manifests.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"p1"));
        assert!(ids.contains(&"p2"));
    }

    #[test]
    fn registry_categories_dedup() {
        let reg = Registry::new();
        reg.register(Arc::new(MockPlugin::new("p1", "P1", PluginCategory::Color))).unwrap();
        reg.register(Arc::new(MockPlugin::new("p2", "P2", PluginCategory::Color))).unwrap();
        reg.register(Arc::new(MockPlugin::new("p3", "P3", PluginCategory::Transform))).unwrap();

        let cats = reg.categories();
        assert_eq!(cats.len(), 2);
        assert!(cats.contains(&PluginCategory::Color));
        assert!(cats.contains(&PluginCategory::Transform));
    }

    #[test]
    fn registry_query_by_category() {
        let reg = Registry::new();
        reg.register(Arc::new(MockPlugin::new("p1", "P1", PluginCategory::Input))).unwrap();
        reg.register(Arc::new(MockPlugin::new("p2", "P2", PluginCategory::Enhance))).unwrap();

        let results = reg.by_category(PluginCategory::Input);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), &"p1".to_string());
    }

    #[test]
    fn registry_query_keyword() {
        let reg = Registry::new();
        reg.register(Arc::new(MockPlugin::new("p1", "Denoise AI", PluginCategory::Enhance))).unwrap();
        reg.register(Arc::new(MockPlugin::new("p2", "Sharpen", PluginCategory::Enhance))).unwrap();

        let q = PluginQuery { keyword: Some("denoise".into()), ..Default::default() };
        let results = reg.query(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name(), "Denoise AI");
    }

    #[test]
    fn registry_is_loaded() {
        let reg = Registry::new();
        assert!(!reg.is_loaded(&"p1".into()));
        reg.register(Arc::new(MockPlugin::new("p1", "P1", PluginCategory::Input))).unwrap();
        assert!(reg.is_loaded(&"p1".into()));
    }

    #[test]
    fn registry_manifest_individual() {
        let reg = Registry::new();
        reg.register(Arc::new(MockPlugin::new("p1", "MyPlugin", PluginCategory::Input))).unwrap();

        let manifest = reg.manifest(&"p1".into());
        assert!(manifest.is_some());
        let m = manifest.unwrap();
        assert_eq!(m.name, "MyPlugin");
        assert_eq!(m.category, PluginCategory::Input);
        assert_eq!(m.version, PluginVersion::new(1, 0, 0));
    }
}

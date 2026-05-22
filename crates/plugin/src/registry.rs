use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use crate::trait_def::*;
use photopipeline_core::{PluginId, PluginCategory, PluginResult};

pub struct Registry {
    entries: DashMap<PluginId, RegistryEntry>,
    manifests: DashMap<PluginId, PluginManifest>,
    load_order: RwLock<Vec<PluginId>>,
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
        self.entries.remove(id).map(|(_, entry)| entry.plugin)
    }

    pub fn get(&self, id: &PluginId) -> Option<Arc<dyn Plugin>> {
        self.entries.get(id).map(|e| e.value().plugin.clone())
    }

    pub fn get_metadata_processor(&self, _id: &PluginId) -> Option<Arc<dyn MetadataProcessor>> {
        None
    }

    pub fn get_pixel_processor(&self, _id: &PluginId) -> Option<Arc<dyn PixelProcessor>> {
        None
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
        cats.sort_by_key(|c| c.to_string());
        cats.dedup_by_key(|c| c.to_string());
        cats
    }

    pub fn is_loaded(&self, id: &PluginId) -> bool {
        self.entries.contains_key(id)
    }
}

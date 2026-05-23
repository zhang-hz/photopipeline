use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::registry::Registry;
use crate::trait_def::*;
use photopipeline_core::{PluginResult, PluginId};

#[async_trait::async_trait]
pub trait PluginLoader: Send + Sync {
    fn name(&self) -> &str;
    fn supported_extensions(&self) -> Vec<&str>;

    async fn probe(&self, path: &Path) -> PluginResult<Option<PluginManifest>>;
    async fn load(&self, manifest: &PluginManifest, path: &Path) -> PluginResult<Box<dyn Plugin>>;
    fn can_hot_reload(&self) -> bool;
}

pub struct BuiltinPluginLoader;

#[async_trait::async_trait]
impl PluginLoader for BuiltinPluginLoader {
    fn name(&self) -> &str { "builtin" }
    fn supported_extensions(&self) -> Vec<&str> { vec![] }
    fn can_hot_reload(&self) -> bool { false }

    async fn probe(&self, _path: &Path) -> PluginResult<Option<PluginManifest>> {
        Ok(None)
    }

    async fn load(&self, _manifest: &PluginManifest, _path: &Path) -> PluginResult<Box<dyn Plugin>> {
        Err(photopipeline_core::PluginError::Other("builtin loader does not load from path".into()))
    }
}

pub struct NativePluginLoader;

#[async_trait::async_trait]
impl PluginLoader for NativePluginLoader {
    fn name(&self) -> &str { "native" }
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["so", "dylib", "dll"]
    }
    fn can_hot_reload(&self) -> bool { false }

    async fn probe(&self, path: &Path) -> PluginResult<Option<PluginManifest>> {
        if !path.exists() || !path.is_file() {
            return Ok(None);
        }
        let manifest_path = path.with_extension("toml");
        if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)
                .map_err(|e| photopipeline_core::PluginError::Io {
                    plugin: PluginId::from("native_loader"),
                    error: e,
                })?;
            let manifest: PluginManifest = toml::from_str(&content)
                .map_err(|e| photopipeline_core::PluginError::Config(e.to_string()))?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    async fn load(&self, manifest: &PluginManifest, path: &Path) -> PluginResult<Box<dyn Plugin>> {
        let _manifest = manifest;
        let _path = path;
        Err(photopipeline_core::PluginError::Other(
            "Native plugin loading requires plugin-specific FFI implementation".into()
        ))
    }
}

pub struct ExternalToolPluginLoader;

#[async_trait::async_trait]
impl PluginLoader for ExternalToolPluginLoader {
    fn name(&self) -> &str { "external_tool" }
    fn supported_extensions(&self) -> Vec<&str> { vec![] }
    fn can_hot_reload(&self) -> bool { false }

    async fn probe(&self, _path: &Path) -> PluginResult<Option<PluginManifest>> {
        Ok(None)
    }

    async fn load(&self, _manifest: &PluginManifest, _path: &Path) -> PluginResult<Box<dyn Plugin>> {
        Err(photopipeline_core::PluginError::Other("external tool loading TBD".into()))
    }
}

#[derive(Default)]
pub struct PluginLoaderManager {
    loaders: Vec<Box<dyn PluginLoader>>,
    search_paths: Vec<PathBuf>,
}

impl PluginLoaderManager {
    pub fn new() -> Self {
        Self {
            loaders: vec![
                Box::new(BuiltinPluginLoader),
                Box::new(NativePluginLoader),
                Box::new(ExternalToolPluginLoader),
            ],
            search_paths: vec![],
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    pub async fn discover_and_load(&self, registry: &Registry) -> PluginResult<Vec<PluginId>> {
        let mut loaded = vec![];
        for dir in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    for loader in &self.loaders {
                        if let Some(manifest) = loader.probe(&path).await? {
                            if !registry.is_loaded(&manifest.id) {
                                match loader.load(&manifest, &path).await {
                                    Ok(plugin) => {
                                        let id = plugin.id().clone();
                                        registry.register(Arc::from(plugin))?;
                                        loaded.push(id);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to load plugin {}: {}", manifest.id, e);
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(loaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_plugin_loader_name() {
        let loader = BuiltinPluginLoader;
        assert_eq!(loader.name(), "builtin");
    }

    #[test]
    fn builtin_plugin_loader_extensions_empty() {
        let loader = BuiltinPluginLoader;
        assert!(loader.supported_extensions().is_empty());
    }

    #[test]
    fn builtin_plugin_loader_cannot_hot_reload() {
        let loader = BuiltinPluginLoader;
        assert!(!loader.can_hot_reload());
    }

    #[test]
    fn native_plugin_loader_name() {
        let loader = NativePluginLoader;
        assert_eq!(loader.name(), "native");
    }

    #[test]
    fn native_plugin_loader_extensions() {
        let loader = NativePluginLoader;
        let exts = loader.supported_extensions();
        assert!(exts.contains(&"so"));
        assert!(exts.contains(&"dylib"));
        assert!(exts.contains(&"dll"));
    }

    #[test]
    fn native_plugin_loader_cannot_hot_reload() {
        let loader = NativePluginLoader;
        assert!(!loader.can_hot_reload());
    }

    #[test]
    fn external_tool_plugin_loader_name() {
        let loader = ExternalToolPluginLoader;
        assert_eq!(loader.name(), "external_tool");
    }

    #[test]
    fn external_tool_plugin_loader_extensions_empty() {
        let loader = ExternalToolPluginLoader;
        assert!(loader.supported_extensions().is_empty());
    }

    #[test]
    fn external_tool_plugin_loader_cannot_hot_reload() {
        let loader = ExternalToolPluginLoader;
        assert!(!loader.can_hot_reload());
    }

    #[test]
    fn plugin_loader_manager_new_has_default_loaders() {
        let manager = PluginLoaderManager::new();
        assert!(!manager.loaders.is_empty());
    }

    #[test]
    fn plugin_loader_manager_new_has_empty_search_paths() {
        let manager = PluginLoaderManager::new();
        assert!(manager.search_paths.is_empty());
    }

    #[test]
    fn plugin_loader_manager_default() {
        let manager = PluginLoaderManager::default();
        assert!(manager.search_paths.is_empty());
    }

    #[test]
    fn plugin_loader_manager_add_search_path() {
        let mut manager = PluginLoaderManager::new();
        manager.add_search_path(std::path::PathBuf::from("/tmp/plugins"));
        assert_eq!(manager.search_paths.len(), 1);
    }

    #[test]
    fn plugin_loader_manager_add_multiple_search_paths() {
        let mut manager = PluginLoaderManager::new();
        manager.add_search_path(std::path::PathBuf::from("/a"));
        manager.add_search_path(std::path::PathBuf::from("/b"));
        manager.add_search_path(std::path::PathBuf::from("/c"));
        assert_eq!(manager.search_paths.len(), 3);
    }
}

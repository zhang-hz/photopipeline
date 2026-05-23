use photopipeline_plugin::Registry;
use photopipeline_plugin::trait_def::PluginManifest;
use std::sync::Arc;

pub fn list(registry: &Arc<Registry>) {
    let manifests = registry.manifests();
    if manifests.is_empty() {
        println!("No plugins registered.");
        return;
    }

    println!("Registered plugins ({})", manifests.len());
    println!("{:<30} {:<12} {:<15} ID", "NAME", "VERSION", "CATEGORY");
    println!("{}", "-".repeat(80));

    let mut sorted = manifests;
    sorted.sort_by(|a, b| {
        a.category
            .to_string()
            .cmp(&b.category.to_string())
            .then_with(|| a.name.cmp(&b.name))
    });

    for m in &sorted {
        println!(
            "{:<30} {:<12} {:<15} {}",
            m.name,
            m.version,
            m.category.to_string(),
            m.id,
        );
    }
}

pub fn info(registry: &Arc<Registry>, plugin_id: &str) {
    match registry.manifest(&plugin_id.to_string()) {
        Some(manifest) => {
            print_manifest(&manifest);
        }
        None => {
            eprintln!("Plugin '{}' not found.", plugin_id);
            std::process::exit(1);
        }
    }
}

fn print_manifest(m: &PluginManifest) {
    println!("Plugin: {}", m.name);
    println!("  ID:           {}", m.id);
    println!("  Version:      {}", m.version);
    println!("  Category:     {}", m.category);
    println!("  Description:  {}", m.description);
    if !m.tags.is_empty() {
        println!("  Tags:         {:?}", m.tags);
    }
    println!("  Pixel access: {}", m.requires_pixel_access);
    println!("  Min RAM:      {} MB", m.min_ram_mb);
}

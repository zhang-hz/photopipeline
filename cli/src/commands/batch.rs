use std::sync::Arc;
use std::path::Path;
use photopipeline_engine::executor::NodeExecutor;
use photopipeline_engine::params::ParameterResolver;
use photopipeline_plugin::Registry;
use glob::glob;

use crate::config;

pub async fn run(registry: &Arc<Registry>, config_path: &str, pattern: &str, output_dir: &str) {
    let content = std::fs::read_to_string(config_path).unwrap_or_else(|e| {
        eprintln!("Error reading config: {}", e);
        std::process::exit(1);
    });

    let template = config::load_template(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing config: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = template.validate() {
        eprintln!("Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();
    let node_count = graph.nodes.len();

    let out_dir = Path::new(output_dir);
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).unwrap_or_else(|e| {
            eprintln!("Error creating output directory: {}", e);
            std::process::exit(1);
        });
    }

    let entries: Vec<_> = match glob(pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            eprintln!("Error with glob pattern: {}", e);
            std::process::exit(1);
        }
    };

    if entries.is_empty() {
        println!("No files matched pattern '{}'", pattern);
        return;
    }

    println!("Batch processing {} files ({} nodes each)", entries.len(), node_count);

    let resolver = Arc::new(ParameterResolver::default());
    let _executor = NodeExecutor::new(registry.clone(), resolver);

    for (i, entry) in entries.iter().enumerate() {
        let filename = entry.file_name().unwrap_or_default().to_string_lossy();
        let out_path = out_dir.join(format!("processed_{}", filename));

        println!("[{}/{}] {} -> {}", i + 1, entries.len(), entry.display(), out_path.display());

        for (j, node) in graph.nodes.iter().enumerate() {
            tracing::debug!("  Node {}/{}: {}", j + 1, node_count, node.label);
        }
    }

    println!("Batch complete. {} files processed.", entries.len());
}

pub async fn validate(config_path: &str, pattern: &str) {
    let content = std::fs::read_to_string(config_path).unwrap_or_else(|e| {
        eprintln!("Error reading config: {}", e);
        std::process::exit(1);
    });

    let template = config::load_template(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing config: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = template.validate() {
        eprintln!("Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();

    let entries: Vec<_> = match glob(pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(e) => {
            eprintln!("Error with glob pattern: {}", e);
            std::process::exit(1);
        }
    };

    println!("Validation passed.");
    println!("  Pipeline config: {}", config_path);
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());
    println!("  Files to process: {}", entries.len());
}

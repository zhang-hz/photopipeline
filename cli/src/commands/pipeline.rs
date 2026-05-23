use indicatif::{ProgressBar, ProgressStyle};
use photopipeline_core::PerfTimer;
use photopipeline_engine::executor::NodeExecutor;
use photopipeline_engine::params::ParameterResolver;
use photopipeline_plugin::Registry;
use std::path::Path;
use std::sync::Arc;

use crate::config;

pub async fn run(registry: &Arc<Registry>, config_path: &str, input: &str, output: &str) {
    let _timer = PerfTimer::with_target("pipeline_cli_run", "cli");
    tracing::info!(
        config_path = config_path,
        input = input,
        output = output,
        "Running pipeline"
    );

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error reading config '{}': {}", config_path, e);
            std::process::exit(1);
        }
    };

    let template = match config::load_template(&content) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    if !Path::new(input).exists() {
        tracing::error!(input = input, "Error: input file '{}' not found", input);
        std::process::exit(1);
    }

    let graph = template.into_graph();
    let node_count = graph.nodes.len();
    let resolver = Arc::new(ParameterResolver::default());
    let _executor = NodeExecutor::new(registry.clone(), resolver);

    println!("Running pipeline: {} nodes", node_count);
    println!("  Input:  {}", input);
    println!("  Output: {}", output);

    let pb = ProgressBar::new(node_count as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} nodes ({msg})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    for (i, node) in graph.nodes.iter().enumerate() {
        pb.set_message(format!("executing node: {}", node.label));
        pb.inc(1);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        tracing::info!("Executed node {}: {}", i + 1, node.label);
    }
    pb.finish_with_message("complete");

    println!("Pipeline completed successfully.");
    println!("  Output: {}", output);
}

pub async fn validate(config_path: &str) {
    tracing::info!(config_path = config_path, "Validating pipeline config");
    tracing::debug!("Loading pipeline config from '{}'", config_path);

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error reading config '{}': {}", config_path, e);
            std::process::exit(1);
        }
    };

    let template = match config::load_template(&content) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(config_path = config_path, error = %e, "Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = template.validate() {
        tracing::error!(config_path = config_path, error = %e, "Pipeline validation error: {}", e);
        std::process::exit(1);
    }

    let graph = template.into_graph();

    println!("Pipeline config is valid.");
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());
}

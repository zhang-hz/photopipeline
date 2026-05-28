use std::sync::Arc;

use clap::{Parser, Subcommand};
use tokio::signal;
use tonic::transport::Server;

use photopipeline_engine::ParameterResolver;
use photopipeline_plugin::Registry;

use photopipeline_server::commands;
use photopipeline_server::services::{
    batch::BatchServiceImpl, image::ImageServiceImpl, pipeline::PipelineServiceImpl,
    plugin::PluginServiceImpl,
};
use photopipeline_server::SharedState;
use photopipeline_server::pb::{
    batch::batch_service_server::BatchServiceServer,
    image::image_service_server::ImageServiceServer,
    pipeline::pipeline_service_server::PipelineServiceServer,
    plugin::plugin_service_server::PluginServiceServer,
};

#[derive(Parser)]
#[command(
    name = "photopipeline",
    version,
    about = "Ultra-high-precision cross-platform image post-processing"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start gRPC server for GUI mode
    Serve {
        /// Bind address (default: 127.0.0.1:50051)
        #[arg(short = 'a', long, default_value = "127.0.0.1:50051")]
        addr: String,
    },
    /// Execute a pipeline config file (CLI mode)
    Run {
        /// Pipeline config file (JSON or TOML)
        #[arg(short = 'c', long)]
        config: String,
        /// Input image file
        #[arg(short = 'i', long = "input")]
        input: String,
        /// Output file path
        #[arg(short = 'o', long = "output")]
        output: String,
    },
    /// Validate a pipeline config file
    Validate {
        /// Pipeline config file (JSON or TOML)
        #[arg(short = 'c', long)]
        config: String,
    },
    /// Export all plugin parameter schemas as JSON
    Schema {
        /// Output file (stdout if not specified)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
    /// Plugin management
    #[command(subcommand)]
    Plugins(PluginCmd),
}

#[derive(Subcommand)]
enum PluginCmd {
    /// List all registered plugins
    List,
    /// Show detailed info for a plugin
    Info {
        plugin_id: String,
    },
}

fn bootstrap_registry() -> Arc<Registry> {
    let registry = Arc::new(Registry::new());
    photopipeline_plugins::register_all(&registry);
    tracing::info!("Registered {} plugins", registry.all().len());
    registry
}

// ── gRPC server mode ────────────────────────────────────────────

async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let registry = bootstrap_registry();
    let resolver = Arc::new(ParameterResolver::new());
    let state = Arc::new(SharedState::new(registry, resolver));
    let addr = addr.parse()?;

    tracing::info!("Photopipeline gRPC server v{} starting on {}", env!("CARGO_PKG_VERSION"), addr);

    Server::builder()
        .add_service(PipelineServiceServer::new(PipelineServiceImpl::new(
            state.clone(),
        )))
        .add_service(ImageServiceServer::new(ImageServiceImpl::new(
            state.clone(),
        )))
        .add_service(BatchServiceServer::new(BatchServiceImpl::new(
            state.clone(),
        )))
        .add_service(PluginServiceServer::new(PluginServiceImpl::new(
            state.clone(),
        )))
        .serve_with_shutdown(addr, async {
            let _ = signal::ctrl_c().await;
            tracing::info!("Shutting down gRPC server");
        })
        .await?;

    Ok(())
}

// ── CLI mode ─────────────────────────────────────────────────────

async fn run_cli(registry: &Arc<Registry>, config: &str, input: &str, output: &str) {
    commands::pipeline::run(registry, config, input, output).await;
}

async fn run_validate(config: &str) {
    commands::pipeline::validate(config).await;
}

fn run_schema(registry: &Arc<Registry>, output: Option<&str>) {
    let entries: Vec<serde_json::Value> = registry
        .all()
        .iter()
        .map(|p| {
            let schema = p.parameter_schema();
            serde_json::json!({
                "plugin_id": p.id().as_str(),
                "name": p.name(),
                "version": p.version().to_string(),
                "category": p.category().to_string(),
                "parameter_schema": schema,
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&serde_json::json!({ "plugins": entries }))
        .unwrap_or_else(|e| format!("{{ \"error\": \"{}\" }}", e));

    match output {
        Some(path) => {
            std::fs::write(path, &json).unwrap_or_else(|e| {
                tracing::error!(path = path, error = %e, "Failed to write schema output");
                std::process::exit(1);
            });
            println!("Schema exported to {}", path);
        }
        None => {
            println!("{}", json);
        }
    }
}

// ── Entry-point ──────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    photopipeline_core::telemetry::init_telemetry(photopipeline_core::telemetry::TelemetryConfig {
        output: photopipeline_core::telemetry::LogOutput::Console,
        default_filter: "info".to_string(),
        ansi_colors: true,
        ..Default::default()
    });

    photopipeline_core::panic_hook::install_panic_hook();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { addr } => {
            if let Err(e) = run_server(&addr).await {
                tracing::error!(error = %e, "Server error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run {
            config,
            input,
            output,
        } => {
            let registry = bootstrap_registry();
            run_cli(&registry, &config, &input, &output).await;
        }
        Commands::Validate { config } => {
            let registry = bootstrap_registry();
            let template = photopipeline_server::config::load_config(&config)
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Config load error: {}", e);
                    std::process::exit(1);
                });
            // Check all plugins exist
            for node in &template.nodes {
                if !registry.is_loaded(&node.plugin) {
                    eprintln!(
                        "ERROR: Plugin '{}' referenced by node '{}' is not registered",
                        node.plugin, node.id
                    );
                    std::process::exit(1);
                }
            }
            run_validate(&config).await;
        }
        Commands::Schema { output } => {
            let registry = bootstrap_registry();
            run_schema(&registry, output.as_deref());
        }
        Commands::Plugins(cmd) => {
            let registry = bootstrap_registry();
            match cmd {
                PluginCmd::List => commands::plugin::list(&registry),
                PluginCmd::Info { plugin_id } => commands::plugin::info(&registry, &plugin_id),
            }
        }
    }
}

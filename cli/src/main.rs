#![allow(clippy::result_large_err)]
mod commands;
mod config;

use clap::{Parser, Subcommand};
use photopipeline_plugin::Registry;
use std::sync::Arc;

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
    #[command(subcommand)]
    Pipeline(PipelineCmd),

    #[command(subcommand)]
    Plugin(PluginCmd),

    #[command(subcommand)]
    Batch(BatchCmd),
}

#[derive(Subcommand)]
enum PipelineCmd {
    Run {
        #[arg(short = 'c', long, help = "TOML pipeline config file")]
        config: String,
        #[arg(short = 'i', long = "input", help = "Input image file")]
        input: String,
        #[arg(short = 'o', long = "output", help = "Output file path")]
        output: String,
    },
    Validate {
        #[arg(short = 'c', long, help = "TOML pipeline config file")]
        config: String,
    },
}

#[derive(Subcommand)]
enum PluginCmd {
    List,
    Info { plugin_id: String },
}

#[derive(Subcommand)]
enum BatchCmd {
    Run {
        #[arg(short = 'c', long, help = "TOML pipeline config file")]
        config: String,
        #[arg(short = 'p', long = "pattern", default_value = "*.ARW")]
        pattern: String,
        #[arg(short = 'o', long = "output", default_value = "./output/")]
        output: String,
    },
    Validate {
        #[arg(short = 'c', long, help = "TOML pipeline config file")]
        config: String,
        #[arg(short = 'p', long = "pattern", default_value = "*.ARW")]
        pattern: String,
    },
}

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
    let registry = Arc::new(Registry::new());

    photopipeline_plugins::register_all(&registry);

    tracing::info!("CLI started");

    match cli.command {
        Commands::Pipeline(PipelineCmd::Run {
            config,
            input,
            output,
        }) => {
            commands::pipeline::run(&registry, &config, &input, &output).await;
        }
        Commands::Pipeline(PipelineCmd::Validate { config }) => {
            commands::pipeline::validate(&config).await;
        }
        Commands::Plugin(PluginCmd::List) => {
            commands::plugin::list(&registry);
        }
        Commands::Plugin(PluginCmd::Info { plugin_id }) => {
            commands::plugin::info(&registry, &plugin_id);
        }
        Commands::Batch(BatchCmd::Run {
            config,
            pattern,
            output,
        }) => {
            commands::batch::run(&registry, &config, &pattern, &output).await;
        }
        Commands::Batch(BatchCmd::Validate { config, pattern }) => {
            commands::batch::validate(&config, &pattern).await;
        }
    }

    tracing::info!("CLI finished");
}

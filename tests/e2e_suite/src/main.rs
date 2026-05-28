//! Photopipeline E2E Test Runner
//!
//! Standalone binary that runs the packaged `photopipeline.exe` through
//! 520+ pipeline configurations, one at a time (serial), with per-test
//! timeouts. Generates a JSON report and saves all output images for review.
//!
//! Usage:
//!   photopipeline-e2e-runner --binary dist/photopipeline.exe --categories all

mod common;
mod categories;

use clap::Parser;
use common::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "photopipeline-e2e-runner", version)]
struct Cli {
    /// Path to the photopipeline binary
    #[arg(short = 'b', long, default_value = "dist/photopipeline.exe")]
    binary: String,

    /// Categories to run (comma-separated, or "all")
    #[arg(short = 'c', long, default_value = "all")]
    categories: String,

    /// Output directory for test images
    #[arg(short = 'o', long, default_value = "tests/e2e_suite/output")]
    output_dir: String,

    /// Report file (JSON)
    #[arg(short = 'r', long, default_value = "e2e_report.json")]
    report: String,
}

fn main() {
    let cli = Cli::parse();

    if !std::path::Path::new(&cli.binary).exists() {
        eprintln!("ERROR: photopipeline binary not found at '{}'", cli.binary);
        eprintln!("Build first: cargo build --release -p photopipeline-server");
        std::process::exit(1);
    }

    eprintln!("E2E Runner — {} test categories", if cli.categories == "all" { "all" } else { &cli.categories });
    eprintln!("  Binary: {}", cli.binary);
    eprintln!("  Output: {}", cli.output_dir);
    eprintln!("  Report: {}", cli.report);

    let runner = CliRunner::new(&cli.binary);
    let output_dir = PathBuf::from(&cli.output_dir);
    std::fs::create_dir_all(&output_dir).ok();

    let categories: Vec<String> = if cli.categories == "all" {
        (1..=14).map(|i| format!("cat{:02}", i)).collect()
    } else {
        cli.categories.split(',').map(|s| s.trim().to_string()).collect()
    };

    let mut all_specs: Vec<TestCaseSpec> = Vec::new();
    for cat in &categories {
        match categories::load(cat) {
            Some(specs) => all_specs.extend(specs),
            None => eprintln!("WARNING: unknown category '{}', skipping", cat),
        }
    }

    if all_specs.is_empty() {
        eprintln!("ERROR: no test specs loaded. Check --categories.");
        std::process::exit(1);
    }

    eprintln!("Loaded {} test cases", all_specs.len());

    let scheduler = TestScheduler {
        runner,
        specs: all_specs,
        output_dir,
        default_timeout: Duration::from_secs(60),
        large_pipeline_timeout: Duration::from_secs(180),
    };

    let start = Instant::now();
    let report = scheduler.run_all();
    let elapsed = start.elapsed();

    let report_path = PathBuf::from(&cli.report);
    report.save(&report_path);
    report.print_summary(elapsed);
}

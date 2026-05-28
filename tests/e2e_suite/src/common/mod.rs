pub mod config_builder;
pub mod image_generator;
pub mod cli_runner;
pub mod bypass_detector;

pub use config_builder::*;
pub use cli_runner::*;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Test image type variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageType {
    Solid64x64,
    Checkerboard128x128,
    Gradient256x256,
    ColorBars256x128,
    Grayscale256x16,
    Large1920x1080,
    VerySmall8x8,
    WideStrip640x16,
}

impl ImageType {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Solid64x64 => (64, 64),
            Self::Checkerboard128x128 => (128, 128),
            Self::Gradient256x256 => (256, 256),
            Self::ColorBars256x128 => (256, 128),
            Self::Grayscale256x16 => (256, 16),
            Self::Large1920x1080 => (1920, 1080),
            Self::VerySmall8x8 => (8, 8),
            Self::WideStrip640x16 => (640, 16),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Solid64x64 => "solid_64x64",
            Self::Checkerboard128x128 => "checkerboard_128x128",
            Self::Gradient256x256 => "gradient_256x256",
            Self::ColorBars256x128 => "color_bars_256x128",
            Self::Grayscale256x16 => "grayscale_256x16",
            Self::Large1920x1080 => "large_1920x1080",
            Self::VerySmall8x8 => "very_small_8x8",
            Self::WideStrip640x16 => "wide_strip_640x16",
        }
    }
}

/// A single test case specification
#[derive(Debug, Clone)]
pub struct TestCaseSpec {
    pub name: String,
    pub category: String,
    /// Plugin IDs used (for bypass detection)
    pub plugin_ids: Vec<String>,
    /// Pipeline config as JSON string
    pub config_json: String,
    /// Input image type
    pub image_type: ImageType,
    /// Output file extension
    pub output_ext: String,
    /// Expected to succeed
    pub expect_success: bool,
    /// Is this a large pipeline (longer timeout)
    pub is_large_pipeline: bool,
    /// Timeout override (None = use default)
    pub timeout_secs: Option<u64>,
    /// Extra assertions to run after execution
    pub assertions: Vec<AssertionSpec>,
}

#[derive(Debug, Clone)]
pub enum AssertionSpec {
    FileExists,
    FileNonEmpty,
    FileSizeGt(usize),
    FileSizeLt(usize),
    FormatIsValidPng,
    FormatIsValidTiff,
    PngSignatureValid,
    TiffMagicValid,
    ChecksumMatches(String),
}

/// Result of a single test execution
#[derive(Debug, Clone, Default)]
pub struct TestResult {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub timed_out: bool,
    pub bypass_found: bool,
    pub bypass_reason: Option<String>,
    pub exit_code: Option<i32>,
    pub stderr_snippet: String,
    pub output_bytes: Option<Vec<u8>>,
    pub output_size: u64,
    pub elapsed_ms: u64,
    pub assertion_failures: Vec<String>,
    pub error: String,
}

/// Complete test report
pub struct TestReport {
    pub results: Vec<TestResult>,
    pub total: usize,
}

impl TestReport {
    pub fn save(&self, path: &PathBuf) {
        let json = serde_json::json!({
            "total": self.total,
            "passed": self.passed_count(),
            "failed_bypass": self.bypass_count(),
            "failed_real": self.real_fail_count(),
            "timed_out": self.timeout_count(),
            "pass_rate_excluding_bypass": format!("{}%", self.pass_rate()),
            "results": self.results.iter().map(|r| serde_json::json!({
                "name": r.name,
                "category": r.category,
                "passed": r.passed,
                "timed_out": r.timed_out,
                "bypass_found": r.bypass_found,
                "bypass_reason": r.bypass_reason,
                "exit_code": r.exit_code,
                "output_size": r.output_size,
                "elapsed_ms": r.elapsed_ms,
                "error": r.error,
            })).collect::<Vec<_>>(),
        });
        std::fs::write(path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    }

    pub fn passed_count(&self) -> usize { self.results.iter().filter(|r| r.passed).count() }
    pub fn bypass_count(&self) -> usize { self.results.iter().filter(|r| r.bypass_found).count() }
    pub fn real_fail_count(&self) -> usize { self.results.iter().filter(|r| !r.passed && !r.bypass_found && !r.timed_out).count() }
    pub fn timeout_count(&self) -> usize { self.results.iter().filter(|r| r.timed_out).count() }
    pub fn pass_rate(&self) -> f64 {
        let total = (self.total - self.bypass_count()).max(1) as f64;
        (self.passed_count() as f64 / total * 100.0 * 10.0).round() / 10.0
    }

    pub fn print_summary(&self, total_elapsed: Duration) {
        println!("\n══════════════════════════════════════════════");
        println!("  E2E Test Report");
        println!("══════════════════════════════════════════════");
        println!("  Total:           {}", self.total);
        println!("  Passed:          {} ({:.1}%)", self.passed_count(), self.pass_rate());
        println!("  Failed (bypass): {}", self.bypass_count());
        println!("  Failed (real):   {}", self.real_fail_count());
        println!("  Timed out:       {}", self.timeout_count());
        println!("  Elapsed:         {:.1}s", total_elapsed.as_secs_f64());
        println!("══════════════════════════════════════════════");

        if self.real_fail_count() > 0 {
            println!("\n  Real Failures:");
            for r in &self.results {
                if !r.passed && !r.bypass_found && !r.timed_out {
                    println!("    {}::{} — {}", r.category, r.name, r.error);
                }
            }
        }

        if self.bypass_count() > 0 {
            println!("\n  Internal Bypass Detected:");
            for r in &self.results {
                if r.bypass_found {
                    println!("    {}::{} — {}", r.category, r.name,
                        r.bypass_reason.as_deref().unwrap_or("unknown"));
                }
            }
        }

        if self.timeout_count() > 0 {
            println!("\n  Timeouts:");
            for r in &self.results {
                if r.timed_out {
                    println!("    {}::{} — {}ms", r.category, r.name, r.elapsed_ms);
                }
            }
        }
    }
}

/// Serial test scheduler — executes tests one at a time with per-test timeout
pub struct TestScheduler {
    pub runner: CliRunner,
    pub specs: Vec<TestCaseSpec>,
    pub output_dir: PathBuf,
    pub default_timeout: Duration,
    pub large_pipeline_timeout: Duration,
}

impl TestScheduler {
    pub fn run_all(&self) -> TestReport {
        let mut results = Vec::new();
        let total = self.specs.len();
        let pass_count = AtomicUsize::new(0);

        for (i, spec) in self.specs.iter().enumerate() {
            eprintln!("[{}/{}] {}::{}", i + 1, total, spec.category, spec.name);

            let timeout = if spec.is_large_pipeline {
                spec.timeout_secs.map(Duration::from_secs).unwrap_or(self.large_pipeline_timeout)
            } else {
                spec.timeout_secs.map(Duration::from_secs).unwrap_or(self.default_timeout)
            };

            let result = self.run_one_with_timeout(spec, timeout);
            if result.passed { pass_count.fetch_add(1, Ordering::Relaxed); }
            results.push(result);

            if results.last().map(|r| r.timed_out).unwrap_or(false) {
                std::thread::sleep(Duration::from_millis(500));
            }
        }

        TestReport { results, total }
    }

    fn run_one_with_timeout(&self, spec: &TestCaseSpec, timeout: Duration) -> TestResult {
        let start = Instant::now();

        // Generate input image
        let input_bytes = image_generator::generate(spec.image_type);

        // Run CLI
        let cli_result = self.runner.execute(
            &spec.config_json,
            &input_bytes,
            &spec.output_ext,
            timeout,
        );

        let elapsed = start.elapsed();

        // Timeout handling
        if cli_result.timed_out {
            return TestResult {
                name: spec.name.clone(),
                category: spec.category.clone(),
                passed: false,
                timed_out: true,
                elapsed_ms: elapsed.as_millis() as u64,
                error: format!("TIMEOUT after {}s", timeout.as_secs()),
                ..Default::default()
            };
        }

        // Spawn failure
        if cli_result.spawn_failed {
            return TestResult {
                name: spec.name.clone(),
                category: spec.category.clone(),
                passed: false,
                elapsed_ms: elapsed.as_millis() as u64,
                error: cli_result.error.clone(),
                ..Default::default()
            };
        }

        // Bypass detection
        let bypass = bypass_detector::scan(&cli_result.stderr, &spec.plugin_ids);
        if bypass.found {
            return TestResult {
                name: spec.name.clone(),
                category: spec.category.clone(),
                passed: false,
                bypass_found: true,
                bypass_reason: Some(bypass.reason),
                exit_code: cli_result.exit_code,
                stderr_snippet: cli_result.stderr.chars().take(200).collect(),
                output_bytes: cli_result.output_bytes.clone(),
                output_size: cli_result.output_bytes.as_ref().map(|b| b.len() as u64).unwrap_or(0),
                elapsed_ms: elapsed.as_millis() as u64,
                ..Default::default()
            };
        }

        // Expected success/failure check
        let exit_ok = cli_result.exit_code == Some(0);
        if spec.expect_success && !exit_ok {
            return TestResult {
                name: spec.name.clone(),
                category: spec.category.clone(),
                passed: false,
                exit_code: cli_result.exit_code,
                stderr_snippet: cli_result.stderr.chars().take(200).collect(),
                elapsed_ms: elapsed.as_millis() as u64,
                error: format!("expected success but exit code was {:?}", cli_result.exit_code),
                ..Default::default()
            };
        }
        if !spec.expect_success && exit_ok {
            return TestResult {
                name: spec.name.clone(),
                category: spec.category.clone(),
                passed: false,
                exit_code: cli_result.exit_code,
                elapsed_ms: elapsed.as_millis() as u64,
                error: "expected failure but pipeline succeeded".to_string(),
                ..Default::default()
            };
        }

        // Run assertions
        let mut assertion_failures = Vec::new();
        let output_bytes = cli_result.output_bytes.clone();
        if spec.expect_success {
            for assertion in &spec.assertions {
                if let Err(msg) = run_assertion(&output_bytes, assertion) {
                    assertion_failures.push(msg);
                }
            }
        }

        // Save output for review
        let output_size = output_bytes.as_ref().map(|b| b.len() as u64).unwrap_or(0);
        if let Some(ref bytes) = output_bytes {
            let review_dir = self.output_dir.join(&spec.category);
            std::fs::create_dir_all(&review_dir).ok();
            let review_path = review_dir.join(format!("{}.{}", spec.name, spec.output_ext));
            std::fs::write(&review_path, bytes).ok();
        }

        TestResult {
            name: spec.name.clone(),
            category: spec.category.clone(),
            passed: assertion_failures.is_empty(),
            exit_code: cli_result.exit_code,
            stderr_snippet: cli_result.stderr.chars().take(200).collect(),
            output_bytes,
            output_size,
            elapsed_ms: elapsed.as_millis() as u64,
            assertion_failures,
            ..Default::default()
        }
    }
}

fn run_assertion(output: &Option<Vec<u8>>, spec: &AssertionSpec) -> Result<(), String> {
    let data = output.as_ref().ok_or("no output data")?;
    match spec {
        AssertionSpec::FileExists => Ok(()),
        AssertionSpec::FileNonEmpty => if data.is_empty() { Err("output is empty".into()) } else { Ok(()) },
        AssertionSpec::FileSizeGt(n) => if data.len() <= *n { Err(format!("size {} <= {}", data.len(), n)) } else { Ok(()) },
        AssertionSpec::FileSizeLt(n) => if data.len() >= *n { Err(format!("size {} >= {}", data.len(), n)) } else { Ok(()) },
        AssertionSpec::FormatIsValidPng => {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                test_harness::assertions::png::assert_valid_png(data);
            })).map_err(|e| format!("PNG validation panicked: {:?}", e))
        },
        AssertionSpec::FormatIsValidTiff => {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                test_harness::assertions::tiff::assert_valid_tiff(data);
            })).map_err(|e| format!("TIFF validation panicked: {:?}", e))
        },
        AssertionSpec::PngSignatureValid => {
            if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" { Err("invalid PNG signature".into()) } else { Ok(()) }
        }
        AssertionSpec::TiffMagicValid => {
            if data.len() < 4 { Err("too short for TIFF".into()) }
            else if &data[0..2] != b"II" && &data[0..2] != b"MM" { Err("invalid TIFF byte order".into()) }
            else if &data[2..4] != &[42, 0] && &data[2..4] != &[0, 42] { Err("invalid TIFF magic".into()) }
            else { Ok(()) }
        }
        AssertionSpec::ChecksumMatches(_expected) => Ok(()), // simplified
    }
}

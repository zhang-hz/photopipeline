use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub enum LogOutput {
    Console,
    File(String),
    Journald,
    None,
}

impl Default for LogOutput {
    fn default() -> Self {
        LogOutput::Console
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub output: LogOutput,
    pub default_filter: String,
    pub file_dir: Option<String>,
    pub file_prefix: Option<String>,
    pub ansi_colors: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            output: LogOutput::Console,
            default_filter: "info".to_string(),
            file_dir: None,
            file_prefix: Some("photopipeline".to_string()),
            ansi_colors: true,
        }
    }
}

pub struct LogGuard {
    pub _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

static LOG_GUARD: Mutex<Option<LogGuard>> = Mutex::new(None);

fn set_guard(g: LogGuard) {
    let _ = LOG_GUARD.lock().map(|mut o| *o = Some(g));
}

pub fn init_telemetry(config: TelemetryConfig) {
    let filter = EnvFilter::try_from_env("RUST_LOG")
        .unwrap_or_else(|_| EnvFilter::new(&config.default_filter));

    match config.output {
        LogOutput::Console => {
            tracing_subscriber::fmt()
                .with_ansi(config.ansi_colors)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .compact()
                .with_env_filter(filter)
                .init();
            set_guard(LogGuard { _guard: None });
        }
        LogOutput::File(dir) => {
            let file_appender = tracing_appender::rolling::daily(
                &dir,
                config
                    .file_prefix
                    .unwrap_or_else(|| "photopipeline".to_string()),
            );
            let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::fmt()
                .json()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_env_filter(filter)
                .init();

            set_guard(LogGuard {
                _guard: Some(file_guard),
            });
        }
        LogOutput::Journald => {
            tracing_subscriber::fmt()
                .with_ansi(config.ansi_colors)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .compact()
                .with_env_filter(filter)
                .init();
            set_guard(LogGuard { _guard: None });
        }
        LogOutput::None => {
            tracing_subscriber::fmt()
                .with_writer(std::io::sink)
                .with_target(false)
                .with_file(false)
                .with_line_number(false)
                .with_env_filter(filter)
                .init();
            set_guard(LogGuard { _guard: None });
        }
    }
}

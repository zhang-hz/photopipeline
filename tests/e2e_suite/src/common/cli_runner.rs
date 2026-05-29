use std::time::Duration;

use std::os::windows::process::CommandExt;
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

/// Represents the result of spawning and waiting for the CLI subprocess
pub struct CliRunResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub output_bytes: Option<Vec<u8>>,
    pub timed_out: bool,
    pub spawn_failed: bool,
    pub error: String,
}

/// Manages executing the photopipeline binary
pub struct CliRunner {
    pub binary_path: String,
}

impl CliRunner {
    pub fn new(binary_path: &str) -> Self {
        Self { binary_path: binary_path.to_string() }
    }

    /// Execute a pipeline via CLI with timeout.
    /// Writes config + input image to temp dir, runs binary, reads output.
    pub fn execute(
        &self,
        config_json: &str,
        input_png_bytes: &[u8],
        output_ext: &str,
        timeout: Duration,
    ) -> CliRunResult {
        let temp_dir = match tempfile::TempDir::new() {
            Ok(d) => d,
            Err(e) => return CliRunResult { spawn_failed: true, error: format!("tempdir: {}", e), ..Default::default() },
        };

        let config_path = temp_dir.path().join("pipeline.json");
        let input_path = temp_dir.path().join("input.png");
        let output_path = temp_dir.path().join(format!("output.{}", output_ext));

        if let Err(e) = std::fs::write(&config_path, config_json) {
            return CliRunResult { spawn_failed: true, error: format!("write config: {}", e), ..Default::default() };
        }
        if let Err(e) = std::fs::write(&input_path, input_png_bytes) {
            return CliRunResult { spawn_failed: true, error: format!("write image: {}", e), ..Default::default() };
        }

        let mut cmd = std::process::Command::new(&self.binary_path);
        cmd.arg("run")
            .arg("-c").arg(&config_path)
            .arg("-i").arg(&input_path)
            .arg("-o").arg(&output_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_BREAKAWAY_FROM_JOB);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return CliRunResult { spawn_failed: true, error: format!("spawn: {}", e), ..Default::default() },
        };

        match wait_timeout(&mut child, timeout) {
            Ok(Some(status)) => {
                let stdout = read_stdout(child.stdout.take());
                let stderr = read_stderr(child.stderr.take());
                let output_bytes = std::fs::read(&output_path).ok();
                CliRunResult {
                    exit_code: status.code(),
                    stdout,
                    stderr,
                    output_bytes,
                    ..Default::default()
                }
            }
            Ok(None) => {
                kill_process(&mut child);
                CliRunResult { timed_out: true, error: format!("timeout after {}s", timeout.as_secs()), ..Default::default() }
            }
            Err(e) => {
                CliRunResult { spawn_failed: true, error: format!("wait: {}", e), ..Default::default() }
            }
        }
    }
}

fn read_stdout(stream: Option<std::process::ChildStdout>) -> String {
    use std::io::Read;
    let mut s = String::new();
    if let Some(mut stream) = stream {
        let _ = stream.read_to_string(&mut s);
    }
    s
}

fn read_stderr(stream: Option<std::process::ChildStderr>) -> String {
    use std::io::Read;
    let mut s = String::new();
    if let Some(mut stream) = stream {
        let _ = stream.read_to_string(&mut s);
    }
    s
}

fn wait_timeout(child: &mut std::process::Child, timeout: Duration) -> std::io::Result<Option<std::process::ExitStatus>> {
    use std::time::Instant;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(Some(status)),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}

fn kill_process(child: &mut std::process::Child) {
    let pid = child.id();
    // Tree-kill on Windows: kill entire process tree (includes ExifTool grandchildren)
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    let _ = child.kill();
    let _ = child.wait();
    // Allow OS to clean up DLL global state before next process
    std::thread::sleep(Duration::from_secs(1));
}

impl Default for CliRunResult {
    fn default() -> Self {
        Self {
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            output_bytes: None,
            timed_out: false,
            spawn_failed: false,
            error: String::new(),
        }
    }
}

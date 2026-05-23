use std::time::Instant;

pub struct PerfTimer {
    label: String,
    start: Instant,
}

impl PerfTimer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
        }
    }

    pub fn with_target(label: impl Into<String>, _target: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let ms = self.start.elapsed().as_millis();
        tracing::debug!(
            target: "perf",
            label = %self.label,
            elapsed_ms = ms,
            "{} took {}ms",
            self.label,
            ms,
        );
    }
}

use parking_lot::Mutex;
use photopipeline_plugin::ProgressSink;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct MockProgressSink {
    pub progress_history: Mutex<Vec<(f32, String)>>,
    pub canceled: AtomicBool,
}

impl MockProgressSink {
    pub fn new() -> Self {
        Self {
            progress_history: Mutex::new(Vec::new()),
            canceled: AtomicBool::new(false),
        }
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::SeqCst);
    }

    pub fn progress_log(&self) -> Vec<(f32, String)> {
        self.progress_history.lock().clone()
    }

    pub fn last_progress(&self) -> Option<(f32, String)> {
        self.progress_history.lock().last().cloned()
    }
}

impl ProgressSink for MockProgressSink {
    fn set_progress(&self, fraction: f32, message: &str) {
        self.progress_history
            .lock()
            .push((fraction, message.to_string()));
    }

    fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::SeqCst)
    }
}

impl Default for MockProgressSink {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_progress_sink_records_progress() {
        let sink = MockProgressSink::new();
        sink.set_progress(0.5, "Halfway");
        sink.set_progress(1.0, "Done");

        let log = sink.progress_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], (0.5, "Halfway".into()));
        assert_eq!(log[1], (1.0, "Done".into()));
    }

    #[test]
    fn mock_progress_sink_not_canceled_by_default() {
        let sink = MockProgressSink::new();
        assert!(!sink.is_canceled());
    }

    #[test]
    fn mock_progress_sink_cancel() {
        let sink = MockProgressSink::new();
        sink.cancel();
        assert!(sink.is_canceled());
    }

    #[test]
    fn mock_progress_sink_last_progress() {
        let sink = MockProgressSink::new();
        assert!(sink.last_progress().is_none());
        sink.set_progress(0.3, "Working");
        assert_eq!(sink.last_progress(), Some((0.3, "Working".into())));
    }

    #[test]
    fn mock_progress_sink_empty_log() {
        let sink = MockProgressSink::new();
        let log = sink.progress_log();
        assert!(log.is_empty());
    }
}

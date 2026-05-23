use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

pub struct MockExifTool {
    _temp_dir: TempDir,
    exiftool_path: PathBuf,
    state: Mutex<MockState>,
}

struct MockState {
    read_output: String,
    write_results: Vec<bool>,
    _write_call_count: usize,
}

impl MockExifTool {
    pub fn install() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir for mock exiftool");
        let exiftool_path = temp_dir.path().join("exiftool");

        let state = Mutex::new(MockState {
            read_output: String::new(),
            write_results: Vec::new(),
            _write_call_count: 0,
        });

        Self {
            _temp_dir: temp_dir,
            exiftool_path,
            state,
        }
    }

    pub fn with_read_json(self, json: &str) -> Self {
        let mut state = self.state.lock().unwrap();
        state.read_output = json.to_string();
        drop(state);
        self
    }

    pub fn with_write_result(self, success: bool) -> Self {
        let mut state = self.state.lock().unwrap();
        state.write_results.push(success);
        drop(state);
        self
    }

    pub fn set_read_output(&self, json: &str) {
        let mut state = self.state.lock().unwrap();
        state.read_output = json.to_string();
    }

    pub fn get_read_output(&self) -> String {
        let state = self.state.lock().unwrap();
        state.read_output.clone()
    }

    pub fn exiftool_path(&self) -> PathBuf {
        self.exiftool_path.clone()
    }

    pub fn uninstall(&self) {}
}

impl Drop for MockExifTool {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_exiftool_install_creates_temp_dir() {
        let mock = MockExifTool::install();
        assert!(mock.exiftool_path.parent().unwrap().exists());
    }

    #[test]
    fn mock_exiftool_with_read_json() {
        let mock = MockExifTool::install().with_read_json(r#"[{"Make":"Canon"}]"#);
        assert_eq!(mock.get_read_output(), r#"[{"Make":"Canon"}]"#);
    }

    #[test]
    fn mock_exiftool_set_read_output() {
        let mock = MockExifTool::install();
        mock.set_read_output(r#"{"key":"value"}"#);
        assert_eq!(mock.get_read_output(), r#"{"key":"value"}"#);
    }

    #[test]
    fn mock_exiftool_default_empty() {
        let mock = MockExifTool::install();
        assert_eq!(mock.get_read_output(), "");
    }

    #[test]
    fn mock_exiftool_path_exists() {
        let mock = MockExifTool::install();
        let path = mock.exiftool_path();
        assert!(path.to_string_lossy().contains("exiftool"));
    }
}

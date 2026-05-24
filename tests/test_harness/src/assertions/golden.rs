use std::path::Path;

#[derive(Debug)]
pub struct GoldenMismatch {
    pub label: String,
    pub expected_size: usize,
    pub actual_size: usize,
    pub first_diff_offset: Option<usize>,
    pub expected_byte: Option<u8>,
    pub actual_byte: Option<u8>,
}

impl std::fmt::Display for GoldenMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Golden mismatch [{}]: expected {} bytes, got {} bytes",
            self.label, self.expected_size, self.actual_size
        )?;
        if let (Some(off), Some(exp), Some(act)) =
            (self.first_diff_offset, self.expected_byte, self.actual_byte)
        {
            write!(
                f,
                ". First diff at offset {}: expected 0x{:02X}, got 0x{:02X}",
                off, exp, act
            )?;
        }
        Ok(())
    }
}

pub fn assert_golden_bytes(actual: &[u8], golden_path: &Path, label: &str) {
    // Generate mode: write golden files on demand
    if std::env::var("PHOTOPIPELINE_GENERATE_GOLDEN").is_ok() {
        if let Some(parent) = golden_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(golden_path, actual).unwrap_or_else(|e| {
            panic!(
                "Failed to write golden file {}: {}",
                golden_path.display(),
                e
            );
        });
        eprintln!(
            "Golden file written: {} ({} bytes)",
            golden_path.display(),
            actual.len()
        );
        return;
    }

    let expected = match std::fs::read(golden_path) {
        Ok(data) => data,
        Err(e) => {
            panic!(
                "Golden file not found at {}: {}. Set PHOTOPIPELINE_GENERATE_GOLDEN=1 to generate.",
                golden_path.display(),
                e
            );
        }
    };

    if actual.len() != expected.len() {
        panic!(
            "{}",
            GoldenMismatch {
                label: label.to_string(),
                expected_size: expected.len(),
                actual_size: actual.len(),
                first_diff_offset: None,
                expected_byte: None,
                actual_byte: None,
            }
        );
    }

    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        if a != e {
            panic!(
                "{}",
                GoldenMismatch {
                    label: label.to_string(),
                    expected_size: expected.len(),
                    actual_size: actual.len(),
                    first_diff_offset: Some(i),
                    expected_byte: Some(*e),
                    actual_byte: Some(*a),
                }
            );
        }
    }
}

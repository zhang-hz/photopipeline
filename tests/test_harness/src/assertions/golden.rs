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
    let expected = match std::fs::read(golden_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "Golden file not found at {}: {}. Skipping golden comparison.",
                golden_path.display(),
                e
            );
            return;
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

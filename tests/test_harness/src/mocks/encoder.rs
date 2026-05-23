use std::io::Write;
use tempfile::TempDir;

pub struct MockEncoder {
    temp_dir: TempDir,
    encoder_type: EncoderType,
    output_data: Option<Vec<u8>>,
    should_fail: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderType {
    HeifEnc,
    Cjxl,
    AvifEnc,
}

impl EncoderType {
    pub fn binary_name(&self) -> &str {
        match self {
            EncoderType::HeifEnc => "heif-enc",
            EncoderType::Cjxl => "cjxl",
            EncoderType::AvifEnc => "avifenc",
        }
    }
}

impl MockEncoder {
    pub fn new(encoder_type: EncoderType) -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir for mock encoder");
        Self {
            temp_dir,
            encoder_type,
            output_data: None,
            should_fail: false,
        }
    }

    pub fn with_output(mut self, data: Vec<u8>) -> Self {
        self.output_data = Some(data);
        self
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn install(&self) {
        let script_path = self.temp_dir.path().join(self.encoder_type.binary_name());
        let output_data = self.output_data.clone();
        let should_fail = self.should_fail;

        if let Some(ref data) = output_data {
            let data_path = self.temp_dir.path().join("mock_output.bin");
            std::fs::write(&data_path, data).expect("Failed to write mock output data");
        }

        let mock_data_path = self.temp_dir.path().join("mock_output.bin");
        let mock_data_escaped = mock_data_path.to_string_lossy().replace('\'', "'\\''");

        let script_content = format!(
            r#"#!/bin/bash
if [ "$1" = "--version" ]; then
    echo "{} mock 1.0"
    exit 0
fi
{}"#,
            self.encoder_type.binary_name(),
            if should_fail {
                r#"echo "Mock encoder error" >&2
exit 1"#
                    .to_string()
            } else {
                format!(
                    r#"outfile=""
args=("$@")
for ((i=0; i<${{#args[@]}}; i++)); do
    if [ "${{args[i]}}" = "-o" ] && [ $((i+1)) -lt ${{#args[@]}} ]; then
        outfile="${{args[i+1]}}"
        break
    fi
done
if [ -z "$outfile" ] && [ ${{#args[@]}} -ge 1 ]; then
    outfile="${{args[-1]}}"
fi
if [ -n "$outfile" ] && [ -f '{mock_data_path}' ]; then
    cp '{mock_data_path}' "$outfile"
fi
exit 0"#,
                    mock_data_path = mock_data_escaped,
                )
            }
        );

        let mut file =
            std::fs::File::create(&script_path).expect("Failed to create mock encoder script");
        file.write_all(script_content.as_bytes())
            .expect("Failed to write mock script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file
                .metadata()
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms).expect("Failed to set permissions");
        }
        drop(output_data);
    }

    pub fn dir_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    pub fn uninstall(&self) {}

    pub fn encoder_type(&self) -> EncoderType {
        self.encoder_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_encoder_new_heif() {
        let mock = MockEncoder::new(EncoderType::HeifEnc);
        assert_eq!(mock.encoder_type(), EncoderType::HeifEnc);
    }

    #[test]
    fn mock_encoder_new_cjxl() {
        let mock = MockEncoder::new(EncoderType::Cjxl);
        assert_eq!(mock.encoder_type(), EncoderType::Cjxl);
    }

    #[test]
    fn mock_encoder_new_avif() {
        let mock = MockEncoder::new(EncoderType::AvifEnc);
        assert_eq!(mock.encoder_type(), EncoderType::AvifEnc);
    }

    #[test]
    fn mock_encoder_with_output() {
        let mock = MockEncoder::new(EncoderType::HeifEnc).with_output(vec![1, 2, 3]);
        assert!(mock.output_data.is_some());
    }

    #[test]
    fn mock_encoder_with_failure() {
        let mock = MockEncoder::new(EncoderType::Cjxl).with_failure();
        assert!(mock.should_fail);
    }

    #[test]
    fn mock_encoder_install_creates_script() {
        let mock = MockEncoder::new(EncoderType::HeifEnc);
        mock.install();
        let script_path = mock.dir_path().join("heif-enc");
        assert!(script_path.exists());
    }

    #[test]
    fn mock_encoder_dir_path_accessible() {
        let mock = MockEncoder::new(EncoderType::HeifEnc);
        assert!(mock.dir_path().exists());
    }

    #[test]
    fn encoder_type_binary_name() {
        assert_eq!(EncoderType::HeifEnc.binary_name(), "heif-enc");
        assert_eq!(EncoderType::Cjxl.binary_name(), "cjxl");
        assert_eq!(EncoderType::AvifEnc.binary_name(), "avifenc");
    }
}

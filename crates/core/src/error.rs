use thiserror::Error;

use crate::types::{GpuBackend, PluginId, PluginVersion, VersionRequirement};

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin '{0}' not found")]
    NotFound(PluginId),

    #[error("plugin '{}' already loaded", .plugin)]
    AlreadyLoaded { plugin: PluginId },

    #[error("plugin '{plugin}' load failed: {reason}")]
    LoadFailed { plugin: PluginId, reason: String },

    #[error("plugin '{plugin}' version {actual} doesn't satisfy {required}")]
    VersionMismatch {
        plugin: PluginId,
        actual: PluginVersion,
        required: VersionRequirement,
    },

    #[error("plugin '{plugin}' parameter '{field}' invalid: {message}")]
    InvalidParameter {
        plugin: PluginId,
        field: String,
        message: String,
    },

    #[error("plugin '{plugin}' missing tool: {tool} (need {required})")]
    MissingTool {
        plugin: PluginId,
        tool: String,
        required: String,
    },

    #[error("plugin '{plugin}' requires GPU backend '{backend}', none available")]
    GpuNotAvailable {
        plugin: PluginId,
        backend: GpuBackend,
    },

    #[error("plugin '{plugin}' out of GPU memory: need {needed}MB, have {available}MB")]
    GpuOutOfMemory {
        plugin: PluginId,
        needed: u64,
        available: u64,
    },

    #[error("plugin '{plugin}' expression error in '{field}': {error}")]
    ExpressionError {
        plugin: PluginId,
        field: String,
        error: String,
    },

    #[error("plugin '{plugin}' processing timeout ({elapsed}s > {timeout}s)")]
    Timeout {
        plugin: PluginId,
        elapsed: f64,
        timeout: f64,
    },

    #[error("plugin '{plugin}' internal error: {message}")]
    Internal { plugin: PluginId, message: String },

    #[error("plugin '{plugin}' canceled by user")]
    Canceled { plugin: PluginId },

    #[error("plugin '{plugin}' IO error: {error}")]
    Io {
        plugin: PluginId,
        #[source]
        error: std::io::Error,
    },

    #[error("pipeline validation failed: {0}")]
    ValidationFailed(String),

    #[error("pipeline execution failed at node '{node}': {message}")]
    NodeExecutionFailed { node: String, message: String },

    #[error("circular dependency detected in pipeline")]
    CircularDependency,

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("encoding failed: {0}")]
    EncodingFailed(String),

    #[error("decoding failed: {0}")]
    DecodingFailed(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ValidationIssue {
    Error { param: String, message: String },
    Warning { param: String, message: String },
    Info { param: String, message: String },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error { param, message } => write!(f, "ERROR({}): {}", param, message),
            Self::Warning { param, message } => write!(f, "WARNING({}): {}", param, message),
            Self::Info { param, message } => write!(f, "INFO({}): {}", param, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_error_not_found_display() {
        let err = PluginError::NotFound("test.plugin".into());
        let msg = err.to_string();
        assert!(msg.contains("test.plugin"));
    }

    #[test]
    fn plugin_error_already_loaded_display() {
        let err = PluginError::AlreadyLoaded {
            plugin: "p1".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("p1"));
    }

    #[test]
    fn plugin_error_load_failed_display() {
        let err = PluginError::LoadFailed {
            plugin: "p2".into(),
            reason: "bad".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("p2"));
        assert!(msg.contains("bad"));
    }

    #[test]
    fn plugin_error_version_mismatch_display() {
        let err = PluginError::VersionMismatch {
            plugin: "p3".into(),
            actual: PluginVersion::new(1, 0, 0),
            required: VersionRequirement {
                min_version: PluginVersion::new(2, 0, 0),
                max_version: None,
            },
        };
        let msg = err.to_string();
        assert!(msg.contains("p3"));
    }

    #[test]
    fn plugin_error_invalid_parameter_display() {
        let err = PluginError::InvalidParameter {
            plugin: "p4".into(),
            field: "quality".into(),
            message: "out of range".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("quality"));
        assert!(msg.contains("out of range"));
    }

    #[test]
    fn plugin_error_missing_tool_display() {
        let err = PluginError::MissingTool {
            plugin: "heif_encoder".into(),
            tool: "heif-enc".into(),
            required: ">=1.0".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("heif-enc"));
    }

    #[test]
    fn plugin_error_gpu_not_available_display() {
        let err = PluginError::GpuNotAvailable {
            plugin: "denoise".into(),
            backend: GpuBackend::CUDA,
        };
        let msg = err.to_string();
        assert!(msg.contains("CUDA"));
    }

    #[test]
    fn plugin_error_gpu_out_of_memory_display() {
        let err = PluginError::GpuOutOfMemory {
            plugin: "denoise".into(),
            needed: 8192,
            available: 4096,
        };
        let msg = err.to_string();
        assert!(msg.contains("8192"));
        assert!(msg.contains("4096"));
    }

    #[test]
    fn plugin_error_expression_error_display() {
        let err = PluginError::ExpressionError {
            plugin: "p5".into(),
            field: "threshold".into(),
            error: "unknown variable".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("threshold"));
    }

    #[test]
    fn plugin_error_timeout_display() {
        let err = PluginError::Timeout {
            plugin: "slow".into(),
            elapsed: 60.0,
            timeout: 30.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("60"));
        assert!(msg.contains("30"));
    }

    #[test]
    fn plugin_error_internal_display() {
        let err = PluginError::Internal {
            plugin: "p6".into(),
            message: "panic".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("panic"));
    }

    #[test]
    fn plugin_error_canceled_display() {
        let err = PluginError::Canceled {
            plugin: "p7".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("canceled"));
    }

    #[test]
    fn plugin_error_io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = PluginError::Io {
            plugin: "reader".into(),
            error: io_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("IO"));
    }

    #[test]
    fn plugin_error_io_source_chain() {
        use std::error::Error;
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = PluginError::Io {
            plugin: "writer".into(),
            error: io_err,
        };
        let source = err.source();
        assert!(source.is_some());
    }

    #[test]
    fn plugin_error_validation_failed_display() {
        let err = PluginError::ValidationFailed("bad config".into());
        assert!(err.to_string().contains("bad config"));
    }

    #[test]
    fn plugin_error_node_execution_failed_display() {
        let err = PluginError::NodeExecutionFailed {
            node: "n1".into(),
            message: "crash".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("n1"));
        assert!(msg.contains("crash"));
    }

    #[test]
    fn plugin_error_circular_dependency_display() {
        let err = PluginError::CircularDependency;
        assert!(err.to_string().contains("circular"));
    }

    #[test]
    fn plugin_error_file_not_found_display() {
        let err = PluginError::FileNotFound("/tmp/missing.txt".into());
        assert!(err.to_string().contains("/tmp/missing.txt"));
    }

    #[test]
    fn plugin_error_unsupported_format_display() {
        let err = PluginError::UnsupportedFormat("bmp".into());
        assert!(err.to_string().contains("bmp"));
    }

    #[test]
    fn plugin_error_encoding_failed_display() {
        let err = PluginError::EncodingFailed("buffer overflow".into());
        assert!(err.to_string().contains("encoding"));
    }

    #[test]
    fn plugin_error_decoding_failed_display() {
        let err = PluginError::DecodingFailed("corrupt header".into());
        assert!(err.to_string().contains("decoding"));
    }

    #[test]
    fn plugin_error_config_display() {
        let err = PluginError::Config("invalid TOML".into());
        assert!(err.to_string().contains("invalid TOML"));
    }

    #[test]
    fn plugin_error_other_display() {
        let err = PluginError::Other("something bad".into());
        assert_eq!(err.to_string(), "something bad");
    }

    #[test]
    fn plugin_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oh no");
        let err = PluginError::Io {
            plugin: "test".into(),
            error: io_err,
        };
        assert!(matches!(err, PluginError::Io { .. }));
    }

    #[test]
    fn validation_issue_error_display() {
        let vi = ValidationIssue::Error {
            param: "q".into(),
            message: "too high".into(),
        };
        let s = vi.to_string();
        assert!(s.contains("ERROR"));
        assert!(s.contains("q"));
    }

    #[test]
    fn validation_issue_warning_display() {
        let vi = ValidationIssue::Warning {
            param: "size".into(),
            message: "large".into(),
        };
        let s = vi.to_string();
        assert!(s.contains("WARNING"));
    }

    #[test]
    fn validation_issue_info_display() {
        let vi = ValidationIssue::Info {
            param: "note".into(),
            message: "ok".into(),
        };
        let s = vi.to_string();
        assert!(s.contains("INFO"));
    }

    #[test]
    fn plugin_error_debug_output_variants() {
        let err = PluginError::NotFound("test".into());
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }
}

//! Shared test case definitions for the Photopipeline test infrastructure.
//!
//! This crate provides the Rust-side parser for shared JSON test case definitions
//! stored in `shared/test_cases/`. These definitions are used by:
//! - Layer 1: Rust Pipeline Integration tests
//! - Layer 2: Rust gRPC E2E tests
//! - Layer 4: C# gRPC Integration tests (via equivalent C# parser)
//! - Layer 5: C# GUI FlaUI E2E tests
//! - Layer 6: Cross-Channel Verification tests
//!
//! # Iron Laws (测试工程铁律)
//!
//! 1. Every test must have at least one FAIL-able assertion
//! 2. No silent skipping — errors must propagate
//! 3. Infrastructure must have consumers first
//! 4. UI tests must launch real processes
//! 5. Adversarial self-review on every test
//! 6. Regression tests must have golden reference images

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when loading or validating test case definitions.
#[derive(Debug, thiserror::Error)]
pub enum TestDefError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Validation failed for test case '{case_id}': {reason}")]
    Validation { case_id: String, reason: String },

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("No test cases found in directory: {0}")]
    NoCasesFound(String),
}

// ---------------------------------------------------------------------------
// TestCategory enum
// ---------------------------------------------------------------------------

/// The category of a test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TestCategory {
    /// Single-plugin tests
    Plugin,
    /// Multi-plugin pipeline chain tests
    Pipeline,
    /// Format conversion/roundtrip tests
    Format,
    /// Batch processing tests
    Batch,
    /// Golden-reference regression tests
    Regression,
}

impl std::fmt::Display for TestCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestCategory::Plugin => write!(f, "plugin"),
            TestCategory::Pipeline => write!(f, "pipeline"),
            TestCategory::Format => write!(f, "format"),
            TestCategory::Batch => write!(f, "batch"),
            TestCategory::Regression => write!(f, "regression"),
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline specification structs
// ---------------------------------------------------------------------------

/// A complete pipeline specification with nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineSpec {
    pub nodes: Vec<PipelineNode>,
    #[serde(default)]
    pub edges: Vec<PipelineEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<serde_json::Value>,
}

/// A single node in the processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineNode {
    /// Unique identifier for this node within the pipeline.
    pub id: String,
    /// The plugin identifier (e.g., "photopipeline.plugins.colorspace").
    pub plugin: String,
    /// Optional human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether this node is enabled. Defaults to true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Plugin-specific parameters as a JSON object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

/// A directed edge connecting two pipeline nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PipelineEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
}

// ---------------------------------------------------------------------------
// Assertions struct
// ---------------------------------------------------------------------------

/// Assertion configuration for test validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assertions {
    /// Per-channel pixel value tolerance (0 = exact match, 255 = anything goes).
    pub tolerance_per_channel: u8,

    /// Primary assertion type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertion_type: Option<String>,

    /// Expected output image format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_format: Option<String>,

    /// Expected output image width in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_width: Option<u32>,

    /// Expected output image height in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_height: Option<u32>,

    /// Expected output bit depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_bit_depth: Option<u8>,

    /// Minimum acceptable PSNR in dB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_psnr: Option<f64>,

    /// Minimum acceptable SSIM (0.0-1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_ssim: Option<f64>,

    /// Maximum acceptable Delta-E color difference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_delta_e: Option<f64>,

    /// Maximum acceptable Mean Absolute Error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_mae: Option<f64>,

    /// Whether to validate output metadata.
    #[serde(default)]
    pub check_metadata: bool,

    /// Whether the test expects an error result.
    #[serde(default)]
    pub expect_error: bool,

    /// Expected error message substring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_error_message: Option<String>,

    /// Path to golden reference image (relative to golden_dir).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub golden_reference: Option<String>,
}

// ---------------------------------------------------------------------------
// Metadata struct
// ---------------------------------------------------------------------------

/// Metadata assertions for EXIF/GPS/XMP validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// Expected EXIF field values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exif_fields: Option<serde_json::Value>,

    /// Expected XMP field values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xmp_fields: Option<serde_json::Value>,

    /// Expected GPS latitude (-90 to 90).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gps_latitude: Option<f64>,

    /// Expected GPS longitude (-180 to 180).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gps_longitude: Option<f64>,

    /// Expected EXIF datetime string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
}

// ---------------------------------------------------------------------------
// TestCase struct (main definition)
// ---------------------------------------------------------------------------

/// A complete test case definition, shared between Rust and C# test systems.
///
/// This struct can be serialized/deserialized from the shared JSON format
/// defined in `shared/test_cases/schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    /// Unique test case identifier (e.g., "CCV-001", "GRPC-012").
    pub id: String,

    /// Test layer (0=Unit through 6=Cross-Channel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<u32>,

    /// Human-readable test name.
    pub name: String,

    /// Test category.
    pub category: TestCategory,

    /// Detailed description of the test.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// If true, the test is temporarily skipped.
    #[serde(default)]
    pub skip: bool,

    /// Reason for skipping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,

    /// Tags for filtering and categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Input image identifiers (I01-I20).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_images: Vec<String>,

    /// Pipeline specification.
    pub pipeline_spec: PipelineSpec,

    /// Assertion configuration.
    pub assertions: Assertions,

    /// Optional metadata assertions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

// ---------------------------------------------------------------------------
// Serialization container
// ---------------------------------------------------------------------------

/// Container for deserializing JSON files that wrap cases in a `cases` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCaseFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    cases: Vec<TestCase>,
}

// ---------------------------------------------------------------------------
// Known plugin IDs for validation
// ---------------------------------------------------------------------------

/// The 14 known plugin IDs in the Photopipeline system.
pub const KNOWN_PLUGIN_IDS: &[&str] = &[
    "photopipeline.plugins.raw_input",
    "photopipeline.plugins.transform",
    "photopipeline.plugins.colorspace",
    "photopipeline.plugins.lut3d",
    "photopipeline.plugins.lens_correct",
    "photopipeline.plugins.ai_denoise",
    "photopipeline.plugins.exif_rw",
    "photopipeline.plugins.gps_set",
    "photopipeline.plugins.time_shift",
    "photopipeline.plugins.avif_encoder",
    "photopipeline.plugins.jxl_encoder",
    "photopipeline.plugins.heif_encoder",
    "photopipeline.plugins.tiff_encoder",
    "photopipeline.plugins.png_encoder",
];

/// Valid ID prefixes for known test case IDs.
const VALID_ID_PREFIXES: &[&str] = &["CCV-", "GE2E-", "GRPC-", "REG-"];

// ---------------------------------------------------------------------------
// TestCase implementation
// ---------------------------------------------------------------------------

impl TestCase {
    /// Load all test cases from a single JSON file.
    ///
    /// The file can contain either a single TestCase or a container object
    /// with a `cases` array.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<TestCase>, TestDefError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| TestDefError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        Self::from_json(&content).map_err(|e| {
            // Attach file path to parse errors for better diagnostics
            if let TestDefError::JsonParse(_) = &e {
                TestDefError::Io {
                    path: path.display().to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
                }
            } else {
                e
            }
        })
    }

    /// Parse test cases from a JSON string.
    ///
    /// Supports both a single TestCase object and a container with a `cases` array.
    pub fn from_json(json: &str) -> Result<Vec<TestCase>, TestDefError> {
        // First, try parsing as a container with a `cases` array
        if let Ok(container) = serde_json::from_str::<TestCaseFile>(json) {
            if !container.cases.is_empty() {
                return Ok(container.cases);
            }
        }

        // If that fails, try parsing as a single TestCase
        let case: TestCase = serde_json::from_str(json)?;
        Ok(vec![case])
    }

    /// Validate this test case definition.
    ///
    /// Checks:
    /// - ID format matches expected pattern
    /// - Pipeline has at least one node
    /// - Edge references valid node IDs
    /// - No duplicate edges
    /// - No self-loop edges
    /// - tolerance_per_channel is within valid range
    /// - Plugin IDs match known plugins
    /// - Input images list is non-empty (for non-error tests)
    ///
    /// Returns `Ok(())` if valid, or `Err(TestDefError::Validation)` with details.
    pub fn validate(&self) -> Result<(), TestDefError> {
        let cid = &self.id;

        // 1. Validate ID format
        self.validate_id()?;

        // 2. Validate pipeline has at least one node
        if self.pipeline_spec.nodes.is_empty() {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "pipeline_spec.nodes must contain at least one node".to_string(),
            });
        }

        // 3. Validate node count does not exceed max
        if self.pipeline_spec.nodes.len() > 20 {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: format!(
                    "pipeline_spec.nodes has {} nodes, maximum is 20",
                    self.pipeline_spec.nodes.len()
                ),
            });
        }

        // 4. Validate node IDs are unique
        let node_ids: HashSet<&str> = self.pipeline_spec.nodes.iter().map(|n| n.id.as_str()).collect();
        if node_ids.len() != self.pipeline_spec.nodes.len() {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "pipeline_spec.nodes contains duplicate node IDs".to_string(),
            });
        }

        // 5. Validate edge references valid nodes and no duplicates/self-loops
        let mut edge_set: HashSet<&PipelineEdge> = HashSet::new();
        for edge in &self.pipeline_spec.edges {
            // Check source node exists
            if !node_ids.contains(edge.from.as_str()) {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!(
                        "edge from='{}' references non-existent node ID",
                        edge.from
                    ),
                });
            }
            // Check target node exists
            if !node_ids.contains(edge.to.as_str()) {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!(
                        "edge to='{}' references non-existent node ID",
                        edge.to
                    ),
                });
            }
            // Check no self-loop
            if edge.from == edge.to {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!("self-loop edge detected: '{}' -> '{}'", edge.from, edge.to),
                });
            }
            // Check no duplicate
            if !edge_set.insert(edge) {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!(
                        "duplicate edge: '{}' -> '{}'",
                        edge.from, edge.to
                    ),
                });
            }
        }

        // 6. tolerance_per_channel is u8, so it is always within [0, 255] by type.
        // No runtime check needed — the Rust type system guarantees this invariant.

        // 7. Validate plugin IDs against known plugins
        for node in &self.pipeline_spec.nodes {
            if !KNOWN_PLUGIN_IDS.contains(&node.plugin.as_str()) {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!(
                        "node '{}' references unknown plugin '{}'",
                        node.id, node.plugin
                    ),
                });
            }
        }

        // 8. Validate input_images count (unless expect_error is set)
        if !self.assertions.expect_error && self.input_images.is_empty() {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "input_images is empty for non-error test case".to_string(),
            });
        }

        // 9. Validate input_images format
        for img in &self.input_images {
            if !img.starts_with('I') || img.len() != 3 {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: format!(
                        "invalid input_image ID '{}': must match pattern I01-I20",
                        img
                    ),
                });
            }
        }

        // 10. Validate expected_width and expected_height if present
        if let Some(w) = self.assertions.expected_width {
            if w == 0 {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: "expected_width is 0, must be at least 1".to_string(),
                });
            }
        }
        if let Some(h) = self.assertions.expected_height {
            if h == 0 {
                return Err(TestDefError::Validation {
                    case_id: cid.clone(),
                    reason: "expected_height is 0, must be at least 1".to_string(),
                });
            }
        }

        // 11. Regression tests must have golden_reference
        if self.category == TestCategory::Regression && self.assertions.golden_reference.is_none() {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "regression test must have a golden_reference image path".to_string(),
            });
        }

        Ok(())
    }

    /// Validate the test case ID format.
    fn validate_id(&self) -> Result<(), TestDefError> {
        let cid = &self.id;

        // Check prefix
        let valid_prefix = VALID_ID_PREFIXES.iter().any(|p| cid.starts_with(p));
        if !valid_prefix {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: format!(
                    "ID '{}' must start with one of: {:?}",
                    cid, VALID_ID_PREFIXES
                ),
            });
        }

        // Check format: PREFIX-NNN where NNN are exactly 3 digits
        let parts: Vec<&str> = cid.splitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "ID must be in format PREFIX-NNN (e.g., CCV-001)".to_string(),
            });
        }

        let suffix = parts[1];
        if suffix.len() != 3 || !suffix.chars().all(|c| c.is_ascii_digit()) {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: "ID suffix must be exactly 3 digits (001-999)".to_string(),
            });
        }

        let num: u32 = suffix.parse().unwrap_or(0);
        if num < 1 || num > 999 {
            return Err(TestDefError::Validation {
                case_id: cid.clone(),
                reason: format!("ID number {} must be between 001 and 999", num),
            });
        }

        Ok(())
    }

    /// Check if this test case is active (not skipped).
    pub fn is_active(&self) -> bool {
        !self.skip
    }

    /// Get the number of nodes in the pipeline.
    pub fn node_count(&self) -> usize {
        self.pipeline_spec.nodes.len()
    }

    /// Get the number of edges in the pipeline.
    pub fn edge_count(&self) -> usize {
        self.pipeline_spec.edges.len()
    }

    /// Get the number of enabled nodes.
    pub fn enabled_node_count(&self) -> usize {
        self.pipeline_spec.nodes.iter().filter(|n| n.enabled).count()
    }
}

// ---------------------------------------------------------------------------
// Load all test cases from directory
// ---------------------------------------------------------------------------

/// Load all test case `.json` files from a directory.
///
/// Scans the given directory for JSON files, parses each one, and returns
/// a flattened vector of all `TestCase` instances.
///
/// Skips `schema.json` automatically.
///
/// # Errors
///
/// Returns `TestDefError::DirectoryNotFound` if the directory does not exist.
/// Returns `TestDefError::NoCasesFound` if no valid test cases were loaded.
pub fn load_all_test_cases(dir: impl AsRef<Path>) -> Result<Vec<TestCase>, TestDefError> {
    let dir = dir.as_ref();

    if !dir.is_dir() {
        return Err(TestDefError::DirectoryNotFound(dir.display().to_string()));
    }

    let mut all_cases: Vec<TestCase> = Vec::new();

    for entry in std::fs::read_dir(dir).map_err(|e| TestDefError::Io {
        path: dir.display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| TestDefError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;
        let path = entry.path();

        // Only process .json files
        if path.extension().map_or(true, |ext| ext != "json") {
            continue;
        }

        // Skip the schema file
        if path.file_name().map_or(false, |n| n == "schema.json") {
            continue;
        }

        let cases = TestCase::from_file(&path)?;
        all_cases.extend(cases);
    }

    if all_cases.is_empty() {
        return Err(TestDefError::NoCasesFound(dir.display().to_string()));
    }

    Ok(all_cases)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // =======================================================================
    // Roundtrip tests
    // =======================================================================

    /// Test that a TestCase can be serialized to JSON and deserialized back
    /// with all fields preserved.
    #[test]
    fn roundtrip_json_basic() {
        let tc = TestCase {
            id: "CCV-001".to_string(),
            layer: Some(6),
            name: "Test Case".to_string(),
            category: TestCategory::Plugin,
            description: Some("A test case".to_string()),
            skip: false,
            skip_reason: None,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: Some("Colorspace".to_string()),
                        enabled: true,
                        params: Some(serde_json::json!({"source": "srgb", "target": "linear"})),
                    },
                ],
                edges: vec![],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: Some("pixel_exact".to_string()),
                expected_format: Some("TIFF".to_string()),
                expected_width: Some(1920),
                expected_height: Some(1080),
                expected_bit_depth: Some(8),
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: Some("golden/test.png".to_string()),
            },
            metadata: None,
        };

        let json = serde_json::to_string_pretty(&tc).expect("serialize");
        let deserialized: TestCase = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(tc.id, deserialized.id);
        assert_eq!(tc.name, deserialized.name);
        assert_eq!(tc.category, deserialized.category);
        assert_eq!(tc.input_images, deserialized.input_images);
        assert_eq!(tc.pipeline_spec.nodes.len(), deserialized.pipeline_spec.nodes.len());
        assert_eq!(
            tc.assertions.tolerance_per_channel,
            deserialized.assertions.tolerance_per_channel
        );
        assert_eq!(
            tc.assertions.golden_reference,
            deserialized.assertions.golden_reference
        );
    }

    /// Test that a TestCase with edges roundtrips correctly.
    #[test]
    fn roundtrip_json_with_edges() {
        let tc = TestCase {
            id: "CCV-021".to_string(),
            layer: Some(6),
            name: "Multi-node pipeline".to_string(),
            category: TestCategory::Pipeline,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.raw_input".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                    PipelineNode {
                        id: "n2".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                    PipelineNode {
                        id: "n3".to_string(),
                        plugin: "photopipeline.plugins.tiff_encoder".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                ],
                edges: vec![
                    PipelineEdge { from: "n1".to_string(), to: "n2".to_string() },
                    PipelineEdge { from: "n2".to_string(), to: "n3".to_string() },
                ],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: Some("pixel_exact".to_string()),
                expected_format: Some("TIFF".to_string()),
                expected_width: Some(1920),
                expected_height: Some(1080),
                expected_bit_depth: Some(8),
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: Some(Metadata {
                exif_fields: Some(serde_json::json!({"ColorSpace": "sRGB"})),
                xmp_fields: None,
                gps_latitude: None,
                gps_longitude: None,
                datetime: None,
            }),
        };

        let json = serde_json::to_string_pretty(&tc).expect("serialize");
        let deserialized: TestCase = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(tc.id, deserialized.id);
        assert_eq!(tc.pipeline_spec.edges.len(), deserialized.pipeline_spec.edges.len());
        assert_eq!(tc.pipeline_spec.edges[0], deserialized.pipeline_spec.edges[0]);
        assert_eq!(tc.pipeline_spec.edges[1], deserialized.pipeline_spec.edges[1]);
        assert!(deserialized.metadata.is_some());
    }

    /// Test that a disabled node roundtrips correctly.
    #[test]
    fn roundtrip_json_disabled_node() {
        let tc = TestCase {
            id: "GRPC-001".to_string(),
            layer: Some(2),
            name: "Disabled node test".to_string(),
            category: TestCategory::Pipeline,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec![],
            pipeline_spec: PipelineSpec {
                nodes: vec![
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                    PipelineNode {
                        id: "n2".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: None,
                        enabled: false,
                        params: None,
                    },
                ],
                edges: vec![],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: None,
                expected_format: None,
                expected_width: None,
                expected_height: None,
                expected_bit_depth: None,
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: true,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: None,
        };

        let json = serde_json::to_string(&tc).expect("serialize");
        let deserialized: TestCase = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.pipeline_spec.nodes[0].enabled, true);
        assert_eq!(deserialized.pipeline_spec.nodes[1].enabled, false);
        assert_eq!(deserialized.enabled_node_count(), 1);
    }

    // =======================================================================
    // Validation tests: ID format
    // =======================================================================

    fn make_minimal_tc(id: &str) -> TestCase {
        TestCase {
            id: id.to_string(),
            layer: None,
            name: "Minimal".to_string(),
            category: TestCategory::Plugin,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![PipelineNode {
                    id: "n1".to_string(),
                    plugin: "photopipeline.plugins.colorspace".to_string(),
                    label: None,
                    enabled: true,
                    params: None,
                }],
                edges: vec![],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: None,
                expected_format: None,
                expected_width: None,
                expected_height: None,
                expected_bit_depth: None,
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: None,
        }
    }

    #[test]
    fn validate_id_ccv_format_valid() {
        let tc = make_minimal_tc("CCV-001");
        assert!(tc.validate().is_ok(), "CCV-001 should be valid");
    }

    #[test]
    fn validate_id_ccv_format_max() {
        let tc = make_minimal_tc("CCV-060");
        assert!(tc.validate().is_ok(), "CCV-060 should be valid");
    }

    #[test]
    fn validate_id_grpc_format_valid() {
        let tc = make_minimal_tc("GRPC-120");
        assert!(tc.validate().is_ok(), "GRPC-120 should be valid");
    }

    #[test]
    fn validate_id_ge2e_format_valid() {
        let tc = make_minimal_tc("GE2E-105");
        assert!(tc.validate().is_ok(), "GE2E-105 should be valid");
    }

    #[test]
    fn validate_id_reg_format_valid() {
        let tc = make_minimal_tc("REG-007");
        assert!(tc.validate().is_ok(), "REG-007 should be valid");
    }

    #[test]
    fn validate_id_invalid_prefix_fails() {
        let tc = make_minimal_tc("BAD-001");
        let result = tc.validate();
        assert!(result.is_err(), "BAD-001 should fail validation");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must start with"), "Error should mention valid prefixes: {err}");
    }

    #[test]
    fn validate_id_no_prefix_fails() {
        let tc = make_minimal_tc("abc123");
        let result = tc.validate();
        assert!(result.is_err(), "abc123 should fail validation");
    }

    #[test]
    fn validate_id_non_numeric_suffix_fails() {
        let tc = make_minimal_tc("CCV-ABC");
        let result = tc.validate();
        assert!(result.is_err(), "CCV-ABC should fail validation");
    }

    #[test]
    fn validate_id_too_short_suffix_fails() {
        let tc = make_minimal_tc("CCV-01");
        let result = tc.validate();
        assert!(result.is_err(), "CCV-01 should fail (needs 3 digits)");
    }

    #[test]
    fn validate_id_too_long_suffix_fails() {
        let tc = make_minimal_tc("CCV-0001");
        let result = tc.validate();
        assert!(result.is_err(), "CCV-0001 should fail (needs exactly 3 digits)");
    }

    #[test]
    fn validate_id_zero_fails() {
        let tc = make_minimal_tc("CCV-000");
        let result = tc.validate();
        assert!(result.is_err(), "CCV-000 should fail (number must be >= 1)");
    }

    // =======================================================================
    // Validation tests: plugin exists
    // =======================================================================

    #[test]
    fn validate_plugin_colorspace_valid() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.nodes[0].plugin = "photopipeline.plugins.colorspace".to_string();
        assert!(tc.validate().is_ok(), "colorspace should be valid");
    }

    #[test]
    fn validate_plugin_raw_input_valid() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.nodes[0].plugin = "photopipeline.plugins.raw_input".to_string();
        assert!(tc.validate().is_ok(), "raw_input should be valid");
    }

    #[test]
    fn validate_plugin_unknown_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.nodes[0].plugin = "photopipeline.plugins.nonexistent".to_string();
        let result = tc.validate();
        assert!(result.is_err(), "nonexistent plugin should fail");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown plugin"), "Error should mention unknown plugin: {err}");
    }

    #[test]
    fn validate_all_14_known_plugins_pass() {
        for plugin_id in KNOWN_PLUGIN_IDS {
            let mut tc = make_minimal_tc("CCV-001");
            tc.pipeline_spec.nodes[0].id = "n1".to_string();
            tc.pipeline_spec.nodes[0].plugin = (*plugin_id).to_string();
            let result = tc.validate();
            assert!(
                result.is_ok(),
                "Known plugin '{plugin_id}' should pass validation, got: {result:?}"
            );
        }
    }

    // =======================================================================
    // Validation tests: pipeline structure
    // =======================================================================

    #[test]
    fn validate_empty_nodes_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.nodes.clear();
        let result = tc.validate();
        assert!(result.is_err(), "empty nodes should fail");
        assert!(result.unwrap_err().to_string().contains("at least one node"));
    }

    #[test]
    fn validate_duplicate_node_ids_fails() {
        let tc = TestCase {
            id: "CCV-001".to_string(),
            layer: None,
            name: "Dup nodes".to_string(),
            category: TestCategory::Pipeline,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.transform".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                ],
                edges: vec![],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: None,
                expected_format: None,
                expected_width: None,
                expected_height: None,
                expected_bit_depth: None,
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: None,
        };
        let result = tc.validate();
        assert!(result.is_err(), "duplicate node IDs should fail");
        assert!(result.unwrap_err().to_string().contains("duplicate node ID"));
    }

    #[test]
    fn validate_edge_missing_source_fails() {
        let tc = TestCase {
            id: "CCV-001".to_string(),
            layer: None,
            name: "Bad edge".to_string(),
            category: TestCategory::Pipeline,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![PipelineNode {
                    id: "n1".to_string(),
                    plugin: "photopipeline.plugins.colorspace".to_string(),
                    label: None,
                    enabled: true,
                    params: None,
                }],
                edges: vec![PipelineEdge {
                    from: "nonexistent".to_string(),
                    to: "n1".to_string(),
                }],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: None,
                expected_format: None,
                expected_width: None,
                expected_height: None,
                expected_bit_depth: None,
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: None,
        };
        let result = tc.validate();
        assert!(result.is_err(), "edge with missing source should fail");
        assert!(result.unwrap_err().to_string().contains("non-existent node ID"));
    }

    #[test]
    fn validate_self_loop_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.edges = vec![PipelineEdge {
            from: "n1".to_string(),
            to: "n1".to_string(),
        }];
        let result = tc.validate();
        assert!(result.is_err(), "self-loop should fail");
        assert!(result.unwrap_err().to_string().contains("self-loop"));
    }

    #[test]
    fn validate_duplicate_edge_fails() {
        let tc = TestCase {
            id: "CCV-001".to_string(),
            layer: None,
            name: "Dup edge".to_string(),
            category: TestCategory::Pipeline,
            description: None,
            skip: false,
            skip_reason: None,
            tags: vec![],
            input_images: vec!["I01".to_string()],
            pipeline_spec: PipelineSpec {
                nodes: vec![
                    PipelineNode {
                        id: "n1".to_string(),
                        plugin: "photopipeline.plugins.colorspace".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                    PipelineNode {
                        id: "n2".to_string(),
                        plugin: "photopipeline.plugins.transform".to_string(),
                        label: None,
                        enabled: true,
                        params: None,
                    },
                ],
                edges: vec![
                    PipelineEdge { from: "n1".to_string(), to: "n2".to_string() },
                    PipelineEdge { from: "n1".to_string(), to: "n2".to_string() },
                ],
                overrides: None,
            },
            assertions: Assertions {
                tolerance_per_channel: 0,
                assertion_type: None,
                expected_format: None,
                expected_width: None,
                expected_height: None,
                expected_bit_depth: None,
                min_psnr: None,
                min_ssim: None,
                max_delta_e: None,
                max_mae: None,
                check_metadata: false,
                expect_error: false,
                expected_error_message: None,
                golden_reference: None,
            },
            metadata: None,
        };
        let result = tc.validate();
        assert!(result.is_err(), "duplicate edge should fail");
        assert!(result.unwrap_err().to_string().contains("duplicate edge"));
    }

    // =======================================================================
    // Validation tests: assertions
    // =======================================================================

    #[test]
    fn validate_regression_must_have_golden_reference() {
        let mut tc = make_minimal_tc("REG-001");
        tc.category = TestCategory::Regression;
        tc.assertions.golden_reference = None;
        let result = tc.validate();
        assert!(result.is_err(), "regression without golden_reference should fail");
        assert!(
            result.unwrap_err().to_string().contains("golden_reference"),
        );
    }

    #[test]
    fn validate_regression_with_golden_passes() {
        let mut tc = make_minimal_tc("REG-001");
        tc.category = TestCategory::Regression;
        tc.assertions.golden_reference = Some("golden/test.png".to_string());
        assert!(tc.validate().is_ok(), "regression with golden should pass");
    }

    #[test]
    fn validate_expected_width_zero_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.assertions.expected_width = Some(0);
        let result = tc.validate();
        assert!(result.is_err(), "expected_width=0 should fail");
    }

    #[test]
    fn validate_expected_height_zero_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.assertions.expected_height = Some(0);
        let result = tc.validate();
        assert!(result.is_err(), "expected_height=0 should fail");
    }

    // =======================================================================
    // Validation tests: input_images
    // =======================================================================

    #[test]
    fn validate_input_images_empty_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.input_images.clear();
        let result = tc.validate();
        assert!(result.is_err(), "empty input_images should fail for non-error test");
    }

    #[test]
    fn validate_input_images_empty_ok_when_expect_error() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.input_images.clear();
        tc.assertions.expect_error = true;
        assert!(tc.validate().is_ok(), "empty input_images should be OK for error tests");
    }

    #[test]
    fn validate_invalid_input_image_format_fails() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.input_images = vec!["XYZ".to_string()];
        let result = tc.validate();
        assert!(result.is_err(), "invalid input image format should fail");
    }

    // =======================================================================
    // Helper method tests
    // =======================================================================

    #[test]
    fn test_is_active() {
        let mut tc = make_minimal_tc("CCV-001");
        assert!(tc.is_active());
        tc.skip = true;
        assert!(!tc.is_active());
    }

    #[test]
    fn test_node_count() {
        let tc = make_minimal_tc("CCV-001");
        assert_eq!(tc.node_count(), 1);
        assert_eq!(tc.enabled_node_count(), 1);
    }

    #[test]
    fn test_edge_count() {
        let tc = make_minimal_tc("CCV-001");
        assert_eq!(tc.edge_count(), 0);
    }

    // =======================================================================
    // from_json tests
    // =======================================================================

    #[test]
    fn from_json_single_case() {
        let json = r#"{
            "id": "CCV-001",
            "name": "Test",
            "category": "plugin",
            "pipeline_spec": {
                "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.colorspace"}],
                "edges": []
            },
            "assertions": {"tolerance_per_channel": 0}
        }"#;
        let cases = TestCase::from_json(json).expect("parse");
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].id, "CCV-001");
        assert_eq!(cases[0].category, TestCategory::Plugin);
    }

    #[test]
    fn from_json_container_with_cases_array() {
        let json = r#"{
            "title": "Test Cases",
            "cases": [
                {
                    "id": "CCV-001",
                    "name": "Test 1",
                    "category": "plugin",
                    "pipeline_spec": {
                        "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.colorspace"}],
                        "edges": []
                    },
                    "assertions": {"tolerance_per_channel": 0}
                },
                {
                    "id": "CCV-002",
                    "name": "Test 2",
                    "category": "pipeline",
                    "pipeline_spec": {
                        "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.transform"}],
                        "edges": []
                    },
                    "assertions": {"tolerance_per_channel": 1}
                }
            ]
        }"#;
        let cases = TestCase::from_json(json).expect("parse");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].id, "CCV-001");
        assert_eq!(cases[1].id, "CCV-002");
    }

    #[test]
    fn from_json_parse_error() {
        let json = "not valid json";
        let result = TestCase::from_json(json);
        assert!(result.is_err(), "invalid JSON should fail");
    }

    // =======================================================================
    // File loading tests
    // =======================================================================

    #[test]
    fn from_file_nonexistent_returns_error() {
        let result = TestCase::from_file("/nonexistent/path/test.json");
        assert!(result.is_err(), "nonexistent file should fail");
    }

    #[test]
    fn from_file_valid_json() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.json");
        let json = r#"{
            "id": "CCV-001",
            "name": "File Test",
            "category": "plugin",
            "pipeline_spec": {
                "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.colorspace"}],
                "edges": []
            },
            "assertions": {"tolerance_per_channel": 0}
        }"#;
        std::fs::write(&file_path, json).expect("write test file");

        let cases = TestCase::from_file(&file_path).expect("load file");
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "File Test");
    }

    // =======================================================================
    // load_all_test_cases tests
    // =======================================================================

    #[test]
    fn load_all_test_cases_from_dir() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Write a test case file
        let json = r#"{
            "title": "Test Cases",
            "cases": [
                {
                    "id": "CCV-001",
                    "name": "Dir Test 1",
                    "category": "plugin",
                    "pipeline_spec": {
                        "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.colorspace"}],
                        "edges": []
                    },
                    "assertions": {"tolerance_per_channel": 0}
                }
            ]
        }"#;
        std::fs::write(dir.path().join("test_cases.json"), json).expect("write");

        let all = load_all_test_cases(dir.path()).expect("load all");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Dir Test 1");
    }

    #[test]
    fn load_all_test_cases_skips_schema() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Write schema.json (should be skipped)
        std::fs::write(dir.path().join("schema.json"), "{}").expect("write schema");

        // Write a test case file
        let json = r#"{
            "cases": [
                {
                    "id": "CCV-001",
                    "name": "Schema Skip Test",
                    "category": "plugin",
                    "pipeline_spec": {
                        "nodes": [{"id": "n1", "plugin": "photopipeline.plugins.colorspace"}],
                        "edges": []
                    },
                    "assertions": {"tolerance_per_channel": 0}
                }
            ]
        }"#;
        std::fs::write(dir.path().join("cases.json"), json).expect("write");

        let all = load_all_test_cases(dir.path()).expect("load all");
        assert_eq!(all.len(), 1, "schema.json should be skipped");
        assert_eq!(all[0].name, "Schema Skip Test");
    }

    #[test]
    fn load_all_test_cases_empty_dir_returns_error() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let result = load_all_test_cases(dir.path());
        assert!(result.is_err(), "empty directory should fail");
    }

    #[test]
    fn load_all_test_cases_nonexistent_dir_returns_error() {
        let result = load_all_test_cases("/nonexistent/directory");
        assert!(result.is_err(), "nonexistent directory should fail");
    }

    // =======================================================================
    // Adversarial self-review tests (铁律 5)
    // =======================================================================

    /// If tolerance_per_channel is set to 255, the validation should still
    /// pass, but the test author should reconsider their assertion strength.
    #[test]
    fn adversarial_tolerance_255_still_validates() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.assertions.tolerance_per_channel = 255;
        assert!(tc.validate().is_ok(), "tolerance=255 should pass validation");
    }

    /// If all nodes are disabled, the test should still validate structurally
    /// but the consumer should check enabled_node_count before executing.
    #[test]
    fn adversarial_all_nodes_disabled() {
        let mut tc = make_minimal_tc("CCV-001");
        tc.pipeline_spec.nodes[0].enabled = false;
        assert!(tc.validate().is_ok(), "all-disabled should pass structural validation");
        assert_eq!(tc.enabled_node_count(), 0, "but consumer should check enabled count");
    }

    /// Verify that the KNOWN_PLUGIN_IDS constant has exactly 14 entries.
    #[test]
    fn known_plugin_ids_count_is_14() {
        assert_eq!(
            KNOWN_PLUGIN_IDS.len(),
            14,
            "KNOWN_PLUGIN_IDS must have exactly 14 entries"
        );
        // Adversarial: verify no duplicates
        let unique: HashSet<&&str> = KNOWN_PLUGIN_IDS.iter().collect();
        assert_eq!(
            unique.len(),
            KNOWN_PLUGIN_IDS.len(),
            "KNOWN_PLUGIN_IDS must not have duplicates"
        );
    }
}

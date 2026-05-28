/// Patterns that indicate internal bypass (plugin fell back to pure Rust / degraded path).
const BYPASS_PATTERNS: &[(&str, &str)] = &[
    ("colorspace", "using pure Rust matrix conversion"),
    ("colorspace", "lcms2 native not compiled"),
    ("transform", "(bilinear for now)"),
    ("jxl_encoder", "libjxl native FFI and OIIO both unavailable"),
    ("jxl_encoder", "libjxl native not compiled"),
    ("heif_encoder", "libheif native not compiled"),
    ("heif_encoder", "OIIO both unavailable"),
    ("raw_input", "LibRaw native not compiled"),
    ("lens_correct", "LensFun database unavailable"),
    ("lens_correct", "failed to load bundled LensFun"),
    ("exif_rw", "exiftool not available for writing"),
];

pub struct BypassResult {
    pub found: bool,
    pub reason: String,
}

/// Scan stderr for internal bypass patterns for the specified plugin IDs.
/// Returns the first matching bypass found.
pub fn scan(stderr: &str, plugin_ids: &[String]) -> BypassResult {
    let stderr_lower = stderr.to_lowercase();
    for (plugin_id, pattern) in BYPASS_PATTERNS {
        if plugin_ids.iter().any(|p| p.as_str() == *plugin_id || p.contains(plugin_id)) {
            if stderr_lower.contains(&pattern.to_lowercase()) {
                return BypassResult {
                    found: true,
                    reason: format!("plugin '{}' bypassed: '{}'", plugin_id, pattern),
                };
            }
        }
    }
    BypassResult { found: false, reason: String::new() }
}

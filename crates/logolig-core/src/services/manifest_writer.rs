//! Web manifest generator (v1.8.0).
//!
//! Assembles a `manifest.webmanifest` JSON string from
//! `WebManifestSettings` and `ExportPlan.png_sizes`.
//!
//! ## Why a service?
//!
//! The icon array must use the same file-naming convention as the exporter
//! (`favicon-{size}.png`). Placing both in the service layer keeps the
//! naming rule co-located and prevents drift between the manifest and
//! the actual files written to disk.

use serde_json::json;

use crate::domain::WebManifestSettings;

/// Output file name (including extension). W3C recommends `.webmanifest`;
/// browsers also accept `.json`, but logolig follows the recommendation.
pub const MANIFEST_FILENAME: &str = "manifest.webmanifest";

/// Generate the `manifest.webmanifest` JSON string from
/// `WebManifestSettings` and the PNG size set. Output is pretty-printed.
///
/// If `png_sizes` is empty, the `icons` array is empty (not recommended for
/// a PWA, but v1.8 respects the user's choice).
pub fn build_manifest_json(settings: &WebManifestSettings, png_sizes: &[u32]) -> String {
    // Build the icons array. File names in the manifest must match
    // exporter output names; the `favicon-{size}.png` convention is inlined here.
    let icons: Vec<serde_json::Value> = png_sizes
        .iter()
        .map(|size| {
            json!({
                "src": format!("favicon-{size}.png"),
                "sizes": format!("{size}x{size}"),
                "type": "image/png",
                "purpose": "any"
            })
        })
        .collect();

    let manifest = json!({
        "name": settings.name,
        "short_name": settings.short_name,
        "icons": icons,
        "start_url": "/",
        "display": "standalone",
        "theme_color": settings.theme_color,
        "background_color": settings.background_color
    });

    // Pretty-print; add a trailing newline (POSIX text file convention).
    let mut out = serde_json::to_string_pretty(&manifest)
        .expect("serde_json: serializing well-formed JSON should not fail");
    out.push('\n');
    out
}

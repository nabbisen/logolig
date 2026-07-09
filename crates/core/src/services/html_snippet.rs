//! HTML `<head>` snippet generator (§7.2).
//!
//! Design principles (see docs/src/export-spec.md):
//! - **Semantic** — correct use of `rel="icon"` / `rel="apple-touch-icon"`
//! - **Modern** — no legacy `msapplication-*` or `browserconfig.xml`
//! - **Minimal** — only references artifacts present in the export plan
//! - **Non-destructive** — only `<link>` elements; no other HTML changed
//! - **Paste-ready** — bare `<link>` block, no wrapper or full-document

use crate::domain::ExportPlan;

/// URL root prefix for output paths. Default is the site root (`/`).
pub const DEFAULT_BASE: &str = "/";

/// Returns a `<head>` snippet reflecting the given `ExportPlan`.
///
/// `base` is typically `"/"`. Use `"/static/favicons/"` when assets are in a sub-path.
/// A trailing slash is added internally if missing.
///
/// Output order (v1.2.0):
/// 1. `<link rel="icon" type="image/svg+xml">` — preferred by modern browsers
/// 2. `<link rel="icon" href="/favicon.ico" sizes="any">` — legacy fallback
/// 3. PNG sizes (ascending)
/// 4. `<link rel="apple-touch-icon">` — iOS/Safari
///
/// Why SVG first: browsers pick the "most suitable" from multiple `<link rel="icon">` tags.
/// Modern browsers prefer SVG (great on HiDPI); legacy browsers fall back to ICO/PNG.
pub fn render(plan: &ExportPlan, base: &str) -> String {
    let base = normalize_base(base);
    let mut out = String::new();

    // SVG: highest priority for modern browsers.
    // Order in HTML reflects browser preference; SVG before ICO/PNG (§7.2).
    if plan.include_svg {
        out.push_str(&format!(
            "<link rel=\"icon\" type=\"image/svg+xml\" href=\"{base}favicon.svg\">\n"
        ));
    }

    // ICO: second — maximum legacy compatibility.
    // `sizes="any"` indicates the ICO is scalable (modern convention).
    if plan.include_ico {
        out.push_str(&format!(
            "<link rel=\"icon\" href=\"{base}favicon.ico\" sizes=\"any\">\n"
        ));
    }

    // PNG sizes: one `<link>` per size for modern browsers.
    let mut png_sizes = plan.png_sizes.clone();
    png_sizes.sort_unstable();
    png_sizes.dedup();
    for size in &png_sizes {
        out.push_str(&format!(
            "<link rel=\"icon\" type=\"image/png\" sizes=\"{size}x{size}\" href=\"{base}favicon-{size}.png\">\n"
        ));
    }

    // Apple touch icon: dedicated rel. `sizes` is optional but good practice.
    if plan.include_apple_touch {
        out.push_str(&format!(
            "<link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"{base}apple-touch-icon.png\">\n"
        ));
    }

    // v1.8.0: Add manifest link when web manifest is included.
    // `<link rel="manifest">` is the PWA standard.
    // MIME configuration (`application/manifest+json`) is the user's responsibility;
    // logolig only handles the HTML link.
    if plan.web_manifest.is_some() {
        out.push_str(&format!(
            "<link rel=\"manifest\" href=\"{base}manifest.webmanifest\">\n"
        ));
    }

    out
}

/// Ensure `base` ends with a slash. Replace an empty string with the default.
fn normalize_base(base: &str) -> String {
    let trimmed = base.trim();
    if trimmed.is_empty() {
        return DEFAULT_BASE.to_string();
    }
    if trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}

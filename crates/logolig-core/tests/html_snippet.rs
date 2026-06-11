//! HTML snippet generation tests (§7.2).
//!
//! - Default plan produces the expected `<link>` elements
//! - Artifacts excluded from the plan are not referenced
//! - Base-path normalisation (trailing-slash insertion)
//! - Output contains no legacy `msapplication-*` or `browserconfig.xml`

use logolig_core::services::html_snippet::{render, DEFAULT_BASE};
use logolig_core::ExportPlan;

#[test]
fn default_plan_renders_modern_minimal_set() {
    let html = render(&ExportPlan::default(), DEFAULT_BASE);
    // SVG link comes first (v1.2.0 — preferred by modern browsers)
    assert!(html.contains(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#));
    // ICO link is second, for legacy compatibility
    assert!(html.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    // PNG sizes (default 32 / 192 / 512) each appear
    assert!(html.contains(r#"sizes="32x32" href="/favicon-32.png""#));
    assert!(html.contains(r#"sizes="192x192" href="/favicon-192.png""#));
    assert!(html.contains(r#"sizes="512x512" href="/favicon-512.png""#));
    // Apple touch icon
    assert!(html.contains(r#"rel="apple-touch-icon" sizes="180x180""#));

    // Order check: SVG → ICO → PNG → apple-touch
    let pos_svg = html.find("favicon.svg").unwrap();
    let pos_ico = html.find("favicon.ico").unwrap();
    let pos_png = html.find("favicon-32.png").unwrap();
    let pos_apple = html.find("apple-touch-icon").unwrap();
    assert!(pos_svg < pos_ico, "SVG must precede ICO for modern browsers");
    assert!(pos_ico < pos_png);
    assert!(pos_png < pos_apple);
}

#[test]
fn excluded_artifacts_do_not_appear_in_html() {
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_ico = false;
    plan.include_svg = false;

    let html = render(&plan, DEFAULT_BASE);
    assert!(!html.contains("apple-touch-icon"));
    assert!(!html.contains("favicon.ico"));
    assert!(!html.contains("favicon.svg"));
    // PNG references remain
    assert!(html.contains("favicon-32.png"));
}

#[test]
fn svg_omitted_when_include_svg_is_false() {
    let mut plan = ExportPlan::default();
    plan.include_svg = false;

    let html = render(&plan, DEFAULT_BASE);
    assert!(!html.contains("favicon.svg"));
    assert!(!html.contains(r#"type="image/svg+xml""#));
    // ICO and PNG references remain
    assert!(html.contains("favicon.ico"));
    assert!(html.contains("favicon-32.png"));
}

#[test]
fn legacy_microsoft_tags_are_never_emitted() {
    // §7.2 "reflect a modern favicon reference set"
    let html = render(&ExportPlan::default(), DEFAULT_BASE);
    assert!(!html.contains("msapplication"));
    assert!(!html.contains("browserconfig"));
    assert!(!html.contains("mstile"));
    assert!(!html.contains("apple-touch-icon-precomposed"));
}

#[test]
fn base_path_normalization_appends_slash() {
    let plan = ExportPlan::default();
    let html_no_slash = render(&plan, "/static/icons");
    let html_slash = render(&plan, "/static/icons/");
    // Trailing slash presence/absence must not change the output
    assert_eq!(html_no_slash, html_slash);
    assert!(html_no_slash.contains("/static/icons/favicon.ico"));
}

#[test]
fn empty_base_falls_back_to_root() {
    let plan = ExportPlan::default();
    let html = render(&plan, "");
    assert!(html.contains(r#"href="/favicon.ico""#));
}

#[test]
fn png_sizes_are_sorted_and_deduped() {
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![512, 32, 32, 192];
    let html = render(&plan, "/");
    // PNG sizes appear in ascending order
    let pos_32 = html.find("favicon-32.png").unwrap();
    let pos_192 = html.find("favicon-192.png").unwrap();
    let pos_512 = html.find("favicon-512.png").unwrap();
    assert!(pos_32 < pos_192 && pos_192 < pos_512);
    // 32 appears exactly once (duplicate-free)
    assert_eq!(html.matches("favicon-32.png").count(), 1);
}

// ---------------------------------------------------------------------------
// v1.8.0: web_manifest generates <link rel="manifest">
// ---------------------------------------------------------------------------

#[test]
fn manifest_link_is_emitted_when_web_manifest_is_some() {
    let plan = ExportPlan {
        web_manifest: Some(logolig_core::WebManifestSettings::default()),
        ..ExportPlan::default()
    };
    let html = render(&plan, "/");
    assert!(
        html.contains(r#"<link rel="manifest" href="/manifest.webmanifest">"#),
        "manifest link missing: {html}"
    );
}

#[test]
fn manifest_link_omitted_when_web_manifest_is_none() {
    let plan = ExportPlan {
        web_manifest: None,
        ..ExportPlan::default()
    };
    let html = render(&plan, "/");
    assert!(
        !html.contains("manifest"),
        "unexpected manifest reference: {html}"
    );
}

#[test]
fn manifest_link_uses_normalized_base_path() {
    let plan = ExportPlan {
        web_manifest: Some(logolig_core::WebManifestSettings::default()),
        ..ExportPlan::default()
    };
    let html = render(&plan, "/assets/icons");
    assert!(
        html.contains(r#"<link rel="manifest" href="/assets/icons/manifest.webmanifest">"#),
        "base-prefixed manifest link missing: {html}"
    );
}

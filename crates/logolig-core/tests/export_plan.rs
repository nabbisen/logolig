//! Default-value regression tests for domain types.
//!
//! Prevents accidental changes to the defaults specified in §7.1 and §6.2.

use logolig_core::{ExportPlan, ResizeAlgorithm, ThemeMode};

#[test]
fn export_plan_default_is_minimal_modern_set() {
    // §7.1: favicon.ico, apple-touch-icon.png, high-res PNG, HTML snippet
    // v1.2.0: favicon.svg added to default (SVG source → raw copy; raster → vectorise)
    // v1.4.1: vtracer_preset = Default added (v1.2.0 compatible)
    let plan = ExportPlan::default();
    assert!(plan.include_ico);
    assert!(plan.include_apple_touch);
    assert!(plan.include_html_snippet);
    assert!(plan.include_svg);
    assert!(plan.vectorize_on_raster);
    assert_eq!(plan.vtracer_preset, logolig_core::VtracerPreset::Default);
    // Minimal modern set: 32 / 192 / 512 (browser tab, PWA, hi-DPI)
    assert_eq!(plan.png_sizes, vec![32, 192, 512]);
    // ICO contains 16/32/48 for legacy compatibility
    assert_eq!(plan.ico_sizes, vec![16, 32, 48]);
}

#[test]
fn resize_default_is_lanczos3() {
    // §6.2 "default to quality-first"
    assert_eq!(ResizeAlgorithm::default(), ResizeAlgorithm::Lanczos3);
}

#[test]
fn theme_mode_cycles() {
    // Theme cycle: System → Light → Dark → System
    assert_eq!(ThemeMode::System.next(), ThemeMode::Light);
    assert_eq!(ThemeMode::Light.next(), ThemeMode::Dark);
    assert_eq!(ThemeMode::Dark.next(), ThemeMode::System);
}

#[test]
fn artifact_count_matches_default_plan() {
    // v1.2.0: svg(1) + ico(1) + apple(1) + html(1) + png_sizes(3) = 7
    assert_eq!(ExportPlan::default().artifact_count(), 7);
}

// ---------------------------------------------------------------------------
// v1.3.0: size-set editing API
// ---------------------------------------------------------------------------

#[test]
fn add_png_size_keeps_set_sorted_and_unique() {
    let mut plan = ExportPlan::default();
    // Initial: [32, 192, 512]
    assert!(plan.add_png_size(64));
    assert_eq!(plan.png_sizes, vec![32, 64, 192, 512]);
    // Duplicates are not added
    assert!(!plan.add_png_size(64));
    assert_eq!(plan.png_sizes, vec![32, 64, 192, 512]);
    // Verify insertion at head and tail
    assert!(plan.add_png_size(16));
    assert!(plan.add_png_size(1024));
    assert_eq!(plan.png_sizes, vec![16, 32, 64, 192, 512, 1024]);
}

#[test]
fn add_png_size_rejects_out_of_range() {
    let mut plan = ExportPlan::default();
    let baseline = plan.png_sizes.clone();
    // Below minimum
    assert!(!plan.add_png_size(15));
    assert!(!plan.add_png_size(0));
    // Above maximum
    assert!(!plan.add_png_size(1025));
    assert!(!plan.add_png_size(99999));
    assert_eq!(plan.png_sizes, baseline);
}

#[test]
fn remove_png_size_returns_false_for_missing() {
    let mut plan = ExportPlan::default();
    assert!(plan.remove_png_size(192));
    assert_eq!(plan.png_sizes, vec![32, 512]);
    // Removing a non-existent size returns false
    assert!(!plan.remove_png_size(192));
    assert!(!plan.remove_png_size(7));
}

#[test]
fn add_ico_size_respects_256_upper_limit() {
    let mut plan = ExportPlan::default();
    // ICO spec maximum is 256
    assert!(plan.add_ico_size(256));
    assert_eq!(plan.ico_sizes, vec![16, 32, 48, 256]);
    // 257+ is rejected
    assert!(!plan.add_ico_size(257));
    assert!(!plan.add_ico_size(512));
}

#[test]
fn ico_size_set_can_become_empty() {
    // Empty ico_sizes is structurally allowed (user manages via include_ico=false).
    let mut plan = ExportPlan::default();
    for size in [16, 32, 48] {
        plan.remove_ico_size(size);
    }
    assert!(plan.ico_sizes.is_empty());
}

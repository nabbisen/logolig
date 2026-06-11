//! `manifest_writer` behaviour tests (v1.8.0).
//!
//! Properties verified:
//! 1. Output is valid JSON
//! 2. All required fields are present (`name`, `short_name`, `icons`,
//!    `start_url`, `display`, `theme_color`, `background_color`)
//! 3. `icons` array matches `png_sizes`
//! 4. File names follow the `favicon-{size}.png` convention
//! 5. `start_url` and `display` are fixed (`/` and `standalone`)

use logolig_core::services::manifest_writer::{build_manifest_json, MANIFEST_FILENAME};
use logolig_core::WebManifestSettings;
use serde_json::Value;

fn parse(json: &str) -> Value {
    serde_json::from_str(json).expect("manifest output should be valid JSON")
}

#[test]
fn manifest_filename_is_webmanifest_extension() {
    // W3C recommends the `.webmanifest` extension; v1.8 follows that.
    assert_eq!(MANIFEST_FILENAME, "manifest.webmanifest");
}

#[test]
fn output_is_valid_json() {
    let s = WebManifestSettings::default();
    let json_str = build_manifest_json(&s, &[32, 192, 512]);
    let _ = parse(&json_str); // valid JSON if this does not panic
}

#[test]
fn required_top_level_fields_are_present() {
    let s = WebManifestSettings::default();
    let v = parse(&build_manifest_json(&s, &[32]));
    let obj = v.as_object().expect("top level must be an object");
    for key in [
        "name",
        "short_name",
        "icons",
        "start_url",
        "display",
        "theme_color",
        "background_color",
    ] {
        assert!(obj.contains_key(key), "missing top-level field: {key}");
    }
}

#[test]
fn name_and_short_name_are_passed_through() {
    let s = WebManifestSettings {
        name: "Acme Banking".to_string(),
        short_name: "Acme".to_string(),
        ..Default::default()
    };
    let v = parse(&build_manifest_json(&s, &[]));
    assert_eq!(v["name"], "Acme Banking");
    assert_eq!(v["short_name"], "Acme");
}

#[test]
fn theme_and_background_colors_are_passed_through() {
    let s = WebManifestSettings {
        theme_color: "#FF8800".to_string(),
        background_color: "#101010".to_string(),
        ..Default::default()
    };
    let v = parse(&build_manifest_json(&s, &[]));
    assert_eq!(v["theme_color"], "#FF8800");
    assert_eq!(v["background_color"], "#101010");
}

#[test]
fn icons_array_matches_png_sizes() {
    let s = WebManifestSettings::default();
    let sizes = [32u32, 192, 512];
    let v = parse(&build_manifest_json(&s, &sizes));
    let icons = v["icons"].as_array().expect("icons must be an array");
    assert_eq!(icons.len(), sizes.len());

    for (icon, size) in icons.iter().zip(sizes.iter()) {
        assert_eq!(icon["src"], format!("favicon-{size}.png"));
        assert_eq!(icon["sizes"], format!("{size}x{size}"));
        assert_eq!(icon["type"], "image/png");
        assert_eq!(icon["purpose"], "any");
    }
}

#[test]
fn empty_png_sizes_produces_empty_icons_array() {
    // Not recommended for a PWA, but logolig respects the user's choice.
    let s = WebManifestSettings::default();
    let v = parse(&build_manifest_json(&s, &[]));
    let icons = v["icons"].as_array().expect("icons must be an array");
    assert!(icons.is_empty());
}

#[test]
fn start_url_is_root() {
    // Fixed value in v1.8; not configurable from the UI.
    let s = WebManifestSettings::default();
    let v = parse(&build_manifest_json(&s, &[]));
    assert_eq!(v["start_url"], "/");
}

#[test]
fn display_is_standalone() {
    // Fixed value in v1.8; not configurable from the UI.
    let s = WebManifestSettings::default();
    let v = parse(&build_manifest_json(&s, &[]));
    assert_eq!(v["display"], "standalone");
}

#[test]
fn output_ends_with_newline() {
    // POSIX text file convention.
    let s = WebManifestSettings::default();
    let json_str = build_manifest_json(&s, &[]);
    assert!(json_str.ends_with('\n'));
}

#[test]
fn settings_default_uses_friendly_placeholders() {
    // Guarantee "output as-is is a valid manifest".
    let s = WebManifestSettings::default();
    assert!(!s.name.is_empty());
    assert!(!s.short_name.is_empty());
    assert!(WebManifestSettings::is_valid_color(&s.theme_color));
    assert!(WebManifestSettings::is_valid_color(&s.background_color));
    assert!(s.has_required_text());
}

#[test]
fn is_valid_color_accepts_hex_rrggbb() {
    assert!(WebManifestSettings::is_valid_color("#FFFFFF"));
    assert!(WebManifestSettings::is_valid_color("#000000"));
    assert!(WebManifestSettings::is_valid_color("#abcdef"));
    assert!(WebManifestSettings::is_valid_color("#ABCdef"));
}

#[test]
fn is_valid_color_rejects_other_forms() {
    // logolig restricts to #RRGGBB (rejects #RGB and named colours).
    assert!(!WebManifestSettings::is_valid_color("#FFF"));
    assert!(!WebManifestSettings::is_valid_color("FFFFFF"));
    assert!(!WebManifestSettings::is_valid_color("#GGGGGG"));
    assert!(!WebManifestSettings::is_valid_color("white"));
    assert!(!WebManifestSettings::is_valid_color(""));
    assert!(!WebManifestSettings::is_valid_color("#FFFFFF "));
}

#[test]
fn has_required_text_rejects_blank_strings() {
    let mut s = WebManifestSettings::default();
    assert!(s.has_required_text());

    s.name = "".to_string();
    assert!(!s.has_required_text());

    s.name = "   ".to_string(); // whitespace-only
    assert!(!s.has_required_text());

    s.name = "OK".to_string();
    s.short_name = "".to_string();
    assert!(!s.has_required_text());
}

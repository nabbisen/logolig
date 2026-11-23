//! Web manifest input validation in the core (RFC 012, job K2).
//!
//! * One function decides what a colour is: `#RRGGBB` or `#RGB`, any letter case, surrounding
//!   white space trimmed, canonicalised to `#RRGGBB` in upper case; everything else is invalid.
//! * Names are valid when they are not empty after trimming.
//! * `sanitized` (the loader's defence): invalid values of a settings file become the defaults,
//!   valid ones are kept in their canonical form.
//! * The exporter refuses invalid manifest settings with a typed error (defence in depth: only a
//!   hand-edited file can reach it), and the output for valid settings is unchanged.

mod fixtures;

use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;
use logolig::{AppError, ExportPlan, MessageKey, WebManifestSettings};

#[test]
fn normalize_color_accepts_the_two_forms_in_any_case_and_canonicalises() {
    for (input, want) in [
        ("#1A2B3C", "#1A2B3C"),
        ("#1a2b3c", "#1A2B3C"),
        ("#1A2b3C", "#1A2B3C"),
        ("#FFF", "#FFFFFF"),
        ("#fff", "#FFFFFF"),
        ("#ffF", "#FFFFFF"),
        ("#1ab", "#11AABB"),
        ("#000", "#000000"),
        ("  #fff  ", "#FFFFFF"),
        ("\t#1A2B3C\n", "#1A2B3C"),
    ] {
        assert_eq!(
            WebManifestSettings::normalize_color(input).as_deref(),
            Some(want),
            "{input:?}"
        );
    }
}

#[test]
fn normalize_color_rejects_everything_else() {
    for input in [
        "",
        " ",
        "#",
        "#F",
        "#FF",
        "#FFFF",
        "#FFFFF",
        "#12345",
        "#1234567",
        "#FFFFFFF",
        "#FFFFFFFF",
        "FFF",
        "FFFFFF",
        "1A2B3C",
        "#GGG",
        "#GGGGGG",
        "#12G",
        "#fff;",
        "#fff ;",
        "# fff",
        "#f f f",
        "#1A 2B3C",
        "rgb(1,2,3)",
        "red",
        "transparent",
        "##FFF",
        "0xFFFFFF",
        "#ＦＦＦ",
        "#ÿÿÿ",
    ] {
        assert_eq!(
            WebManifestSettings::normalize_color(input),
            None,
            "{input:?} must be invalid"
        );
    }
}

#[test]
fn the_old_strict_predicates_are_unchanged() {
    // Public items stay: `is_valid_color` is still the strict #RRGGBB test, and `has_required_text`
    // still looks at the two names.
    assert!(WebManifestSettings::is_valid_color("#1A2B3C"));
    assert!(!WebManifestSettings::is_valid_color("#FFF"));
    let mut m = WebManifestSettings::default();
    assert!(m.has_required_text());
    m.short_name = "   ".into();
    assert!(!m.has_required_text());
}

#[test]
fn normalize_text_is_the_trimmed_name_and_empty_is_invalid() {
    assert_eq!(
        WebManifestSettings::normalize_text("My App").as_deref(),
        Some("My App")
    );
    assert_eq!(
        WebManifestSettings::normalize_text("  My App \t").as_deref(),
        Some("My App")
    );
    assert_eq!(WebManifestSettings::normalize_text(""), None);
    assert_eq!(WebManifestSettings::normalize_text("   "), None);
    assert_eq!(WebManifestSettings::normalize_text("\n\t"), None);
}

#[test]
fn sanitized_replaces_only_what_is_invalid_and_canonicalises_the_rest() {
    let bad = WebManifestSettings {
        name: "  Fine  ".into(),
        short_name: "   ".into(),
        theme_color: "red".into(),
        background_color: "#abc".into(),
    };
    let (clean, replaced) = bad.sanitized();
    assert!(replaced, "an invalid value was replaced");
    let d = WebManifestSettings::default();
    assert_eq!(clean.name, "Fine", "a valid name is kept, trimmed");
    assert_eq!(
        clean.short_name, d.short_name,
        "an empty name is the default"
    );
    assert_eq!(
        clean.theme_color, d.theme_color,
        "a bad colour is the default"
    );
    assert_eq!(
        clean.background_color, "#AABBCC",
        "#abc is canonicalised, not replaced"
    );

    // Nothing wrong: nothing reported, even if a value only needed canonicalising.
    let ok = WebManifestSettings {
        theme_color: "#fff".into(),
        ..WebManifestSettings::default()
    };
    let (clean, replaced) = ok.sanitized();
    assert!(!replaced, "canonicalising is not a replacement");
    assert_eq!(clean.theme_color, "#FFFFFF");
    // The defaults themselves are already clean.
    assert_eq!(
        WebManifestSettings::default().sanitized(),
        (WebManifestSettings::default(), false)
    );
}

fn plan_with(manifest: WebManifestSettings) -> ExportPlan {
    ExportPlan {
        web_manifest: Some(manifest),
        ..ExportPlan::default()
    }
}

#[test]
fn the_exporter_refuses_invalid_manifest_settings_with_a_typed_error() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    for (what, manifest) in [
        (
            "theme_color",
            WebManifestSettings {
                theme_color: "red".into(),
                ..WebManifestSettings::default()
            },
        ),
        (
            "background_color",
            WebManifestSettings {
                background_color: "#12".into(),
                ..WebManifestSettings::default()
            },
        ),
        (
            "name",
            WebManifestSettings {
                name: "".into(),
                ..WebManifestSettings::default()
            },
        ),
        (
            "short_name",
            WebManifestSettings {
                short_name: "   ".into(),
                ..WebManifestSettings::default()
            },
        ),
    ] {
        let err = run_in_memory(&asset, &plan_with(manifest)).expect_err(what);
        match &err {
            AppError::ManifestInvalid { field } => assert_eq!(*field, what),
            other => panic!("{what}: {other:?}"),
        }
        assert_eq!(err.key(), MessageKey::ErrorManifestInvalid);
        assert_eq!(err.args(), vec![("field", what.to_string())]);
    }
}

#[test]
fn the_exporter_accepts_the_defaults_and_valid_values_and_the_output_is_unchanged() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let ok = WebManifestSettings {
        name: "Acme".into(),
        short_name: "Acme".into(),
        theme_color: "#1A2B3C".into(),
        background_color: "#abcdef".into(), // lower case hex digits were always valid
    };
    for manifest in [WebManifestSettings::default(), ok.clone()] {
        let arts = run_in_memory(&asset, &plan_with(manifest.clone())).expect("valid manifest");
        let json = arts
            .iter()
            .find(|a| a.relative_path.to_string_lossy() == "manifest.webmanifest")
            .expect("the manifest is made");
        let text = String::from_utf8(json.bytes.clone()).unwrap();
        assert!(text.contains(&manifest.theme_color), "{text}");
        assert!(text.contains(&manifest.name), "{text}");
    }
    // Without a manifest nothing is validated.
    run_in_memory(&asset, &ExportPlan::default()).expect("no manifest, no check");
}

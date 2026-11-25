//! The "Interface size" setting (RFC 034, job J24): three steps named in words, a lenient read, and a
//! settings file that older and newer versions both load.

use logolig::{InterfaceSize, PersistedSettings};

#[test]
fn the_three_steps_map_to_their_factors() {
    assert_eq!(InterfaceSize::Normal.factor(), 1.0);
    assert_eq!(InterfaceSize::Large.factor(), 1.25);
    assert_eq!(InterfaceSize::ExtraLarge.factor(), 1.5);
    assert_eq!(
        InterfaceSize::all(),
        [
            InterfaceSize::Normal,
            InterfaceSize::Large,
            InterfaceSize::ExtraLarge
        ]
    );
    assert_eq!(InterfaceSize::default(), InterfaceSize::Normal);
}

/// The file is hand-editable and may come from another version: whatever is stored under the key,
/// the application starts, at Normal unless it is one of the three names.
#[test]
fn an_unknown_or_malformed_stored_value_loads_as_normal() {
    let load = |json: &str| -> InterfaceSize {
        let text = format!(r#"{{"interface_size": {json}}}"#);
        serde_json::from_str::<PersistedSettings>(&text)
            .unwrap_or_else(|e| panic!("{json} must load: {e}"))
            .interface_size
    };
    // The three names.
    assert_eq!(load(r#""normal""#), InterfaceSize::Normal);
    assert_eq!(load(r#""large""#), InterfaceSize::Large);
    assert_eq!(load(r#""extra_large""#), InterfaceSize::ExtraLarge);
    // Everything else is Normal: an unknown name, an empty string, the wrong case, a spelling that
    // is not canonical, a number (a factor or a percentage), null, a boolean, a list, an object.
    for bad in [
        r#""huge""#,
        r#""""#,
        r#""Large""#,
        r#""LARGE""#,
        r#""Extra_Large""#,
        r#""extra-large""#,
        r#""extra large""#,
        r#""150""#,
        r#""200%""#,
        "1.5",
        "150",
        "0",
        "-1",
        "null",
        "true",
        "[]",
        r#"["large"]"#,
        r#"{"large": 1}"#,
    ] {
        assert_eq!(load(bad), InterfaceSize::Normal, "{bad}");
    }
}

#[test]
fn a_1_28_file_without_the_field_loads_as_normal() {
    // The shape 1.28 wrote: no `interface_size`.
    let old = r#"{"export_plan": {}, "theme": "Dark", "locale": "ja"}"#;
    let s: PersistedSettings = serde_json::from_str(old).unwrap();
    assert_eq!(s.interface_size, InterfaceSize::Normal);
    assert_eq!(s.theme, logolig::ThemeMode::Dark);
    assert_eq!(s.locale.as_deref(), Some("ja"));
}

#[test]
fn a_file_with_an_unknown_extra_key_loads() {
    // What a later version may add: unknown fields stay ignored (downgrade-safe).
    let newer = r#"{"interface_size": "large", "zoom_shortcut": true, "theme": "Light"}"#;
    let s: PersistedSettings = serde_json::from_str(newer).unwrap();
    assert_eq!(s.interface_size, InterfaceSize::Large);
    assert_eq!(s.theme, logolig::ThemeMode::Light);
}

#[test]
fn every_step_round_trips_in_its_canonical_lowercase_form() {
    for (step, name) in [
        (InterfaceSize::Normal, "normal"),
        (InterfaceSize::Large, "large"),
        (InterfaceSize::ExtraLarge, "extra_large"),
    ] {
        let settings = PersistedSettings {
            interface_size: step,
            ..PersistedSettings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(
            json.contains(&format!(r#""interface_size":"{name}""#)),
            "{json}"
        );
        let back: PersistedSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.interface_size, step);
    }
}

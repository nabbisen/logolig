//! The single-choice groups (RFC 021, job J10): Settings (language, theme) and the preview
//! panel ("View as", "Surface"), each a `snora::radio_group`.
//!
//! Questions: is each group **one** Tab stop; do the arrow keys and a click publish the message
//! the buttons published before (a choice made by an arrow selects at once); do Enter, Space and a
//! click on the option already chosen publish nothing; is a disabled group out of the Tab order
//! and deaf to the pointer.

use iced::Size;
use iced::keyboard::{Key, key::Named};
use iced::mouse::Cursor;
use logolig_i18n::Locale;

use super::controls::{Region, stop_at};
use super::support::{Case, Combo, Probe, click_events, key_press, probe_of};
use crate::app::{AppState, Message};

const SIZE: Size = Size::new(720.0, 1700.0);

fn state(case: Case) -> AppState {
    Combo {
        case,
        size: SIZE,
        locale: Locale::En,
        dark: false,
    }
    .state()
}

fn debug(messages: &[Message]) -> Vec<String> {
    messages.iter().map(|m| format!("{m:?}")).collect()
}

/// Focuses the Tab stop that holds the text `label` (the group whose option it is), presses `key`,
/// and returns what the page published. Groups are found by their labels: the page's other stops
/// (the navigation, the buttons) come before and after them in the Tab order.
fn press(state: &AppState, label: &str, key: Named) -> Vec<String> {
    let mut p = probe_of(state, SIZE);
    let at = centre_of(&mut p, label);
    let stop =
        stop_at(&mut p, at, Region::Page).unwrap_or_else(|| panic!("{label:?} is in no Tab stop"));
    p.focus_target(stop);
    let (_status, messages) = p.send(&[key_press(Key::Named(key))], Cursor::Unavailable);
    debug(&messages)
}

/// Clicks the centre of the text `label` and returns what the page published.
fn click(state: &AppState, label: &str) -> Vec<String> {
    let mut p = probe_of(state, SIZE);
    let at = centre_of(&mut p, label);
    let (_status, messages) = p.send(&click_events(at), Cursor::Available(at));
    debug(&messages)
}

fn centre_of(p: &mut Probe<'_, Message>, label: &str) -> iced::Point {
    p.walk()
        .iter()
        .find(|s| s.text == label)
        .unwrap_or_else(|| panic!("no text {label:?} on the page"))
        .bounds
        .center()
}

#[test]
fn each_group_is_one_tab_stop() {
    // The stop that holds a group's options: the same for all of them, a different one per group.
    let stop_of = |state: &AppState, label: &str| {
        let mut p = probe_of(state, SIZE);
        let at = centre_of(&mut p, label);
        stop_at(&mut p, at, Region::Page)
    };
    let one_stop = |state: &AppState, labels: &[&str]| -> usize {
        let first =
            stop_of(state, labels[0]).unwrap_or_else(|| panic!("{:?} is in no stop", labels[0]));
        for label in labels {
            assert_eq!(
                stop_of(state, label),
                Some(first),
                "{label:?} is in the group's stop"
            );
        }
        first
    };
    let settings = state(Case::Settings);
    let language = one_stop(&settings, &["日本語", "English"]);
    let theme = one_stop(&settings, &["Light", "Dark"]);
    assert_ne!(language, theme, "two groups, two stops");

    let preview = state(Case::ResultPreview);
    let view_as = one_stop(
        &preview,
        &["Browser tab (16×16)", "Smartphone home", "Checker"],
    );
    let surface = one_stop(&preview, &["Light", "Dark"]);
    assert_ne!(view_as, surface);

    // With the checker the Surface group is disabled: no Tab stop.
    let checker = state(Case::ResultPreviewChecker);
    one_stop(&checker, &["Browser tab (16×16)", "Smartphone home"]);
    assert_eq!(
        stop_of(&checker, "Dark"),
        None,
        "the disabled group is no stop"
    );
}

/// The keys and clicks of the Settings groups publish the messages the rows published.
#[test]
fn settings_groups_publish_the_messages_the_rows_published() {
    let settings = state(Case::Settings); // English and Light selected
    // Language: Ja, En, System in this order; English is selected.
    assert_eq!(
        press(&settings, "日本語", Named::ArrowDown),
        ["LocalePicked(None)"],
        "next: System"
    );
    assert_eq!(
        press(&settings, "日本語", Named::ArrowUp),
        ["LocalePicked(Some(Ja))"],
        "previous: 日本語"
    );
    assert_eq!(click(&settings, "日本語"), ["LocalePicked(Some(Ja))"]);
    // "System" is in both groups; the first one on the page is the language group's.
    assert_eq!(click(&settings, "System"), ["LocalePicked(None)"]);
    // Theme: Light, Dark, System; Light is selected.
    assert_eq!(
        press(&settings, "Light", Named::ArrowDown),
        ["ThemePicked(Dark)"]
    );
    assert_eq!(
        press(&settings, "Light", Named::ArrowUp),
        ["ThemePicked(System)"],
        "wraps to the last"
    );
    // The option that is already chosen, Enter and Space publish nothing.
    assert!(
        click(&settings, "English").is_empty(),
        "a click on the chosen option"
    );
    assert!(
        click(&settings, "Light").is_empty(),
        "a click on the chosen option"
    );
    for key in [Named::Enter, Named::Space] {
        assert!(
            press(&settings, "日本語", key).is_empty(),
            "{key:?} on the language group"
        );
        assert!(
            press(&settings, "Light", key).is_empty(),
            "{key:?} on the theme group"
        );
    }
}

/// The preview groups publish what the buttons published.
#[test]
fn preview_groups_publish_the_messages_the_buttons_published() {
    let preview = state(Case::ResultPreview); // Browser tab and System selected
    assert_eq!(
        press(&preview, "Browser tab (16×16)", Named::ArrowDown),
        ["PreviewContextSelected(SmartphoneIcon)"]
    );
    assert_eq!(
        click(&preview, "Smartphone home"),
        ["PreviewContextSelected(SmartphoneIcon)"]
    );
    assert_eq!(
        click(&preview, "Checker"),
        ["PreviewContextSelected(TransparencyChecker)"]
    );
    assert_eq!(
        press(&preview, "Dark", Named::ArrowDown),
        ["PreviewBackgroundSelected(Light)"]
    );
    assert_eq!(click(&preview, "Dark"), ["PreviewBackgroundSelected(Dark)"]);
    assert!(
        click(&preview, "Browser tab (16×16)").is_empty(),
        "the chosen option"
    );
    assert!(click(&preview, "System").is_empty(), "the chosen option");
}

/// A disabled group (Surface during the checker) ignores the pointer.
#[test]
fn the_disabled_surface_group_ignores_a_click() {
    let checker = state(Case::ResultPreviewChecker);
    assert!(
        click(&checker, "Dark").is_empty(),
        "the Surface group is disabled"
    );
    // The View as group still works.
    assert_eq!(
        click(&checker, "Smartphone home"),
        ["PreviewContextSelected(SmartphoneIcon)"]
    );
}

// ---- the Customize page: SVG conversion and the resize algorithm ------------------------------

/// The Customize page with every option on, tall enough to show everything.
fn customize() -> AppState {
    Combo {
        case: Case::CustomizeAdvanced,
        size: Size::new(720.0, 4000.0),
        locale: Locale::En,
        dark: false,
    }
    .state()
}

/// The keys of a group of the Customize page, found by its first option's label, whatever its
/// place in the Tab order is.
fn customize_press(label: &str, key: Named) -> Vec<String> {
    let state = customize();
    let size = Size::new(720.0, 4000.0);
    let mut p = probe_of(&state, size);
    let at = centre_of(&mut p, label);
    let index = p
        .focus_targets()
        .iter()
        .position(|t| t.bounds.contains(at))
        .unwrap_or_else(|| panic!("{label:?} is in no focus target"));
    p.focus_target(index);
    let (_status, messages) = p.send(&[key_press(Key::Named(key))], Cursor::Unavailable);
    debug(&messages)
}

fn customize_click(label: &str) -> Vec<String> {
    let state = customize();
    let mut p = probe_of(&state, Size::new(720.0, 4000.0));
    let at = centre_of(&mut p, label);
    let (_status, messages) = p.send(&click_events(at), Cursor::Available(at));
    debug(&messages)
}

/// One group of three named presets edits `vtracer_preset`; the old slider and the pick list are
/// gone. The names are the descriptive ones.
#[test]
fn the_svg_conversion_group_publishes_the_preset_messages() {
    // Default (balanced) is selected.
    assert_eq!(
        customize_press("Sharp (logos, icons)", Named::ArrowDown),
        ["VtracerPresetChanged(PhotoRich)"]
    );
    assert_eq!(
        customize_press("Sharp (logos, icons)", Named::ArrowUp),
        ["VtracerPresetChanged(Sharp)"]
    );
    assert_eq!(
        customize_click("Sharp (logos, icons)"),
        ["VtracerPresetChanged(Sharp)"]
    );
    assert_eq!(
        customize_click("Photo-rich (gradients)"),
        ["VtracerPresetChanged(PhotoRich)"]
    );
    assert!(
        customize_click("Default (balanced)").is_empty(),
        "the chosen option"
    );
    // The same three names appear once on the page: no second control for the same setting.
    let state = customize();
    let mut p = probe_of(&state, Size::new(720.0, 4000.0));
    for name in [
        "Sharp (logos, icons)",
        "Default (balanced)",
        "Photo-rich (gradients)",
    ] {
        let count = p.walk().iter().filter(|s| s.text == name).count();
        assert_eq!(count, 1, "{name:?} is on the page {count} times");
    }
    assert!(
        !p.walk()
            .iter()
            .any(|s| s.text == "Simple" || s.text == "Detailed"),
        "the slider's labels are gone"
    );
}

#[test]
fn the_resize_algorithm_group_publishes_the_algorithm_messages() {
    // Lanczos3 (sharp) is selected; the five in order.
    assert_eq!(
        customize_press("Lanczos3 (sharp)", Named::ArrowDown),
        ["AlgorithmChanged(MitchellNetravali)"]
    );
    assert_eq!(
        customize_press("Lanczos3 (sharp)", Named::ArrowUp),
        ["AlgorithmChanged(Nearest)"],
        "wraps to the last"
    );
    assert_eq!(customize_click("Bilinear"), ["AlgorithmChanged(Bilinear)"]);
    assert!(
        customize_click("Lanczos3 (sharp)").is_empty(),
        "the chosen option"
    );
}

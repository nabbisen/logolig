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

/// Focuses the `index`-th stop **of the page** (the navigation items are not counted), presses
/// `key`, and returns what the page published.
fn press(state: &AppState, index: usize, key: Named) -> Vec<String> {
    let mut p = probe_of(state, SIZE);
    let stops = p.focus_targets();
    let global = stops
        .iter()
        .enumerate()
        .filter(|(i, _)| !p.is_navigation_stop(*i, stops.len()))
        .nth(index)
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("no page stop number {index}"));
    p.focus_target(global);
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
    // The navigation items are stops too (RFC 027 J14b); the groups are the rest.
    let page_stops = |case: Case| {
        let state = state(case);
        let mut p = probe_of(&state, SIZE);
        let stops = p.focus_targets();
        (0..stops.len())
            .filter(|i| !p.is_navigation_stop(*i, stops.len()))
            .count()
    };
    assert_eq!(
        page_stops(Case::Settings),
        2,
        "the language group and the theme group"
    );
    // Both preview groups are stops; with the checker the Surface group is disabled: no stop.
    assert_eq!(page_stops(Case::ResultPreview), 2, "View as, Surface");
    assert_eq!(page_stops(Case::ResultPreviewChecker), 1, "View as only");
}

/// The keys and clicks of the Settings groups publish the messages the rows published.
#[test]
fn settings_groups_publish_the_messages_the_rows_published() {
    let settings = state(Case::Settings); // English and Light selected
    // Language: Ja, En, System in this order; English is selected.
    assert_eq!(
        press(&settings, 0, Named::ArrowDown),
        ["LocalePicked(None)"],
        "next: System"
    );
    assert_eq!(
        press(&settings, 0, Named::ArrowUp),
        ["LocalePicked(Some(Ja))"],
        "previous: 日本語"
    );
    assert_eq!(click(&settings, "日本語"), ["LocalePicked(Some(Ja))"]);
    // "System" is in both groups; the first one on the page is the language group's.
    assert_eq!(click(&settings, "System"), ["LocalePicked(None)"]);
    // Theme: Light, Dark, System; Light is selected.
    assert_eq!(press(&settings, 1, Named::ArrowDown), ["ThemePicked(Dark)"]);
    assert_eq!(
        press(&settings, 1, Named::ArrowUp),
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
            press(&settings, 0, key).is_empty(),
            "{key:?} on the language group"
        );
        assert!(
            press(&settings, 1, key).is_empty(),
            "{key:?} on the theme group"
        );
    }
}

/// The preview groups publish what the buttons published.
#[test]
fn preview_groups_publish_the_messages_the_buttons_published() {
    let preview = state(Case::ResultPreview); // Browser tab and System selected
    assert_eq!(
        press(&preview, 0, Named::ArrowDown),
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
        press(&preview, 1, Named::ArrowDown),
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

//! `update` tests: web manifest input (RFC 012, job K2).
//!
//! The four fields have edit buffers (like `png_size_input`). The plan, and so `settings.json`, only
//! ever receive a **valid** value, canonicalised (`#RRGGBB` upper case; names trimmed); the previous
//! valid value stays while the text is invalid; half-typed text is never saved. Without a blur event in
//! iced 0.14's `text_input`, the plan takes every keystroke that makes the text valid and **Enter**
//! also shows the canonical form in the field. An error is shown only once the field has been typed
//! in or submitted.

use super::manifest_inputs::ManifestField::{BackgroundColor, Name, ShortName, ThemeColor};
use super::manifest_inputs::{ManifestField, ManifestInputs};
use super::persist::sanitize_loaded_manifest;
use super::*;

fn with_manifest() -> AppState {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::IncludeWebManifestToggled(true));
    state
}

fn type_into(state: &mut AppState, field: ManifestField, text: &str) {
    let message = match field {
        Name => Message::WebManifestNameChanged(text.into()),
        ShortName => Message::WebManifestShortNameChanged(text.into()),
        ThemeColor => Message::WebManifestThemeColorChanged(text.into()),
        BackgroundColor => Message::WebManifestBackgroundColorChanged(text.into()),
    };
    let task = update(state, message);
    // Typing starts no work. The one task there may be is the scroll that keeps the line under the
    // field in view while that line is shown (release 1.29.0, R2b).
    let expected = if state.manifest_inputs.shows_error(field) {
        crate::focus::keep_in_view::<Message>().units()
    } else {
        0
    };
    assert_eq!(
        task.units(),
        expected,
        "typing starts nothing but the scroll"
    );
}

fn submit(state: &mut AppState, field: ManifestField) {
    let _ = update(state, Message::WebManifestSubmitted(field));
}

fn saved(state: &AppState) -> String {
    format!("{:?}", snapshot_persisted(state))
}

fn manifest(state: &AppState) -> logolig::WebManifestSettings {
    state.export_plan.web_manifest.clone().expect("on")
}

#[test]
fn half_typed_text_is_never_saved_and_the_previous_valid_color_stays() {
    let mut state = with_manifest();
    type_into(&mut state, ThemeColor, "#1A2"); // valid shorthand: committed as #11AA22
    assert_eq!(manifest(&state).theme_color, "#11AA22");
    let after_valid = saved(&state);
    // Typing on: "#1A2B" (4 digits), "#1A2B3" (5): invalid. The plan keeps #11AA22, nothing is saved.
    for partial in ["#1A2B", "#1A2B3"] {
        type_into(&mut state, ThemeColor, partial);
        assert_eq!(
            manifest(&state).theme_color,
            "#11AA22",
            "{partial}: previous valid value"
        );
        assert_eq!(saved(&state), after_valid, "{partial}: nothing was saved");
        assert_eq!(
            state.manifest_inputs.get(ThemeColor),
            partial,
            "the buffer holds what was typed"
        );
    }
    type_into(&mut state, ThemeColor, "#1A2B3C");
    assert_eq!(manifest(&state).theme_color, "#1A2B3C");
    assert_ne!(saved(&state), after_valid);
}

#[test]
fn starting_a_color_from_nothing_never_saves_the_prefixes() {
    let mut state = with_manifest();
    let before = saved(&state);
    for prefix in ["", "#", "#F", "#FF"] {
        type_into(&mut state, BackgroundColor, prefix);
        assert_eq!(saved(&state), before, "{prefix:?}: not saved");
        assert_eq!(manifest(&state).background_color, "#FFFFFF");
    }
}

#[test]
fn enter_shows_the_canonical_form_and_keystrokes_do_not() {
    let mut state = with_manifest();
    type_into(&mut state, ThemeColor, "#abc");
    assert_eq!(
        state.manifest_inputs.get(ThemeColor),
        "#abc",
        "the field is left as typed"
    );
    assert_eq!(
        manifest(&state).theme_color,
        "#AABBCC",
        "the value written is canonical"
    );
    submit(&mut state, ThemeColor);
    assert_eq!(
        state.manifest_inputs.get(ThemeColor),
        "#AABBCC",
        "Enter: what you see is what is written"
    );
    // Lower case and white space too.
    type_into(&mut state, BackgroundColor, "  #a1b2c3 ");
    submit(&mut state, BackgroundColor);
    assert_eq!(state.manifest_inputs.get(BackgroundColor), "#A1B2C3");
    assert_eq!(manifest(&state).background_color, "#A1B2C3");
}

#[test]
fn submitting_an_invalid_value_changes_nothing_but_shows_the_error() {
    let mut state = with_manifest();
    let before = saved(&state);
    type_into(&mut state, ThemeColor, "red");
    submit(&mut state, ThemeColor);
    assert_eq!(
        state.manifest_inputs.get(ThemeColor),
        "red",
        "the text is not rewritten"
    );
    assert_eq!(manifest(&state).theme_color, "#FFFFFF");
    assert_eq!(saved(&state), before);
    assert!(state.manifest_inputs.shows_error(ThemeColor));
    // An untouched field shows no error; a submit on an untouched default is a valid commit.
    assert!(!state.manifest_inputs.shows_error(BackgroundColor));
    submit(&mut state, BackgroundColor);
    assert!(!state.manifest_inputs.shows_error(BackgroundColor));
    assert_eq!(
        saved(&state),
        before,
        "committing the unchanged default saves nothing new"
    );
}

#[test]
fn names_are_trimmed_and_may_not_be_empty() {
    let mut state = with_manifest();
    type_into(&mut state, Name, "  Acme Inc ");
    assert_eq!(manifest(&state).name, "Acme Inc");
    assert_eq!(
        state.manifest_inputs.get(Name),
        "  Acme Inc ",
        "spaces stay typable"
    );
    submit(&mut state, Name);
    assert_eq!(state.manifest_inputs.get(Name), "Acme Inc");
    let before = saved(&state);
    type_into(&mut state, Name, "   ");
    assert_eq!(
        manifest(&state).name,
        "Acme Inc",
        "an empty name does not replace the last one"
    );
    assert_eq!(saved(&state), before);
    assert!(state.manifest_inputs.shows_error(Name));
    type_into(&mut state, ShortName, "");
    assert!(state.manifest_inputs.shows_error(ShortName));
    assert_eq!(manifest(&state).short_name, "App");
}

#[test]
fn the_error_goes_when_the_text_is_valid_again_and_is_never_shown_on_an_untouched_field() {
    let mut state = with_manifest();
    for field in [Name, ShortName, ThemeColor, BackgroundColor] {
        assert!(
            !state.manifest_inputs.shows_error(field),
            "{field:?}: untouched default"
        );
    }
    type_into(&mut state, ThemeColor, "#GGG");
    assert!(state.manifest_inputs.shows_error(ThemeColor));
    type_into(&mut state, ThemeColor, "#123");
    assert!(!state.manifest_inputs.shows_error(ThemeColor));
}

#[test]
fn a_valid_commit_changes_only_its_own_leaf_of_settings_json() {
    let mut state = with_manifest();
    let before = saved(&state);
    type_into(&mut state, ThemeColor, "#1A2B3C");
    let after = saved(&state);
    let lines = |s: &str| {
        s.split(',')
            .map(str::trim)
            .map(String::from)
            .collect::<Vec<_>>()
    };
    let (b, a) = (lines(&before), lines(&after));
    let removed: Vec<_> = b.iter().filter(|l| !a.contains(l)).collect();
    let added: Vec<_> = a.iter().filter(|l| !b.contains(l)).collect();
    assert_eq!(removed.len(), 1, "{removed:?}");
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(
        added[0].contains("#1A2B3C") && removed[0].contains("#FFFFFF"),
        "{added:?} {removed:?}"
    );
}

#[test]
fn turning_the_manifest_off_and_on_or_resetting_clears_the_buffers_and_the_errors() {
    let mut state = with_manifest();
    type_into(&mut state, ThemeColor, "red");
    let _ = update(&mut state, Message::IncludeWebManifestToggled(false));
    let _ = update(&mut state, Message::IncludeWebManifestToggled(true));
    assert_eq!(state.manifest_inputs.get(ThemeColor), "#FFFFFF");
    assert!(!state.manifest_inputs.shows_error(ThemeColor));
    type_into(&mut state, Name, "");
    let _ = update(&mut state, Message::ExportPlanResetRequested);
    assert_eq!(state.manifest_inputs, ManifestInputs::default());
}

#[test]
fn typing_raises_no_toast() {
    let mut state = with_manifest();
    for text in ["#", "#F", "red", "#1A2B3C"] {
        type_into(&mut state, ThemeColor, text);
        submit(&mut state, ThemeColor);
    }
    assert!(state.toasts.is_empty());
}

// ---- the loader's defence ----------------------------------------------------------------------

fn loaded(manifest: logolig::WebManifestSettings) -> AppState {
    let mut state = AppState::default();
    state.export_plan.web_manifest = Some(manifest);
    sanitize_loaded_manifest(&mut state);
    state
}

#[test]
fn a_settings_file_with_bad_manifest_values_gets_defaults_and_one_warning() {
    let state = loaded(logolig::WebManifestSettings {
        name: "".into(),
        short_name: "Fine".into(),
        theme_color: "red".into(),
        background_color: "#12".into(),
    });
    let m = manifest(&state);
    let d = logolig::WebManifestSettings::default();
    assert_eq!(m.name, d.name);
    assert_eq!(m.short_name, "Fine", "a valid value is kept");
    assert_eq!(m.theme_color, d.theme_color);
    assert_eq!(m.background_color, d.background_color);
    assert_eq!(
        state.toasts.len(),
        1,
        "one warning, however many values were bad"
    );
    assert_eq!(
        state.toasts[0].title,
        state.translator.t(MessageKey::ToastManifestResetTitle)
    );
    // The buffers show what is in use, untouched (no error).
    assert_eq!(state.manifest_inputs.get(ThemeColor), d.theme_color);
    assert!(!state.manifest_inputs.shows_error(ThemeColor));
}

#[test]
fn valid_values_are_kept_canonicalised_without_a_toast_and_no_manifest_is_left_alone() {
    let state = loaded(logolig::WebManifestSettings {
        name: " Acme ".into(),
        short_name: "A".into(),
        theme_color: "#abc".into(),
        background_color: "#a1b2c3".into(),
    });
    let m = manifest(&state);
    assert_eq!(
        (
            m.name.as_str(),
            m.theme_color.as_str(),
            m.background_color.as_str()
        ),
        ("Acme", "#AABBCC", "#A1B2C3")
    );
    assert!(state.toasts.is_empty());
    let mut off = AppState::default();
    sanitize_loaded_manifest(&mut off);
    assert!(off.export_plan.web_manifest.is_none() && off.toasts.is_empty());
}

//! The web manifest form, drawn and driven (RFC 012, job K2).
//!
//! * A field that holds an invalid value shows an **icon and a plain sentence** under it (never
//!   colour alone, never the text glyph `⚠`), in both languages; an untouched field shows nothing.
//! * The error line is always in the tree: a field with an error is still a Tab stop, and the field
//!   being typed in keeps its focus when the error appears.
//! * Enter in a field publishes the submit message.
//!
//! The error state is part of the layout matrix (`Case::CustomizeManifestErrors`): every property
//! of the matrix (no overflow, hit areas, tab stops, focus in view, ...) runs on it at every size,
//! in both languages and both themes.

use iced::Size;
use iced::keyboard::{Key, key::Named};
use iced::mouse::Cursor;
use logolig_i18n::Locale;

use super::focus_after::Session;
use super::support::{Case, Combo, key_press, probe_of};
use crate::app::{ManifestField, Message};

const SIZE: Size = Size::new(1280.0, 4000.0);
/// The `AlertTriangle` icon of the bundled lucide font.
const WARNING_ICON: &str = "\u{e193}";

fn texts(case: Case, locale: Locale) -> Vec<String> {
    let state = Combo {
        case,
        size: SIZE,
        locale,
        dark: false,
    }
    .state();
    probe_of(&state, SIZE)
        .walk()
        .iter()
        .map(|s| s.text.clone())
        .collect()
}

#[test]
fn every_invalid_field_shows_an_icon_and_a_sentence_in_both_languages() {
    for (locale, color, name) in [
        (
            Locale::En,
            "Use a color like #1A2B3C or #1AB",
            "Enter a name",
        ),
        (
            Locale::Ja,
            "#1A2B3C や #1AB のような色を指定してください",
            "名前を入力してください",
        ),
    ] {
        let all = texts(Case::CustomizeManifestErrors, locale);
        let count = |what: &str| all.iter().filter(|t| t.as_str() == what).count();
        assert_eq!(count(color), 2, "{locale:?}: one for each colour field");
        assert_eq!(count(name), 2, "{locale:?}: one for each name field");
        assert_eq!(
            count(WARNING_ICON),
            4,
            "{locale:?}: an icon with each sentence"
        );
        assert!(
            !all.iter().any(|t| t.contains('\u{26A0}')),
            "no text glyph as a state cue"
        );
    }
}

#[test]
fn untouched_fields_show_no_error() {
    let all = texts(Case::CustomizeAdvanced, Locale::En);
    assert!(
        !all.iter().any(|t| t == WARNING_ICON),
        "no icon on the defaults"
    );
    assert!(
        !all.iter().any(|t| t.starts_with("Use a color")),
        "no sentence"
    );
}

#[test]
fn a_field_with_an_error_is_still_a_tab_stop() {
    let stops = |case| {
        let state = Combo {
            case,
            size: SIZE,
            locale: Locale::En,
            dark: false,
        }
        .state();
        probe_of(&state, SIZE).focus_targets().len()
    };
    assert_eq!(
        stops(Case::CustomizeAdvanced),
        stops(Case::CustomizeManifestErrors),
        "an error adds no stop and removes none"
    );
}

/// The centre y of the text `label` on the page.
fn label_y(s: &mut Session, label: &str) -> f32 {
    s.with(|p| {
        p.walk()
            .iter()
            .find(|w| w.text == label)
            .unwrap_or_else(|| panic!("no text {label:?}"))
            .bounds
            .center()
            .y
    })
}

#[test]
fn the_field_keeps_its_focus_when_its_error_appears() {
    let mut s = Session::new(Case::CustomizeAdvanced, SIZE);
    // Text inputs in order: the custom size, then name, short name, theme colour, background colour.
    s.with(|p| p.focus_text_input(3));
    let (focused, _) = s.focus();
    let at = focused.expect("the field has the focus");
    let theme_y = label_y(&mut s, "Theme color");
    let stop_y = s.with(|p| p.focus_targets()[at].bounds.center().y);
    assert!(
        (stop_y - theme_y).abs() < 12.0,
        "the focused field is the theme colour field ({stop_y} vs {theme_y})"
    );

    s.apply(Message::WebManifestThemeColorChanged("red".into()));
    assert!(
        s.state
            .manifest_inputs
            .shows_error(ManifestField::ThemeColor)
    );
    let all: Vec<String> = s.with(|p| p.walk().iter().map(|w| w.text.clone()).collect());
    assert!(
        all.iter().any(|t| t == WARNING_ICON),
        "the error line is there"
    );
    assert_eq!(
        s.focus().0,
        Some(at),
        "the same field still holds the focus"
    );

    // And it goes on to work: the error goes when the text is valid, still focused.
    s.apply(Message::WebManifestThemeColorChanged("#123".into()));
    assert_eq!(s.focus().0, Some(at));
}

#[test]
fn enter_in_a_manifest_field_publishes_the_submit_message() {
    let state = Combo {
        case: Case::CustomizeAdvanced,
        size: SIZE,
        locale: Locale::En,
        dark: false,
    }
    .state();
    for (k, field) in [
        (1, "Name"),
        (2, "ShortName"),
        (3, "ThemeColor"),
        (4, "BackgroundColor"),
    ] {
        let mut p = probe_of(&state, SIZE);
        p.focus_text_input(k);
        let (_status, messages) =
            p.send(&[key_press(Key::Named(Named::Enter))], Cursor::Unavailable);
        let got: Vec<String> = messages.iter().map(|m| format!("{m:?}")).collect();
        assert_eq!(got, [format!("WebManifestSubmitted({field})")], "field {k}");
    }
}

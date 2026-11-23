//! `update` tests: "System" follows the operating system (RFC 013, job K3).
//!
//! `AppState::system_theme` is what iced last reported: the one-shot query at boot and every
//! `theme_changes()` event arrive as `Message::SystemThemeMode`, in order, the last wins. `None` means
//! "no preference or nothing answered" (the program cannot tell which), and `System` then resolves to
//! Light. The setting itself stays `System` in `settings.json`: the resolved mode lives only in the state.

use crate::ui::theme;
use iced::theme::Mode;

use super::*;

fn state_with(theme: ThemeMode, mode: Mode) -> AppState {
    AppState {
        theme,
        system_theme: mode,
        ..AppState::default()
    }
}

const MODES: [Mode; 3] = [Mode::None, Mode::Light, Mode::Dark];

#[test]
fn resolve_theme_over_every_system_mode_and_every_setting() {
    for mode in MODES {
        // System follows the mode; None means Light.
        let expected = if mode == Mode::Dark {
            theme::dark()
        } else {
            theme::light()
        };
        assert_eq!(
            resolve_theme(&state_with(ThemeMode::System, mode)),
            expected,
            "System with {mode:?}"
        );
        // Light and Dark ignore the system.
        assert_eq!(
            resolve_theme(&state_with(ThemeMode::Light, mode)),
            theme::light(),
            "Light with {mode:?}"
        );
        assert_eq!(
            resolve_theme(&state_with(ThemeMode::Dark, mode)),
            theme::dark(),
            "Dark with {mode:?}"
        );
    }
}

#[test]
fn resolve_mode_is_never_system() {
    for mode in MODES {
        for setting in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
            assert_ne!(resolve_mode(setting, mode), ThemeMode::System);
        }
    }
    assert_eq!(resolve_mode(ThemeMode::System, Mode::Dark), ThemeMode::Dark);
    assert_eq!(
        resolve_mode(ThemeMode::System, Mode::None),
        ThemeMode::Light
    );
}

fn report(state: &mut AppState, modes: &[Mode]) {
    for mode in modes {
        let task = update(state, Message::SystemThemeMode(*mode));
        assert_eq!(task.units(), 0, "a theme report starts nothing");
    }
}

#[test]
fn the_default_is_not_reported_and_resolves_to_light() {
    let state = AppState::default();
    assert_eq!(state.system_theme, Mode::None);
    assert_eq!(resolve_theme(&state), theme::light());
}

#[test]
fn a_leading_none_then_dark_ends_dark() {
    // What iced does at start (RFC 013, the spike): the window step reports None first, then the
    // portal's value, in one batch.
    let mut state = AppState::default();
    report(&mut state, &[Mode::None, Mode::Dark, Mode::Dark]);
    assert_eq!(state.system_theme, Mode::Dark);
    assert_eq!(resolve_theme(&state), theme::dark());
}

#[test]
fn dark_then_none_ends_light_and_a_live_change_follows() {
    let mut state = AppState::default();
    report(&mut state, &[Mode::Dark]);
    assert_eq!(resolve_theme(&state), theme::dark());
    report(&mut state, &[Mode::None]); // the service went away or has no preference
    assert_eq!(resolve_theme(&state), theme::light());
    report(&mut state, &[Mode::Light, Mode::Dark, Mode::Light]);
    assert_eq!(
        resolve_theme(&state),
        theme::light(),
        "the last report wins"
    );
}

#[test]
fn a_report_changes_nothing_that_is_saved_and_raises_no_toast() {
    for mode in MODES {
        let mut state = AppState::default();
        let before = format!("{:?}", snapshot_persisted(&state));
        report(&mut state, &[mode]);
        assert_eq!(
            format!("{:?}", snapshot_persisted(&state)),
            before,
            "{mode:?}: settings.json"
        );
        assert_eq!(
            state.theme,
            ThemeMode::System,
            "the setting is still System"
        );
        assert!(state.toasts.is_empty());
    }
}

#[test]
fn picking_a_theme_is_independent_of_the_report() {
    let mut state = AppState::default();
    report(&mut state, &[Mode::Dark]);
    let _ = update(&mut state, Message::ThemePicked(ThemeMode::Light));
    assert_eq!(
        resolve_theme(&state),
        theme::light(),
        "an explicit Light wins"
    );
    let _ = update(&mut state, Message::ThemePicked(ThemeMode::System));
    assert_eq!(
        resolve_theme(&state),
        theme::dark(),
        "System follows the report again"
    );
}

#[test]
fn the_system_option_says_honestly_what_it_does_in_both_languages() {
    use logolig_i18n::{Locale, Translator};
    for (locale, light, dark, none) in [
        (
            Locale::En,
            "System (currently Light)",
            "System (currently Dark)",
            "System (not reported; using Light)",
        ),
        (
            Locale::Ja,
            "システム設定に従う (現在: ライト)",
            "システム設定に従う (現在: ダーク)",
            "システム設定に従う (取得できないためライトを使用)",
        ),
    ] {
        let mut state = AppState {
            translator: Translator::for_locale(locale),
            ..AppState::default()
        };
        for (mode, want) in [(Mode::Light, light), (Mode::Dark, dark), (Mode::None, none)] {
            state.system_theme = mode;
            assert_eq!(
                crate::ui::settings_page::system_option_label(&state),
                want,
                "{locale:?} {mode:?}"
            );
        }
    }
}

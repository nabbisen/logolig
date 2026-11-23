//! "System" follows the operating system, drawn (RFC 013, job K3).
//!
//! * A report of the system theme **repaints**: the page drawn with the theme the application
//!   resolves is dark after `Dark` and light after `Light` (and `None`), measured as the mean
//!   luminance of the pixels, in the same session driver as the keyboard tests.
//! * The Settings page says what "System" does, in the three states (text, never colour alone).
//! * The preview's "System" surface follows the same value: the mock-up draws the chrome colour of
//!   the resolved mode (exact flat colours, counted in the drawn pixels).

use std::sync::Arc;

use iced::Size;
use iced::theme::Mode;
use logolig_i18n::Locale;

use super::focus_after::Session;
use super::support::{Case, Combo, probe_of};
use crate::app::{AppState, Message, resolve_theme};

const SIZE: Size = Size::new(720.0, 700.0);
/// Tall, so that the preview section below the result cards is inside the window (nothing scrolls).
const TALL: Size = Size::new(800.0, 3000.0);

fn mean_luminance(rgba: &[u8]) -> f32 {
    let pixels = rgba.as_chunks::<4>().0;
    let sum: f64 = pixels
        .iter()
        .map(|p| 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]))
        .sum();
    (sum / pixels.len() as f64) as f32
}

fn drawn_mean(session: &mut Session) -> f32 {
    let theme = resolve_theme(&session.state);
    session.with(|p| mean_luminance(&p.rgba(&theme, 1.0)))
}

#[test]
fn a_system_theme_report_repaints_the_page() {
    let mut s = Session::new(Case::Settings, SIZE);
    s.state.theme = logolig::ThemeMode::System;
    let light = drawn_mean(&mut s);
    assert!(light > 160.0, "not reported: Light, mean {light}");

    s.apply(Message::SystemThemeMode(Mode::Dark));
    let dark = drawn_mean(&mut s);
    assert!(dark < 90.0, "Dark reported: the page is dark, mean {dark}");

    s.apply(Message::SystemThemeMode(Mode::Light));
    assert!(drawn_mean(&mut s) > 160.0, "Light reported: light again");

    s.apply(Message::SystemThemeMode(Mode::Dark));
    s.apply(Message::SystemThemeMode(Mode::None));
    assert!(drawn_mean(&mut s) > 160.0, "the report goes away: Light");
}

#[test]
fn an_explicit_theme_ignores_the_report_when_drawn() {
    let mut s = Session::new(Case::Settings, SIZE);
    s.state.theme = logolig::ThemeMode::Light;
    s.apply(Message::SystemThemeMode(Mode::Dark));
    assert!(
        drawn_mean(&mut s) > 160.0,
        "Light stays light under a dark system"
    );
    s.state.theme = logolig::ThemeMode::Dark;
    s.apply(Message::SystemThemeMode(Mode::Light));
    assert!(
        drawn_mean(&mut s) < 90.0,
        "Dark stays dark under a light system"
    );
}

#[test]
fn the_settings_page_says_what_system_does_in_the_three_states() {
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
        for (mode, want) in [(Mode::Light, light), (Mode::Dark, dark), (Mode::None, none)] {
            let mut state = Combo {
                case: Case::Settings,
                size: SIZE,
                locale,
                dark: false,
            }
            .state();
            state.system_theme = mode;
            let mut p = probe_of(&state, SIZE);
            let texts: Vec<String> = p.walk().iter().map(|s| s.text.clone()).collect();
            assert!(
                texts.iter().any(|t| t == want),
                "{locale:?} {mode:?}: the page should show {want:?}; it shows {texts:?}"
            );
        }
    }
}

/// A preview cache by hand (the matrix states have none: they show the loading placeholder).
fn preview_cache() -> logolig::PreviewCache {
    let raster = |side: u32| {
        logolig::Rgba8::try_from_raw(
            side,
            side,
            Arc::<[u8]>::from(vec![0x80; (side * side * 4) as usize]),
        )
        .expect("a square")
    };
    logolig::PreviewCache {
        source_path: std::path::PathBuf::from("logo.png"),
        algorithm: logolig::ResizeAlgorithm::Lanczos3,
        tab_16: raster(16),
        icon_120: raster(120),
    }
}

/// How many pixels of the drawn page have exactly this colour.
fn count(rgba: &[u8], colour: [u8; 3]) -> usize {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] == colour)
        .count()
}

fn preview_state(surface: logolig::ThemeMode, mode: Mode) -> AppState {
    let mut state = Combo {
        case: Case::ResultPreview,
        size: TALL,
        locale: Locale::En,
        dark: false,
    }
    .state();
    state.theme = logolig::ThemeMode::Light; // the page itself stays Light: only the surface varies
    state.preview_cache = Some(preview_cache());
    state.preview = Some(logolig::PreviewProfile {
        background: surface,
        ..logolig::PreviewProfile::default()
    });
    state.system_theme = mode;
    state
}

#[test]
fn the_preview_system_surface_follows_the_report() {
    // The browser-tab mock-up paints its chrome with (0x2b,0x2b,0x2e) on Dark and (0xee,0xee,0xf2) on Light.
    const DARK_CHROME: [u8; 3] = [0x2b, 0x2b, 0x2e];
    const LIGHT_CHROME: [u8; 3] = [0xee, 0xee, 0xf2];
    let draw = |surface, mode| {
        let state = preview_state(surface, mode);
        let theme = resolve_theme(&state);
        probe_of(&state, TALL).rgba(&theme, 1.0)
    };
    let system_dark = draw(logolig::ThemeMode::System, Mode::Dark);
    let system_none = draw(logolig::ThemeMode::System, Mode::None);
    let explicit_dark = draw(logolig::ThemeMode::Dark, Mode::None);
    assert!(
        count(&system_dark, DARK_CHROME) > 500,
        "System + Dark draws the dark chrome: {}",
        count(&system_dark, DARK_CHROME)
    );
    assert_eq!(count(&system_dark, LIGHT_CHROME), 0, "and no light chrome");
    assert!(
        count(&system_none, LIGHT_CHROME) > 500,
        "System + not reported draws the light chrome: {}",
        count(&system_none, LIGHT_CHROME)
    );
    assert_eq!(count(&system_none, DARK_CHROME), 0, "and no dark chrome");
    assert_eq!(
        count(&system_dark, DARK_CHROME),
        count(&explicit_dark, DARK_CHROME),
        "System + Dark draws what an explicit Dark surface draws"
    );
}

#[test]
fn the_surface_picker_keeps_the_plain_system_while_settings_says_more() {
    // RFC 013 review: in the narrow Surface picker "System" is enough (the mock-up beside it shows what
    // it resolves to); the honest long label is the Settings page's.
    for mode in [Mode::None, Mode::Light, Mode::Dark] {
        let state = preview_state(logolig::ThemeMode::System, mode);
        let texts: Vec<String> = probe_of(&state, TALL)
            .walk()
            .iter()
            .map(|s| s.text.clone())
            .collect();
        assert!(texts.iter().any(|t| t == "System"), "{mode:?}: {texts:?}");
        assert!(
            !texts.iter().any(|t| t.starts_with("System (")),
            "{mode:?}: no long label in the picker"
        );
    }
}

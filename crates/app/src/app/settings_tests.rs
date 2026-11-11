//! `update` tests: navigation page, theme, and export-plan settings.

use super::*;

#[test]
fn nav_page_selected_switches_page() {
    let mut state = AppState::default();
    assert_eq!(state.nav_page, NavPage::Home); // precondition

    let _ = update(&mut state, Message::NavPageSelected(NavPage::Settings));
    assert_eq!(state.nav_page, NavPage::Settings);

    let _ = update(&mut state, Message::NavPageSelected(NavPage::Home));
    assert_eq!(state.nav_page, NavPage::Home);
}

#[test]
fn theme_picked_updates_theme() {
    let mut state = AppState::default();
    assert_eq!(state.theme, ThemeMode::System); // default
    let _ = update(&mut state, Message::ThemePicked(ThemeMode::Dark));
    assert_eq!(state.theme, ThemeMode::Dark);
}

#[test]
fn keep_transparency_toggle_sets_plan() {
    let mut state = AppState::default();
    assert!(state.export_plan.keep_transparency); // default true
    let _ = update(&mut state, Message::KeepTransparencyToggled(false));
    assert!(!state.export_plan.keep_transparency);
    let _ = update(&mut state, Message::KeepTransparencyToggled(true));
    assert!(state.export_plan.keep_transparency);
}

#[test]
fn png_preset_size_add_inserts_sorted() {
    let mut state = AppState::default();
    assert_eq!(state.export_plan.png_sizes, vec![32, 192, 512]); // default
    let _ = update(&mut state, Message::PngPresetSizeAdded(64));
    assert_eq!(state.export_plan.png_sizes, vec![32, 64, 192, 512]);
}

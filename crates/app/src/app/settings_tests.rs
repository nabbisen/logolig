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

// ---- the settings file changes exactly its own leaf (RFC 011, logic level) -----------------
//
// `persist_settings` writes `snapshot_persisted(state)`. Each test takes that snapshot before and
// after one message and compares the pretty-printed lines: what the message changed must be
// exactly the expected lines, and nothing else (no other leaf, no neighbour).

use logolig_i18n::Locale;

/// The lines that are only in `before` and the lines that are only in `after`, each sorted.
fn changed_lines(
    before: &logolig::PersistedSettings,
    after: &logolig::PersistedSettings,
) -> (Vec<String>, Vec<String>) {
    let lines = |s: &logolig::PersistedSettings| -> Vec<String> {
        format!("{s:#?}")
            .lines()
            .map(|l| l.trim().to_string())
            .collect()
    };
    let (b, a) = (lines(before), lines(after));
    let only = |x: &[String], y: &[String]| -> Vec<String> {
        let mut rest = y.to_vec();
        let mut out = Vec::new();
        for l in x {
            match rest.iter().position(|r| r == l) {
                Some(i) => {
                    rest.remove(i);
                }
                None => out.push(l.clone()),
            }
        }
        out.sort();
        out
    };
    (only(&b, &a), only(&a, &b))
}

/// Applies `message` and returns what it changed in the persisted settings.
fn leaves_changed_by(state: &mut AppState, message: Message) -> (Vec<String>, Vec<String>) {
    let before = snapshot_persisted(state);
    let _ = update(state, message);
    changed_lines(&before, &snapshot_persisted(state))
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn settings_leaf_keep_transparency_changes_only_itself() {
    let mut state = AppState::default();
    let (removed, added) = leaves_changed_by(&mut state, Message::KeepTransparencyToggled(false));
    assert_eq!(removed, strings(&["keep_transparency: true,"]));
    assert_eq!(added, strings(&["keep_transparency: false,"]));
}

#[test]
fn settings_leaf_png_preset_checkbox_changes_only_the_size_list() {
    let mut state = AppState::default();
    let (removed, added) = leaves_changed_by(&mut state, Message::PngPresetSizeAdded(16));
    assert_eq!(removed, Vec::<String>::new());
    assert_eq!(
        added,
        strings(&["16,"]),
        "one more size in png_sizes, nothing else"
    );
}

#[test]
fn settings_leaf_custom_size_changes_only_the_size_list() {
    let mut state = AppState::default();
    // Typing into the field changes the transient buffer, not the settings.
    let (removed, added) =
        leaves_changed_by(&mut state, Message::PngSizeInputChanged("256".into()));
    assert!(
        removed.is_empty() && added.is_empty(),
        "typing is not persisted"
    );
    let (removed, added) = leaves_changed_by(&mut state, Message::PngSizeAddRequested);
    assert_eq!(removed, Vec::<String>::new());
    assert_eq!(
        added,
        strings(&["256,"]),
        "one more size in png_sizes, nothing else"
    );
}

#[test]
fn settings_leaf_language_changes_only_the_locale() {
    let mut state = AppState::default();
    let (removed, added) = leaves_changed_by(&mut state, Message::LocalePicked(Some(Locale::Ja)));
    assert_eq!(removed, strings(&["locale: None,"]));
    assert_eq!(added, strings(&["\"ja\",", "),", "locale: Some("]));
}

#[test]
fn settings_leaf_theme_changes_only_the_theme() {
    let mut state = AppState::default();
    let (removed, added) = leaves_changed_by(&mut state, Message::ThemePicked(ThemeMode::Dark));
    assert_eq!(removed, strings(&["theme: System,"]));
    assert_eq!(added, strings(&["theme: Dark,"]));
}

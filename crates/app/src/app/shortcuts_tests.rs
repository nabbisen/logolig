//! `update` tests: global shortcuts (RFC 019 phase 2).

use super::*;

use super::test_support::{dummy_result_assets, dummy_source_asset};

use crate::shortcuts::Shortcut;

// ---- RFC 019 phase 2: global shortcuts -------------------------------

fn result_state() -> AppState {
    AppState {
        screen: Screen::Result,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    }
}

fn press(state: &mut AppState, shortcut: Shortcut) -> Task<Message> {
    update(state, Message::ShortcutPressed(shortcut))
}

/// The task the mouse path would start for `message`, as a count of started units.
fn units_of(state: &mut AppState, message: Message) -> usize {
    update(state, message).units()
}

#[test]
fn choose_file_starts_the_same_task_as_the_button_on_empty_and_result() {
    for screen in [Screen::Empty, Screen::Result] {
        let mut state = AppState {
            screen,
            ..result_state()
        };
        let mouse = units_of(&mut state, Message::PickFileRequested);
        assert!(mouse > 0, "the mouse path starts a task");
        let key = press(&mut state, Shortcut::ChooseFile).units();
        assert_eq!(key, mouse, "{screen:?}");
    }
}

#[test]
fn choose_file_does_nothing_while_converting_or_off_the_home_page() {
    let mut converting = AppState {
        screen: Screen::Converting,
        ..AppState::default()
    };
    assert_eq!(press(&mut converting, Shortcut::ChooseFile).units(), 0);
    for page in [NavPage::Customize, NavPage::Settings] {
        let mut state = AppState {
            nav_page: page,
            ..AppState::default()
        };
        assert_eq!(
            press(&mut state, Shortcut::ChooseFile).units(),
            0,
            "{page:?}"
        );
    }
}

#[test]
fn download_all_works_only_on_the_result_screen_with_assets() {
    let mut state = result_state();
    let mouse = units_of(&mut state, Message::DownloadAllRequested);
    assert!(mouse > 0);
    assert_eq!(press(&mut state, Shortcut::DownloadAll).units(), mouse);

    let mut empty = AppState::default();
    assert_eq!(press(&mut empty, Shortcut::DownloadAll).units(), 0, "Empty");

    let mut converting = AppState {
        screen: Screen::Converting,
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    };
    assert_eq!(
        press(&mut converting, Shortcut::DownloadAll).units(),
        0,
        "Converting"
    );

    let mut no_assets = AppState {
        screen: Screen::Result,
        ..AppState::default()
    };
    assert_eq!(
        press(&mut no_assets, Shortcut::DownloadAll).units(),
        0,
        "no assets"
    );

    for page in [NavPage::Customize, NavPage::Settings] {
        let mut state = AppState {
            nav_page: page,
            ..result_state()
        };
        assert_eq!(
            press(&mut state, Shortcut::DownloadAll).units(),
            0,
            "{page:?}"
        );
    }
}

#[test]
fn escape_dismisses_the_newest_toast_first_and_does_not_navigate() {
    let mut state = result_state();
    push_warning_toast(&mut state, "first", "one");
    push_warning_toast(&mut state, "second", "two");
    let (first_id, second_id) = (state.toasts[0].id, state.toasts[1].id);

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].id, first_id, "the newest one went first");
    assert_ne!(first_id, second_id);
    assert_eq!(
        state.screen,
        Screen::Result,
        "no navigation while a toast was open"
    );

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert!(state.toasts.is_empty());
    assert_eq!(state.screen, Screen::Result);

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.screen, Screen::Empty, "with no toast, Esc goes back");
}

#[test]
fn escape_dismisses_a_toast_on_any_page_and_screen() {
    for page in [NavPage::Home, NavPage::Customize, NavPage::Settings] {
        for screen in [Screen::Empty, Screen::Converting, Screen::Result] {
            let mut state = AppState {
                nav_page: page,
                screen,
                ..AppState::default()
            };
            push_error_toast(&mut state, AppError::unsupported_file("x.png"));
            let _ = press(&mut state, Shortcut::EscapeOrBack);
            assert!(state.toasts.is_empty(), "{page:?} {screen:?}");
            assert_eq!(state.screen, screen);
            assert_eq!(state.nav_page, page);
        }
    }
}

#[test]
fn escape_without_a_toast_goes_back_only_from_the_home_result_screen() {
    for (page, screen) in [
        (NavPage::Home, Screen::Empty),
        (NavPage::Home, Screen::Converting),
        (NavPage::Customize, Screen::Result),
        (NavPage::Settings, Screen::Result),
    ] {
        let mut state = AppState {
            nav_page: page,
            screen,
            ..result_state()
        };
        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert_eq!(state.screen, screen, "{page:?} {screen:?}");
        assert_eq!(state.nav_page, page);
    }
    let mut state = result_state();
    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.screen, Screen::Empty);
    assert!(
        state.result_assets.is_some(),
        "history is kept, as for the Back button"
    );
}

#[test]
fn page_shortcuts_and_help_set_the_nav_page_from_anywhere() {
    let mut state = AppState::default();
    for (shortcut, page) in [
        (Shortcut::Page(NavPage::Customize), NavPage::Customize),
        (Shortcut::Page(NavPage::Settings), NavPage::Settings),
        (Shortcut::Page(NavPage::Home), NavPage::Home),
        (Shortcut::Help, NavPage::Settings),
    ] {
        let _ = press(&mut state, shortcut);
        assert_eq!(state.nav_page, page, "{shortcut:?}");
    }
    // While converting, switching pages is allowed (the sidebar allows it too).
    let mut converting = AppState {
        screen: Screen::Converting,
        ..AppState::default()
    };
    let _ = press(&mut converting, Shortcut::Page(NavPage::Customize));
    assert_eq!(converting.nav_page, NavPage::Customize);
    assert_eq!(converting.screen, Screen::Converting);
}

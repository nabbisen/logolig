//! Handlers for going back, showing the last result, and the keyboard shortcuts
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use iced::Task;

use super::super::{AppState, Message, NavPage, Screen};
use super::update;

/// Handler for `Message::EditCancelled`.
pub(super) fn edit_cancelled(state: &mut AppState) -> Task<Message> {
    state.file_drag_hovering = false;
    // v1.24.0: result_assets and source_asset are intentionally kept so
    // the history card can offer "View results →" on the Empty screen.
    // Clearing them is deferred until a new file is ingested.
    state.preview = None;
    state.preview_cache = None;
    state.transparency = None;
    state.result_preview_open = false;
    state.screen = Screen::Empty;
    Task::none()
}

/// Handler for `Message::ShowLastResultRequested`.
pub(super) fn show_last_result_requested(state: &mut AppState) -> Task<Message> {
    // Return to the Result screen without re-converting.
    // result_assets is always Some here because the history card
    // only appears when it is Some.
    state.screen = Screen::Result;
    Task::none()
}

/// Apply a keyboard shortcut (RFC 019 phase 2).
///
/// A shortcut acts exactly when the control it stands for is available to the mouse, and it
/// then goes through `update` as the message that control sends, so there is one code path for
/// the action and one set of state checks (this also keeps a second ingest from starting while
/// one is running).
pub(super) fn handle_shortcut(
    state: &mut AppState,
    shortcut: crate::shortcuts::Shortcut,
) -> Task<Message> {
    use crate::shortcuts::Shortcut;

    let on_home = state.nav_page == NavPage::Home;
    let message = match shortcut {
        // The drop zone, or Back and then the drop zone, is reachable.
        Shortcut::ChooseFile
            if on_home && matches!(state.screen, Screen::Empty | Screen::Result) =>
        {
            Message::PickFileRequested
        }
        // The "Download all (ZIP)" button is on the Result screen when there are assets.
        Shortcut::DownloadAll
            if on_home && state.screen == Screen::Result && state.result_assets.is_some() =>
        {
            Message::DownloadAllRequested
        }
        Shortcut::EscapeOrBack => match state.toasts.last() {
            // Toasts are pushed in order, so the last one is the newest.
            Some(toast) => Message::DismissToast(toast.id),
            None if on_home && state.screen == Screen::Result => Message::EditCancelled,
            None => return Task::none(),
        },
        Shortcut::Page(page) => Message::NavPageSelected(page),
        Shortcut::Help => Message::NavPageSelected(NavPage::Settings),
        Shortcut::ChooseFile | Shortcut::DownloadAll => return Task::none(),
    };
    update(state, message)
}

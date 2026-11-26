//! Handlers for the download messages: the save dialogs and the write results
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use std::path::PathBuf;

use iced::Task;

use logolig::MessageKey;

use super::super::saved_path::saved_path_text;
use super::super::toast::push_long_success_toast;
use super::super::{AppState, Message};

/// Handler for `Message::DownloadOneRequested`.
pub(super) fn download_one_requested(state: &mut AppState, idx: usize) -> Task<Message> {
    // Extract the filename and open a save dialog with the right extension.
    let Some(assets) = state.result_assets.as_ref() else {
        return Task::none();
    };
    let Some(item) = assets.items.get(idx) else {
        return Task::none();
    };
    // RFC 031: one dialog at a time, of any kind.
    if state.dialog_open {
        return Task::none();
    }
    let name = item.file_name.clone();
    state.dialog_open = true;
    crate::task_queue::pick_save_one_task(idx, name)
}

/// Handler for `Message::DownloadAllRequested`.
pub(super) fn download_all_requested(state: &mut AppState) -> Task<Message> {
    // RFC 031: one dialog at a time, of any kind.
    if state.dialog_open {
        return Task::none();
    }
    state.dialog_open = true;
    crate::task_queue::pick_save_all_task()
}

/// Handler for `Message::DownloadOneTargetPicked` and `DownloadAllTargetPicked` with `None`: the
/// Save dialog was cancelled. Only the dialog flag changes.
pub(super) fn save_cancelled(state: &mut AppState) -> Task<Message> {
    state.dialog_open = false;
    Task::none()
}

/// Handler for `Message::DownloadOneTargetPicked`.
pub(super) fn download_one_target_picked(
    state: &mut AppState,
    idx: usize,
    path: PathBuf,
) -> Task<Message> {
    // The dialog has answered (RFC 031).
    state.dialog_open = false;
    let Some(assets) = state.result_assets.as_ref() else {
        return Task::none();
    };
    let Some(item) = assets.items.get(idx) else {
        return Task::none();
    };
    let bytes = item.bytes.clone();
    crate::task_queue::write_one_task(path, bytes)
}

/// Handler for `Message::DownloadAllTargetPicked`.
pub(super) fn download_all_target_picked(state: &mut AppState, path: PathBuf) -> Task<Message> {
    // The dialog has answered (RFC 031).
    state.dialog_open = false;
    let Some(assets) = state.result_assets.as_ref() else {
        return Task::none();
    };
    // ZIP assembly runs as a task to avoid blocking the UI thread.
    let items_clone = assets.items.clone();
    crate::task_queue::write_zip_task(path, items_clone)
}

/// Handler for `Message::DownloadOneCompleted`.
pub(super) fn download_one_completed(state: &mut AppState, path: PathBuf) -> Task<Message> {
    let title = state.translator.t(MessageKey::ToastExportTitle);
    // A document-portal path (Flatpak) means nothing to the user: the file name is shown instead.
    let body = saved_path_text(&path);
    push_long_success_toast(state, &title, &body);
    Task::none()
}

/// Handler for `Message::DownloadAllCompleted`.
pub(super) fn download_all_completed(state: &mut AppState, path: PathBuf) -> Task<Message> {
    let title = state.translator.t(MessageKey::ToastExportTitle);
    // A document-portal path (Flatpak) means nothing to the user: the file name is shown instead.
    let body = saved_path_text(&path);
    push_long_success_toast(state, &title, &body);
    Task::none()
}

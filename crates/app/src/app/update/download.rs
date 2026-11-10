//! Handlers for the download messages: the save dialogs and the write results
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use std::path::PathBuf;

use iced::Task;

use logolig::MessageKey;

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
    crate::task_queue::pick_save_one_task(idx, item.file_name.clone())
}

/// Handler for `Message::DownloadOneTargetPicked`.
pub(super) fn download_one_target_picked(
    state: &mut AppState,
    idx: usize,
    path: PathBuf,
) -> Task<Message> {
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
    let body = format!("{}", path.display());
    push_long_success_toast(state, &title, &body);
    Task::none()
}

/// Handler for `Message::DownloadAllCompleted`.
pub(super) fn download_all_completed(state: &mut AppState, path: PathBuf) -> Task<Message> {
    let title = state.translator.t(MessageKey::ToastExportTitle);
    let body = format!("{}", path.display());
    push_long_success_toast(state, &title, &body);
    Task::none()
}

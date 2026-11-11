//! Handlers for the preview context and background, and the common error handler
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use iced::Task;

use logolig::{AppError, PreviewContext, ThemeMode};

use super::super::toast::push_error_toast;
use super::super::{AppState, Message, Screen};

/// Handler for `Message::PreviewContextSelected`.
pub(super) fn preview_context_selected(state: &mut AppState, ctx: PreviewContext) -> Task<Message> {
    if let Some(p) = state.preview.as_mut() {
        p.context = ctx;
    }
    Task::none()
}

/// Handler for `Message::PreviewBackgroundSelected`.
pub(super) fn preview_background_selected(state: &mut AppState, theme: ThemeMode) -> Task<Message> {
    if let Some(p) = state.preview.as_mut() {
        p.background = theme;
    }
    Task::none()
}

/// Handler for `Message::ConvertCompleted` with a result.
pub(super) fn convert_completed(
    state: &mut AppState,
    job: u64,
    assets: crate::result::ResultAssets,
) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    state.result_assets = Some(assets);
    state.screen = Screen::Result;
    Task::none()
}

/// Handler for `Message::ConvertCompleted` with an error.
pub(super) fn convert_failed(state: &mut AppState, job: u64, err: AppError) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    fail(state, err)
}

/// Common error handler.
///
/// - returns to Result if a source is loaded, otherwise Empty
///   (v1.16: the former Preview state was removed)
/// - pushes a persistent error toast (stays until dismissed)
pub(super) fn fail(state: &mut AppState, err: AppError) -> Task<Message> {
    state.screen = if state.result_assets.is_some() {
        Screen::Result
    } else {
        Screen::Empty
    };
    push_error_toast(state, err);
    Task::none()
}

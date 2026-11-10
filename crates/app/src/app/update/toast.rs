//! Handler for the reader's attention on a toast (RFC 032), extracted from `update`
//! (RFC 017: the body is the old arm body unchanged).

use std::time::Instant;

use iced::Task;

use super::super::{AppState, Message};

/// Handler for `Message::ToastAttention`.
pub(super) fn toast_attention(
    state: &mut AppState,
    id: u64,
    attention: snora::ToastAttention,
) -> Task<Message> {
    state
        .holds
        .attend(id, attention, &mut state.toasts, Instant::now());
    Task::none()
}
